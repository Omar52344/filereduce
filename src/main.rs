

use std::fs::File;
use std::io::{BufReader, BufWriter};
use filereduce::processor::process;

fn main() {
    let input = File::open("input.edi").unwrap();
    let output = File::create("out.jsonl").unwrap();

    process(BufReader::new(input), &mut BufWriter::new(output));
}
