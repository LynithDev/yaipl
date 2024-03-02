use std::{fs, io::{stdin, stdout, Write}, process::exit};

use another_interpreted_language::{error, errors::DynamicError, evaluator::{object::{Object, ObjectType}, Evaluator}, lexer::{token::Tokens, Lexer}, parser::{ast::Node, Parser}, utils::colors::{BLUE, BOLD, CYAN, GREEN, MAGENTA, RED, RESET, UNDERLINE, YELLOW}};

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

pub fn repl() {
    println!("{}{}{} - {}REPL Mode{}", 
        format!("{}{}{}", GREEN, BOLD, UNDERLINE),
        NAME,
        RESET,
        format!("{}{}", BLUE, BOLD), 
        RESET
    );
    
    let stdin = stdin();
    let mut buf = String::new();

    loop {
        print!("\n{}{}>>>{} ", BOLD, BLUE, RESET);
        let _ = stdout().flush();
        let _ = stdin.read_line(&mut buf);

        let (_, _, result) = match interpret(buf.to_owned()) {
            Ok(res) => res,
            Err(err) => {
                handle_errors(err, None);
                buf.clear();
                continue;
            }
        };

        println!("{}", result.to_string_with_type());
    }
}

fn interpret(input: String) -> Result<(Tokens, Vec<Node>, Object), DynamicError> {
    let mut lexer = Lexer::from(&input);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::from(&tokens);
    let ast = parser.parse()?;

    if let Node::Program(ast) = ast {
        let mut evaluator = Evaluator::new(&ast);
        let result = evaluator.eval()?;

        return Ok((tokens.to_owned(), ast, result));
    }

    error!("AST is not a program node.");
}

pub fn parse_file(path: &String) -> Result<(), DynamicError> {
    let content = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            println!("Could not read file: {}", err);
            exit(1)
        } 
    };

    let (_, _, result) = interpret(content)?;

    if !result.is(ObjectType::Void) {
        println!("{}", result);
    }

    Ok(())
}

fn handle_errors(err: DynamicError, path: Option<String>) {
    let as_str = err.to_string()
        .replace(r"{{path}}", &path.unwrap_or("unknown_path".to_string()))
        .replace("&r", RED)
        .replace("&g", GREEN)
        .replace("&b", BLUE)
        .replace("&c", CYAN)
        .replace("&m", MAGENTA)
        .replace("&y", YELLOW)
        .replace("&-", RESET)
        .replace("&_", UNDERLINE)
        .replace("&*", BOLD);

    println!("{}{}{}{}", RESET, RED, as_str, RESET);
}

