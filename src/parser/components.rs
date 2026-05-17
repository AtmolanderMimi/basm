//! Component patterns, these are generic util patterns used to make other patterns.

use std::marker::PhantomData;

use either::Either;

use crate::lexer::token::Token;

use crate::parser::{Pattern, PatternResult};

#[derive(Debug, Clone, PartialEq, Default)]
/// Requires one of the patterns to be valid.
/// If both are valid, then the first one to be completed gets returned.
/// If both are completed at the same time, then the pattern `T` is prioritised.
pub struct Or<T, U>
where T: Pattern, U: Pattern {
    _phantom: PhantomData<(T, U)>
}

impl<T, U> Pattern for Or<T, U>
where T: Pattern, U: Pattern {
    type ParseResult = Either<T::ParseResult, U::ParseResult>;

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        if let Ok(ok) = T::solve(tokens) {
            Ok((ok.0, Either::Left(ok.1)))
        } else {
            // TODO: this code chooses the only returns the error of U
            // they should be combined into one error
            U::solve(tokens)
                .map(|ok| (ok.0, Either::Right(ok.1)))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
/// Requires both patterns to be valid in order (`T` → `U`).
pub struct Then<T, U>
where T: Pattern, U: Pattern {
    _phantom: PhantomData<(U, T)>
}

impl<T, U> Pattern for Then<T, U>
where T: Pattern, U: Pattern {
    type ParseResult = (T::ParseResult, U::ParseResult);

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let (t_tokens_consumed, t_parse_result) = T::solve(tokens)?;

        // skips the tokens already grabbed by U
        let next_tokens = &tokens[t_tokens_consumed..];
        let (u_tokens_consumed, u_parse_result) = U::solve(next_tokens)?;

        let tokens_consumed = t_tokens_consumed + u_tokens_consumed;
        Ok((tokens_consumed, (t_parse_result, u_parse_result)))
    }
}

/// Greedly matches the provided pattern zero or more times.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Many<T: Pattern> {
    _phantom: PhantomData<T>,
}

impl<T: Pattern> Pattern for Many<T> {
    type ParseResult = Vec<T::ParseResult>;

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let mut parse_results = Vec::new();

        let mut total_tokens_consumed = 0;
        while let Ok((tokens_consumed, parse_result)) = T::solve(&tokens[total_tokens_consumed..]) {
            total_tokens_consumed += tokens_consumed;
            parse_results.push(parse_result);
        }

        Ok((total_tokens_consumed, parse_results))
    }
}

/// The pattern may or may not be present
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Maybe<T>
where T: Pattern {
    _phantom: PhantomData<T>
}

impl<T> Pattern for Maybe<T>
where T: Pattern {
    type ParseResult = Option<T::ParseResult>;

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        if let Ok(ok) = T::solve(tokens) {
            Ok((ok.0, Some(ok.1)))
        } else {
            Ok((0, None))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::token::TokenType, parser::terminals::*, source::SfSlice};

    use std::assert_matches;

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn or_token_pattern() {
        let token = bogus_token(TokenType::Ident("joe".to_string()));
        let tokens = vec![token];
        let res = <Or<Ident, Or<NumLit, CharLit>>>::solve(&tokens);
        if let Ok(_) = res {
            // yay
        } else {
            panic!("should have been valid")
        }

        let token = bogus_token(TokenType::CharLit('c'));
        let tokens = vec![token];
        let res = <Or<Ident, Or<NumLit, CharLit>>>::solve(&tokens);
        if let Ok(_) = res {
            // yay
        } else {
            panic!("should have been valid")
        }

        let token = bogus_token(TokenType::Plus);
        let tokens = vec![token];
        let res = <Or<Ident, Or<NumLit, CharLit>>>::solve(&tokens);
        if let Err(_) = res {
            // yay
        } else {
            panic!("should NOT have been valid")
        }
    }

    #[test]
    fn then_token_pattern() {
        let tokens = vec![
            bogus_token(TokenType::CharLit('c')),
            bogus_token(TokenType::Minus),
            bogus_token(TokenType::Ident("a".to_string())),
        ];

        let res = <Then<CharLit, Then<Minus, Ident>>>::solve(&tokens);
        if res.is_err() {
            panic!("did not complete")
        }

        let tokens = vec![
            bogus_token(TokenType::CharLit('c')),
            bogus_token(TokenType::Ident("a".to_string())),
            bogus_token(TokenType::Minus),
        ];
        let res = <Then<CharLit, Then<Minus, Ident>>>::solve(&tokens);
        if res.is_ok() {
            panic!("did complete")
        }
    }

    #[test]
    fn many_token_pattern() {
        let tokens = vec![
            bogus_token(TokenType::Ident("tavgha".to_string())),
            bogus_token(TokenType::Ident("a".to_string())),
            bogus_token(TokenType::Minus),
            bogus_token(TokenType::Ident("the_secrets_of_732".to_string())),
            bogus_token(TokenType::Eof),
        ];
        let res = <Many<Ident>>::solve(&tokens);
        assert_eq!(res.unwrap().1.len(), 2);

        let tokens = vec![
            bogus_token(TokenType::Ident("tavgha".to_string())),
            bogus_token(TokenType::Ident("a".to_string())),
            bogus_token(TokenType::Minus),
            bogus_token(TokenType::Ident("the_secrets_of_732".to_string())),
            bogus_token(TokenType::Eof),
        ];
        let res = <Then<Then<Many<Ident>, Minus>, Many<Ident>>>::solve(&tokens);
        let res = res.unwrap();
        assert_eq!(res.1.0.0.len(), 2);
        assert_eq!(res.1.1.len(), 1);

        let tokens = vec![
            bogus_token(TokenType::Ident("tavgha".to_string())),
            bogus_token(TokenType::Ident("a".to_string())),
            bogus_token(TokenType::Eof),
        ];
        let res = <Then<Many<Ident>, Ident>>::solve(&tokens);
        assert!(res.is_err());
    }

    #[test]
    fn maybe_no_match() {
        let tokens = [ bogus_token(TokenType::Comma) ];

        // with one token
        let res = Maybe::<Ident>::solve(&tokens);
        assert_matches!(
            res,
            Ok((0, None))
        );

        // with no token
        let res = Maybe::<Ident>::solve(&[]);
        assert_matches!(
            res,
            Ok((0, None))
        )
    }

    #[test]
    fn maybe_with_match() {
        let tokens = [ bogus_token(TokenType::Comma) ];

        // with one token
        let res = Maybe::<Comma>::solve(&tokens);
        assert_matches!(
            res,
            Ok((_, Some(_)))
        );
    }
}