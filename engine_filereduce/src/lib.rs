pub mod executor {
    pub mod executor;
}
pub mod query {
    pub mod ast;
    pub mod parser;
}
pub mod reader {
    pub mod edi;
    pub mod reader;
}
pub mod row;

use executor::executor::Executor;
use query::parser::parse;
use reader::edi::EdiReader;

pub fn run_edi_query(file: &str, sql: &str) -> Vec<row::Row> {
    let reader = EdiReader::new(file);
    let query = parse(sql);
    let mut exec = Executor::new(reader, query);

    let mut out = Vec::new();
    while let Some(row) = exec.next() {
        out.push(row);
    }

    out
}
