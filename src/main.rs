mod ast;
mod token;

use logos::Logos;
use std::env;
use std::fs;
use token::Token;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source.c>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let source = fs::read_to_string(input_path).unwrap_or_else(|err| {
        eprintln!("Error reading file {}: {}", input_path, err);
        std::process::exit(1);
    });

    let mut lexer = Token::lexer(&source);
    println!("Scanning tokens from {}", input_path);
    while let Some(tok_res) = lexer.next() {
        match tok_res {
            Ok(tok) => println!("  {:?} ({:?})", tok, lexer.slice()),
            Err(_) => println!("  Lexing error at: {:?}", lexer.slice()),
        }
    }
}
