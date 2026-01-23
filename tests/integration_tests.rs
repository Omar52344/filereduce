use filereduce::processor::{process, FileFormat};
use filereduce::sink::file::FileDataSink;
use filereduce::sink::DataSink;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[tokio::test]
async fn test_edifact_processing() {
    let input = File::open("tests/fixtures/sample.edifact").expect("Failed to open EDI file");
    let mut output = Vec::new();
    {
        let mut writer = BufWriter::new(&mut output);
        let mut sink = FileDataSink::new(&mut writer);
        process(BufReader::new(input), &mut sink, FileFormat::Edifact, None)
            .await
            .expect("Failed to process EDI file");
        sink.flush().await.expect("Failed to flush");
    }

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");
    let lines: Vec<&str> = output_str.lines().collect();

    assert_eq!(lines.len(), 2, "Should process 2 documents");

    let first_doc: serde_json::Value =
        serde_json::from_str(lines[0]).expect("Failed to parse first document as JSON");

    assert_eq!(
        first_doc["document_number"], "ORDER001",
        "First document number should be ORDER001"
    );
    // Note: field names changed in processor.rs (buyer -> buyer, etc preserved but number -> document_number)
}

#[tokio::test]
async fn test_xml_processing() {
    let input = File::open("tests/fixtures/sample.xml").expect("Failed to open XML file");
    let mut output = Vec::new();
    let mut writer = BufWriter::new(&mut output);
    let mut sink = FileDataSink::new(&mut writer);

    let result = process(BufReader::new(input), &mut sink, FileFormat::Xml, None).await;

    // In current async implementation, XML is explicitly not supported yet.
    assert!(result.is_err(), "Xml is not yet supported in async mode");
}

#[tokio::test]
async fn test_jsonl_processing() {
    let input = File::open("tests/fixtures/sample.jsonl").expect("Failed to open JSONL file");
    let mut output = Vec::new();
    let mut writer = BufWriter::new(&mut output);
    let mut sink = FileDataSink::new(&mut writer);

    let result = process(BufReader::new(input), &mut sink, FileFormat::Json, None).await;

    // In current async implementation, JSON is explicitly not supported yet.
    assert!(result.is_err(), "Json is not yet supported in async mode");
}

#[tokio::test]
async fn test_json_processing() {
    let input = File::open("tests/fixtures/sample.json").expect("Failed to open JSON file");
    let mut output = Vec::new();
    let mut writer = BufWriter::new(&mut output);
    let mut sink = FileDataSink::new(&mut writer);

    let result = process(BufReader::new(input), &mut sink, FileFormat::Json, None).await;

    assert!(result.is_err(), "Processing JSON should fail");
}

#[tokio::test]
async fn test_empty_files_handling() {
    let temp_json = "tests/fixtures/empty.jsonl";
    File::create(temp_json).expect("Failed to create empty file");

    let input = File::open(temp_json).expect("Failed to open empty file");
    let mut output = Vec::new();
    let mut writer = BufWriter::new(&mut output);
    let mut sink = FileDataSink::new(&mut writer);

    let result = process(BufReader::new(input), &mut sink, FileFormat::Json, None).await;
    // Json/Jsonl returns error now
    assert!(result.is_err());

    std::fs::remove_file(temp_json).ok();
}
