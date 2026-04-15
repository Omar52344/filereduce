use crate::error::Result;
use crate::model::streaming::StreamingDocument;
use crate::translations::TranslationRegistry;
use std::collections::HashMap;

pub struct EdifactSerializer {
    registry: TranslationRegistry,
    reverse_map: HashMap<String, (String, Option<String>, String, Option<String>)>, // label -> (segment, qualifier, position, subposition)
}

impl EdifactSerializer {
    pub fn new(registry: TranslationRegistry) -> Self {
        // TODO: build reverse mapping when dynamic reconstruction is needed
        let reverse_map = HashMap::new();
        Self {
            registry,
            reverse_map,
        }
    }

    #[allow(dead_code)]
    fn build_reverse_map(
        registry: &TranslationRegistry,
    ) -> HashMap<String, (String, Option<String>, String, Option<String>)> {
        // TODO: implement when TranslationRegistry exposes iteration
        HashMap::new()
    }

    #[allow(dead_code, unused_variables)]
    fn insert_mapping(
        map: &mut HashMap<String, (String, Option<String>, String, Option<String>)>,
        segment_code: &str,
        qualifier: Option<&str>,
        position: &str,
        elem_config: &crate::translations::ElementConfig,
    ) {
        // TODO: implement when needed
    }

    pub fn serialize_document(&self, doc: &StreamingDocument) -> Result<String> {
        let mut segments = Vec::new();

        // UNB segment
        segments.push(format!(
            "UNB+UNOC:3+{}:14+{}:14+{}:0'",
            doc.sender, doc.receiver, doc.interchange_id
        ));

        // UNH segment
        segments.push("UNH+1'".to_string());

        // BGM segment
        let bgm_msg_name = match doc.doc_type.as_str() {
            "ORDERS" => "220",
            _ => &doc.doc_type,
        };
        segments.push(format!("BGM+{}+{}'", bgm_msg_name, doc.document_number));

        // DTM segments
        if let Some(date) = &doc.document_date {
            segments.push(format!("DTM+137:{}'", date));
        }
        if let Some(delivery_date) = &doc.requested_delivery_date {
            segments.push(format!("DTM+2:{}'", delivery_date));
        }

        // NAD segments
        if let Some(buyer) = &doc.buyer {
            segments.push(format!("NAD+BY+{}'", buyer));
        }
        if let Some(seller) = &doc.seller {
            segments.push(format!("NAD+SU+{}'", seller));
        }

        // CUX segment
        if !doc.currency.is_empty() && doc.currency != "UNKNOWN" {
            segments.push(format!("CUX+2:{}'", doc.currency));
        }

        // Lines
        for line in &doc.lines {
            segments.push(format!("LIN+{}+++{}'", line.line_no, line.sku));
            if let Some(qty) = line.qty {
                let uom = line.uom.as_deref().unwrap_or("");
                let suffix = if uom.is_empty() {
                    String::new()
                } else {
                    format!(":{}", uom)
                };
                segments.push(format!("QTY+1:{}{}'", qty, suffix));
            }
            if let Some(amount) = line.amount {
                segments.push(format!("MOA+1:{}'", amount));
            }
        }

        // CNT segment for line count
        if let Some(cnt) = doc.line_count_check {
            segments.push(format!("CNT+2:{}'", cnt));
        }

        // UNT segment (count of segments from UNH to UNT inclusive)
        let segment_count = segments.len() + 1; // including UNT itself
        segments.push(format!("UNT+{}+1'", segment_count));

        // UNZ segment (interchange trailer)
        segments.push("UNZ+1+0'".to_string());

        Ok(segments.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::streaming::{StreamingDocument, StreamingLine};

    #[test]
    fn test_serialize_basic_document() {
        let registry = TranslationRegistry::new().unwrap();
        let serializer = EdifactSerializer::new(registry);
        let doc = StreamingDocument {
            interchange_id: "12345".to_string(),
            sender: "SENDER".to_string(),
            receiver: "RECEIVER".to_string(),
            doc_type: "ORDERS".to_string(),
            document_number: "ORDER001".to_string(),
            document_date: Some("20240415".to_string()),
            requested_delivery_date: Some("20240420".to_string()),
            currency: "USD".to_string(),
            buyer: Some("BUYER001".to_string()),
            seller: Some("SELLER001".to_string()),
            line_count_check: Some(2),
            lines: vec![
                StreamingLine {
                    line_no: 1,
                    sku: "SKU001".to_string(),
                    qty: Some(10.0),
                    uom: Some("KGM".to_string()),
                    amount: Some(100.0),
                    extra: Default::default(),
                },
                StreamingLine {
                    line_no: 2,
                    sku: "SKU002".to_string(),
                    qty: Some(5.0),
                    uom: Some("PCE".to_string()),
                    amount: Some(50.0),
                    extra: Default::default(),
                },
            ],
            extra: Default::default(),
        };
        let edifact = serializer.serialize_document(&doc).unwrap();
        assert!(edifact.contains("UNB+UNOC:3+SENDER:14+RECEIVER:14+12345:0'"));
        assert!(edifact.contains("BGM+220+ORDER001'"));
        assert!(edifact.contains("DTM+137:20240415'"));
        assert!(edifact.contains("DTM+2:20240420'"));
        assert!(edifact.contains("NAD+BY+BUYER001'"));
        assert!(edifact.contains("NAD+SU+SELLER001'"));
        assert!(edifact.contains("CUX+2:USD'"));
        assert!(edifact.contains("LIN+1+++SKU001'"));
        assert!(edifact.contains("QTY+1:10:KGM'"));
        assert!(edifact.contains("MOA+1:100'"));
        assert!(edifact.contains("LIN+2+++SKU002'"));
        assert!(edifact.contains("QTY+1:5:PCE'"));
        assert!(edifact.contains("MOA+1:50'"));
        assert!(edifact.contains("CNT+2:2'"));
        assert!(edifact.contains("UNT+"));
        assert!(edifact.contains("UNZ+1+0'"));
    }
}
