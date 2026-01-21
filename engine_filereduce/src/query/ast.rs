use crate::row::RowKind;
use crate::row::Value;

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub select: Vec<String>,
    pub filter: Option<Expr>,
    pub limit: Option<usize>,
    pub order_by: Option<(String, SortOrder)>,
    pub aggregates: Vec<Aggregate>,
}

#[derive(Debug, Clone)]
pub enum Aggregate {
    Count(String),
    Sum(String),
    Avg(String),
    Min(String),
    Max(String),
}

#[derive(Debug, Clone)]
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
