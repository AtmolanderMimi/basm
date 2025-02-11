//! Componant patterns, these are generic util patterns used to make other patterns.

use std::mem;

use either::Either;

use crate::lexer::token::Token;

use super::{Advancement, AdvancementState as AdvState, Pattern};

#[derive(Debug, Clone, PartialEq, Default)]
/// Requires one of the patterns to be valid.
/// If both are valid, then the first one to be completed gets returned.
/// If both are completed at the same time, then the pattern `T` is prioritised.
pub(super) struct Or<T, U>
where T: Pattern, U: Pattern {
    t: T,
    u: Option<U>, // only loads u if it is needed
}

impl<T, U> Pattern for Or<T, U>
where T: Pattern, U: Pattern {
    type ParseResult = Either<T::ParseResult, U::ParseResult>;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        if let Some(u) = &mut self.u {
            let adv = u.advance(token);
            let overeach = adv.overeach;
            match adv.state {
                AdvState::Advancing => {
                    return Advancement::new(AdvState::Advancing, overeach)
                },
                AdvState::Done(u_res) => {
                    return Advancement::new(AdvState::Done(Either::Right(u_res)), overeach)
                },
                // TODO: simply error forwarding here, a compound error would probably be right'er
                AdvState::Error(e) => {
                    return Advancement::new(AdvState::Error(e), overeach)
                },
            }
        }

        
        let adv = self.t.advance(token);
        let overeach = adv.overeach;
        match adv.state {
            AdvState::Advancing => {
                Advancement::new(AdvState::Advancing, overeach)
            },
            AdvState::Done(t_res) => {
                Advancement::new(AdvState::Done(Either::Left(t_res)), overeach)
            },
            AdvState::Error(_) => {
                self.u = Some(U::default());
                Advancement::new(AdvState::Advancing, overeach)
            },
        }

    }
}

#[derive(Debug, Clone, PartialEq)]
/// Requires both patterns to be valid in order (`T` → `U`).
pub(super) struct Then<T, U>
where T: Pattern, U: Pattern {
    t: T,
    t_res: Option<T::ParseResult>,
    u: U,
    token_count: usize,
}

impl<T, U> Default for Then<T, U>
where T: Pattern, U: Pattern {
    fn default() -> Self {
        Then {
            t: T::default(),
            t_res: None,
            u: U::default(),
            token_count: 0,
        }
    }
}

impl<T, U> Pattern for Then<T, U>
where T: Pattern, U: Pattern {
    type ParseResult = (T::ParseResult, U::ParseResult);

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        self.token_count += 1;

        if let Some(t_res) = &self.t_res {
            let adv = self.u.advance(token);
            let overeach = adv.overeach;

            let adv_return = match adv.state {
                AdvState::Advancing => {
                    self.token_count -= overeach;
                    Advancement::new(AdvState::Advancing, overeach)
                },
                AdvState::Done(u_res) => {
                    self.token_count -= overeach;
                    Advancement::new(AdvState::Done((t_res.clone(), u_res)), overeach)
                },
                AdvState::Error(e) => {
                    Advancement::new(AdvState::Error(e), self.token_count)
                }
            };

            return adv_return
        }


        let adv = self.t.advance(token);
        let overeach = adv.overeach;
        match adv.state {
            AdvState::Advancing => {
                self.token_count -= overeach;
                Advancement::new(AdvState::Advancing, overeach)
            },
            AdvState::Done(t_res) => {
                self.token_count -= overeach;
                self.t_res = Some(t_res);
                Advancement::new(AdvState::Advancing, overeach)
            },
            AdvState::Error(e) => {
                Advancement::new(AdvState::Error(e), self.token_count)
            }
        }
    }
}

/// Greedly matches the provided pattern zero or more times.
#[derive(Debug, Clone, PartialEq)]
pub struct Many<T: Pattern> {
    t: T,
    results: Vec<T::ParseResult>,
}

impl<T: Pattern> Default for Many<T> {
    fn default() -> Self {
        Many {
            t: T::default(),
            results: Vec::new(),
        }
    }
}

impl<T: Pattern> Pattern for Many<T> {
    type ParseResult = Vec<T::ParseResult>;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.t.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                self.results.push(res);
                self.t = T::default();

                Advancement::new(AdvState::Advancing, overeach)
            },
            // TODO: bad error handly here too
            AdvState::Error(_) => {
                let results = mem::take(&mut self.results);
                Advancement::new(AdvState::Done(results), overeach)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::token::TokenType, parser::{solve_pattern, terminals::*}, source::SfSlice};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn or_token_pattern() {
        let token = bogus_token(TokenType::Ident("joe".to_string()));
        let tokens = vec![token];
        let res = solve_pattern::<Or<IdentPattern, Or<NumLitPattern, CharLitPattern>>>(&tokens);
        if let Ok(_) = res {
            // yay
        } else {
            panic!("should have been valid")
        }

        let token = bogus_token(TokenType::CharLit('c'));
        let tokens = vec![token];
        let res = solve_pattern::<Or<IdentPattern, Or<NumLitPattern, CharLitPattern>>>(&tokens);
        if let Ok(_) = res {
            // yay
        } else {
            panic!("should have been valid")
        }

        let token = bogus_token(TokenType::Plus);
        let tokens = vec![token];
        let res = solve_pattern::<Or<IdentPattern, Or<NumLitPattern, CharLitPattern>>>(&tokens);
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

        let res = solve_pattern::<Then<CharLitPattern, Then<MinusPattern, IdentPattern>>>(&tokens);
        if res.is_err() {
            panic!("did not complete")
        }

        let tokens = vec![
            bogus_token(TokenType::CharLit('c')),
            bogus_token(TokenType::Ident("a".to_string())),
            bogus_token(TokenType::Minus),
        ];
        let res = solve_pattern::<Then<CharLitPattern, Then<MinusPattern, IdentPattern>>>(&tokens);
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
        let res = solve_pattern::<Many<IdentPattern>>(&tokens);
        assert_eq!(res.unwrap().len(), 2);

        let tokens = vec![
            bogus_token(TokenType::Ident("tavgha".to_string())),
            bogus_token(TokenType::Ident("a".to_string())),
            bogus_token(TokenType::Minus),
            bogus_token(TokenType::Ident("the_secrets_of_732".to_string())),
            bogus_token(TokenType::Eof),
        ];
        let res = solve_pattern::<Then<Then<Many<IdentPattern>, MinusPattern>, Many<IdentPattern>>>(&tokens);
        let res = res.unwrap();
        assert_eq!(res.0.0.len(), 2);
        assert_eq!(res.1.len(), 1);

        let tokens = vec![
            bogus_token(TokenType::Ident("tavgha".to_string())),
            bogus_token(TokenType::Ident("a".to_string())),
            bogus_token(TokenType::Eof),
        ];
        let res = solve_pattern::<Then<Many<IdentPattern>, IdentPattern>>(&tokens);
        assert!(res.is_err());
    }
}