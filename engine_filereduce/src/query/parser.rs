use crate::query::ast::Query;

pub fn parse(sql: &str) -> Query {
    // ⚠️ ultra simple, mejorable luego
    let sql = sql.to_lowercase();

    let select_part = sql.split("from").next().unwrap().replace("select", "");

    let select = select_part
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let limit = if let Some(pos) = sql.find("limit") {
        sql[pos + 5..].trim().parse().ok()
    } else {
        None
    };

    Query {
        select,
        filter: None,
        limit,
    }
}
