use std::io::{BufRead};
use crate::parser::edifact::parse_segment;
use crate::model::document::{Document, Line};
use crate::parser::segment::Segment;
pub fn process<R: BufRead>(reader: R, writer: &mut dyn std::io::Write) {
    let mut current_doc: Option<Document> = None;
    let mut current_line: Option<Line> = None;

    for line in reader.lines() {
        let raw = line.unwrap();
        let segment = parse_segment(&raw);

        match segment {
            Segment::UNH => {
                current_doc = Some(Document::default());
            }
            Segment::BGM(num) => {
                if let Some(doc) = current_doc.as_mut() {
                    doc.number = num.to_string();
                }
            }
            Segment::NAD("BY", id) => {
                current_doc.as_mut().unwrap().buyer = Some(id.to_string());
            }
            Segment::NAD("SU", id) => {
                current_doc.as_mut().unwrap().seller = Some(id.to_string());
            }
            Segment::LIN(sku) => {
                current_line = Some(Line {
                    sku: sku.to_string(),
                    ..Default::default()
                });
            }
            Segment::QTY(_, qty) => {
                if let Some(line) = current_line.as_mut() {
                    line.qty = qty.parse().ok();
                }
            }
            Segment::MOA(_, amt) => {
                if let Some(line) = current_line.as_mut() {
                    line.amount = amt.parse().ok();
                }
            }
            Segment::UNT => {
                if let Some(line) = current_line.take() {
                    current_doc.as_mut().unwrap().lines.push(line);
                }

                let json = serde_json::to_string(&current_doc.take().unwrap()).unwrap();
                writeln!(writer, "{}", json).unwrap();
            }
            _ => {}
        }
    }
}
