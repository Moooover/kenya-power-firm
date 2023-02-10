use multipeek::{multipeek, MultiPeek};
use std::str::Chars;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref LINE_BREAK_REMOVING_REGEX: Regex = Regex::new(r"[\r\n]+").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Colon,
    FullStop,
    Dash,
    Comma,
    OpenBracket,
    CloseBracket,
    Identifier(String),
    Area,
    Date,
    Time,
}

pub struct Scanner<'a> {
    source: MultiPeek<Chars<'a>>,
    current_lexeme: String,
}

fn is_digit(c: char) -> bool {
    ('0'..='9').contains(&c)
}

fn is_alpha(c: char) -> bool {
    ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || ['.', '-', '&', ':'].contains(&c)
}

fn is_alphanumeric(c: char) -> bool {
    is_digit(c) || is_alpha(c)
}

fn is_nextline(c: char) -> bool {
    matches!(c, '\n')
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\r' | '\t' | '\n') || c.is_whitespace()
}

fn is_white_space_or_new_line(c: char) -> bool {
    is_whitespace(c) || is_nextline(c)
}

impl<'a> Scanner<'a> {
    fn new(raw_text: &'a str) -> Self {
        let source = multipeek(raw_text.chars());
        Self {
            source,
            current_lexeme: Default::default(),
        }
    }

    fn advance(&mut self) -> Option<char> {
        let next = self.source.next();
        if let Some(c) = next {
            self.current_lexeme.push(c);
        }
        next
    }

    fn peek_check(&mut self, check: &dyn Fn(char) -> bool) -> bool {
        match self.source.peek() {
            Some(&c) => check(c),
            None => false,
        }
    }

    fn advance_while(&mut self, condition: &dyn Fn(char) -> bool) {
        while self.peek_check(condition) {
            self.advance();
        }
    }

    fn advance_but_discard(&mut self, condition: &dyn Fn(char) -> bool) {
        while self.peek_check(condition) {
            self.source.next();
        }
    }

    fn identifier(&mut self) -> Token {
        self.advance_while(&is_alphanumeric);

        match self.current_lexeme.to_ascii_uppercase().as_ref() {
            "DATE:" => Token::Date,
            "TIME:" => Token::Time,
            "AREA:" => Token::Area,
            _ => Token::Identifier(self.current_lexeme.to_owned()),
        }
    }

    fn scan_next(&mut self) -> Option<Token> {
        self.current_lexeme.clear();

        self.advance_but_discard(&is_white_space_or_new_line);

        let next_char = match self.advance() {
            Some(c) => c,
            None => return None,
        };

        let token = match next_char {
            ':' => Token::Colon,
            '.' => Token::FullStop,
            '-' => Token::Dash,
            ',' => Token::Comma,
            '(' => Token::OpenBracket,
            ')' => Token::CloseBracket,
            _ => self.identifier(),
        };

        Some(token)
    }
}

pub fn scan(text: &str) -> Vec<Token> {
    let scanner = ScannerIter {
        scanner: Scanner::new(text),
    };
    scanner.into_iter().collect()
}

struct ScannerIter<'a> {
    scanner: Scanner<'a>,
}

impl<'a> Iterator for ScannerIter<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.scanner.scan_next()
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::{is_alphanumeric, is_whitespace, scan};

    #[test]
    fn test_alphanumeric() {
        println!("{}", is_alphanumeric('&'))
    }

    #[test]
    fn test_scanned_text() {
        let text = r"Interruption";

        let result = scan(text);

        println!("{:?}", result)
    }

    #[test]
    fn test_white_space() {
        println!("{}", is_whitespace('\n'))
    }
}
