use criterion::{criterion_group, criterion_main, Criterion};
use filereduce::processor::{process, FileFormat};
use filereduce::sink::file::FileDataSink;
use std::io::BufWriter;

fn bench_process_small_edifact(c: &mut Criterion) {
    let data = include_str!("../tests/fixtures/sample.edifact");
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("process_small_edifact", |b| {
        b.to_async(&rt).iter(|| async {
            let input = data.as_bytes();
            let mut output = Vec::new();
            let mut writer = BufWriter::new(&mut output);
            let mut sink = FileDataSink::new(&mut writer);
            process(input, &mut sink, FileFormat::Edifact, None)
                .await
                .unwrap();
        });
    });
}

criterion_group!(benches, bench_process_small_edifact);
criterion_main!(benches);
