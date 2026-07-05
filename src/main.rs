mod generate;
mod lex;
mod parse;

use generate::generate_code;
use lex::lex;
use parse::{TokensCursor, ast};
use std::process::Command;
use std::{env::args, fs};

fn main() {
    let cli_args: Vec<String> = args().skip(1).collect();
    if cli_args.is_empty() {
        eprintln!(
            "usage: my-c-compiler
  <input.c> [output]"
        );
        std::process::exit(1);
    }
    let input_path = &cli_args[0];
    let output_path = cli_args.get(1).cloned().unwrap_or_else(|| {
        input_path
            .strip_suffix(".c")
            .unwrap_or(input_path)
            .to_string()
    });
    let asm_path = format!("{}.s", output_path);

    let to_compile = fs::read_to_string(input_path)
        .expect(format!("failed to read file {}", input_path).as_ref());

    let tokens = lex(to_compile).unwrap_or_else(|err| {
        eprintln!("Lexer error: {}", err.msg);
        std::process::exit(1);
    });

    println!("LEXING ...");
    for token in &tokens {
        println!("{}", token);
    }

    println!("PARSING ...",);
    let cursor = TokensCursor::new(&tokens);
    let ast_tree = ast(cursor).unwrap_or_else(|err| {
        eprintln!("Parser error: {}", err.0);
        std::process::exit(1);
    });
    println!("Parsed:");
    println!("{}", ast_tree);

    println!("GENERATING ...",);
    let asm = generate_code(ast_tree).unwrap_or_else(|err| {
        eprintln!("Generating error: {}", err.0);
        std::process::exit(1);
    });

    println!("Compiled:");
    println!("{}", asm);
    fs::write(&asm_path, asm).expect("failed to write assembly file");

    let status = Command::new("gcc")
        .arg(&asm_path)
        .arg("-o")
        .arg(&output_path)
        .status()
        .expect("failed to run gcc");

    if !status.success() {
        std::process::exit(1);
    }
}
