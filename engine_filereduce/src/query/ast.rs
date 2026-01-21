use crate::row::RowKind;
use crate::row::Value;

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct Query {
    pub select: Vec<String>,
    pub filter: Option<Expr>,
    pub limit: Option<usize>,
    pub order_by: Option<(String, SortOrder)>,
}

#[derive(Debug)]
pub enum Expr {
    Eq(String, Value),
    Gt(String, Value),
    Lt(String, Value),
    Gte(String, Value),
    Lte(String, Value),
    Like(String, String),
    In(String, Vec<Value>),
    Between(String, Value, Value),

    KindEq(RowKind),

    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}
