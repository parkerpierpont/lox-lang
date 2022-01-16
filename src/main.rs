#![feature(once_cell)]
#![feature(box_into_inner)]
mod ast_printer;
mod environment;
mod errors;
mod expr;
mod interpreter;
mod object;
mod parser;
mod runtime_error;
mod scanner;
mod shared_traits;
mod stmt;
mod token;
mod token_type;
use std::{env, fs, io};

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        l if l > 2 => {
            println!("Usage: jlox [script]");
        }
        l if l == 2 => run_file(&args[1]),
        _ => run_prompt(),
    };
}

fn run_file(file_path_str: &String) {
    errors::initialize_managed();
    let file = fs::read_to_string(file_path_str).unwrap();
    run(&file);

    if errors::has_errors() || errors::has_runtime_error() {
        errors::print_all();
        let code = if errors::has_runtime_error() { 70 } else { 65 };
        std::process::exit(code);
    }
}

fn run_prompt() {
    errors::initialize_immediate();
    let mut line = get_user_input();
    while line.is_ok() {
        let v = line.unwrap();
        match v.is_empty() {
            true => {
                line = get_user_input();
                continue;
            }
            false => {
                run(&v);
                errors::reset_errors(); // don't want to crash our whole prompt
                line = get_user_input();
            }
        }
    }
}

fn run(source: &String) {
    let scanner = Scanner::new(source);
    let interpreter = Interpreter::new();
    let tokens = scanner.scan_tokens();
    // for token in &tokens {
    //     println!("{:?}", token)
    // }
    let mut parser = Parser::new(tokens);
    // If we were able to parse without errors, print the expression.
    let statements = parser.parse();
    interpreter.interpret(statements);
}

fn get_user_input() -> io::Result<String> {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    let _ = stdout().flush();
    match stdin().read_line(&mut s) {
        Ok(_) => Ok(s),
        Err(error) => Err(error),
    }
}
