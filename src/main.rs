mod ast;
mod codegen;
mod iloc;
mod parser;
mod symbol;
mod token;
mod typecheck;

use codegen::{ArmEmitter, CodeGen};
use logos::Logos;
use parser::Parser;
use std::env;
use std::fs;
use std::path::Path;
use symbol::SymbolTable;
use token::Token;
use typecheck::TypeChecker;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source.c>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let source = fs::read_to_string(input_path).unwrap_or_else(|err| {
        eprintln!("Error reading file {}: {}", input_path.display(), err);
        std::process::exit(1);
    });

    // 1. Lexing
    let mut lexer = Token::lexer(&source);
    let mut tokens = Vec::new();
    while let Some(tok_res) = lexer.next() {
        match tok_res {
            Ok(tok) => tokens.push(tok),
            Err(_) => {
                eprintln!("Lexical error at token: {:?}", lexer.slice());
                std::process::exit(1);
            }
        }
    }

    // 2. Parsing
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse_program() {
        Ok(tree) => tree,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            std::process::exit(1);
        }
    };

    // 3. Type Checking
    let mut symbols = SymbolTable::new();
    let mut checker = TypeChecker::new(&mut symbols);
    if let Err(err) = checker.check_program(&ast) {
        eprintln!("Type error: {}", err);
        std::process::exit(1);
    }

    // 4. Intermediate Representation (ILOC) Lowering
    let codegen = CodeGen::new();
    let (iloc, stack_bytes, strings) = codegen.generate(&ast);

    // 5. ARM32 Code Emission
    let mut emitter = ArmEmitter::new();
    let asm = emitter.emit_arm(&iloc, stack_bytes, &strings);

    // Write .s output file
    let stem = input_path.file_stem().unwrap().to_str().unwrap();
    let output_file = format!("{}.s", stem);
    fs::write(&output_file, asm).expect("Failed to write output assembly file");

    println!("Compilation successful -> {}", output_file);
}
