use crate::parser::filter_out_comments::CommentsRemover;
use crate::scanner::{KeyWords, Token};
use crate::token::{Area, County, Region};
use multipeek::{multipeek, MultiPeek};
use std::iter;
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
    UnexpectedToken(Token),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let comments_remover = CommentsRemover::new();
        let tokens = comments_remover.remove_comments(tokens);
        Self {
            tokens: multipeek(tokens.into_iter()),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Region>, ParseError> {
        let mut regions = Vec::new();
        let mut error = None;
        loop {
            let result = self.parse_region();
            match result {
                Ok(region) => regions.push(region),
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
        Ok(regions)
    }

    fn parse_region(&mut self) -> Result<Region, ParseError> {
        let next = { self.tokens.peek().ok_or(ParseError::UnexpectedEndOfFile)? };
        match next {
            Token::Region(region) => {
                let counties = self.parse_counties()?;
                Ok(Region {
                    name: region.to_owned(),
                    counties,
                })
            }
            token => Err(ParseError::UnexpectedToken(token.clone())),
        }
    }

    fn parse_counties(&mut self) -> Result<Vec<County>, ParseError> {
        // loop up until we get to another region, returning the list of counties
        todo!()
    }

    fn parse_county(&mut self) -> Result<County, ParseError> {
        // TODO: loop until we get to another county
        todo!()
    }

    fn parse_areas(&mut self) -> Result<Vec<Area>, ParseError> {
        let mut areas = vec![];
        fn matches_county_or_region(token: Option<&Token>) -> bool {
            matches!(token, Some(&Token::County(_) | Some(&Token::Region(_))))
        }

        while matches_county_or_region(self.tokens.peek()) {
            areas.push(self.area()?);
        }

        Ok(areas)
    }

    fn area(&mut self) -> Result<Area, ParseError> {
        // loop until we get to another area
        todo!()
    }

    fn consume_expected_identifier(&mut self) -> Result<String, ParseError> {
        let next = self.tokens.next().ok_or(ParseError::UnexpectedEndOfFile)?;
        match next {
            Token::Identifier(identifier) => Ok(identifier),
            token => Err(ParseError::UnexpectedToken(token)),
        }
    }
}
