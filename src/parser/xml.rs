use crate::error::Result;
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use std::collections::HashMap;
use std::io::{BufRead, Write};

#[derive(Debug)]
pub struct XmlRecord {
    pub fields: HashMap<String, String>,
}

pub fn process_xml<R: BufRead, W: Write>(reader: R, writer: &mut W) -> Result<()> {
    let mut xml_reader = XmlReader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);

    let mut current_record: Option<XmlRecord> = None;
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();
    let mut buf = Vec::new();

    loop {
        let event = xml_reader.read_event_into(&mut buf)?;

        match event {
            Event::Start(ref e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if tag_name == "record" || tag_name == "item" || tag_name == "row" {
                    current_record = Some(XmlRecord {
                        fields: HashMap::new(),
                    });
                } else {
                    current_tag = Some(tag_name);
                    current_text.clear();
                }
            }

            Event::Text(e) => {
                if current_tag.is_some() {
                    current_text.push_str(&e.unescape()?);
                }
            }

            Event::End(ref e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if tag_name == "record" || tag_name == "item" || tag_name == "row" {
                    if let Some(record) = current_record.take() {
                        let json = serde_json::to_string(&record.fields)?;
                        writeln!(writer, "{}", json)?;
                    }
                } else if current_tag.as_ref() == Some(&tag_name) {
                    if let Some(ref mut record) = current_record {
                        if let Some(tag) = current_tag.take() {
                            record.fields.insert(tag, current_text.clone());
                        }
                    }
                }
            }

            Event::Eof => break,

            _ => {}
        }

        buf.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_xml_simple() {
        let xml_input = r#"<records>
            <record>
                <sku>ITEM1</sku>
                <qty>10</qty>
            </record>
            <record>
                <sku>ITEM2</sku>
                <qty>20</qty>
            </record>
        </records>"#;

        let mut output = Vec::new();
        let result = process_xml(xml_input.as_bytes(), &mut output);

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.lines().collect();
        assert_eq!(lines.len(), 2);
    }
}
