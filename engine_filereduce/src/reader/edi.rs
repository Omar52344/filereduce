use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::reader::reader::Reader;
use crate::row::{Row, Value};

pub struct EdiReader {
    reader: BufReader<File>,
}

impl EdiReader {
    pub fn new(path: &str) -> Self {
        let file = File::open(path).expect("Cannot open EDI file");
        Self {
            reader: BufReader::new(file),
        }
    }
}

impl Reader for EdiReader {
    fn next(&mut self) -> Option<Row> {
        let mut line = String::new();

        if self.reader.read_line(&mut line).ok()? == 0 {
            return None;
        }

        let mut row = Row::new();

        for segment in line.trim().split('\'') {
            if segment.is_empty() {
                continue;
            }

            let mut parts = segment.split('+');
            let tag = parts.next().unwrap();

            row.insert("segment".into(), Value::String(tag.into()));

            if let Some(value) = parts.next() {
                row.insert("value".into(), Value::String(value.into()));
            }
        }

        Some(row)
    }
}
