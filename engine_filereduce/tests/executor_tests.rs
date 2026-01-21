use engine_filereduce::executor::executor::Executor;
use engine_filereduce::query::ast::Query;
use engine_filereduce::reader::reader::MockReader;
use engine_filereduce::row::{Row, RowKind, Value};

#[test]
fn test_limit_functionality() {
    let mut rows = Vec::new();
    for i in 1..=10 {
        let mut row = Row::new(RowKind::LIN);
        row.insert("sku", Value::Text(format!("ITEM{}", i)));
        row.insert("qty", Value::Number(i as f64));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: Some(5),
        order_by: None,
        aggregates: vec![],
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);

    let results = executor.collect();

    assert_eq!(results.len(), 5, "LIMIT should return only 5 rows");
}

#[test]
fn test_select_projection() {
    let mut row = Row::new(RowKind::LIN);
    row.insert("sku", Value::Text("ITEM1".into()));
    row.insert("qty", Value::Number(10.0));
    row.insert("price", Value::Number(99.99));

    let query = Query {
        select: vec!["sku".to_string(), "qty".to_string()],
        filter: None,
        limit: None,
        order_by: None,
        aggregates: vec![],
    };

    let reader = MockReader::new(vec![row]);
    let mut executor = Executor::new(reader, query);

    let projected = executor.next().unwrap();

    assert!(
        projected.get("sku").is_some(),
        "sku should be in projection"
    );
    assert!(
        projected.get("qty").is_some(),
        "qty should be in projection"
    );
    assert!(
        projected.get("price").is_none(),
        "price should NOT be in projection"
    );
}
