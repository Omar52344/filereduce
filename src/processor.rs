use crate::error::Result;
use crate::parser::edifact::parse_segment;
use crate::parser::segment::Segment;
use engine_filereduce::executor::executor::eval;
use engine_filereduce::query::ast::Expr;
use engine_filereduce::row::{Row, RowKind, Value};
use serde::Serialize;
use std::io::{BufRead, Write};

#[derive(Serialize)]
struct StreamingDocument {
    interchange_id: String,
    sender: String,
    receiver: String,
    doc_type: String,
    document_number: String,
    document_date: Option<String>,
    requested_delivery_date: Option<String>,
    currency: String,
    buyer: Option<String>,
    seller: Option<String>,
    line_count_check: Option<u64>,
    lines: Vec<StreamingLine>,
}

impl Default for StreamingDocument {
    fn default() -> Self {
        Self {
            interchange_id: Default::default(),
            sender: Default::default(),
            receiver: Default::default(),
            doc_type: Default::default(),
            document_number: Default::default(),
            document_date: Default::default(),
            requested_delivery_date: Default::default(),
            currency: "UNKNOWN".to_string(),
            buyer: Default::default(),
            seller: Default::default(),
            line_count_check: Default::default(),
            lines: Default::default(),
        }
    }
}

#[derive(Default, Serialize)]
struct StreamingLine {
    line_no: u64,
    sku: String,
    qty: Option<f64>,
    uom: Option<String>,
    amount: Option<f64>,
}

pub enum FileFormat {
    Edifact,
    Xml,
    Json,
}

pub fn process<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    format: FileFormat,
    query: Option<&Expr>,
) -> Result<()> {
    match format {
        FileFormat::Edifact => process_edifact(reader, writer, query),
        FileFormat::Xml => crate::parser::xml::process_xml(reader, writer, query),
        FileFormat::Json => crate::parser::json::process_json(reader, writer, query),
    }
}

fn process_edifact<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    query: Option<&Expr>,
) -> Result<()> {
    let mut current_doc: Option<StreamingDocument> = None;
    let mut current_line: Option<StreamingLine> = None;
    let mut interchange_id = String::new();
    let mut sender_id = String::new();
    let mut receiver_id = String::new();

    for line in reader.lines() {
        let raw = line?;
        if raw.trim().is_empty() {
            continue;
        }
        let segment = parse_segment(&raw);

        match segment {
            Segment::UNB(s, r, id) => {
                sender_id = s.to_string();
                receiver_id = r.to_string();
                interchange_id = id.to_string();
            }
            Segment::UNH => {
                current_doc = Some(StreamingDocument {
                    interchange_id: interchange_id.clone(),
                    sender: sender_id.clone(),
                    receiver: receiver_id.clone(),
                    ..Default::default()
                });
            }
            Segment::BGM(code, num) => {
                if let Some(doc) = current_doc.as_mut() {
                    doc.document_number = num.to_string();
                    doc.doc_type = match code {
                        "220" => "ORDERS".to_string(),
                        _ => code.to_string(),
                    };
                }
            }
            Segment::DTM(qualifier, date) => {
                if let Some(doc) = current_doc.as_mut() {
                    match qualifier {
                        "137" => doc.document_date = Some(date.to_string()),
                        "2" => doc.requested_delivery_date = Some(date.to_string()),
                        _ => {}
                    }
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
            Segment::LIN(line_num, sku) => {
                if let Some(line) = current_line.take() {
                    if let Some(doc) = current_doc.as_mut() {
                        doc.lines.push(line);
                    }
                }
                current_line = Some(StreamingLine {
                    line_no: line_num.parse().unwrap_or(0),
                    sku: sku.to_string(),
                    ..Default::default()
                });
            }
            Segment::QTY(_, qty, unit) => {
                if let Some(line) = current_line.as_mut() {
                    line.qty = qty.parse().ok();
                    line.uom = if !unit.is_empty() {
                        Some(unit.to_string())
                    } else {
                        None
                    };
                }
            }
            Segment::MOA(_, amt) => {
                if let Some(line) = current_line.as_mut() {
                    line.amount = amt.parse().ok();
                }
            }
            Segment::CNT(code, val) => {
                if code == "2" {
                    if let Some(doc) = current_doc.as_mut() {
                        doc.line_count_check = val.parse().ok();
                    }
                }
            }
            Segment::CUX(curr) => {
                if let Some(doc) = current_doc.as_mut() {
                    doc.currency = curr.to_string();
                }
            }
            Segment::UNT => {
                if let Some(line) = current_line.take() {
                    if let Some(doc) = current_doc.as_mut() {
                        doc.lines.push(line);
                    }
                }

                if let Some(doc) = current_doc.take() {
                    let should_write = if let Some(expr) = query {
                        let mut matched = false;
                        let mut base_row = Row::new(RowKind::UNH);
                        base_row.insert("number", Value::Text(doc.document_number.clone()));
                        base_row.insert("doc_type", Value::Text(doc.doc_type.clone()));
                        base_row.insert("interchange_id", Value::Text(doc.interchange_id.clone()));
                        base_row.insert("sender", Value::Text(doc.sender.clone()));
                        if let Some(val) = &doc.document_date {
                            base_row.insert("date", Value::Text(val.clone()));
                        }
                        if let Some(val) = &doc.buyer {
                            base_row.insert("buyer", Value::Text(val.clone()));
                        }
                        if let Some(val) = &doc.seller {
                            base_row.insert("seller", Value::Text(val.clone()));
                        }

                        if doc.lines.is_empty() {
                            if eval(expr, &base_row) {
                                matched = true;
                            }
                        } else {
                            for line in &doc.lines {
                                let mut row = base_row.clone();
                                row.kind = RowKind::LIN;
                                row.insert("sku", Value::Text(line.sku.clone()));
                                if let Some(q) = line.qty {
                                    row.insert("qty", Value::Number(q));
                                }
                                if let Some(a) = line.amount {
                                    row.insert("amount", Value::Number(a));
                                }
                                if eval(expr, &row) {
                                    matched = true;
                                    break;
                                }
                            }
                        }
                        matched
                    } else {
                        true
                    };

                    if should_write {
                        let json = serde_json::to_string(&doc)?;
                        writeln!(writer, "{}", json)?;
                    }
                }
            }
            Segment::UNZ => {}
            _ => {}
        }
    }

    Ok(())
}
