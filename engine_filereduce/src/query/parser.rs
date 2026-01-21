use crate::query::ast::Expr;
use crate::row::Value;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<String>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = tokenize(input);
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Expr {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();

        while self.match_token("OR") {
            let right = self.parse_and();
            left = Expr::Or(Box::new(left), Box::new(right));
        }

        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_term();

        while self.match_token("AND") {
            let right = self.parse_term();
            left = Expr::And(Box::new(left), Box::new(right));
        }

        left
    }

    fn parse_term(&mut self) -> Expr {
        if self.match_token("(") {
            let expr = self.parse_or();
            self.expect(")");
            expr
        } else {
            self.parse_factor()
        }
    }

    fn parse_factor(&mut self) -> Expr {
        if self.match_token("NOT") {
            let inner = self.parse_term();
            return Expr::Not(Box::new(inner));
        }

        let field = self.next().to_string();
        let op = self.next();

        match op.as_str() {
            "=" => {
                let value = self.parse_value();
                Expr::Eq(field, value)
            }
            ">" => {
                let value = self.parse_value();
                Expr::Gt(field, value)
            }
            "<" => {
                let value = self.parse_value();
                Expr::Lt(field, value)
            }
            ">=" => {
                let value = self.parse_value();
                Expr::Gte(field, value)
            }
            "<=" => {
                let value = self.parse_value();
                Expr::Lte(field, value)
            }
            "LIKE" => {
                let pattern = self.parse_string();
                Expr::Like(field, pattern)
            }
            "IN" => {
                self.expect("(");
                let mut values = Vec::new();
                loop {
                    let token = self.peek().unwrap_or("");
                    if token == ")" {
                        break;
                    }
                    values.push(self.parse_value());
                    if !self.match_token(",") {
                        break;
                    }
                }
                self.expect(")");
                Expr::In(field, values)
            }
            "BETWEEN" => {
                let start = self.parse_value();
                self.expect("AND");
                let end = self.parse_value();
                Expr::Between(field, start, end)
            }
            _ => panic!("Operador no soportado: {}", op),
        }
    }

    fn parse_value(&mut self) -> Value {
        let token = self.next();

        if token.starts_with('\'') {
            Value::Text(token.trim_matches('\'').to_string())
        } else {
            Value::Number(token.parse().expect("Número inválido"))
        }
    }

    fn parse_string(&mut self) -> String {
        let token = self.next();
        token.trim_matches('\'').to_string()
    }

    fn match_token(&mut self, expected: &str) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: &str) {
        let tok = self.next();
        if tok != expected {
            panic!("Esperado {}, encontrado {}", expected, tok);
        }
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|s| s.as_str())
    }

    fn next(&mut self) -> String {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let result = input
        .replace("(", " ( ")
        .replace(")", " ) ")
        .replace(">=", " >= ")
        .replace("<=", " <= ")
        .replace("!=", " != ")
        .replace(",", " , ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    result
}
