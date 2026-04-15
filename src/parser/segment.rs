#[derive(Debug)]
pub enum Segment<'a> {
    UNB(&'a str, &'a str, &'a str),
    UNH,
    BGM(&'a str, &'a str),
    DTM(&'a str, &'a str),
    NAD(&'a str, &'a str),
    LIN(&'a str, &'a str),
    QTY(&'a str, &'a str, &'a str),
    MOA(&'a str, &'a str),
    CNT(&'a str, &'a str),
    CUX(&'a str),
    UNT,
    UNZ,
    Dynamic {
        code: &'a str,
        qualifier: Option<&'a str>,
        elements: Vec<Vec<&'a str>>,
    },
    Unknown(&'a str),
}
