use std::error::Error;

use crate::{create_error_list, parser::ast::ProgramTree};

use self::environment::Environment;

pub mod environment;
pub mod object;
pub mod yaipl_std;

pub struct Evaluator<'a> {
    global_env: Environment<'a>,
    ast: ProgramTree,
}

create_error_list!(EvaluatorErrors, {});

type EvaluatorResult<T> = Result<T, Box<dyn Error>>; 
