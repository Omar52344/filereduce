use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
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

#[derive(Default, Serialize, Deserialize)]
pub struct StreamingLine {
    pub line_no: u64,
    pub sku: String,
    pub qty: Option<f64>,
    pub uom: Option<String>,
    pub amount: Option<f64>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}
