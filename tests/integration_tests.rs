use filereduce::processor::{process, FileFormat};
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[test]
fn test_edifact_processing() {
    let input = File::open("tests/fixtures/sample.edifact").expect("Failed to open EDI file");
    let mut output = Vec::new();
    {
        let mut writer = BufWriter::new(&mut output);
        process(
            BufReader::new(input),
            &mut writer,
            FileFormat::Edifact,
            None,
        )
        .expect("Failed to process EDI file");
    }

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let lines: Vec<&str> = output_str.lines().collect();

    assert_eq!(lines.len(), 2, "Should process 2 documents");

    let first_doc: serde_json::Value =
        serde_json::from_str(lines[0]).expect("Failed to parse first document as JSON");

    assert_eq!(
        first_doc["number"], "ORDER001",
        "First document number should be ORDER001"
    );

    assert_eq!(
        first_doc["buyer"], "BUYER001",
        "First document buyer should be BUYER001"
    );

    assert_eq!(
        first_doc["seller"], "SELLER001",
        "First document seller should be SELLER001"
    );

    let lines = first_doc["lines"]
        .as_array()
        .expect("Lines should be an array");
    assert_eq!(lines.len(), 3, "First document should have 3 lines");
    assert_eq!(lines[0]["sku"], "SKU001", "First line SKU should be SKU001");
    assert_eq!(lines[0]["qty"], 10.0, "First line qty should be 10");
}

#[test]
fn test_xml_processing() {
    let input = File::open("tests/fixtures/sample.xml").expect("Failed to open XML file");
    let mut output = Vec::new();
    {
        let mut writer = BufWriter::new(&mut output);
        process(BufReader::new(input), &mut writer, FileFormat::Xml, None)
            .expect("Failed to process XML file");
    }

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let lines: Vec<&str> = output_str.lines().collect();

    assert_eq!(lines.len(), 7, "Should process 7 records");

    let first_record: serde_json::Value =
        serde_json::from_str(lines[0]).expect("Failed to parse first record as JSON");

    assert_eq!(
        first_record["number"], "ORDER001",
        "First record number should be ORDER001"
    );

    assert_eq!(
        first_record["buyer"], "BUYER001",
        "First record buyer should be BUYER001"
    );
}

#[test]
fn test_jsonl_processing() {
    let input = File::open("tests/fixtures/sample.jsonl").expect("Failed to open JSONL file");
    let mut output = Vec::new();
    {
        let mut writer = BufWriter::new(&mut output);
        process(BufReader::new(input), &mut writer, FileFormat::Json, None)
            .expect("Failed to process JSONL file");
    }

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let lines: Vec<&str> = output_str.lines().collect();

    assert_eq!(lines.len(), 6, "Should process 6 JSON lines");

    let first_line: serde_json::Value =
        serde_json::from_str(lines[0]).expect("Failed to parse first line as JSON");

    assert_eq!(
        first_line["number"], "ORDER001",
        "First line number should be ORDER001"
    );

    assert_eq!(
        first_line["buyer"], "BUYER001",
        "First line buyer should be BUYER001"
    );

    assert_eq!(
        first_line["sku"], "SKU001",
        "First line SKU should be SKU001"
    );
}

#[test]
fn test_json_processing() {
    let input = File::open("tests/fixtures/sample.json").expect("Failed to open JSON file");
    let mut output = Vec::new();
    let result = process(BufReader::new(input), &mut output, FileFormat::Json, None);

    assert!(
        result.is_err(),
        "Processing JSON array should fail - only JSONL is supported"
    );
}

#[test]
fn test_empty_files_handling() {
    let temp_json = "tests/fixtures/empty.jsonl";
    File::create(temp_json).expect("Failed to create empty file");

    let input = File::open(temp_json).expect("Failed to open empty file");
    let mut output = Vec::new();
    {
        let mut writer = BufWriter::new(&mut output);
        process(BufReader::new(input), &mut writer, FileFormat::Json, None)
            .expect("Failed to process empty file");
    }

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    assert!(
        output_str.lines().count() == 0 || output_str.is_empty(),
        "Empty file should produce no output"
    );

    std::fs::remove_file(temp_json).ok();
}
