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
        if let Ok(ok) = U::solve(tokens) {
            Ok((ok.0, Either::Right(ok.1)))
        } else {
            // TODO: this code chooses the only returns the error of T
            // they should be combined into one error
            T::solve(tokens)
                .map(|ok| (ok.0, Either::Left(ok.1)))
        }
    }

    fn name() -> String {
        format!("{} or {}", T::name(), U::name())
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

    fn name() -> String {
        format!("{} then {}", T::name(), U::name())
    }
}

/// Greedly matches the provided pattern one or more times.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OneOrMore<T: Pattern> {
    _phantom: PhantomData<T>,
}

impl<T: Pattern> Pattern for OneOrMore<T> {
    type ParseResult = Vec<T::ParseResult>;

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let (first_tokens_consumed, first_parse) = T::solve(tokens)?;
        
        let mut parse_results = vec![first_parse];
        let mut total_tokens_consumed = first_tokens_consumed;
        while let Ok((tokens_consumed, parse_result)) = T::solve(&tokens[total_tokens_consumed..]) {
            total_tokens_consumed += tokens_consumed;
            parse_results.push(parse_result);
        }

        Ok((total_tokens_consumed, parse_results))
    }

    fn name() -> String {
        format!("1 or more {}", T::name())
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
        if let Ok((tokens_consumed, parsed_items)) = OneOrMore::<T>::solve(&tokens) {
            return Ok((tokens_consumed, parsed_items))            
        } else {
            return Ok((0, Vec::new()))
        }
    }

    fn name() -> String {
        format!("0 or more {}", T::name())
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

    fn name() -> String {
        format!("1 or 0 {}", T::name())
    }
}

/// This pattern just serves as a wildcard in other component patterns, such as [TerminatedSeperatedMany].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Nothing;

impl Pattern for Nothing {
    type ParseResult = ();

    fn solve(_tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        Ok((0, ()))
    }

    fn name() -> String {
        format!("nothing")
    }
}

/// A list of many items `T` ([Many]) seperated by `U` and terminated by `V`.
/// You may use only the seperation or termination elements using [SeperatedMany] and [TerminatedMany].
/// This component is functionnaly equivalent to Then::<Many<T>, U>.
/// However, it gives better errors, since the default [Many] cannot error.
/// With the component, if the `T` pattern fails before there is a `U`,
/// the error will be reported as a lack of `T` instead of a lack of `U`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TerminatedSeperatedMany<T, U, V>
where T: Pattern, U: Pattern, V: Pattern {
    _phantom: PhantomData<(T, U, V)>
}

impl<T, U, V> Pattern for TerminatedSeperatedMany<T, U, V>
where T: Pattern, U: Pattern, V: Pattern {
    type ParseResult = (Vec<(Option<U::ParseResult>, T::ParseResult)>, V::ParseResult);

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let seperated_res =
        Maybe::<
            Then<
                T,
                Many<Then<U, T>>
            >
        >::solve(&tokens)?;

        // combines the first item with the other items
        let (seperated_tokens_consumed, maybe_items) = seperated_res;
        let items = if let Some((first_item, other_items)) = maybe_items {
            let mut other_items = other_items.into_iter().map(|(c, e)| (Some(c), e))
                .collect::<Vec<_>>();

            other_items.insert(0, (None, first_item));

            other_items
        } else {
            Vec::new()
        };

        let mut rest_tokens = &tokens[seperated_tokens_consumed..];
        let Ok((v_tokens_consumed, terminator)) = V::solve(rest_tokens) else {
            // If this is not the first item, then we also need to check if there is a seperator
            if !items.is_empty() {
                let (u_consumed_tokens, _) =  U::solve(rest_tokens)?;
                rest_tokens = &rest_tokens[u_consumed_tokens..];
            }

            // if there is no terminator, this means that the many did not parse all the tokens until the terminator
            // we rerun just a single T parse to know the error
            T::solve(rest_tokens)?;

            panic!("T or U should always error");
        };

        Ok((seperated_tokens_consumed + v_tokens_consumed, (items, terminator)))
    }

    fn name() -> String {
        format!("0 or more {} sperated by {} and terminated by {}", T::name(), U::name(), V::name())
    }
}

/// A list of many items `T` ([Many]) seperated by `U`.
/// The seperator cannot be at the end of the list.
/// The first item of the returned list (if not empty) is guarentied to not have a seperator
/// (i.e the first element of the enum is `None`).
/// 
/// Equivalent to `TerminatedSeperatedMany<T, U, Nothing>`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SeperatedMany<T, U>
where T: Pattern, U: Pattern {
    _phantom: PhantomData<(T, U)>
}

impl<T, U> Pattern for SeperatedMany<T, U>
where T: Pattern, U: Pattern {
    type ParseResult = Vec<(Option<U::ParseResult>, T::ParseResult)>;

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = TerminatedSeperatedMany::<T, U, Nothing>::solve(tokens)?;

        Ok((res.0, res.1.0))
    }

    fn name() -> String {
        format!("{} seperated list of {}", T::name(), U::name())
    }
}

/// A list of many items `T` ([Many]) terminated by `U`.
/// This is practically equivalent to Then::<Many<T>, U>, except the error messages are better.
/// The error messages will blame the lack of `T` instead of the lack of `U` when an error occurs.
/// 
/// Equivalent to `TerminatedSeperatedMany<T, Nothing, U>`
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TerminatedMany<T, U>
where T: Pattern, U: Pattern {
    _phantom: PhantomData<(T, U)>
}

impl<T, U> Pattern for TerminatedMany<T, U>
where T: Pattern, U: Pattern {
    type ParseResult = (Vec<T::ParseResult>, U::ParseResult);

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = TerminatedSeperatedMany::<T, Nothing, U>::solve(tokens)?;

        let items = res.1.0.into_iter()
            .map(|(_, i)| i).collect();

        Ok((res.0, (items, res.1.1)))
    }

    fn name() -> String {
        format!("0 or more {} terminated by {}", T::name(), U::name())
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::{lex_string, token::TokenType}, parser::{expression::Expression, terminals::*}, source::SfSlice};

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
    fn one_or_more_none_does_not_parse() {
        let tokens = lex_string("").unwrap();

        let res = OneOrMore::<Ident>::solve(&tokens);
        res.unwrap_err();
    }

    #[test]
    fn one_or_more_one_item_does_parse() {
        let tokens = lex_string("hello ").unwrap();

        let res = OneOrMore::<Ident>::solve(&tokens);
        let (tokens_taken, items) = res.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(tokens_taken, tokens.len()-1);
    }

    #[test]
    fn one_or_more_multiple_items_does_parse() {
        let tokens = lex_string("hello welcome even").unwrap();

        let res = OneOrMore::<Ident>::solve(&tokens);
        let (tokens_taken, items) = res.unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(tokens_taken, tokens.len()-1);
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

    #[test]
    fn seperated_many_no_match() {
        let tokens = [ bogus_token(TokenType::Comma) ];

        let res = SeperatedMany::<Ident, Comma>::solve(&tokens);
        let items = res.unwrap().1;
        assert!(items.is_empty());
    }

    #[test]
    fn seperated_many_single_match() {
        let tokens = lex_string("hello").unwrap();

        let res = SeperatedMany::<Ident, Comma>::solve(&tokens);
        let items = res.unwrap().1;
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn seperated_many_multiple_match() {
        let tokens = lex_string("hello, world,and,people").unwrap();

        let res = SeperatedMany::<Ident, Comma>::solve(&tokens);
        let items = res.unwrap().1;
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn seperated_many_ending_in_seperator_does_not_include_it() {
        let tokens = lex_string("hello, world,and,people,").unwrap();

        let res = SeperatedMany::<Ident, Comma>::solve(&tokens);
        let (tokens_taken, items) = res.unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(tokens_taken, 7);
    }

    #[test]
    fn seperated_many_minimal() {
        let tokens = lex_string("").unwrap();

        let res = SeperatedMany::<Ident, Comma>::solve(&tokens);
        let (tokens_taken, items) = res.unwrap();
        assert_eq!(items.len(), 0);
        assert_eq!(tokens_taken, tokens.len()-1);
    }

    #[test]
    fn terminated_many_parses_valid() {
        let tokens = lex_string("hello hi welcome;").unwrap();

        let res = TerminatedMany::<Ident, Semicolon>::solve(&tokens);
        let (tokens_taken, (items, _)) = res.unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(tokens_taken, tokens.len()-1);
    }

    #[test]
    fn terminated_many_does_not_parse_invalid_many() {
        let tokens = lex_string("hello hi welcome 34;").unwrap();

        let res = TerminatedMany::<Ident, Semicolon>::solve(&tokens);
        res.unwrap_err();
    }

    #[test]
    fn terminated_many_does_not_parse_missing_terminator() {
        let tokens = lex_string("hello hi welcome").unwrap();

        let res = TerminatedMany::<Ident, Semicolon>::solve(&tokens);
        res.unwrap_err();
    }

    #[test]
    fn terminated_many_minimal() {
        let tokens = lex_string(";").unwrap();

        let res = TerminatedMany::<Ident, Semicolon>::solve(&tokens);
        let (tokens_taken, (items, _)) = res.unwrap();
        assert_eq!(items.len(), 0);
        assert_eq!(tokens_taken, tokens.len()-1);
    }

    #[test]
    fn terminated_seperated_many_parses_valid() {
        let tokens = lex_string("3 + 4, hi@[3,5], 'c';").unwrap();

        let res = TerminatedSeperatedMany::<Expression, Comma, Semicolon>::solve(&tokens);
        let (tokens_taken, (items, _)) = res.unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(tokens_taken, tokens.len()-1);
    }

    #[test]
    fn terminated_seperated_many_does_not_parse_missing_seperator() {
        let tokens = lex_string("3 + 4, hi@[3,5] 'c';").unwrap();

        let res = TerminatedSeperatedMany::<Expression, Comma, Semicolon>::solve(&tokens);
        res.unwrap_err();
    }

    #[test]
    fn terminated_seperated_many_does_not_parse_missing_terminator() {
        let tokens = lex_string("3 + 4, hi@[3,5], 'c'").unwrap();

        let res = TerminatedSeperatedMany::<Expression, Comma, Semicolon>::solve(&tokens);
        res.unwrap_err();
    }

    #[test]
    fn terminated_seperated_many_minimal() {
        let tokens = lex_string(";").unwrap();

        let res = TerminatedSeperatedMany::<Expression, Comma, Semicolon>::solve(&tokens);
        let (tokens_taken, (items, _)) = res.unwrap();
        assert_eq!(items.len(), 0);
        assert_eq!(tokens_taken, tokens.len()-1);
    }
}