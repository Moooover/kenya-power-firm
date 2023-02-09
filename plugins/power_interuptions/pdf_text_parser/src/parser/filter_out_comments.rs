use crate::scanner::Token;
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
                .all(|(i, token)| matches!(tokens.peek_nth(i), Some(token))),
        }
    }

    fn does_end_match(&self, tokens: &mut MultiPeek<IntoIter<Token>>) -> bool {
        match tokens.peek() {
            None => false,
            Some(_) => self
                .end
                .iter()
                .enumerate()
                .all(|(i, token)| matches!(tokens.peek_nth(i), Some(token))),
        }
    }
}

struct CommentsRemover {
    filters: Vec<Filter>,
}

impl CommentsRemover {
    fn new() -> Self {
        // TODO: Provide these filters as an env configuration
        let start = vec![Token::Identifier("Interruption".to_owned())];
        let end = vec![
            Token::Identifier("during".to_owned()),
            Token::Identifier("road".to_owned()),
            Token::Identifier("construction".to_owned()),
            Token::Comma,
            Token::Identifier("etc".to_owned()),
            Token::FullStop,
            Token::CloseBracket,
        ];
        let filter = Filter::new(start, end);

        Self {
            filters: vec![filter],
        }
    }
    fn remove_comments(&self, tokens: Vec<Token>) -> Vec<Token> {
        let mut result = tokens;

        for filter in self.filters.iter() {
            result = self.remove_comments_per_filter(filter, result);
        }

        result
    }

    fn remove_comments_per_filter(&self, filter: &Filter, tokens: Vec<Token>) -> Vec<Token> {
        let mut result: Vec<Option<Token>> = Vec::new();
        let mut tokens = multipeek(tokens.into_iter());
        // while tokens.peek().is_some() {
        //     if filter.does_start_match(&mut tokens) {
        //         self.advance_and_discard_until_end(filter, &mut tokens)
        //     } else {
        //         result.push(tokens.next());
        //     }
        // }
        loop {
            match tokens.peek() {
                None => break,
                Some(_) => {
                    if filter.does_start_match(&mut tokens) {
                        self.advance_and_discard_until_end(filter, &mut tokens)
                    } else {
                        result.push(tokens.next());
                    }
                }
            }
        }
        result.into_iter().flatten().collect()
    }

    fn advance_and_discard_until_end(
        &self,
        filter: &Filter,
        tokens: &mut MultiPeek<IntoIter<Token>>,
    ) {
        // TODO: Debug this issue
        while !filter.does_end_match(tokens) {
            tokens.next();
        }

        tokens.next();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::filter_out_comments::CommentsRemover;
    use crate::scanner::Token;

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
        ];
        let comment_remover = CommentsRemover::new();

        let result = comment_remover.remove_comments(tokens);

        println!("{:?}", result)
    }
}
