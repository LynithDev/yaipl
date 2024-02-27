// use crate::evaluator::object::{NativeFunctionObject, ObjectValue};

// pub fn get_native_function<'a>(list: &'a Vec<ObjectValue>, name: &String) -> Option<&'a ObjectValue> {
//     list.iter().find(|func| {
//         if let ObjectValue::NativeFunction(native) = func {
//             native.name == *name
//         } else {
//             false
//         }
//     })
// }

// pub fn initialize() -> Vec<ObjectValue> {
//     let mut functions: Vec<ObjectValue> = Vec::new();

//     macro_rules! function {
//         ($name:tt, [$($args:tt),*], ($arg_param:tt) => $body:block) => {
//             {
//                 functions.push(ObjectValue::NativeFunction(
//                     NativeFunctionObject::new($name.to_string(), vec!($($args.to_string()),*), |$arg_param| {
//                         $body
//                     })
//                 ))
//             }
//         };
//     }

//     function!("print", ["arg"], (args) => {
//         let value = if args.len() > 0 {
//             args[0].to_string()
//         } else {
//             String::from("")
//         };

//         print!("{}", value);
//         ObjectValue::Void
//     });

//     function!("println", ["arg"], (args) => {
//         let value = if args.len() > 0 {
//             args[0].to_string()
//         } else {
//             String::from("")
//         };

//         println!("{}", value);
//         ObjectValue::Void
//     });

//     function!("typeof", ["arg"], (args) => {
//         let value = if args.len() > 0 {
//             args[0].name()
//         } else {
//             String::from("")
//         };

//         ObjectValue::String(value)
//     });

//     function!("sleep", ["ms"], (args) => {
//         let ms = if args.len() > 0 {
//             args[0].to_string().parse::<u64>().unwrap()
//         } else {
//             0
//         };

//         std::thread::sleep(std::time::Duration::from_millis(ms));
//         ObjectValue::Void
//     });

//     functions
// }