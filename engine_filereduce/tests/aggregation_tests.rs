use engine_filereduce::executor::executor::Executor;
use engine_filereduce::query::aggregation::execute_aggregates;
use engine_filereduce::query::ast::{Aggregate, Query};
use engine_filereduce::reader::reader::MockReader;
use engine_filereduce::row::{Row, RowKind, Value};

#[test]
fn test_count_aggregation() {
    let mut rows = Vec::new();
    for i in 1..=5 {
        let mut row = Row::new(RowKind::LIN);
        row.insert("qty", Value::Number(i as f64));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: None,
        order_by: None,
        aggregates: vec![Aggregate::Count("qty".to_string())],
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);
    let result = executor.collect_with_aggregates();

    assert_eq!(result.count, Some(5));
}

#[test]
fn test_sum_aggregation() {
    let mut rows = Vec::new();
    for i in 1..=5 {
        let mut row = Row::new(RowKind::LIN);
        row.insert("qty", Value::Number(i as f64));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: None,
        order_by: None,
        aggregates: vec![Aggregate::Sum("qty".to_string())],
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);
    let result = executor.collect_with_aggregates();

    assert_eq!(result.sum, Some(15.0));
}

#[test]
fn test_avg_aggregation() {
    let mut rows = Vec::new();
    for i in [10.0, 20.0, 30.0] {
        let mut row = Row::new(RowKind::LIN);
        row.insert("price", Value::Number(i));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: None,
        order_by: None,
        aggregates: vec![Aggregate::Avg("price".to_string())],
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);
    let result = executor.collect_with_aggregates();

    assert_eq!(result.avg, Some(20.0));
}

#[test]
fn test_min_max_aggregation() {
    let mut rows = Vec::new();
    for i in [10.0, 50.0, 30.0, 70.0, 20.0] {
        let mut row = Row::new(RowKind::LIN);
        row.insert("qty", Value::Number(i));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: None,
        order_by: None,
        aggregates: vec![
            Aggregate::Min("qty".to_string()),
            Aggregate::Max("qty".to_string()),
        ],
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);
    let result = executor.collect_with_aggregates();

    assert_eq!(result.min, Some(10.0));
    assert_eq!(result.max, Some(70.0));
}

#[test]
fn test_multiple_aggregations() {
    let mut rows = Vec::new();
    for i in 1..=5 {
        let mut row = Row::new(RowKind::LIN);
        row.insert("qty", Value::Number(i as f64));
        row.insert("price", Value::Number((i * 10) as f64));
        rows.push(row);
    }

    let query = Query {
        select: vec![],
        filter: None,
        limit: None,
        order_by: None,
        aggregates: vec![
            Aggregate::Count("qty".to_string()),
            Aggregate::Sum("qty".to_string()),
            Aggregate::Avg("price".to_string()),
        ],
    };

    let reader = MockReader::new(rows);
    let executor = Executor::new(reader, query);
    let result = executor.collect_with_aggregates();

    assert_eq!(result.count, Some(5));
    assert_eq!(result.sum, Some(15.0));
    assert_eq!(result.avg, Some(30.0));
}
