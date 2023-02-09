use crate::scanner::Token;
use crate::token::Ast;
use multipeek::{multipeek, MultiPeek};
use std::vec::IntoIter;

mod filter_out_comments;

fn parse(tokens: Vec<Token>) -> Vec<Ast> {
    let peekable = tokens.iter();
    todo!()
}

struct Parser {
    tokens: MultiPeek<IntoIter<Token>>,
}

pub enum ParseError {
    UnexpectedEndOfFile,
    UnexpectedElement(String),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: multipeek(tokens.into_iter()),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Ast>, ParseError> {
        let mut ast_collection = Vec::new();
        let mut error = None;
        loop {
            let result = self.parse_entry_point();
            match result {
                Ok(ast) => ast_collection.push(ast),
                Err(ParseError::UnexpectedEndOfFile) => {
                    break;
                }
                Err(err) => {
                    error = Some(err);
                    break;
                }
            }
        }
        if let Some(error) = error {
            return Err(error);
        }
        Ok(ast_collection)
    }

    fn parse_entry_point(&mut self) -> Result<Ast, ParseError> {
        let next = { self.tokens.peek().ok_or(ParseError::UnexpectedEndOfFile)? };
        match next {
            token => Err(ParseError::UnexpectedElement(format!("{token:?}"))),
        }
    }
    fn parse_identifier(&mut self, identifier: &str) -> Result<Ast, ParseError> {
        todo!()
    }
}
