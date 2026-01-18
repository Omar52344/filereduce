use crate::query::ast::Query;
use crate::reader::reader::Reader;
use crate::row::Row;

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

            return Some(projected);
        }
        None
    }
}
