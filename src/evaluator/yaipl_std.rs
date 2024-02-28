use super::object::{NativeFunctionObject, Object, ObjectType};

pub fn get_native_function<'a>(list: &'a Vec<Object>, name: &String) -> Option<&'a Object> {
    list.iter().find(|func| {
        if func.is(ObjectType::NativeFunction) {
            func.as_native_function().expect("").0 == name
        } else {
            false
        }
    })
}

pub fn initialize() -> Vec<Object> {
    let mut functions: Vec<Object> = Vec::new();

    macro_rules! function {
        ($name:tt, [$($args:tt),*], ($arg_param:tt) => $body:block) => {
            {
                functions.push(Object::native_function(
                    &NativeFunctionObject($name, vec!($($args),*), |$arg_param| {
                        $body
                    })
                ))
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

    // function!("sleep", ["ms"], (args) => {
    //     let ms = if args.len() > 0 {
    //         args[0].to_string().parse::<u64>().unwrap()
    //     } else {
    //         0
    //     };

    //     std::thread::sleep(std::time::Duration::from_millis(ms));
    //     ObjectValue::Void
    // });

    functions
}