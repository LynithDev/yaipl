use std::{fs, process::exit};

use another_interpreted_language::{errors::ErrorList, lexer::Lexer, parser::Parser};

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
    if let Err(err) = parse_file(file_path) {
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
    println!("----\nTokens\n{}\nTokens\n----\n", Lexer::tokens_to_string(&tokens));

    let mut parser = Parser::from(&tokens);
    let ast = parser.parse()?;
    println!("\n----\nProgram\n{:#?}\nProgram\n----", ast);

    Ok(())
}

