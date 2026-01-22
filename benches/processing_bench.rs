use criterion::{criterion_group, criterion_main, Criterion};
use filereduce::processor::{process, FileFormat};
use std::io::{BufRead, BufWriter};

fn bench_process_small_jsonl(c: &mut Criterion) {
    let data = include_str!("../tests/fixtures/sample.jsonl");
    c.bench_function("process_small_jsonl", |b| {
        let mut input = data.as_bytes();
        let mut output = Vec::new();
        b.iter(|| {
            output.clear();
            let mut writer = BufWriter::new(&mut output);
            process(&mut input, &mut writer, FileFormat::Json, None).unwrap();
        });
    });
}

criterion_group!(benches, bench_process_small_jsonl);
criterion_main!(benches);
