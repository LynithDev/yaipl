use crate::parser::ast::Literal;

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub value: ObjectValue,
}

impl Object {
    pub fn new(value: ObjectValue) -> Object {
        Object {
            value
        }
    }

    pub fn get_type(&self) -> String {
        self.value.name()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectValue {
    Integer(i32),
    Float(f32),
    Boolean(i8),
    String(String),
    Void
    // Function(Vec<Object>, Box<Environment>)
}

impl ObjectValue {
    pub fn to_string(&self) -> String {
        match self {
            ObjectValue::Integer(i) => i.to_string(),
            ObjectValue::Float(f) => f.to_string(),
            ObjectValue::Boolean(b) => b.to_string(),
            ObjectValue::String(s) => s.to_string(),
            ObjectValue::Void => "void".to_string()
        }
    }

    pub fn name(&self) -> String {
        match self {
            ObjectValue::Integer(_) => "int".to_string(),
            ObjectValue::Float(_) => "float".to_string(),
            ObjectValue::Boolean(_) => "bool".to_string(),
            ObjectValue::String(_) => "string".to_string(),
            ObjectValue::Void => "void".to_string()
        }
    }
}

impl From<Literal> for ObjectValue {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::String(s) => ObjectValue::String(s.0),
            Literal::Integer(i) => ObjectValue::Integer(i.0),
            Literal::Float(f) => ObjectValue::Float(f.0),
            Literal::Boolean(b) => ObjectValue::Boolean(b.0)
        }
    }
}
