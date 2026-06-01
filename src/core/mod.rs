#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f32),
    Array(Vec<Value>),
    Bool(bool),
    Null,
    Func(SeruUserFunc),
}

// TODO: custom error type으로 변경
pub type SeruUserFunc = fn(args: Vec<Value>) -> anyhow::Result<Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    String,
    Number,
    Array,
    Bool,
    Null,
    Func,
}

impl Value {
    pub fn ty(&self) -> ValueType {
        match self {
            Value::String(_) => ValueType::String,
            Value::Number(_) => ValueType::Number,
            Value::Array(_) => ValueType::Array,
            Value::Bool(_) => ValueType::Bool,
            Value::Null => ValueType::Null,
            Value::Func(_) => ValueType::Func,
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
}
