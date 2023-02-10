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

    fn is_contained_in_tokens(&self, tokens: &mut MultiPeek<IntoIter<Token>>) -> bool {
        self.is_start_in_tokens(&mut tokens.clone()) && self.is_end_in_tokens(&mut tokens.clone())
    }

    fn is_start_in_tokens(&self, tokens: &mut MultiPeek<IntoIter<Token>>) -> bool {
        let mut result = false;

        while tokens.peek().is_some() {
            let did_start_start = self.does_start_match(tokens);
            tokens.next();
            if did_start_start {
                result = true;
                break;
            }
        }
        result
    }

    fn is_end_in_tokens(&self, tokens: &mut MultiPeek<IntoIter<Token>>) -> bool {
        let mut result = false;

        while tokens.peek().is_some() {
            let did_end_match = self.does_end_match(tokens);
            tokens.next();
            if did_end_match {
                result = true;
                break;
            }
        }
        result
    }

    fn remove_comments(&self, tokens: Vec<Token>) -> Vec<Token> {
        let mut tokens = multipeek(tokens.into_iter());

        println!("{}", self.is_contained_in_tokens(&mut tokens.clone()));
        if self.is_contained_in_tokens(&mut tokens.clone()) {
            let mut result = vec![];

            while tokens.peek().is_some() {
                if self.does_start_match(&mut tokens) {
                    for _ in self.start.iter() {
                        tokens.next();
                    }
                    while !self.does_end_match(&mut tokens) {
                        tokens.next();
                    }

                    if tokens.peek().is_some() {
                        for _ in self.end.iter() {
                            tokens.next();
                        }
                    }
                } else {
                    result.push(tokens.next());
                }
            }
            result.into_iter().flatten().collect()
        } else {
            tokens.into_iter().collect()
        }
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

        let interruptions = Filter::new(scan("Interruption"), scan("construction"));

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
mod tests {
    use crate::parser::filter_out_comments::{CommentsRemover, Filter};
    use crate::scanner::Token;
    use multipeek::multipeek;

    #[test]
    fn test_comment_removal() {
        let tokens = vec![
            Token::Identifier("Interruption".to_owned()),
            Token::Identifier("test".to_owned()),
            Token::Identifier("not".to_owned()),
            Token::Identifier("removed".to_owned()),
            Token::Identifier("during".to_owned()),
            Token::Identifier("road".to_owned()),
            Token::Identifier("construction".to_owned()),
            Token::Comma,
            Token::Identifier("etc".to_owned()),
            Token::FullStop,
            Token::CloseBracket,
            Token::Identifier("Yes".to_owned()),
        ];
        let comment_remover = CommentsRemover::new();

        let result = comment_remover.remove_comments(tokens);

        println!("{:?}", result)
    }

    #[test]
    fn test_filter_end() {
        let tokens = vec![
            Token::Identifier("Interruption".to_owned()),
            Token::Identifier("test".to_owned()),
            Token::Identifier("not".to_owned()),
        ];
        let filter = Filter::new(vec![Token::FullStop], tokens.clone());

        let t = vec![
            Token::Identifier("Interruptions".to_owned()),
            Token::Identifier("test".to_owned()),
            Token::Identifier("not".to_owned()),
        ];
        let mut iter = multipeek(t.into_iter());

        println!("{}", filter.does_end_match(&mut iter));
    }
}
