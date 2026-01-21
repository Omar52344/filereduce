use crate::query::ast::{Expr, Query, SortOrder};
use crate::reader::reader::Reader;
use crate::row::Row;
use crate::row::Value;
use std::cmp::Ordering;

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

        if let Some((field, order)) = &self.query.order_by {
            results.sort_by(|a, b| {
                let a_val = a.get(field);
                let b_val = b.get(field);

                let cmp = match (a_val, b_val) {
                    (Some(Value::Number(a)), Some(Value::Number(b))) => {
                        a.partial_cmp(b).unwrap_or(Ordering::Equal)
                    }
                    (Some(Value::Text(a)), Some(Value::Text(b))) => a.cmp(b),
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    _ => Ordering::Equal,
                };

                match order {
                    SortOrder::Asc => cmp,
                    SortOrder::Desc => cmp.reverse(),
                }
            });
        }

        if let Some(limit) = self.query.limit {
            results.truncate(limit);
        }

        results
    }
}

pub fn eval(expr: &Expr, row: &Row) -> bool {
    match expr {
        Expr::Eq(field, value) => row.fields.get(field) == Some(value),

        Expr::Gt(field, value) => match (row.fields.get(field), value) {
            (Some(Value::Number(v)), Value::Number(n)) => v > n,
            _ => false,
        },

        Expr::Lt(field, value) => match (row.fields.get(field), value) {
            (Some(Value::Number(v)), Value::Number(n)) => v < n,
            _ => false,
        },

        Expr::Gte(field, value) => match (row.fields.get(field), value) {
            (Some(Value::Number(v)), Value::Number(n)) => v >= n,
            _ => false,
        },

        Expr::Lte(field, value) => match (row.fields.get(field), value) {
            (Some(Value::Number(v)), Value::Number(n)) => v <= n,
            _ => false,
        },

        Expr::Like(field, pattern) => match row.fields.get(field) {
            Some(Value::Text(text)) => matches_like(text, pattern),
            _ => false,
        },

        Expr::In(field, values) => match row.fields.get(field) {
            Some(field_val) => values.contains(field_val),
            _ => false,
        },

        Expr::Between(field, start, end) => match (row.fields.get(field), start, end) {
            (Some(Value::Number(v)), Value::Number(s), Value::Number(e)) => v >= s && v <= e,
            _ => false,
        },

        Expr::KindEq(kind) => &row.kind == kind,

        Expr::And(left, right) => eval(left, row) && eval(right, row),

        Expr::Or(left, right) => eval(left, row) || eval(right, row),

        Expr::Not(inner) => !eval(inner, row),
    }
}

fn matches_like(text: &str, pattern: &str) -> bool {
    let pattern_regex = pattern.replace('%', ".*").replace('_', ".");

    let re = regex::Regex::new(&format!("^{}$", pattern_regex)).unwrap();
    re.is_match(text)
}
