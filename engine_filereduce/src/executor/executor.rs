use crate::query::ast::Expr;
use crate::query::ast::Query;
use crate::reader::reader::Reader;
use crate::row::Row;
use crate::row::Value;

pub struct Executor<R: Reader> {
    reader: R,
    query: Query,
    count: usize,
}

impl<R: Reader> Executor<R> {
    pub fn new(reader: R, query: Query) -> Self {
        Self {
            reader,
            query,
            count: 0,
        }
    }

    pub fn next(&mut self) -> Option<Row> {
        while let Some(row) = self.reader.next() {
            self.count += 1;

            if let Some(limit) = self.query.limit {
                if self.count > limit {
                    return None;
                }
            }

            if let Some(ref filter) = self.query.filter {
                if !eval(filter, &row) {
                    continue;
                }
            }

            let projected = if self.query.select.is_empty() {
                row.clone()
            } else {
                let mut projected = Row::new(row.kind.clone());
                for field in &self.query.select {
                    if let Some(val) = row.get(field) {
                        projected.insert(field.as_str(), val.clone());
                    }
                }
                projected
            };

            return Some(projected);
        }
        None
    }

    pub fn collect(mut self) -> Vec<Row> {
        let mut results = Vec::new();
        while let Some(row) = self.next() {
            results.push(row);
        }
        results
    }
}
pub fn eval(expr: &Expr, row: &Row) -> bool {
    match expr {
        Expr::Eq(field, value) => row.fields.get(field) == Some(value),

        Expr::Gt(field, Value::Number(n)) => {
            matches!(
                row.fields.get(field),
                Some(Value::Number(v)) if v > n
            )
        }

        Expr::Lt(field, Value::Number(n)) => {
            matches!(
                row.fields.get(field),
                Some(Value::Number(v)) if v < n
            )
        }

        Expr::KindEq(kind) => &row.kind == kind,

        Expr::And(left, right) => eval(left, row) && eval(right, row),

        Expr::Or(left, right) => eval(left, row) || eval(right, row),

        Expr::Not(inner) => !eval(inner, row),

        _ => false,
    }
}
