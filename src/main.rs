#![feature(once_cell)]
mod errors;
mod input_handler;
mod scanner;
mod shared_traits;
mod token;
mod token_type;
use std::{
    env,
    fs::{self, File},
    io,
    io::{BufRead, BufReader},
};

use scanner::Scanner;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        l if l > 2 => {
            println!("Usage: jlox [script]");
            Ok(())
        }
        l if l == 2 => run_file(&args[1]),
        _ => run_prompt(),
    }
}

fn run_file(file_path_str: &String) -> io::Result<()> {
    errors::initialize_managed();
    let file = fs::read_to_string(file_path_str)?;
    run(&file);

    if errors::has_errors() {
        errors::print_all();
        panic!("Completed with errors.");
    }

    Ok(())
}

fn run_prompt() -> io::Result<()> {
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

    Ok(())
}

fn run(source: &String) {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();

    // For now, just print the tokens.
    for token in scanner.tokens() {
        println!("{:?}", token);
    }
}
