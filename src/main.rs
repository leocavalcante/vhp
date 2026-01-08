mod ast;
mod interpreter;
mod lexer;
mod parser;
mod test_runner;
mod token;
mod vm;

use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use test_runner::TestRunner;

/// Run source with bytecode VM
fn run(source: &str) -> Result<(), String> {
    use vm::compiler::Compiler;
    use vm::VM;

    // Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Compile to bytecode
    let compiler = Compiler::new("<main>".to_string());
    let compilation = compiler.compile_program(&program)?;

    // Execute with VM
    let output = std::io::stdout();
    let mut vm_instance = VM::new(output, std::ptr::null_mut());
    vm_instance.register_builtins();
    vm_instance.register_functions(compilation.functions);
    vm_instance.register_classes(compilation.classes);
    vm_instance.register_interfaces(compilation.interfaces);
    vm_instance.register_traits(compilation.traits);
    vm_instance.register_enums(compilation.enums);
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
    eprintln!("  {} <file.php>              Run a PHP file", program);
    eprintln!("  {} -r <code>               Run code directly", program);
    eprintln!("  {} test [dir|file] [-v]    Run .vhpt tests", program);
    eprintln!();
    eprintln!("Options:");
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
