//! Componant patterns, these are generic util patterns used to make other patterns.

use std::marker::PhantomData;

use either::Either;

use crate::lexer::token::Token;

use super::{Advancement, AdvancementState as AdvState, Pattern, PatternMatchingError};

#[derive(Debug, Clone, PartialEq, Default)]
/// Requires one of the patterns to be valid.
/// If both are valid, then the first one to be completed gets returned.
/// If both are completed at the same time, then the pattern `T` is prioritised.
pub(super) struct Or<'a, T, U>
where T: Pattern<'a>, U: Pattern<'a> {
    _phantom: PhantomData<&'a ()>,
    checking_u: bool,
    t: T,
    u: U,
}

impl<'a, T, U> Pattern<'a> for Or<'a, T, U>
where T: Pattern<'a>, U: Pattern<'a> {
    type ParseResult = Either<T::ParseResult, U::ParseResult>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
        if !self.checking_u {
            let adv = self.t.advance(&token);
            let overeach = adv.overeach;
            let adv_return = match adv.state {
                AdvState::Advancing => {
                    Advancement::new(AdvState::Advancing, overeach)
                },
                AdvState::Done(t_res) => {
                    Advancement::new(AdvState::Done(Either::Left(t_res)), overeach)
                },
                AdvState::Error(e) => {
                    self.checking_u = true;
                    Advancement::new(AdvState::Advancing, overeach)
                },
            };

            return adv_return;
        }

        let adv = self.u.advance(&token);
        let overeach = adv.overeach;
        match adv.state {
            AdvState::Advancing => {
                Advancement::new(AdvState::Advancing, overeach)
            },
            AdvState::Done(u_res) => {
                Advancement::new(AdvState::Done(Either::Right(u_res)), overeach)
            },
            // TODO: simply error forwarding here, a compound error would probably be right'er
            AdvState::Error(e) => {
                Advancement::new(AdvState::Error(e), overeach)
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Requires both patterns to be valid in order (`T` → `U`).
pub(super) struct Then<'a, T, U>
where T: Pattern<'a>, U: Pattern<'a> {
    t: T,
    t_res: Option<T::ParseResult>,
    u: U,
    token_count: usize,
}

impl<'a, T, U> Default for Then<'a, T, U>
where T: Pattern<'a>, U: Pattern<'a> {
    fn default() -> Self {
        Then {
            t: T::default(),
            t_res: None,
            u: U::default(),
            token_count: 0,
        }
    }
}

impl<'a, T, U> Pattern<'a> for Then<'a, T, U>
where T: Pattern<'a>, U: Pattern<'a> {
    type ParseResult = (T::ParseResult, U::ParseResult);

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
        self.token_count += 1;

        if let Some(t_res) = &self.t_res {
            let adv = self.u.advance(&token);
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


        let adv = self.t.advance(&token);
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

#[cfg(test)]
mod tests {
    use crate::{lexer::token::TokenType, parser::{solve_pattern, terminals::*}, source::SfSlice};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token<'static> {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn or_token_pattern() {
        let token = bogus_token(TokenType::Ident("joe".to_string()));
        let tokens = vec![token];
        let res = solve_pattern::<Or::<IdentPattern, Or<NumLitPattern, CharLitPattern>>>(&tokens);
        if let Ok(_) = res {
            // yay
        } else {
            panic!("should have been valid")
        }

        let token = bogus_token(TokenType::CharLit('c'));
        let tokens = vec![token];
        let res = solve_pattern::<Or::<IdentPattern, Or<NumLitPattern, CharLitPattern>>>(&tokens);
        if let Ok(_) = res {
            // yay
        } else {
            panic!("should have been valid")
        }

        let token = bogus_token(TokenType::Plus);
        let tokens = vec![token];
        let res = solve_pattern::<Or::<IdentPattern, Or<NumLitPattern, CharLitPattern>>>(&tokens);
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

        let res = solve_pattern::<Then::<CharLitPattern, Then<MinusPattern, IdentPattern>>>(&tokens);
        if res.is_err() {
            panic!("did not complete")
        }

        let tokens = vec![
            bogus_token(TokenType::CharLit('c')),
            bogus_token(TokenType::Ident("a".to_string())),
            bogus_token(TokenType::Minus),
        ];
        let res = solve_pattern::<Then::<CharLitPattern, Then<MinusPattern, IdentPattern>>>(&tokens);
        if res.is_ok() {
            panic!("did complete")
        }
    }
}