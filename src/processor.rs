use crate::error::Result;
use crate::parser::edifact::parse_segment_with_registry;
use crate::parser::segment::Segment;
use crate::sink::{DataSink, SinkItem};
use crate::translations::{ElementConfig, TranslationRegistry};
use tracing;
use engine_filereduce::executor::executor::eval;
use engine_filereduce::query::ast::Expr;
use engine_filereduce::row::{Row, RowKind, Value};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{BufRead, Write};

#[derive(Serialize)]
pub struct StreamingDocument {
    pub interchange_id: String,
    pub sender: String,
    pub receiver: String,
    pub doc_type: String,
    pub document_number: String,
    pub document_date: Option<String>,
    pub requested_delivery_date: Option<String>,
    pub currency: String,
    pub buyer: Option<String>,
    pub seller: Option<String>,
    pub line_count_check: Option<u64>,
    pub lines: Vec<StreamingLine>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
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
            extra: Default::default(),
        }
    }
}

#[derive(Default, Serialize)]
pub struct StreamingLine {
    pub line_no: u64,
    pub sku: String,
    pub qty: Option<f64>,
    pub uom: Option<String>,
    pub amount: Option<f64>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

pub enum FileFormat {
    Edifact,
    Xml,
    Json,
}

pub async fn process<R: BufRead + Send>(
    reader: R,
    sink: &mut dyn DataSink,
    format: FileFormat,
    query: Option<&Expr>,
) -> Result<()> {
    match format {
        FileFormat::Edifact => process_edifact(reader, sink, query).await,
        FileFormat::Xml => Err(crate::error::FileReduceError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "XML not yet supported with async Sink",
        ))),
        FileFormat::Json => Err(crate::error::FileReduceError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "JSON not yet supported with async Sink",
        ))),
    }
}

fn apply_dynamic_segment(
    segment_code: &str,
    qualifier: Option<&str>,
    element_groups: &[Vec<&str>],
    registry: &TranslationRegistry,
    current_doc: &mut Option<StreamingDocument>,
    current_line: &mut Option<StreamingLine>,
) {
    // Determine if segment affects document or line
    let is_line_segment = matches!(segment_code, "LIN" | "QTY" | "MOA");
    
    let Some(segment_config) = registry.get_segment(segment_code) else {
        return;
    };
    
    let mut field_values = HashMap::new();
    
    // Get appropriate elements configuration based on qualifier
    let elements_config = if segment_config.use_qualifier {
        if let Some(q) = qualifier {
            // Try to get qualifier-specific elements
            registry.get_qualifier(segment_code, q)
                .map(|sub| sub.elements)
                .unwrap_or_else(|| segment_config.elements.clone())
        } else {
            segment_config.elements.clone()
        }
    } else {
        segment_config.elements.clone()
    };
    
    // Build map from position to component group
    let mut pos_to_group = std::collections::HashMap::new();
    for (idx, group) in element_groups.iter().enumerate() {
        let pos = idx + 1;
        pos_to_group.insert(pos, group);
    }
    
    for (pos_str, config) in elements_config.iter() {
        let pos_num: usize = pos_str.parse().unwrap_or(0);
        if pos_num == 0 {
            continue;
        }
        let Some(components) = pos_to_group.get(&pos_num) else {
            continue;
        };
        match config {
            ElementConfig::Simple(label) => {
                // Take first component
                if let Some(&value) = components.first() {
                    field_values.insert(label.clone(), value.to_string());
                }
            }
            ElementConfig::Composite { label: _, components: comp_map } => {
                // Map each subcomponent according to comp_map
                for (sub_pos_str, sub_label) in comp_map.iter() {
                    let sub_pos: usize = sub_pos_str.parse().unwrap_or(0);
                    if sub_pos == 0 || sub_pos > components.len() {
                        continue;
                    }
                    let value = components[sub_pos - 1];
                    field_values.insert(sub_label.clone(), value.to_string());
                }
            }
        }
    }
    
    // Apply field values to appropriate target
    // Special handling for known segment types to populate fixed fields
    match segment_code {
        "BGM" => {
            if let Some(doc) = current_doc.as_mut() {
                // Try to get document_number from field values
                if let Some(num) = field_values.get("DocumentNumber") {
                    doc.document_number = num.clone();
                }
                if let Some(msg) = field_values.get("MessageName") {
                    doc.doc_type = match msg.as_str() {
                        "220" => "ORDERS".to_string(),
                        _ => msg.clone(),
                    };
                }
            }
        }
        "DTM" => {
            if let Some(doc) = current_doc.as_mut() {
                if let Some(qual) = qualifier {
                    match qual {
                        "137" => {
                            if let Some(date) = field_values.get("Value") {
                                doc.document_date = Some(date.clone());
                            }
                        }
                        "2" => {
                            if let Some(date) = field_values.get("Value") {
                                doc.requested_delivery_date = Some(date.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        "NAD" => {
            if let Some(doc) = current_doc.as_mut() {
                if let Some(qual) = qualifier {
                    match qual {
                        "BY" => {
                            if let Some(id) = field_values.get("PartyId") {
                                doc.buyer = Some(id.clone());
                            }
                        }
                        "SU" => {
                            if let Some(id) = field_values.get("PartyId") {
                                doc.seller = Some(id.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        "LIN" => {
            if let Some(line) = current_line.as_mut() {
                if let Some(line_no) = field_values.get("LineNumber") {
                    line.line_no = line_no.parse().unwrap_or(0);
                }
                if let Some(sku) = field_values.get("ProductId") {
                    line.sku = sku.clone();
                }
            }
        }
        "QTY" => {
            if let Some(line) = current_line.as_mut() {
                if let Some(qty) = field_values.get("Value") {
                    line.qty = qty.parse().ok();
                }
                if let Some(uom) = field_values.get("Format") {
                    line.uom = Some(uom.clone());
                }
            }
        }
        "MOA" => {
            if let Some(line) = current_line.as_mut() {
                if let Some(amt) = field_values.get("Value") {
                    line.amount = amt.parse().ok();
                }
            }
        }
        "CNT" => {
            if let Some(doc) = current_doc.as_mut() {
                if let Some(code) = qualifier {
                    if code == "2" {
                        if let Some(val) = field_values.get("Value") {
                            doc.line_count_check = val.parse().ok();
                        }
                    }
                }
            }
        }
        "CUX" => {
            if let Some(doc) = current_doc.as_mut() {
                if let Some(curr) = field_values.get("Currency") {
                    doc.currency = curr.clone();
                }
            }
        }
        _ => {}
    }
    
    // Apply extra fields
    if is_line_segment {
        if let Some(line) = current_line.as_mut() {
            for (key, val) in &field_values {
                line.extra.insert(key.clone(), val.clone());
            }
        }
    } else if let Some(doc) = current_doc.as_mut() {
        for (key, val) in &field_values {
            doc.extra.insert(key.clone(), val.clone());
        }
    }
}

async fn process_edifact<R: BufRead>(
    reader: R,
    sink: &mut dyn DataSink,
    query: Option<&Expr>,
) -> Result<()> {
    let mut current_doc: Option<StreamingDocument> = None;
    let mut current_line: Option<StreamingLine> = None;
    let mut interchange_id = String::new();
    let mut sender_id = String::new();
    let mut receiver_id = String::new();
    let registry = crate::translations::TranslationRegistry::new().ok();

    for line in reader.lines() {
        let raw = line?;
        if raw.trim().is_empty() {
            continue;
        }
        let segment = parse_segment_with_registry(&raw, registry.as_ref());

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
                        sink.send(SinkItem::Document(doc)).await?;
                    }
                }
            }
            Segment::UNZ => {}
            Segment::Dynamic { code, qualifier, elements: element_groups } => {
                if let Some(reg) = &registry {
                    apply_dynamic_segment(
                        code,
                        qualifier,
                        &element_groups,
                        reg,
                        &mut current_doc,
                        &mut current_line,
                    );
                }
            }
            Segment::Unknown(code) => {
                tracing::warn!("Unknown segment encountered: {}", code);
            }
            _ => {}
        }
    }

    Ok(())
}
