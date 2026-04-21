use filereduce::core::EdifactProcessor;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn test_parse_generated_edifact() {
    println!("Test starting");
    let file = File::open("test_sample.edi").unwrap();
    let reader = BufReader::new(file);
    let mut processor = EdifactProcessor::new();
    let result = processor.process_to_vec(reader);
    // Should not panic
    assert!(result.is_ok());
    let jsonl_bytes = result.unwrap();
    let jsonl_str = String::from_utf8(jsonl_bytes).unwrap();
    // Should have at least one line
    assert!(jsonl_str.lines().count() > 0);
    // Should have at least one line
    assert!(jsonl_str.lines().count() > 0);
    // Check that price amounts are present (from PRI segments)
    let has_amount = jsonl_str.lines().any(|line| line.contains("\"amount\":"));
    assert!(has_amount, "No amount field found in JSONL output");
    println!("Parsed {} lines", jsonl_str.lines().count());
}
