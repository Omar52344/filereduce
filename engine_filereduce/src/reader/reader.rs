use crate::row::Row;

pub trait Reader {
    fn next(&mut self) -> Option<Row>;
}

#[derive(Clone)]
pub struct MockReader {
    rows: Vec<Row>,
    index: usize,
}

impl MockReader {
    pub fn new(rows: Vec<Row>) -> Self {
        Self { rows, index: 0 }
    }
}

impl Reader for MockReader {
    fn next(&mut self) -> Option<Row> {
        if self.index < self.rows.len() {
            let row = self.rows[self.index].clone();
            self.index += 1;
            Some(row)
        } else {
            None
        }
    }
}
