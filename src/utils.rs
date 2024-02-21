#[macro_export]
macro_rules! create_error {
    ($name:ident, { $($field:ident : $field_type:ty),* $(,)? }) => {
        #[derive(Debug)]
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
    ($error_type:expr) => {{
        return Err($error_type.into());
    }}
}

pub type DynamicError = Box<dyn std::error::Error>;
