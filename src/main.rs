use std::{error::Error, fs, process::exit};

use another_interpreted_language::{lexer::tokenize, parser::{ast::{Expression, Literal, Operator, Program, Statement}, parse_tokens}};

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

fn parse_file(path: &String) -> Result<(), Box<dyn Error>> {
    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            println!("Could not read file: {}", err);
            exit(1)
        } 
    };

    let mut tokens = tokenize(&content)?;
    println!("----\nTokens\n{:#?}\nTokens\n----", tokens);
    let ast = parse_tokens(&mut tokens);
    println!("----\nProgram\n{:#?}\nProgram\n----", ast);

    // let test = Program::from([
    //     Statement::Expr(
    //         Expression::FunctionCall(
    //             Box::from(
    //                 Expression::Literal(
    //                     Literal::String("print".to_string())
    //                 )
    //             ),
    //             vec!(
    //                 Expression::Infix(
    //                     Operator::Plus,
    //                     Box::from(
    //                         Expression::Literal(
    //                             Literal::Integer(5)
    //                         )
    //                     ),
    //                     Box::from(
    //                         Expression::Literal(
    //                             Literal::Integer(10)
    //                         )
    //                     ),
    //                 )
    //             )
    //         )
    //     )
    // ]);

    // println!("{:#?}", test);

    Ok(())
}