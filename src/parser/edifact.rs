use super::segment::Segment;
use super::tokenizer::tokenize_segment;
use crate::translations::TranslationRegistry;

pub fn parse_segment<'a>(raw: &'a str) -> Segment<'a> {
    parse_segment_with_registry(raw, None)
}

pub fn parse_segment_with_registry<'a>(
    raw: &'a str,
    registry: Option<&TranslationRegistry>,
) -> Segment<'a> {
    let tokens = tokenize_segment(raw);
    let segment_code = tokens[0][0];

    // Try dynamic parsing if registry provided and segment is defined
    if let Some(reg) = registry {
        if let Some(segment_config) = reg.get_segment(segment_code) {
            let (qualifier, elements) = extract_dynamic_elements(&tokens, &segment_config);
            return Segment::Dynamic {
                code: segment_code,
                qualifier,
                elements,
            };
        }
    }

    // Fallback to static parsing
    match segment_code {
        "UNB" => Segment::UNB(
            tokens.get(2).and_then(|v| v.get(0)).copied().unwrap_or(""),
            tokens.get(3).and_then(|v| v.get(0)).copied().unwrap_or(""),
            tokens.get(5).and_then(|v| v.get(0)).copied().unwrap_or(""),
        ),
        "UNH" => Segment::UNH,
        "BGM" => Segment::BGM(
            tokens.get(1).and_then(|v| v.get(0)).copied().unwrap_or(""),
            tokens.get(2).and_then(|v| v.get(0)).copied().unwrap_or(""),
        ),
        "DTM" => {
            let v = &tokens[1];
            Segment::DTM(
                v.get(0).copied().unwrap_or(""),
                v.get(1).copied().unwrap_or(""),
            )
        }
        "NAD" => Segment::NAD(
            tokens[1][0],
            tokens.get(2).and_then(|v| v.get(0)).copied().unwrap_or(""),
        ),
        "LIN" => Segment::LIN(
            tokens.get(1).and_then(|v| v.get(0)).copied().unwrap_or(""),
            tokens.get(3).and_then(|v| v.get(0)).copied().unwrap_or(""),
        ),
        "QTY" => Segment::QTY(
            tokens[1][0],
            tokens[1][1],
            tokens[1].get(2).copied().unwrap_or(""),
        ),
        "MOA" => Segment::MOA(tokens[1][0], tokens[1][1]),
        "CNT" => Segment::CNT(tokens[1][0], tokens[1][1]),
        "CUX" => Segment::CUX(tokens.get(1).and_then(|v| v.get(1)).copied().unwrap_or("")),
        "UNT" => Segment::UNT,
        "UNZ" => Segment::UNZ,
        other => Segment::Unknown(other),
    }
}

fn extract_dynamic_elements<'a>(
    tokens: &[Vec<&'a str>],
    segment_config: &crate::translations::SegmentConfig,
) -> (Option<&'a str>, Vec<Vec<&'a str>>) {
    let mut qualifier = None;
    let mut elements = Vec::new();

    if tokens.len() < 2 {
        return (qualifier, elements);
    }

    let data_tokens = &tokens[1..];

    for (group_idx, group) in data_tokens.iter().enumerate() {
        if segment_config.use_qualifier && group_idx == 0 {
            // First element may have qualifier as first component
            if !group.is_empty() {
                qualifier = Some(group[0]);
                // The rest of components belong to this element
                let remaining: Vec<&'a str> = group.iter().skip(1).copied().collect();
                if !remaining.is_empty() {
                    elements.push(remaining);
                }
            }
        } else {
            // Whole group is an element
            elements.push(group.clone());
        }
    }

    (qualifier, elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translations::TranslationRegistry;

    #[test]
    fn test_parse_segment_with_registry() {
        let registry = TranslationRegistry::new().unwrap();

        // BGM segment defined in translations.json
        let seg = parse_segment_with_registry("BGM+220+12345", Some(&registry));
        match seg {
            Segment::Dynamic {
                code,
                qualifier,
                elements,
            } => {
                assert_eq!(code, "BGM");
                assert_eq!(qualifier, None);
                // elements should contain the two values
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0][0], "220");
                assert_eq!(elements[1][0], "12345");
            }
            _ => panic!("Expected Dynamic segment, got {:?}", seg),
        }

        // DTM segment with qualifier
        let seg = parse_segment_with_registry("DTM+137:20240414", Some(&registry));
        match seg {
            Segment::Dynamic {
                code,
                qualifier,
                elements,
            } => {
                assert_eq!(code, "DTM");
                assert_eq!(qualifier, Some("137"));
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0][0], "20240414");
            }
            _ => panic!("Expected Dynamic segment, got {:?}", seg),
        }

        // Unknown segment (not in translations) should fallback to static parsing
        let seg = parse_segment_with_registry("UNB+...", Some(&registry));
        // UNB is not in translations.json, so should be parsed statically
        match seg {
            Segment::UNB(_, _, _) => {}
            _ => panic!("Expected UNB segment, got {:?}", seg),
        }
    }
}
