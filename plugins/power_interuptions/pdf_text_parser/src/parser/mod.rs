use crate::parser::filter_out_comments::CommentsRemover;
use crate::scanner::{KeyWords, Token};
use crate::token::{Area, County, Region};
use multipeek::{multipeek, MultiPeek};
use std::iter;
use std::vec::IntoIter;

mod filter_out_comments;

fn parse(tokens: Vec<Token>) -> Vec<Region> {
    let peekable = tokens.into_iter();
    todo!()
}

struct Parser {
    tokens: MultiPeek<IntoIter<Token>>,
}

pub struct UnexpectedToken {
    found: Token,
    expected: String,
}

pub enum ParseError {
    UnexpectedEndOfFile,
    UnexpectedToken(UnexpectedToken),
}

macro_rules! consume_expected_token {
    ($tokens:expr, $expected:pat, $transform_token:expr, $required_element:expr) => {
        match $tokens.next() {
            Some($expected) => Ok($transform_token),
            Some(token) => {
                let unexpected_token = UnexpectedToken {
                    found: token.clone(),
                    expected: $required_element,
                };
                Err(ParseError::UnexpectedToken(unexpected_token))
            }
            None => Err(ParseError::UnexpectedEndOfFile),
        }
    };
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
        let region = consume_expected_token!(
            self.tokens,
            Token::Region(literal),
            literal,
            "Region".to_string()
        )?;
        let counties = self.parse_counties()?;
        Ok(Region {
            name: region,
            counties,
        })
    }

    fn parse_counties(&mut self) -> Result<Vec<County>, ParseError> {
        let mut counties = vec![];
        // loop up until we get to another region, returning the list of counties
        fn does_region_match(token: Option<&Token>) -> bool {
            matches!(token, Some(&Token::Region(_)))
        }
        while !does_region_match(self.tokens.peek()) {
            counties.push(self.parse_county()?);
        }
        Ok(counties)
    }

    fn parse_county(&mut self) -> Result<County, ParseError> {
        let county = consume_expected_token!(
            self.tokens,
            Token::County(literal),
            literal,
            "County".to_string()
        )?;

        let areas = self.parse_areas()?;

        Ok(County {
            name: county,
            areas,
        })
    }

    fn parse_areas(&mut self) -> Result<Vec<Area>, ParseError> {
        let mut areas = vec![];
        fn matches_county_or_region(token: Option<&Token>) -> bool {
            matches!(token, Some(&Token::County(_)) | Some(&Token::Region(_)))
        }

        while !matches_county_or_region(self.tokens.peek()) {
            areas.push(self.area()?);
        }

        Ok(areas)
    }

    fn area(&mut self) -> Result<Area, ParseError> {
        let area_lines = consume_expected_token!(
            self.tokens,
            Token::Area(literal),
            literal
                .split(",")
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            "Area".to_string()
        )?;
        let date =
            consume_expected_token!(self.tokens, Token::Date(date), date, "Date".to_owned())?;
        let time =
            consume_expected_token!(self.tokens, Token::Time(time), time, "Time".to_owned())?;
        let pins = self.pins()?;

        Ok(Area {
            lines: area_lines,
            date,
            time,
            pins,
        })
    }

    fn pins(&mut self) -> Result<Vec<String>, ParseError> {
        let mut results = vec![];
        fn end_of_pins(token: Option<&Token>) -> bool {
            matches!(token, Some(&Token::Keyword(KeyWords::EndOfAreaPins)))
        }

        while !end_of_pins(self.tokens.peek()) {
            let token = self.tokens.next().ok_or(ParseError::UnexpectedEndOfFile)?;
            match token {
                Token::Comma => continue,
                Token::Identifier(ident) => {
                    results.push(ident);
                }
                token => {
                    return Err(ParseError::UnexpectedToken(UnexpectedToken {
                        found: token,
                        expected: "Identifier".to_string(),
                    }))
                }
            }
        }

        // consume the end of pins keyword
        self.tokens.next();

        Ok(results)
    }
}
