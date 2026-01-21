use crate::row::RowKind;
use crate::row::Value;
#[derive(Debug)]
pub struct Query {
    pub select: Vec<String>,
    pub filter: Option<Expr>,
    pub limit: Option<usize>,
}

#[derive(Debug)]
pub enum Expr {
    Eq(String, Value),
    Gt(String, Value),
    Lt(String, Value),

    KindEq(RowKind),

    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}
