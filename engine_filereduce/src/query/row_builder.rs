use crate::query::segment::Segment;
use crate::row::{Row, RowKind, Value};

pub struct RowBuilder {
    current: Option<Row>,
    rows: Vec<Row>,
}

impl RowBuilder {
    pub fn new() -> Self {
        Self {
            current: None,
            rows: Vec::new(),
        }
    }

    pub fn consume(&mut self, seg: &Segment) {
        match seg.tag.as_str() {
            "BGM" => self.emit_simple(RowKind::BGM, seg),
            "NAD" => self.emit_simple(RowKind::NAD, seg),
            "LIN" => self.start_lin(seg),
            "QTY" | "MOA" | "PRI" => self.enrich_lin(seg),
            _ => {}
        }
    }

    fn emit_simple(&mut self, kind: RowKind, seg: &Segment) {
        let mut row = Row::new(kind);
        row.insert("raw", Value::Text(seg.elements.join("|")));
        self.rows.push(row);
    }

    fn start_lin(&mut self, seg: &Segment) {
        if let Some(row) = self.current.take() {
            self.rows.push(row);
        }

        let mut row = Row::new(RowKind::LIN);
        if let Some(line) = seg.elements.get(0).and_then(|v| v.parse().ok()) {
            row.insert("line", Value::Number(line));
        }
        if let Some(code) = seg.elements.get(1) {
            row.insert("item", Value::Text(code.clone()));
        }

        self.current = Some(row);
    }

    fn enrich_lin(&mut self, seg: &Segment) {
        if let Some(row) = self.current.as_mut() {
            match seg.tag.as_str() {
                "QTY" => {
                    if let Some(qty) = seg.elements.get(1).and_then(|v| v.parse().ok()) {
                        row.insert("qty", Value::Number(qty));
                    }
                }
                "MOA" => {
                    if let Some(amount) = seg.elements.get(1).and_then(|v| v.parse().ok()) {
                        row.insert("amount", Value::Number(amount));
                    }
                }
                "PRI" => {
                    if let Some(price) = seg.elements.get(1).and_then(|v| v.parse().ok()) {
                        row.insert("price", Value::Number(price));
                    }
                }
                _ => {}
            }
        }
    }

    pub fn finish(mut self) -> Vec<Row> {
        if let Some(row) = self.current.take() {
            self.rows.push(row);
        }
        self.rows
    }
}
