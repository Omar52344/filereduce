use criterion::{criterion_group, criterion_main, Criterion};
use engine_filereduce::executor::executor::Executor;
use engine_filereduce::query::ast::{Expr, Query};
use engine_filereduce::reader::reader::MockReader;
use engine_filereduce::row::{Row, RowKind, Value};

fn bench_query_simple(c: &mut Criterion) {
    let mut rows = Vec::new();
    for i in 0..1000 {
        let mut row = Row::new(RowKind::LIN);
        row.insert("id", Value::Number(i as f64));
        row.insert("qty", Value::Number((i % 100 + 1) as f64));
        rows.push(row);
    }

    c.bench_function("query_simple", |b| {
        let reader = MockReader::new(rows.clone());
        let query = Query {
            select: vec![],
            filter: None,
            limit: None,
            order_by: None,
            aggregates: vec![],
        };
        b.iter(|| {
            let mut executor = Executor::new(reader.clone(), query.clone());
            let _results = executor.collect();
        });
    });
}

fn bench_query_with_filter(c: &mut Criterion) {
    let mut rows = Vec::new();
    for i in 0..1000 {
        let mut row = Row::new(RowKind::LIN);
        row.insert("id", Value::Number(i as f64));
        row.insert("qty", Value::Number((i % 100 + 1) as f64));
        rows.push(row);
    }

    c.bench_function("query_with_filter", |b| {
        let reader = MockReader::new(rows.clone());
        let query = Query {
            select: vec![],
            filter: Some(Expr::Gt("qty".to_string(), Value::Number(50.0))),
            limit: None,
            order_by: None,
            aggregates: vec![],
        };
        b.iter(|| {
            let mut executor = Executor::new(reader.clone(), query.clone());
            let _results = executor.collect();
        });
    });
}

criterion_group!(benches, bench_query_simple, bench_query_with_filter);
criterion_main!(benches);
