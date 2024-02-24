use std::{fs, process::exit};

use another_interpreted_language::{errors::ErrorList, extract_type, lexer::Lexer, parser::{Parser, ParserErrors, TokenMismatch}, utils::{BLUE, BOLD, MAGENTA, RED, RESET, UNDERLINE}};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        let binary: String = if args.len() < 1 {
            String::from("yaipl")
        } else {
            args[0].to_owned()
        };

        println!("Missing input file");
        println!("USAGE: {} <file>", binary);
        exit(1);
    }

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
        extract_type!(err, ParserErrors, TokenMismatch, (mismatch) => {
            let path = format!("{}:{}:{}", pretty_path, mismatch.position.line, mismatch.position.col);
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
}

fn parse_file(path: &String) -> Result<(), Box<dyn ErrorList>> {
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

