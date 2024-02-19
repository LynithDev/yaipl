use std::{fs, process::exit};

use another_interpreted_language::{parser, SyntaxError, TokenError};

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
    parse_file(file_path)
}

fn parse_file(path: &String) {
    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            println!("Could not read file: {}", err);
            exit(1)
        } 
    };

    match parser::parse(&content) {
        Ok(ret) => {
            println!("{}", ret);
        },
        Err(err) => {
            if let Some(err) = err.downcast_ref::<SyntaxError>() {
                println!("SyntaxError @ '{}:{}:{}'", path, err.line, err.col);
                for line in err.err.lines() {
                    println!("  {}", red(line));
                }
            } else if let Some(err) = err.downcast_ref::<TokenError>() {
                let token_name = match err.token.to_owned() {
                    Some(token) => token.get_name(),
                    None => String::from("unknown-token")
                };

                println!("TokenError @ '{}' for token '{}'", path, token_name);
                for line in err.err.lines() {
                    println!("  {}", red(line));
                }
            } else {
                println!("{}", err);
            }
    
            exit(2)
        }
    }
}

fn red(s: &str) -> String {
    format!("\x1B[0;31m{}\x1B[0m", s)
}
