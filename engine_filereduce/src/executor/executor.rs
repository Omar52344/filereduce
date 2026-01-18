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
            let mut projected = Row::new();

            for field in &self.query.select {
                if let Some(val) = row.get(field) {
                    projected.insert(field.clone(), val.clone());
                }
            }

            self.count += 1;

            if let Some(limit) = self.query.limit {
                if self.count > limit {
                    return None;
                }
            }
            if let Some(ref filter) = self.query.filter {
                if !Self::eval(filter, &row) {
                    continue;
                }
            }

            return Some(projected);
        }
        None
    }
    fn eval(expr: &Expr, row: &Row) -> bool {
        match expr {
            Expr::Eq(field, val) => {
                matches!(row.get(field), Some(v) if v == val)
            }
            Expr::Gt(field, Value::Number(n)) => {
                matches!(row.get(field), Some(Value::Number(v)) if v > n)
            }
            Expr::Lt(field, Value::Number(n)) => {
                matches!(row.get(field), Some(Value::Number(v)) if v < n)
            }
            _ => false,
        }
    }
}
