mod ast;
mod interpreter;
mod lexer;
mod parser;
mod test_runner;
mod token;
mod vm;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use test_runner::TestRunner;

/// Run source with tree-walking interpreter (legacy mode)
fn run_legacy(source: &str) -> Result<(), String> {
    // Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Execution
    let mut interpreter = Interpreter::default();
    interpreter
        .execute(&program)
        .map_err(|e| format!("Runtime error: {}", e))?;

    Ok(())
}

/// Run source with bytecode VM, falling back to interpreter if compilation fails
fn run(source: &str) -> Result<(), String> {
    use vm::compiler::Compiler;
    use vm::VM;

    // Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Try to compile to bytecode
    let compiler = Compiler::new("<main>".to_string());
    match compiler.compile_program(&program) {
        Ok(compilation) => {
            // Execute with VM
            let output = std::io::stdout();
            let mut vm_instance = VM::new(output, std::ptr::null_mut());
            vm_instance.register_functions(compilation.functions);
            vm_instance
                .execute(compilation.main)
                .map_err(|e| format!("VM error: {}", e))?;
        }
        Err(_reason) => {
            // Fall back to tree-walking interpreter
            // Note: Silent fallback for now, could add --verbose flag to show reason
            let mut interpreter = Interpreter::default();
            interpreter
                .execute(&program)
                .map_err(|e| format!("Runtime error: {}", e))?;
        }
    }

    Ok(())
}

fn run_vm(source: &str) -> Result<(), String> {
    use vm::compiler::Compiler;
    use vm::VM;

    // Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Compile to bytecode (no fallback - fail if unsupported)
    let compiler = Compiler::new("<main>".to_string());
    let compilation = compiler.compile_program(&program)?;

    // Execute with VM
    let output = std::io::stdout();
    let mut vm_instance = VM::new(output, std::ptr::null_mut());
    vm_instance.register_functions(compilation.functions);
    vm_instance
        .execute(compilation.main)
        .map_err(|e| format!("VM error: {}", e))?;

    Ok(())
}

fn run_tests(test_dir: &str, verbose: bool) -> Result<(), String> {
    let path = Path::new(test_dir);
    let runner = TestRunner::new(path, verbose);
    let summary = runner.run_all()?;

    if summary.failed > 0 || summary.errors > 0 {
        process::exit(1);
    }

    Ok(())
}

fn print_usage(program: &str) {
    eprintln!(
        "VHP: Vibe-coded Hypertext Preprocessor v{}",
        env!("CARGO_PKG_VERSION")
    );
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  {} <file.php>              Run a PHP file (bytecode VM with fallback)", program);
    eprintln!("  {} --legacy <file.php>     Run with tree-walking interpreter", program);
    eprintln!("  {} --vm <file.php>         Run with bytecode VM (no fallback)", program);
    eprintln!("  {} -r <code>               Run code directly", program);
    eprintln!("  {} test [dir|file] [-v]    Run .vhpt tests", program);
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --legacy                   Use tree-walking interpreter");
    eprintln!("  --vm                       Use bytecode VM only (fail if unsupported)");
    eprintln!("  -v, --verbose              Verbose test output");
    eprintln!();
    eprintln!("Test file format (.vhpt):");
    eprintln!("  --TEST--                   Test name (required)");
    eprintln!("  --DESCRIPTION--            Test description");
    eprintln!("  --FILE--                   PHP code to execute (required)");
    eprintln!("  --EXPECT--                 Expected output (required unless --EXPECT_ERROR--)");
    eprintln!("  --EXPECT_ERROR--           Expected error message");
    eprintln!("  --SKIPIF--                 Reason to skip this test");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let result = match args[1].as_str() {
        "--legacy" | "--tree-walker" => {
            if args.len() < 3 {
                eprintln!("Error: --legacy requires a filename");
                process::exit(1);
            }
            match fs::read_to_string(&args[2]) {
                Ok(source) => run_legacy(&source),
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", &args[2], e);
                    process::exit(1);
                }
            }
        }
        "--vm" => {
            if args.len() < 3 {
                eprintln!("Error: --vm requires a filename");
                process::exit(1);
            }
            match fs::read_to_string(&args[2]) {
                Ok(source) => run_vm(&source),
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", &args[2], e);
                    process::exit(1);
                }
            }
        }
        "-r" => {
            if args.len() < 3 {
                eprintln!("Error: -r requires code argument");
                process::exit(1);
            }
            let code = format!("<?php {}", &args[2]);
            run(&code)
        }
        "test" => {
            let verbose = args.iter().any(|a| a == "-v" || a == "--verbose");
            let test_dir = args
                .iter()
                .skip(2)
                .find(|a| !a.starts_with('-'))
                .map(|s| s.as_str())
                .unwrap_or("tests");
            run_tests(test_dir, verbose)
        }
        "-h" | "--help" => {
            print_usage(&args[0]);
            Ok(())
        }
        filename => match fs::read_to_string(filename) {
            Ok(source) => run(&source),
            Err(e) => {
                eprintln!("Error reading file '{}': {}", filename, e);
                process::exit(1);
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
