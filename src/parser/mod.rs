//! Parser module for VHP
//!
//! This module contains the recursive descent parser that transforms
//! tokens into an AST.

mod expr;
mod precedence;
mod stmt;

use crate::ast::Program;
use crate::token::{Token, TokenKind};
use stmt::StmtParser;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        let current = self.tokens.get(self.pos).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
            column: 0,
        });
        std::mem::discriminant(&current.kind) == std::mem::discriminant(kind)
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::Eof) {
            let mut stmt_parser = StmtParser::new(&self.tokens, &mut self.pos);
            if let Some(stmt) = stmt_parser.parse_statement()? {
                statements.push(stmt);
            }
        }

        Ok(Program { statements })
    }
}
