use std::{error::Error, fs, process::exit};

use another_interpreted_language::token::tokenize;

fn main() -> Result<(), Box<dyn Error>> {
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
    let content = fs::read_to_string(file_path)?;

    println!("Tokens: {:#?}", tokenize(content.as_str())?);

    Ok(())
}
