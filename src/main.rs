mod lexer;
mod parser;
mod ast;
mod interpreter;
mod errors;
mod stdlib;

use std::env;
use std::fs;
use std::process;

fn print_usage() {
    eprintln!("Usage: elang <command> [<file>]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  run <file>       Run an elang file");
    eprintln!("  check <file>     Check for errors without running");
    eprintln!("  version          Print version information");
    eprintln!("  help             Print this help message");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "version" => {
            println!("elang v0.1.0");
        }
        "help" => {
            print_usage();
        }
        "run" | "check" => {
            if args.len() < 3 {
                eprintln!("elang: missing file argument");
                eprintln!("Usage: elang {} <file>", command);
                process::exit(1);
            }

            let path = &args[2];

            let source = match fs::read_to_string(path) {
                Ok(content) => content,
                Err(_) => {
                    eprintln!("elang: cannot find file '{}'", path);
                    process::exit(1);
                }
            };

            let tokens = match lexer::tokenize(&source) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };

            let program = match parser::parse(tokens) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };

            if command == "check" {
                println!("elang: no errors found");
                return;
            }

            if let Err(e) = interpreter::run(program) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        _ => {
            eprintln!("elang: unknown command '{}'", command);
            eprintln!();
            print_usage();
            process::exit(1);
        }
    }
}
