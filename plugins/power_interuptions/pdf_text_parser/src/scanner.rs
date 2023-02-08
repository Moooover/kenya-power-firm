use multipeek::{multipeek, MultiPeek};
use std::str::Chars;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref LINE_BREAK_REMOVING_REGEX: Regex = Regex::new(r"[\r\n]+").unwrap();
}

#[derive(Debug)]
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
        let text = r"
         Interruption of
Electricity Supply
Notice is hereby given under Rule 27 of the Electric Power Rules
That the electricity supply will be interrupted as here under:
(It  is  necessary  to  interrupt  supply  periodically  in  order  to
facilitate maintenance and upgrade of power lines to the network;
to connect new customers or to replace power lines during road
construction, etc.)

NAIROBI REGION

AREA: GRAIN BULK
DATE: Sunday 05.02.2023                                  TIME: 9.00 A.M. – 5.00 P.M.
Heavy Engineering, Grain Bulk Handlers, Posh Auto body, SGR Head office
& adjacent customers.

AREA: REDHILL ROAD
DATE: Tuesday 07.02.2023                         TIME: 9.00 A.M. – 5.00 P.M.
Redhill Rd, Rosslyn Green, Part of Nyari, Embassy of Switzerland, Gachie,
Karura  SDA  Church,  Hospital  Hill  Sec  Sch,  Commission   for  University
Education, Trio Est & adjacent customers.

AREA: KIHARA, KINANDA
DATE: Tuesday 07.02.2023                                TIME: 9.00 A.M. – 5.00 P.M.
Kihara  Village,  Old  Karura  Rd,  Kihara  Mkt,  Kitsuru  Ridge  Villas  Est, White
Cottage Sch, Weaverbird Kenya, Part of Kirawa Rd, Kitsuru Country Homes,
Hotani Close, Parazoro Institute & adjacent customers.

         ";

        let result = scan(text);

        println!("{:?}", result)
    }

    #[test]
    fn test_white_space() {
        println!("{}", is_whitespace('\n'))
    }
}
