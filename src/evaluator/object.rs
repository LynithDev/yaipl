use std::{alloc::{alloc, dealloc, Layout}, cmp::Ordering, fmt::Display, ptr::{addr_of_mut, drop_in_place}};

use crate::parser::ast::FunctionDeclareExpression;

// largely based on https://github.com/dannyvankooten/nederlang/blob/tree-walker/src/object.rs


#[derive(Clone, Debug)]
pub struct Object(*mut u8);

// unsafe impl Sync for Object {}
// unsafe impl Send for Object {}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectType {
    Integer,
    Boolean,
    Function,
    Float,
    String,
    Void
}

const TAG_MASK: usize = 0b111;
const PTR_MASK: usize = !TAG_MASK;
const VALUE_SHIFT_BITS: usize = 3;

impl<'a> Object {
    fn from_type(pointer: *mut u8, object_type: ObjectType) -> Self {
        Self((pointer as usize | object_type as usize) as _)
    }

    pub fn void() -> Self {
        Self::from_type(0 as _, ObjectType::Void)
    }

    pub fn integer(value: i32) -> Self {
        Self::from_type((value << VALUE_SHIFT_BITS) as _, ObjectType::Integer)
    }

    pub fn boolean(value: bool) -> Self {
        match value {
            false => Self::from_type(0 as _, ObjectType::Boolean),
            true => Self::from_type((1 << VALUE_SHIFT_BITS) as _, ObjectType::Boolean)
        }
    }

    pub fn function(func: &'a FunctionDeclareExpression) -> Self {
        Self::from_type(func as *const FunctionDeclareExpression as _, ObjectType::Function)
    }

    pub fn string(value: &'a str) -> Self {
        YaiplString::from_str(value)
    }

    pub fn float(value: f32) -> Self {
        YaiplFloat::from_f32(value)
    }

    pub fn get_type(&self) -> ObjectType {
        unsafe {
            std::mem::transmute((self.0 as usize & TAG_MASK) as u8)
        }
    }

    pub fn is(&self, object_type: ObjectType) -> bool {
        self.get_type() == object_type
    }
    
    pub fn as_boolean(&self) -> Option<bool> {
        match self.get_type() {
            ObjectType::Boolean => Some((self.0 as u8 >> VALUE_SHIFT_BITS) as u8 != 0),
            _ => None
        }
    }

    pub fn as_integer(&self) -> Option<i32> {
        match self.get_type() {
            ObjectType::Integer => Some(self.0 as i32 >> VALUE_SHIFT_BITS),
            _ => None
        }
    }

    pub fn as_function(&self) -> Option<&'a FunctionDeclareExpression> {
        match self.get_type() {
            ObjectType::Function => Some(unsafe { self.get::<FunctionDeclareExpression>() }),
            _ => None
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self.get_type() {
            ObjectType::Float => Some(unsafe { let result: f32 = YaiplFloat::read(self).value; result }),
            _ => None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self.get_type() {
            ObjectType::String => Some(unsafe { self.get::<YaiplString>().value.as_str() }),
            _ => None
        }
    }
    
    pub fn as_ptr(&self) -> *mut u8 {
        (self.0 as usize & PTR_MASK) as *mut u8
    }

    pub unsafe fn get<T>(&self) -> &'a T {
        &*(self.as_ptr() as *const T)
    }

    pub unsafe fn get_mut<T>(&self) -> &'a mut T {
        &mut *(self.as_ptr() as *mut T)
    }

    pub fn free(self) {
        unsafe {
            match self.get_type() {
                ObjectType::Float => YaiplFloat::destroy(self),
                ObjectType::String => YaiplString::destroy(self),
                _ => {}
            }
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get_type() {
            ObjectType::Integer => write!(f, "Integer({})", self.as_integer().expect("Couldn't take as integer")),
            ObjectType::Boolean => write!(f, "Boolean({})", self.as_boolean().unwrap()),
            ObjectType::Function => write!(f, "Function"),
            ObjectType::Float => write!(f, "Float({})", self.as_f32().expect("Couldn't take as f32")),
            ObjectType::String => write!(f, "String({})", self.as_str().expect("Couldn't take as str")),
            ObjectType::Void => write!(f, "Void")
        }
    }
}

#[repr(C)]
pub struct Header {
    pub marked: bool
}

impl Header {
    pub unsafe fn read(obj: &Object) -> &mut Self {
        obj.get_mut::<Self>()
    }
}

fn allocate(layout: Layout) -> *mut u8 {
    unsafe { alloc(layout) }
}

#[repr(C)]
pub struct YaiplFloat {
    header: Header,
    value: f32,
}

impl YaiplFloat {
    unsafe fn read(obj: &Object) -> &Self {
        obj.get::<Self>()
    }

    unsafe fn destroy(obj: Object) {
        drop_in_place(obj.as_ptr() as *mut Self);
        dealloc(obj.as_ptr(), Layout::new::<Self>());
    }

    fn from_f32(value: f32) -> Object {
        let ptr = Object::from_type(allocate(Layout::new::<Self>()), ObjectType::Float);
        let obj = unsafe { ptr.get_mut::<Self>() };
        obj.header.marked = false;
        unsafe { addr_of_mut!(obj.value).write(value); }

        ptr
    }
}

#[repr(C)]
pub struct YaiplString {
    header: Header,
    value: String,
}

impl YaiplString {
    unsafe fn destroy(obj: Object) {
        drop_in_place(obj.as_ptr() as *mut Self);
        dealloc(obj.as_ptr(), Layout::new::<Self>());
    }

    fn from_str(value: &str) -> Object {
        let ptr = Object::from_type(allocate(Layout::new::<Self>()), ObjectType::String);
        let obj = unsafe { ptr.get_mut::<Self>() };
        obj.header.marked = false;
        unsafe { addr_of_mut!(obj.value).write(value.to_string()); }

        ptr
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        match self.get_type() {
            ObjectType::Boolean | ObjectType::Integer | ObjectType::Void | ObjectType::Function => self.0 == other.0,
            ObjectType::Float => self.as_f32().expect("Couldn't take as f32") == other.as_f32().expect("Couldn't take as f32"),
            ObjectType::String => self.as_str().expect("Couldn't take as str") == other.as_str().expect("Couldn't take as str")
        }
    }
}

impl PartialOrd for Object {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        debug_assert_eq!(self.get_type(), other.get_type());
        match self.get_type() {
            ObjectType::Boolean | ObjectType::Integer => self.0.partial_cmp(&other.0),
            ObjectType::Float => self.as_f32().expect("Couldn't take as f32").partial_cmp(&other.as_f32().expect("Couldn't take as f32")),
            ObjectType::String => self.as_str().expect("Couldn't take as string").partial_cmp(other.as_str().expect("Couldn't take as string")),
            ObjectType::Function => None,
            ObjectType::Void => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    TypeError(String),
    SyntaxError(String),
    ReferenceError(String),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::TypeError(msg) => write!(f, "Type error: {}", msg),
            Error::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            Error::ReferenceError(msg) => write!(f, "Reference error: {}", msg),
        }
    }
}

macro_rules! impl_arithmetic {
    ($func_name:ident, $op:tt) => {
        impl_arithmetic!($func_name, $op, (lhs, rhs) => {});
    };

    ($func_name:ident, $op:tt, ($lhs:ident, $rhs:ident) => { $($pat:pat => $result:expr),* }) => {
        pub fn $func_name(self, rhs: Self) -> Result<Object, Error> {
            
            let ($lhs, $rhs) = (self, rhs);
            
            let result = match ($lhs.get_type(), $rhs.get_type()) {
                (ObjectType::Integer, ObjectType::Integer) => Object::integer($lhs.as_integer().expect("Couldn't take as integer") $op $rhs.as_integer().expect("Couldn't take as integer")),
                (ObjectType::Float, ObjectType::Float) => Object::float($lhs.as_f32().expect("Couldn't take as f32") $op $rhs.as_f32().expect("Couldn't take as f32")),
                (ObjectType::Float, ObjectType::Integer) => Object::float($lhs.as_f32().expect("Couldn't take as f32") $op $rhs.as_integer().expect("Couldn't take as integer") as f32),
                (ObjectType::Integer, ObjectType::Float) => Object::float($lhs.as_integer().expect("Couldn't take as integer") as f32 $op $rhs.as_f32().expect("Couldn't take as f32")),
                $($pat => $result,)*
                _ => return Err(Error::TypeError(format!("Operator {:?} cannot be used for type {:?}", stringify!($op), $lhs.get_type()))),
            };

            Ok(result)
        }
    };
}

macro_rules! impl_logical {
    ($func_name:ident, $op:tt) => {
        pub fn $func_name(self, rhs: Self) -> Result<Object, Error> {
            let result = match (self.get_type(), rhs.get_type()) {
                (ObjectType::Boolean, ObjectType::Boolean) => Object::boolean(self.as_boolean().expect("Couldn't take as boolean") $op rhs.as_boolean().expect("Couldn't take as boolean")),
                _ => return Err(Error::TypeError(format!("Operator {} does not support types {:?} and {:?}", stringify!($op), self.get_type(), rhs.get_type())))
            };
            Ok(result)
        }
    };
}

macro_rules! impl_comparison {
    ($func_name:ident, $op:tt) => {
        pub fn $func_name(self, rhs: Self) -> Result<Object, Error> {
            if self.get_type() != rhs.get_type() {
                return Err(Error::TypeError(format!("Comparison does not support types {:?} and {:?}", self.get_type(), rhs.get_type())));
            }

            Ok(Object::boolean(self $op rhs))
        }
    };
}

impl Object {
    impl_arithmetic!(add, +, (lhs, rhs) => {
        (ObjectType::String, _) => Object::string(&(lhs.as_str().expect("Couldn't take as str").to_string() + rhs.to_string().as_str())),
        (_, ObjectType::String) => Object::string(&(lhs.to_string() + rhs.as_str().expect("Couldn't take as str")))
    });

    impl_arithmetic!(sub, -);
    impl_arithmetic!(mul, *);
    impl_arithmetic!(div, /);
    impl_arithmetic!(rem, %);
    pub fn pow(self, rhs: Self) -> Result<Object, Error> {
        let result = match (self.get_type(), rhs.get_type()) {
            (ObjectType::Integer, ObjectType::Integer) => Object::integer(self.as_integer().expect("Couldn't take as integer").pow(rhs.as_integer().expect("Couldn't take as integer") as u32)),
            (ObjectType::Float, ObjectType::Float) => Object::float(self.as_f32().expect("Couldn't take as f32").powf(rhs.as_f32().expect("Couldn't take as f32"))),
            (ObjectType::Float, ObjectType::Integer) => Object::float(self.as_f32().expect("Couldn't take as f32").powf(rhs.as_integer().expect("Couldn't take as integer") as f32)),
            (ObjectType::Integer, ObjectType::Float) => Object::float((self.as_integer().expect("Couldn't take as integer") as f32).powf(rhs.as_f32().expect("Couldn't take as f32"))),
            _ => return Err(Error::TypeError(format!("Operator {:?} cannot be used for type {:?}", "**", self.get_type())))
        };

        Ok(result)
    }

    impl_comparison!(gt, >);
    impl_comparison!(gte, >=);
    impl_comparison!(lt, <);
    impl_comparison!(lte, <=);
    impl_comparison!(eq, ==);
    impl_comparison!(neq, !=);

    impl_logical!(and, &&);
    impl_logical!(or, ||);
}
