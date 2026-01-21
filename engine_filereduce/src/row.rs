use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RowKind {
    BGM,
    NAD,
    LIN,
    UNH,
    UNS,
    UNT,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
}

pub type RowData = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Row {
    pub kind: RowKind,
    pub fields: HashMap<String, Value>,
}

impl Row {
    pub fn new(kind: RowKind) -> Self {
        Self {
            kind,
            fields: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: Value) {
        self.fields.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }
}
