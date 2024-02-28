use super::{environment::Environment, object::{NativeFunctionObject, Object}};

pub fn initialize(env: &mut Environment) {

    macro_rules! function {
        ($name:literal, [$($args:tt),*], ($arg_param:tt) => $body:block) => {
            {
                let function = Object::native_function(
                    &NativeFunctionObject(concat!("__fc_", $name), vec!($($args),*), |$arg_param| {
                        $body
                    })
                );

                env.set(concat!("__fc_", $name), function);
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
        Object::void()
    });

    function!("println", ["arg"], (args) => {
        let value = if args.len() > 0 {
            args[0].to_string()
        } else {
            String::from("")
        };

        println!("{}", value);
        Object::void()
    });

    function!("typeof", ["arg"], (args) => {
        let value: String = if args.len() > 0 {
            args[0].get_type().to_string()
        } else {
            String::new()
        };

        Object::string(&value)
    });

    function!("sleep", ["ms"], (args) => {
        let ms = if args.len() > 0 {
            args[0].to_string().parse::<u64>().unwrap()
        } else {
            0
        };

        std::thread::sleep(std::time::Duration::from_millis(ms));
        Object::void()
    });
}