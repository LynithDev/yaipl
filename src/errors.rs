use std::fmt::Debug;

#[macro_export]
macro_rules! create_error {
    ($name:ident, { $($field:ident : $field_type:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            pub err: String,
            $(pub $field : $field_type),*
        }

        impl $name {
            pub fn from(err: String, $($field : $field_type),*) -> Self {
                Self {
                    err: err,
                    $($field),*
                }
            }

            pub fn from_str(err: &str, $($field : $field_type),*) -> Self {
                $name::from(err.to_string(), $($field),*)
            }
        }

        impl std::error::Error for $name {}
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.err)
            }
        }
    };
}

#[macro_export]
macro_rules! error {
    ($error_type:expr) => {
        {
            return Err($error_type.into());
        }
    }
}

pub trait ErrorList {
    fn list_name(&self) -> String;
    fn print(&self) -> String;
    fn error_name(&self) -> String;
}

impl Debug for dyn ErrorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.print())
    }
}

#[macro_export]
macro_rules! create_error_list {
    ($name:ident, { $($error:ident),* $(,)? }) => {
        #[derive(Debug)]
        pub enum $name {
            String(String),
            Error(Box<dyn std::error::Error>),
            $($error($error)),*
        }

        impl From<&str> for $name {
            fn from(err: &str) -> Self {
                $name::String(err.to_string())
            }
        }

        impl From<Box<dyn std::error::Error>> for $name {
            fn from(err: Box<dyn std::error::Error>) -> Self {
                $name::Error(err)
            }
        }

        impl From<$name> for Box<dyn crate::errors::ErrorList> {
            fn from(error: $name) -> Self {
                Box::new(error)
            }
        }

        impl From<Box<dyn crate::errors::ErrorList>> for $name {
            fn from(err: Box<dyn crate::errors::ErrorList>) -> Self {
                $name::String(err.list_name())
            }
        }

        impl crate::errors::ErrorList for $name {
            fn list_name(&self) -> String {
                return stringify!($name).to_string();
            }

            fn print(&self) -> String {
                match self {
                    $name::Error(err) => format!("{:#?}", err),
                    $name::String(err) => err.to_string(),
                    $($name::$error(err) => format!("{:#?}", err)),*
                }
            }

            fn error_name(&self) -> String {
                match self {
                    $name::Error(err) => format!("{:#?}", err).to_string().split_whitespace().next().unwrap_or("").to_string(),
                    $name::String(err) => err.to_string(),
                    $($name::$error(_) => stringify!($name).to_string()),*
                }
            }
        }

        impl std::error::Error for $name {}
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $name::Error(err) => write!(f, "{:#?}", err),
                    $name::String(err) => write!(f, "{}", err),
                    $($name::$error(err) => write!(f, "{}", err)),*
                }
            }
        }
    };
}

pub type DynamicError = Box<dyn std::error::Error>;