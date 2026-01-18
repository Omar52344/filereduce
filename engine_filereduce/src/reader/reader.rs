use crate::row::Row;

pub trait Reader {
    fn next(&mut self) -> Option<Row>;
}
