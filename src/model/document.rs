use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Document {
    pub doc_type: String,
    pub number: String,
    pub date: Option<String>,
    pub buyer: Option<String>,
    pub seller: Option<String>,
    pub lines: Vec<Line>,
    pub total: Option<f64>,
}

#[derive(Default, Serialize)]
pub struct Line {
    pub sku: String,
    pub qty: Option<f64>,
    pub amount: Option<f64>,
}
