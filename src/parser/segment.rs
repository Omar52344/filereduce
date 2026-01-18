#[derive(Debug)]
pub enum Segment<'a> {
    UNH,
    BGM(&'a str),
    DTM(&'a str, &'a str),
    NAD(&'a str, &'a str),
    LIN(&'a str),
    QTY(&'a str, &'a str),
    MOA(&'a str, &'a str),
    UNT,
    UNZ,
    Unknown(&'a str),
}
