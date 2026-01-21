//use crate::expr::Expr;
//use crate::value::Value;

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
        self.parse_expr()
    }

    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();

        while self.match_token("AND") {
            let right = self.parse_term();
            left = Expr::And(Box::new(left), Box::new(right));
        }

        left
    }

    fn parse_term(&mut self) -> Expr {
        if self.match_token("(") {
            let expr = self.parse_expr();
            self.expect(")");
            expr
        } else {
            self.parse_factor()
        }
    }

    fn parse_factor(&mut self) -> Expr {
        let field = self.next().to_string();
        let op = self.next();
        let value = self.parse_value();

        match op.as_str() {
            "=" => Expr::Eq(field, value),
            ">" => Expr::Gt(field, value),
            "<" => Expr::Lt(field, value),
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
    input
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}
