use crate::scanner::{scan, Token};
use multipeek::{multipeek, MultiPeek};
use std::vec::IntoIter;

#[derive(Debug)]
struct Filter {
    start: Vec<Token>,
    end: Vec<Token>,
}

impl Filter {
    fn new(start: Vec<Token>, end: Vec<Token>) -> Self {
        Self { start, end }
    }
    fn does_start_match(&self, tokens: &mut MultiPeek<IntoIter<Token>>) -> bool {
        match tokens.peek() {
            None => false,
            Some(_) => self
                .start
                .iter()
                .enumerate()
                .all(|(i, token)| tokens.peek_nth(i) == Some(token)),
        }
    }

    fn does_end_match(&self, tokens: &mut MultiPeek<IntoIter<Token>>) -> bool {
        match tokens.peek() {
            None => false,
            Some(_) => self
                .end
                .iter()
                .enumerate()
                .all(|(i, token)| tokens.peek_nth(i) == Some(token)),
        }
    }

    fn remove_comments(&self, tokens: Vec<Token>) -> Vec<Token> {
        let reset = tokens.clone();
        let mut tokens = multipeek(tokens.into_iter());

        let mut result = vec![];
        while tokens.peek().is_some() {
            if !self.does_start_match(&mut tokens) {
                result.push(tokens.next());
                continue;
            }
            for _ in self.start.iter() {
                tokens.next();
            }
            while !self.does_end_match(&mut tokens) && tokens.peek().is_some() {
                tokens.next();
            }
            // the end wasn't in the tokens so just the initial tokens.
            if tokens.peek().is_none() {
                return reset;
            }

            for _ in self.end.iter() {
                tokens.next();
            }
        }
        result.into_iter().flatten().collect()
    }
}

pub struct CommentsRemover {
    filters: Vec<Filter>,
}

impl CommentsRemover {
    pub fn new() -> Self {
        // TODO: Provide these filters as a config file
        let communications_filter = Filter::new(
            scan("For further information, contact"),
            scan("Interruption notices may be viewed at www.kplc.co.ke"),
        );

        let interruptions = Filter::new(
            scan("Interruption of Electricity Supply"),
            scan("road construction, etc.)"),
        );

        Self {
            filters: vec![communications_filter, interruptions],
        }
    }
    pub fn remove_comments(&self, tokens: Vec<Token>) -> Vec<Token> {
        let mut result = tokens;

        for filter in self.filters.iter() {
            result = filter.remove_comments(result.clone());
        }

        result
    }
}

#[cfg(test)]
mod tests {}
