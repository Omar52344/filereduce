/*use std::fs::File;
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
        let mut buffer = String::new();

        if self.reader.read_line(&mut buffer).ok()? == 0 {
            return None;
        }

        for segment in buffer.split('\'') {
            if segment.trim().is_empty() {
                continue;
            }

            let mut row = Row::new();
            row.insert("raw".into(), Value::Text(segment.to_string()));

            let mut parts = segment.split('+');
            let tag = parts.next().unwrap_or("");

            row.insert("segment".into(), Value::Text(tag.to_string()));

            match tag {
                "NAD" => {
                    let qualifier = parts.next();
                    let party = parts.next();

                    if let Some(q) = qualifier {
                        row.insert("qualifier".into(), Value::Text(q.to_string()));
                    }

                    if let Some(p) = party {
                        let id = p.split(':').next().unwrap_or(p);
                        row.insert("party_id".into(), Value::Text(id.to_string()));
                    }
                }

                "LIN" => {
                    let line = parts.next();
                    let _empty = parts.next();
                    let sku = parts.next();

                    if let Some(l) = line {
                        row.insert("line".into(), Value::Text(l.to_string()));
                    }

                    if let Some(s) = sku {
                        let code = s.split(':').next().unwrap_or(s);
                        row.insert("sku".into(), Value::Text(code.to_string()));
                    }
                }

                "MOA" => {
                    let data = parts.next();
                    if let Some(d) = data {
                        let mut it = d.split(':');
                        let qualifier = it.next();
                        let amount = it.next();

                        if let Some(q) = qualifier {
                            row.insert("amount_type".into(), Value::Text(q.to_string()));
                        }

                        if let Some(a) = amount {
                            row.insert("amount".into(), Value::Number(a.parse().unwrap_or(0.0)));
                        }
                    }
                }

                _ => {}
            }

            return Some(row);
        }

        None
    }
}
pub fn parse_segment(segment: &str, line: usize) -> Row {
    let mut row = Row::new();

    let seg_id = segment.split('+').next().unwrap_or("");

    row.insert("segment".into(), Value::Text(seg_id.into()));
    row.insert("raw".into(), Value::Text(segment.into()));
    row.insert("line".into(), Value::Number(line as f64));

    // ⚠️ todavía no parseamos campos
    row
}
*/
