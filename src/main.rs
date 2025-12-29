mod ast;
mod interpreter;
mod lexer;
mod parser;
mod test_runner;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use test_runner::TestRunner;

fn run(source: &str) -> Result<(), String> {
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
    eprintln!("  {} <file.php>           Run a PHP file", program);
    eprintln!("  {} -r <code>            Run code directly", program);
    eprintln!("  {} test [dir] [-v]      Run .vhpt tests", program);
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -v, --verbose           Verbose test output");
    eprintln!();
    eprintln!("Test file format (.vhpt):");
    eprintln!("  --TEST--                Test name (required)");
    eprintln!("  --DESCRIPTION--         Test description");
    eprintln!("  --FILE--                PHP code to execute (required)");
    eprintln!("  --EXPECT--              Expected output (required unless --EXPECT_ERROR--)");
    eprintln!("  --EXPECT_ERROR--        Expected error message");
    eprintln!("  --SKIPIF--              Reason to skip this test");
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
