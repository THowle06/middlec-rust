mod ast;
mod parser;
mod symbol;
mod token;
mod typecheck;

use logos::Logos;
use std::env;
use std::fs;
use token::Token;

use crate::parser::Parser;
use crate::symbol::SymbolTable;
use crate::typecheck::TypeChecker;

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
    let ast = match parser.parse_program() {
        Ok(tree) => tree,
        Err(err) => {
            eprint!("Parse error: {}", err);
            std::process::exit(1);
        }
    };

    println!("AST successfully constructed.");

    let mut symbols = SymbolTable::new();
    let mut checker = TypeChecker::new(&mut symbols);
    match checker.check_program(&ast) {
        Ok(()) => {
            println!("Type checking successful: zero semantic errors.");
            println!(
                "Local stack footprint: {} bytes",
                symbols.total_stack_bytes()
            );
        }
        Err(err) => {
            eprintln!("Semantic Error: {}", err);
            std::process::exit(1);
        }
    }
}
