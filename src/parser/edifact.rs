use super::segment::Segment;
use super::tokenizer::tokenize_segment;

pub fn parse_segment<'a>(raw: &'a str) -> Segment<'a> {
    let tokens = tokenize_segment(raw);

    match tokens[0][0] {
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
        "QTY" => Segment::QTY(tokens[1][0], tokens[1][1]),
        "MOA" => Segment::MOA(tokens[1][0], tokens[1][1]),
        "CNT" => Segment::CNT(tokens[1][0], tokens[1][1]),
        "CUX" => Segment::CUX(tokens.get(1).and_then(|v| v.get(1)).copied().unwrap_or("")),
        "UNT" => Segment::UNT,
        "UNZ" => Segment::UNZ,
        other => Segment::Unknown(other),
    }
}
