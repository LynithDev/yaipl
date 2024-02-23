use crate::{error, errors::DynamicError};

pub fn unwrap_result<T>(element: Option<T>) -> Result<T, DynamicError> {
    match element {
        Some(element) => Ok(element),
        None => error!("Item is none")
    }
}
