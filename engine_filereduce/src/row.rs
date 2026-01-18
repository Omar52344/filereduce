use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

pub type Row = HashMap<String, Value>;
