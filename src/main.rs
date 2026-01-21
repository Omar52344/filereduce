//use engine_filereduce::reader::edi::parse_segment;

use engine_filereduce::executor::executor::eval;
use engine_filereduce::executor::executor::Executor;
use engine_filereduce::query::ast::Expr;
use engine_filereduce::query::row_builder::RowBuilder;
use engine_filereduce::query::segment::Segment;
use engine_filereduce::row::Row;
use engine_filereduce::row::RowKind;
use engine_filereduce::row::Value;
use std::collections::HashMap;
//use engine_filereduce::run_edi_query;
use filereduce::processor::process;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() {
    let input = File::open("input.edi").unwrap();
    let output = File::create("out.jsonl").unwrap();

    process(BufReader::new(input), &mut BufWriter::new(output));
}

/*#[test]
fn test_edi_query() {
    let rows = run_edi_query("test.edi", "SELECT segment, value LIMIT 5");

    assert!(!rows.is_empty());
}*/

/*#[test]
fn test_edi_row_model() {
    let row = parse_segment("LIN+1++ABC123:IN'", 1);

    assert_eq!(row.get("segment"), Some(&Value::Text("LIN".into())));
    assert_eq!(row.get("line"), Some(&Value::Number(1.0)));
}*/

fn seg(tag: &str, elems: Vec<&str>) -> Segment {
    Segment {
        tag: tag.into(),
        elements: elems.into_iter().map(String::from).collect(),
    }
}

/*#[test]
fn builds_lin_rows() {
    let mut b = RowBuilder::new();

    b.consume(&seg("LIN", vec!["1", "ITEM1"]));
    b.consume(&seg("QTY", vec!["47", "10"]));
    b.consume(&seg("MOA", vec!["203", "200"]));
    b.consume(&seg("LIN", vec!["2", "ITEM2"]));

    let rows = b.finish();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].kind, RowKind::LIN);
}*/

#[test]
fn where_kind_and_qty() {
    let mut fields = HashMap::new();
    fields.insert("qty".into(), Value::Number(10.0));

    let row = Row {
        kind: RowKind::LIN,
        fields,
    };

    let expr = Expr::And(
        Box::new(Expr::KindEq(RowKind::LIN)),
        Box::new(Expr::Gt("qty".into(), Value::Number(5.0))),
    );

    assert!(eval(&expr, &row));
}
