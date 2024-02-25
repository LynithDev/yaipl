use crate::evaluator::object::{NativeFunctionObject, ObjectValue};

use super::environment::Environment;

pub fn initialize(env: &mut Environment) {
    macro_rules! function {
        ($name:tt, [$($args:tt),*], ($arg_param:tt) => $body:block) => {
            {
                let func = NativeFunctionObject::new($name.to_string(), vec!($($args.to_string()),*), |$arg_param| {
                    $body
                });

                env.set_function($name.to_string(), ObjectValue::NativeFunction(func))
            }
        };
    }

    function!("print", ["arg"], (args) => {
        let value = if args.len() > 0 {
            args[0].to_string()
        } else {
            String::from("")
        };

        print!("{}", value);
        ObjectValue::Void
    });

    function!("println", ["arg"], (args) => {
        let value = if args.len() > 0 {
            args[0].to_string()
        } else {
            String::from("")
        };

        println!("{}", value);
        ObjectValue::Void
    });

    function!("typeof", ["arg"], (args) => {
        let value = if args.len() > 0 {
            args[0].name()
        } else {
            String::from("")
        };

        ObjectValue::String(value)
    });
}