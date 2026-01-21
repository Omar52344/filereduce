use engine_filereduce::executor::executor::Executor;
use engine_filereduce::query::ast::{Query, SortOrder};
use engine_filereduce::reader::reader::MockReader;
use engine_filereduce::row::{Row, RowKind, Value};

#[test]
fn test_order_by_asc() {
    let mut rows = Vec::new();
    for i in [5, 2, 8, 1, 9] {
        let mut row = Row::new(RowKind::LIN);
        row.insert("sku", Value::Text(format!("ITEM{}", i)));
        row.insert("qty", Value::Number(i as f64));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: None,
        order_by: Some(("qty".to_string(), SortOrder::Asc)),
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);
    let results = executor.collect();

    assert_eq!(results.len(), 5);
    assert_eq!(results[0].get("qty"), Some(&Value::Number(1.0)));
    assert_eq!(results[1].get("qty"), Some(&Value::Number(2.0)));
    assert_eq!(results[2].get("qty"), Some(&Value::Number(5.0)));
    assert_eq!(results[3].get("qty"), Some(&Value::Number(8.0)));
    assert_eq!(results[4].get("qty"), Some(&Value::Number(9.0)));
}

#[test]
fn test_order_by_desc() {
    let mut rows = Vec::new();
    for i in [5, 2, 8, 1, 9] {
        let mut row = Row::new(RowKind::LIN);
        row.insert("sku", Value::Text(format!("ITEM{}", i)));
        row.insert("qty", Value::Number(i as f64));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: None,
        order_by: Some(("qty".to_string(), SortOrder::Desc)),
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);
    let results = executor.collect();

    assert_eq!(results.len(), 5);
    assert_eq!(results[0].get("qty"), Some(&Value::Number(9.0)));
    assert_eq!(results[1].get("qty"), Some(&Value::Number(8.0)));
    assert_eq!(results[2].get("qty"), Some(&Value::Number(5.0)));
    assert_eq!(results[3].get("qty"), Some(&Value::Number(2.0)));
    assert_eq!(results[4].get("qty"), Some(&Value::Number(1.0)));
}

#[test]
fn test_order_by_with_limit() {
    let mut rows = Vec::new();
    for i in 1..=10 {
        let mut row = Row::new(RowKind::LIN);
        row.insert("qty", Value::Number(i as f64));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: Some(3),
        order_by: Some(("qty".to_string(), SortOrder::Desc)),
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);
    let results = executor.collect();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].get("qty"), Some(&Value::Number(10.0)));
    assert_eq!(results[1].get("qty"), Some(&Value::Number(9.0)));
    assert_eq!(results[2].get("qty"), Some(&Value::Number(8.0)));
}
