mod ast;
mod parser;
mod token;

use logos::Logos;
use std::env;
use std::fs;
use token::Token;

use crate::parser::Parser;

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
    let mut tokens = Vec::new();
    let mut had_lex_error = false;

    while let Some(tok_res) = lexer.next() {
        match tok_res {
            Ok(tok) => tokens.push(tok),
            Err(_) => {
                eprintln!("Lexical error at token slice: {:?}", lexer.slice());
                had_lex_error = true;
            }
        }
    }

    if had_lex_error {
        std::process::exit(1);
    }

    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(ast) => {
            println!("AST successfully constructed:\n{:#?}", ast);
        }
        Err(err) => {
            eprintln!("Parse error: {}", err);
            std::process::exit(1);
        }
    }
}
