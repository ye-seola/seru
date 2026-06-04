use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ImageWrap(pub skia_safe::Image);

impl PartialEq for ImageWrap {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f32),
    Array(Vec<Value>),
    Dict(HashMap<String, Value>),
    Bool(bool),
    Null,
    Func(SeruUserFunc),
    Image(ImageWrap),
}

// TODO: custom error type으로 변경
pub type SeruUserFunc = fn(args: Vec<Value>) -> anyhow::Result<Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    String,
    Number,
    Array,
    Dict,
    Bool,
    Null,
    Func,
    Image,
}

impl Value {
    pub fn ty(&self) -> ValueType {
        match self {
            Value::String(_) => ValueType::String,
            Value::Number(_) => ValueType::Number,
            Value::Array(_) => ValueType::Array,
            Value::Dict(_) => ValueType::Dict,
            Value::Bool(_) => ValueType::Bool,
            Value::Null => ValueType::Null,
            Value::Func(_) => ValueType::Func,
            Value::Image(_) => ValueType::Image,
        }
    }

    pub fn into_string(self) -> anyhow::Result<String> {
        match self {
            Value::String(v) => Ok(v),
            _ => anyhow::bail!("expected string"),
        }
    }

    pub fn into_number(self) -> anyhow::Result<f32> {
        match self {
            Value::Number(v) => Ok(v),
            _ => anyhow::bail!("expected number"),
        }
    }

    pub fn into_bool(self) -> anyhow::Result<bool> {
        match self {
            Value::Bool(v) => Ok(v),
            _ => anyhow::bail!("expected bool"),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::String(val) => val.is_empty(),
            Value::Number(val) => (*val) != 0.0,
            Value::Array(values) => !values.is_empty(),
            Value::Dict(dict) => !dict.is_empty(),
            Value::Bool(bool) => *bool,
            Value::Null => false,
            Value::Func(_) => true,
            Value::Image(_) => true,
        }
    }
}
