use engine_filereduce::run_edi_query;
use filereduce::processor::process;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() {
    let input = File::open("input.edi").unwrap();
    let output = File::create("out.jsonl").unwrap();

    process(BufReader::new(input), &mut BufWriter::new(output));
}

#[test]
fn test_edi_query() {
    let rows = run_edi_query("test.edi", "SELECT segment, value LIMIT 5");

    assert!(!rows.is_empty());
}
