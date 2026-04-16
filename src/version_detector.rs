use crate::parser::tokenizer::tokenize_segment;

/// Extracts EDIFACT version from UNH segment
/// UNH format: UNH+<message reference>+<message type>:<version>:<release>:<controlling agency>+...
/// Returns combined version string like "D96A"
pub fn extract_version_from_unh(unh_segment: &str) -> Option<String> {
    let tokens = tokenize_segment(unh_segment);
    if tokens.is_empty() || tokens[0][0] != "UNH" {
        return None;
    }

    // UNH+1+ORDERS:D:96A:UN+...
    // tokens[0] = ["UNH"]
    // tokens[1] = ["1"]
    // tokens[2] = ["ORDERS", "D", "96A", "UN"]
    if tokens.len() >= 3 && tokens[2].len() >= 3 {
        let version = tokens[2][1]; // D
        let release = tokens[2][2]; // 96A
        return Some(format!("{}{}", version, release));
    }

    // Alternative format: UNH+1+ORDERS:D:96A:UN:EAN008'
    // tokens[2] may have more components, but we need position 1 and 2
    None
}

/// Detects EDIFACT version by scanning lines until UNH segment is found
/// Returns (version, line_index) where line_index is the line number (0-based) of UNH
pub fn detect_version_from_lines<I: Iterator<Item = String>>(lines: I) -> Option<(String, usize)> {
    for (idx, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(version) = extract_version_from_unh(&line) {
            return Some((version, idx));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_from_unh() {
        let unh = "UNH+1+ORDERS:D:96A:UN";
        assert_eq!(extract_version_from_unh(unh), Some("D96A".to_string()));

        let unh = "UNH+123+INVOIC:D:01B:UN";
        assert_eq!(extract_version_from_unh(unh), Some("D01B".to_string()));

        let unh = "UNH+1+ORDERS:D:96A:UN:EAN008'";
        assert_eq!(extract_version_from_unh(unh), Some("D96A".to_string()));

        // Invalid segments
        assert_eq!(extract_version_from_unh("UNB+..."), None);
        assert_eq!(extract_version_from_unh("BGM+..."), None);
    }

    #[test]
    fn test_detect_version_from_lines() {
        let lines = vec![
            "UNB+...".to_string(),
            "UNH+1+ORDERS:D:96A:UN".to_string(),
            "BGM+...".to_string(),
        ];
        let result = detect_version_from_lines(lines.into_iter());
        assert_eq!(result, Some(("D96A".to_string(), 1)));

        let lines = vec!["UNB+...".to_string(), "UNH+123+INVOIC:D:01B:UN".to_string()];
        let result = detect_version_from_lines(lines.into_iter());
        assert_eq!(result, Some(("D01B".to_string(), 1)));

        let lines = vec!["BGM+...".to_string()];
        let result = detect_version_from_lines(lines.into_iter());
        assert_eq!(result, None);
    }
}
