use crate::{error, errors::DynamicError};

pub fn unwrap_result<T>(element: Option<T>) -> Result<T, DynamicError> {
    match element {
        Some(element) => Ok(element),
        None => error!("Unwrapped result is none")
    }
}

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const UNDERLINE: &str = "\x1b[4m";
