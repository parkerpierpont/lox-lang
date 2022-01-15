#![feature(once_cell)]
mod ast_printer;
mod errors;
mod expr;
mod input_handler;
mod parser;
mod scanner;
mod shared_traits;
mod token;
mod token_type;
use std::{env, fs, io::BufReader};

use ast_printer::AstPrinter;
use expr::VisitorTarget;
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

    if errors::has_errors() {
        errors::print_all();
        std::process::exit(1);
    }
}

fn run_prompt() {
    errors::initialize_immediate();
    let input = std::io::stdin();
    let mut input = input_handler::Input::new(BufReader::new(input.lock()));
    let mut line = input.line().next();
    while line.is_some() {
        let v = line.unwrap();
        match v.is_empty() {
            true => {
                line = input.line().next();
                continue;
            }
            false => {
                run(&v);
                errors::reset_errors(); // don't want to crash our whole prompt
                line = input.line().next();
            }
        }
    }
}

fn run(source: &String) {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    for token in &tokens {
        println!("{:?}", token)
    }
    let mut parser = Parser::new(tokens);
    // If we were able to parse without errors, print the expression.
    if let Some(expression) = parser.parse() {
        println!("Parser Result: \n{}", expression.accept(&AstPrinter));
    }
}
