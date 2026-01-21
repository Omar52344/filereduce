use crate::error::Result;
use crate::parser::edifact::parse_segment;
use crate::parser::segment::Segment;
use serde::Serialize;
use std::io::{BufRead, Write};

#[derive(Default, Serialize)]
struct StreamingDocument {
    doc_type: String,
    number: String,
    buyer: Option<String>,
    seller: Option<String>,
    lines: Vec<StreamingLine>,
}

#[derive(Default, Serialize)]
struct StreamingLine {
    sku: String,
    qty: Option<f64>,
    amount: Option<f64>,
}

pub enum FileFormat {
    Edifact,
    Xml,
    Json,
}

pub fn process<R: BufRead, W: Write>(reader: R, writer: &mut W, format: FileFormat) -> Result<()> {
    match format {
        FileFormat::Edifact => process_edifact(reader, writer),
        FileFormat::Xml => crate::parser::xml::process_xml(reader, writer),
        FileFormat::Json => crate::parser::json::process_json(reader, writer),
    }
}

fn process_edifact<R: BufRead, W: Write>(reader: R, writer: &mut W) -> Result<()> {
    let mut current_doc: Option<StreamingDocument> = None;
    let mut current_line: Option<StreamingLine> = None;

    for line in reader.lines() {
        let raw = line?;
        let segment = parse_segment(&raw);

        match segment {
            Segment::UNH => {
                current_doc = Some(StreamingDocument::default());
            }
            Segment::BGM(num) => {
                if let Some(doc) = current_doc.as_mut() {
                    doc.number = num.to_string();
                }
            }
            Segment::NAD("BY", id) => {
                if let Some(doc) = current_doc.as_mut() {
                    doc.buyer = Some(id.to_string());
                }
            }
            Segment::NAD("SU", id) => {
                if let Some(doc) = current_doc.as_mut() {
                    doc.seller = Some(id.to_string());
                }
            }
            Segment::LIN(sku) => {
                if let Some(line) = current_line.take() {
                    if let Some(doc) = current_doc.as_mut() {
                        doc.lines.push(line);
                    }
                }
                current_line = Some(StreamingLine {
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
                    if let Some(doc) = current_doc.as_mut() {
                        doc.lines.push(line);
                    }
                }

                if let Some(doc) = current_doc.take() {
                    let json = serde_json::to_string(&doc)?;
                    writeln!(writer, "{}", json)?;
                }
            }
            Segment::UNZ => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
