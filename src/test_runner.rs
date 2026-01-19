use std::fs;
use std::path::{Path, PathBuf};

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::vm::compiler::Compiler;
use crate::vm::VM;

#[derive(Debug, Default)]
pub struct TestCase {
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    #[allow(dead_code)]
    pub file: String,
    pub code: String,
    pub expected: Option<String>,
    pub expected_error: Option<String>,
    pub skip: Option<String>,
}

#[derive(Debug)]
pub enum TestResult {
    Pass,
    Fail { expected: String, actual: String },
    Error(String),
    Skipped(String),
}

impl TestCase {
    pub fn parse(content: &str, file_path: &str) -> Result<Self, String> {
        let mut test = TestCase {
            file: file_path.to_string(),
            ..Default::default()
        };

        let mut current_section: Option<&str> = None;
        let mut current_content = String::new();

        for line in content.lines() {
            if line.starts_with("--") && line.ends_with("--") && line.len() > 4 {
                // Save previous section
                if let Some(section) = current_section {
                    Self::set_section(&mut test, section, &current_content)?;
                }

                // Start new section
                current_section = Some(line.trim_matches('-'));
                current_content = String::new();
            } else if current_section.is_some() {
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            }
        }

        // Save last section
        if let Some(section) = current_section {
            Self::set_section(&mut test, section, &current_content)?;
        }

        // Validation
        if test.name.is_empty() {
            return Err(format!(
                "Test file {} is missing --TEST-- section",
                file_path
            ));
        }
        if test.code.is_empty() {
            return Err(format!(
                "Test file {} is missing --FILE-- section",
                file_path
            ));
        }
        if test.expected.is_none() && test.expected_error.is_none() {
            return Err(format!(
                "Test file {} is missing --EXPECT-- or --EXPECTF-- or --EXPECT_ERROR-- section",
                file_path
            ));
        }

        Ok(test)
    }

    fn set_section(test: &mut TestCase, section: &str, content: &str) -> Result<(), String> {
        let content = content.trim();
        match section {
            "TEST" => test.name = content.to_string(),
            "DESCRIPTION" => test.description = content.to_string(),
            "FILE" => test.code = content.to_string(),
            "EXPECT" | "EXPECTF" => test.expected = Some(content.to_string()),
            "EXPECT_ERROR" => test.expected_error = Some(content.to_string()),
            "SKIPIF" => test.skip = Some(content.to_string()),
            _ => {} // Ignore unknown sections for forward compatibility
        }
        Ok(())
    }

    pub fn run(&self) -> TestResult {
        // Check skip condition
        if let Some(reason) = &self.skip {
            return TestResult::Skipped(reason.clone());
        }

        // Run the code
        let result = run_code(&self.code);

        match result {
            Ok(output) => {
                if let Some(expected_error) = &self.expected_error {
                    TestResult::Fail {
                        expected: format!("Error: {}", expected_error),
                        actual: output,
                    }
                } else if let Some(expected) = &self.expected {
                    if compare_output(&output, expected) {
                        TestResult::Pass
                    } else {
                        TestResult::Fail {
                            expected: expected.clone(),
                            actual: output,
                        }
                    }
                } else {
                    TestResult::Error("No expected output specified".to_string())
                }
            }
            Err(error) => {
                if let Some(expected_error) = &self.expected_error {
                    if error.contains(expected_error) {
                        TestResult::Pass
                    } else {
                        TestResult::Fail {
                            expected: expected_error.clone(),
                            actual: error,
                        }
                    }
                } else {
                    TestResult::Error(error)
                }
            }
        }
    }
}

fn run_code(source: &str) -> Result<String, String> {
    // Clear global registries for test isolation
    crate::runtime::builtins::spl::clear_autoloaders();
    crate::runtime::builtins::spl::clear_psr4_registry();
    crate::vm::clear_required_files();

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Compile to bytecode
    let compiler = Compiler::new("<test>".to_string());
    let compilation = compiler.compile_program(&program)?;

    // Execute with VM
    let mut output = Vec::new();
    let mut vm = VM::new(&mut output);
    vm.register_builtins();
    vm.register_functions(compilation.functions);
    vm.register_classes(compilation.classes);
    vm.register_interfaces(compilation.interfaces);
    vm.register_traits(compilation.traits);
    vm.register_enums(compilation.enums);

    // Handle exit() as a special case - it's not an error, just termination
    match vm.execute(compilation.main) {
        Ok(_) => {}
        Err(e) if e.starts_with("__EXIT__:") => {
            // exit() was called - this is expected behavior, not an error
        }
        Err(e) => return Err(format!("VM error: {}", e)),
    }

    String::from_utf8(output).map_err(|e| format!("Output encoding error: {}", e))
}

fn compare_output(actual: &str, expected: &str) -> bool {
    // Normalize line endings and trim
    let actual = actual.trim().replace("\r\n", "\n");
    let expected = expected.trim().replace("\r\n", "\n");
    actual == expected
}

pub struct TestRunner {
    test_dir: PathBuf,
    verbose: bool,
}

#[derive(Debug, Default)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub skipped: usize,
    pub failures: Vec<(String, String, String)>, // (name, expected, actual)
}

impl TestRunner {
    pub fn new(test_dir: &Path, verbose: bool) -> Self {
        Self {
            test_dir: test_dir.to_path_buf(),
            verbose,
        }
    }

    pub fn discover_tests(&self) -> Result<Vec<PathBuf>, String> {
        let mut tests = Vec::new();

        // Check if the path is a file or directory
        if self.test_dir.is_file() {
            // Single file - check if it has .vhpt extension
            if self.test_dir.extension().is_some_and(|ext| ext == "vhpt") {
                tests.push(self.test_dir.clone());
            } else {
                return Err(format!(
                    "File must have .vhpt extension: {:?}",
                    self.test_dir
                ));
            }
        } else if self.test_dir.is_dir() {
            // Directory - discover recursively
            self.discover_recursive(&self.test_dir, &mut tests)?;
            tests.sort();
        } else {
            return Err(format!("Path does not exist: {:?}", self.test_dir));
        }

        Ok(tests)
    }

    fn discover_recursive(&self, dir: &Path, tests: &mut Vec<PathBuf>) -> Result<(), String> {
        if !dir.exists() {
            return Err(format!("Test directory does not exist: {:?}", dir));
        }

        let entries =
            fs::read_dir(dir).map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                self.discover_recursive(&path, tests)?;
            } else if path.extension().is_some_and(|ext| ext == "vhpt") {
                tests.push(path);
            }
        }

        Ok(())
    }

    pub fn run_all(&self) -> Result<TestSummary, String> {
        let tests = self.discover_tests()?;
        let mut summary = TestSummary::default();

        if tests.is_empty() {
            println!("No tests found in {:?}", self.test_dir);
            return Ok(summary);
        }

        println!("Running {} tests...\n", tests.len());

        for test_path in &tests {
            summary.total += 1;

            // For single files, show the filename; for directories, show relative path
            let relative_path = if self.test_dir.is_file() {
                test_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| test_path.display().to_string())
            } else {
                test_path
                    .strip_prefix(&self.test_dir)
                    .unwrap_or(test_path)
                    .display()
                    .to_string()
            };

            let content = fs::read_to_string(test_path)
                .map_err(|e| format!("Failed to read test file {:?}: {}", test_path, e))?;

            match TestCase::parse(&content, &relative_path) {
                Ok(test_case) => {
                    let result = test_case.run();

                    match &result {
                        TestResult::Pass => {
                            summary.passed += 1;
                            if self.verbose {
                                println!("  \x1b[32mPASS\x1b[0m {}", test_case.name);
                            } else {
                                print!("\x1b[32m.\x1b[0m");
                            }
                        }
                        TestResult::Fail { expected, actual } => {
                            summary.failed += 1;
                            summary.failures.push((
                                test_case.name.clone(),
                                expected.clone(),
                                actual.clone(),
                            ));
                            if self.verbose {
                                println!("  \x1b[31mFAIL\x1b[0m {}", test_case.name);
                            } else {
                                print!("\x1b[31mF\x1b[0m");
                            }
                        }
                        TestResult::Error(err) => {
                            summary.errors += 1;
                            summary.failures.push((
                                test_case.name.clone(),
                                "No error".to_string(),
                                err.clone(),
                            ));
                            if self.verbose {
                                println!("  \x1b[31mERROR\x1b[0m {}: {}", test_case.name, err);
                            } else {
                                print!("\x1b[31mE\x1b[0m");
                            }
                        }
                        TestResult::Skipped(reason) => {
                            summary.skipped += 1;
                            if self.verbose {
                                println!("  \x1b[33mSKIP\x1b[0m {}: {}", test_case.name, reason);
                            } else {
                                print!("\x1b[33mS\x1b[0m");
                            }
                        }
                    }
                }
                Err(e) => {
                    summary.errors += 1;
                    summary.failures.push((
                        relative_path.clone(),
                        "Valid test file".to_string(),
                        e.clone(),
                    ));
                    if self.verbose {
                        println!("  \x1b[31mERROR\x1b[0m {}: {}", relative_path, e);
                    } else {
                        print!("\x1b[31mE\x1b[0m");
                    }
                }
            }
        }

        if !self.verbose {
            println!();
        }

        println!();
        self.print_summary(&summary);

        Ok(summary)
    }

    fn print_summary(&self, summary: &TestSummary) {
        // Print failures in detail
        if !summary.failures.is_empty() {
            println!("\n\x1b[31mFailures:\x1b[0m\n");
            for (i, (name, expected, actual)) in summary.failures.iter().enumerate() {
                println!("{}. {}", i + 1, name);
                println!("   Expected:\n   {}", expected.replace('\n', "\n   "));
                println!("   Actual:\n   {}", actual.replace('\n', "\n   "));
                println!();
            }
        }

        // Print summary line
        let status_color = if summary.failed > 0 || summary.errors > 0 {
            "\x1b[31m" // Red
        } else if summary.skipped > 0 {
            "\x1b[33m" // Yellow
        } else {
            "\x1b[32m" // Green
        };

        println!(
            "{}Tests: {} total, {} passed, {} failed, {} errors, {} skipped\x1b[0m",
            status_color,
            summary.total,
            summary.passed,
            summary.failed,
            summary.errors,
            summary.skipped
        );
    }
}
