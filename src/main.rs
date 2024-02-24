use std::{fs, io::{stdin, stdout, Read, Write}, process::exit};

use another_interpreted_language::{errors::ErrorList, extract_type, lexer::Lexer, parser::{Parser, ParserErrors, TokenMismatch}, utils::{BLUE, BOLD, MAGENTA, RED, RESET}};

pub const NAME: &str = "YAIPL";
pub const NAME_LONG: &str = "Yet Another Interpreted Programming Language";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        // No input file provided, run REPL
        repl();
        exit(1);
    }

    // Input file provided
    let file_path = &args[1];
    let cwd = std::env::current_dir().expect("Could not get current directory");
    let absolute_path = match fs::canonicalize(file_path) {
        Ok(path) => path,
        Err(err) => {
            println!("Could not get absolute path: {}", err);
            exit(1)
        }
    };

    let pretty_path = match absolute_path.strip_prefix(&cwd) {
        Ok(path) => path.display().to_string(),
        Err(_) => absolute_path.display().to_string()
    };

    if let Err(err) = parse_file(file_path) {
        handle_errors(err, Some(pretty_path));
    }
}

pub fn repl() -> Result<(), Box<dyn ErrorList>> {
    println!("{} - REPL Mode", NAME);
    
    let stdin = stdin();
    let mut buf = String::new();

    loop {
        print!("{}{}>>>{} ", BOLD, BLUE, RESET);
        let _ = stdout().flush();
        let _ = stdin.read_line(&mut buf);

        let mut lexer = Lexer::from(&buf);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::from(&tokens);
        let ast = parser.parse()?;

        println!("{:#?}", ast);
    }
}

pub fn parse_file(path: &String) -> Result<(), Box<dyn ErrorList>> {
    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            println!("Could not read file: {}", err);
            exit(1)
        } 
    };

    let mut lexer = Lexer::from(&content);
    let tokens = lexer.tokenize()?;
    // println!("----\nTokens\n{:#?}\nTokens\n----\n", &tokens);

    let mut parser = Parser::from(&tokens);
    let ast = parser.parse()?;
    println!("\n----\nProgram\n{:#?}\nProgram\n----", ast);

    Ok(())
}

fn handle_errors(err: Box<dyn ErrorList>, path: Option<String>) {
    extract_type!(err, ParserErrors, TokenMismatch, (mismatch) => {
        let path = match path {
            Some(path) => format!("{}:{}:{}", path, mismatch.position.line, mismatch.position.col),
            None => format!("{}:{}", mismatch.position.line, mismatch.position.col)
        };

        println!("{}{}{} error{} at '{}{}{}'", RED, BOLD, mismatch.get_name(), RESET, BLUE, path, RESET);

        print!("->{}{} ", MAGENTA, BOLD);
        if mismatch.expected.len() > 1 {
            println!("Expected tokens of type {:?} but found '{:?}'", mismatch.expected, mismatch.found);
        } else {
            println!("Expected token of type '{:?}' but found '{:?}'", mismatch.expected[0], mismatch.found);
        }
        print!("{}", RESET);

        return;
    });

    println!("{:#?}", err);
}

