use crate::error::Result;
use crate::parser::edifact::parse_segment_with_registry;
use crate::parser::segment::Segment;
use crate::translations::{ElementConfig, TranslationRegistry};
use std::collections::HashMap;
use std::io::{BufRead, Write};

// Re-export the document structures
pub use crate::model::streaming::{StreamingDocument, StreamingLine};

/// Synchronous EDIFACT processor that writes to any Write implementor
pub struct EdifactProcessor {
    registry: Option<TranslationRegistry>,
}

impl EdifactProcessor {
    pub fn new() -> Self {
        Self {
            registry: TranslationRegistry::new().ok(),
        }
    }

    pub fn with_registry(registry: TranslationRegistry) -> Self {
        Self {
            registry: Some(registry),
        }
    }

    pub fn process_to_writer<R: BufRead, W: Write>(
        &mut self,
        reader: R,
        writer: &mut W,
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
            let segment = parse_segment_with_registry(&raw, self.registry.as_ref());

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
                        serde_json::to_writer(&mut *writer, &doc)?;
                        writer.write_all(b"\n")?;
                    }
                }
                Segment::UNZ => {}
                Segment::Dynamic {
                    code,
                    qualifier,
                    elements: element_groups,
                } => {
                    if let Some(reg) = &self.registry {
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
                    // Log unknown segments (could be captured for telemetry)
                    eprintln!("Unknown segment encountered: {}", code);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Process EDIFACT data and return as a Vec of bytes (JSONL)
    pub fn process_to_vec<R: BufRead>(&mut self, reader: R) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        self.process_to_writer(reader, &mut output)?;
        Ok(output)
    }

    /// Process EDIFACT string and return JSONL string
    pub fn process_to_string(&mut self, input: &str) -> Result<String> {
        let reader = std::io::Cursor::new(input);
        let bytes = self.process_to_vec(reader)?;
        String::from_utf8(bytes).map_err(|e| crate::error::FileReduceError::Parse(e.to_string()))
    }
}

pub(crate) fn apply_dynamic_segment(
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
            registry
                .get_qualifier(segment_code, q)
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
            ElementConfig::Composite {
                label: _,
                components: comp_map,
            } => {
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

    // Special handling for known segment types to populate fixed fields
    match segment_code {
        "BGM" => {
            if let Some(doc) = current_doc.as_mut() {
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
