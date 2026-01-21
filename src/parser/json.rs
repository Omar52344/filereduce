use crate::error::Result;
use serde_json::Value;
use std::io::{BufRead, Write};

pub fn process_json<R: BufRead, W: Write>(reader: R, writer: &mut W) -> Result<()> {
    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line)?;

        let normalized = normalize_json(&json_value);
        writeln!(writer, "{}", normalized)?;
    }

    Ok(())
}

fn normalize_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let result: serde_json::Value = map
                .iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k.clone(), normalize_value(v)))
                .collect();
            serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
        }
        Value::Array(arr) => {
            let result: Vec<serde_json::Value> = arr.iter().map(normalize_value).collect();
            serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
        }
        _ => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
    }
}

fn normalize_value(value: &Value) -> serde_json::Value {
    match value {
        Value::Object(map) => {
            let result: serde_json::Value = map
                .iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k.clone(), normalize_value(v)))
                .collect();
            result
        }
        Value::Array(arr) => {
            let result: Vec<serde_json::Value> = arr.iter().map(normalize_value).collect();
            serde_json::Value::Array(result)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_json() {
        let json_input = r#"{"sku": "ITEM1", "qty": 10}
{"sku": "ITEM2", "qty": 20}"#;

        let mut output = Vec::new();
        let result = process_json(json_input.as_bytes(), &mut output);

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_normalize_json() {
        let input = serde_json::json!({
            "sku": "ITEM1",
            "qty": 10,
            "null_field": null
        });

        let result = normalize_json(&input);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert!(parsed.get("sku").is_some());
        assert!(parsed.get("qty").is_some());
        assert!(parsed.get("null_field").is_none());
    }
}
