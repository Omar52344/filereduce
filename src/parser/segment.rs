#[derive(Debug)]
pub enum Segment<'a> {
    UNB(&'a str, &'a str, &'a str),
    UNH,
    BGM(&'a str, &'a str),
    DTM(&'a str, &'a str),
    NAD(&'a str, &'a str),
    LIN(&'a str),
    QTY(&'a str, &'a str),
    MOA(&'a str, &'a str),
    CNT(&'a str, &'a str),
    UNT,
    UNZ,
    Unknown(&'a str),
}
