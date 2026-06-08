//! Defines operators, and how they operate

use thiserror::Error;

use crate::{compiler::{expression::{Expression, ExpressionEvaluationError}, scope::Scope, value::{PropertyError, Value}}, newtype_wrapper};

use crate::parser::{BinaryOperator as ParsedBinaryOperator, LeftAssociativeUnaryOperator as ParsedLeftAssociativeUnaryOperator, RightAssociativeUnaryOperator as ParsedRightAssociativeUnaryOperator};

newtype_wrapper!(BinaryOperator, ParsedBinaryOperator);
newtype_wrapper!(LeftAssociativeUnaryOperator, ParsedLeftAssociativeUnaryOperator);
newtype_wrapper!(RightAssociativeUnaryOperator, ParsedRightAssociativeUnaryOperator);

/// An error which occured while evaluating an operation.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum OperationError {
    #[error("an over/underflowed occured")]
    Overflow,
    #[error("a division by zero occured")]
    DivisionByZero,
    #[error("tried to index out of bound (indicies: {:?}, length: {length})", &indices)]
    IndexOutOfBound {
        indices: Vec<i32>,
        length: usize
    },
    #[error("{inner}")]
    PropertyError { // during property operation
        inner: PropertyError
    },
    #[error("{op_name} cannot be done between a {lhs_type} and a {rhs_type}")]
    InvalidTypeBinary {
        lhs_type: String,
        op_name: String,
        rhs_type: String,
    },
    #[error("{op_name} cannot be done with a {value_type}")]
    InvalidTypeUnary {
        value_type: String,
        op_name: String,
    },
    #[error("{inner}")]
    CouldNotEvaluateIndex {
        inner: Box<ExpressionEvaluationError>,
        expression: Expression,
    },
}

impl BinaryOperator {
    pub fn evaluate(&self, lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        match self.0 {
            ParsedBinaryOperator::Plus(_) => Self::add(lhs, rhs),
            ParsedBinaryOperator::Minus(_) => Self::substract(lhs, rhs),
            ParsedBinaryOperator::Multiply(_) => Self::multiply(lhs, rhs),
            ParsedBinaryOperator::Divide(_) => Self::divide(lhs, rhs),
            ParsedBinaryOperator::Modulo(_) => Self::modulo(lhs, rhs),
            ParsedBinaryOperator::LogicalEqual(_) => Self::equal(lhs, rhs),
            ParsedBinaryOperator::LogicalInequal(_) => Self::inequal(lhs, rhs),
            ParsedBinaryOperator::GreaterThan(_) => Self::greater_than(lhs, rhs),
            ParsedBinaryOperator::GreaterThanEqual(_) => Self::greater_than_equal(lhs, rhs),
            ParsedBinaryOperator::LessThan(_) => Self::lesser_than(lhs, rhs),
            ParsedBinaryOperator::LessThanEqual(_) => Self::lesser_than_equal(lhs, rhs),
            ParsedBinaryOperator::LogicalOr(_) => Self::logical_or(lhs, rhs),
            ParsedBinaryOperator::LogicalAnd(_) => Self::logical_and(lhs, rhs),
            ParsedBinaryOperator::Property(_) => Self::property(lhs, rhs),
        }
    }

    fn add(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            // number, number (adds)
            (Value::Number(lhs_n), Value::Number(rhs_n)) => {
                let Some (res) = lhs_n.checked_add(rhs_n) else {
                    return Err(OperationError::Overflow);
                };
                
                Ok(Value::Number(res))
            },
            // list, list (appends)
            (Value::List(mut lhs_l), Value::List(mut rhs_l)) => {
                lhs_l.append(&mut rhs_l);
                Ok(Value::List(lhs_l))
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "addition".to_string(), rhs_type })
        }
    }

    fn substract(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            // number, number (substracts)
            (Value::Number(lhs_n), Value::Number(rhs_n)) => {
                let Some (res) = lhs_n.checked_sub(rhs_n) else {
                    return Err(OperationError::Overflow);
                };
                
                Ok(Value::Number(res))
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "substraction".to_string(), rhs_type })
        }
    }

    fn multiply(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            // number, number (multiply)
            (Value::Number(lhs_n), Value::Number(rhs_n)) => {
                let Some (res) = lhs_n.checked_mul(rhs_n) else {
                    return Err(OperationError::Overflow);
                };
                
                Ok(Value::Number(res))
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "multiplication".to_string(), rhs_type })
        }
    }

    fn divide(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            // number, number (divide)
            (Value::Number(lhs_n), Value::Number(rhs_n)) => {
                if rhs_n == 0 {
                    return Err(OperationError::DivisionByZero);
                }

                let Some (res) = lhs_n.checked_div(rhs_n) else {
                    return Err(OperationError::Overflow);
                };
                
                Ok(Value::Number(res))
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "division".to_string(), rhs_type })
        }
    }

    fn modulo(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            // number, number (modulo)
            (Value::Number(lhs_n), Value::Number(rhs_n)) => {
                if rhs_n == 0 {
                    return Err(OperationError::DivisionByZero);
                }

                Ok(Value::Number(lhs_n % rhs_n))
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "modulus".to_string(), rhs_type })
        }
    }

    fn equal(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            (Value::Number(lhs_n), Value::Number(rhs_n)) => {
                let res = if lhs_n == rhs_n {
                    Value::TRUE
                } else {
                    Value::FALSE
                };

                Ok(res)
            },
            // item wise equality
            (Value::List(lhs_l), Value::List(rhs_l)) => {
                if lhs_l.len() != rhs_l.len() {
                    return Ok(Value::FALSE);
                }

                let all_items_are_equal = lhs_l.into_iter().zip(rhs_l.into_iter())
                    .find_map(|(l, r)| {
                        let res = BinaryOperator::equal(l, r);
                        if res.is_err() || res.unwrap() == Value::FALSE {
                            Some(())
                        } else {
                            None
                        }
                    })
                    .is_none();

                let res = if all_items_are_equal {
                    Value::TRUE
                } else {
                    Value::FALSE
                };
                
                Ok(res)
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "equality".to_string(), rhs_type })
        }
    }

    fn inequal(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        RightAssociativeUnaryOperator::not(Self::equal(lhs, rhs)?)
    }

    fn greater_than(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            (Value::Number(lhs_n), Value::Number(rhs_n)) => {
                let res = if lhs_n > rhs_n {
                    Value::TRUE
                } else {
                    Value::FALSE
                };

                Ok(res)
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "comparaison".to_string(), rhs_type })
        }
    }

    fn greater_than_equal(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            (Value::Number(lhs_n), Value::Number(rhs_n)) => {
                let res = if lhs_n >= rhs_n {
                    Value::TRUE
                } else {
                    Value::FALSE
                };

                Ok(res)
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "comparaison".to_string(), rhs_type })
        }
    }

    fn lesser_than(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        RightAssociativeUnaryOperator::not(Self::greater_than_equal(lhs, rhs)?)
    }

    fn lesser_than_equal(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        RightAssociativeUnaryOperator::not(Self::greater_than(lhs, rhs)?)
    }

    fn logical_or(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (&lhs, &rhs) {
            (Value::Number(_), Value::Number(_)) => {
                let res = if lhs != Value::FALSE || rhs != Value::FALSE {
                    Value::TRUE
                } else {
                    Value::FALSE
                };

                Ok(res)
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "logical or".to_string(), rhs_type })
        }
    }

    fn logical_and(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (&lhs, &rhs) {
            (Value::Number(_), Value::Number(_)) => {
                let res = if lhs != Value::FALSE && rhs != Value::FALSE {
                    Value::TRUE
                } else {
                    Value::FALSE
                };

                Ok(res)
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "logical and".to_string(), rhs_type })
        }
    }

    fn property(lhs: Value, rhs: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let rhs_type = rhs.type_name().to_string();

        match (lhs, rhs) {
            // indexing by number
            (value, property) if property.as_string().is_some() => {
                let property_value = value.get_property(&property.as_string().unwrap())
                    .map_err(|err| OperationError::PropertyError { inner: err })?;

                Ok(property_value)
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "finding a property".to_string(), rhs_type })
        }
    }
}

impl LeftAssociativeUnaryOperator {
    pub fn evaluate(&self, ctx: &Scope, arg: Value) -> Result<Value, OperationError> {
        match &self.0 {
            ParsedLeftAssociativeUnaryOperator::Index(_, index, _) => {
                let index = <&Expression>::from(&**index);
                let index_value = index.evaluate(ctx)
                    .map_err(|err| OperationError::CouldNotEvaluateIndex { inner: Box::new(err), expression: index.clone() })?;

                Self::index(arg, index_value)
            },
        }
    }

    fn index(lhs: Value, index: Value) -> Result<Value, OperationError> {
        let lhs_type = lhs.type_name().to_string();
        let index_type = index.type_name().to_string();

        match (lhs, index) {
            // indexing by number
            (Value::List(mut lhs_l), Value::Number(index_n)) => {
                // tries to convert into index
                let Ok(index): Result<usize, _> = index_n.try_into() else {
                    return Err(OperationError::IndexOutOfBound { indices: vec![index_n], length: lhs_l.len() });
                };

                // tries to get the item
                let Some(element) = lhs_l.try_remove(index) else {
                    return Err(OperationError::IndexOutOfBound { indices: vec![index_n], length: lhs_l.len() });
                };

                Ok(element)
            },
            // indexing by list of number
            (Value::List(lhs_l), Value::List(index_l)) => {
                let mut return_list = Vec::new();

                for index_value in index_l {
                    let Value::Number(index_n) = index_value else {
                        return Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "indexing".to_string(), rhs_type: index_value.type_name().to_string() });
                    };

                    // tries to convert into index
                    let Ok(index): Result<usize, _> = index_n.try_into() else {
                        return Err(OperationError::IndexOutOfBound { indices: vec![index_n], length: lhs_l.len() });
                    };

                    // tries to get the item
                    let Some(element) = lhs_l.get(index) else {
                        return Err(OperationError::IndexOutOfBound { indices: vec![index_n], length: lhs_l.len() });
                    };

                    // this is bad, cloning can lose a lot of performance
                    return_list.push(element.clone());
                }

                Ok(Value::List(return_list))
            },
            _ => Err(OperationError::InvalidTypeBinary { lhs_type, op_name: "indexing".to_string(), rhs_type: index_type })
        }
    }
}

impl RightAssociativeUnaryOperator {
    pub fn evaluate(&self, arg: Value) -> Result<Value, OperationError> {
        match self.0 {
            ParsedRightAssociativeUnaryOperator::UnaryPlus(_) => Self::unary_plus(arg),
            ParsedRightAssociativeUnaryOperator::UnaryMinus(_) => Self::unary_minus(arg),
            ParsedRightAssociativeUnaryOperator::LogicalNot(_) => Self::not(arg),
        }
    }

    fn unary_plus(value: Value) -> Result<Value, OperationError> {
        let value_type = value.type_name().to_string();

        // only on numbers
        let Value::Number(_) = value else {
            return Err(OperationError::InvalidTypeUnary { value_type, op_name: "unary plus".to_string() });
        };

        Ok(value)
    }

    fn unary_minus(value: Value) -> Result<Value, OperationError> {
        let value_type = value.type_name().to_string();

        // only on numbers
        let Value::Number(num) = value else {
            return Err(OperationError::InvalidTypeUnary { value_type, op_name: "unary plus".to_string() });
        };

        let Some(num) = num.checked_neg() else {
            return Err(OperationError::Overflow);
        };

        Ok(Value::Number(num))
    }

    fn not(value: Value) -> Result<Value, OperationError> {
        let value_type = value.type_name().to_string();

        // only on numbers
        let Value::Number(_) = value else {
            return Err(OperationError::InvalidTypeUnary { value_type, op_name: "logical not".to_string() });
        };

        if value == Value::FALSE {
            Ok(Value::TRUE)
        } else {
            Ok(Value::FALSE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Value::*;

    use std::assert_matches;

    #[test]
    fn add_two_numbers() {
        let res = BinaryOperator::add(Number(3), Number(-42)).unwrap();
        assert_eq!(res, Number(-39));
    }

    #[test]
    fn add_two_numbers_overflow() {
        let res = BinaryOperator::add(Number(3), Number(i32::MAX)).unwrap_err();
        assert_eq!(res, OperationError::Overflow);
    }

    #[test]
    fn add_two_lists() {
        let res = BinaryOperator::add(List(vec![Number(1)]), List(vec![Number(2), Number(3), List(Vec::new())])).unwrap();
        assert_eq!(res, List(vec![Number(1), Number(2), Number(3), List(Vec::new())]));
    }

    #[test]
    fn add_two_empty_lists() {
        let res = BinaryOperator::add(List(Vec::new()), List(Vec::new())).unwrap();
        assert_eq!(res, List(Vec::new()));
    }

    #[test]
    fn substract_two_numbers() {
        let res = BinaryOperator::substract(Number(3), Number(-42)).unwrap();
        assert_eq!(res, Number(45));
    }

    #[test]
    fn substract_two_numbers_overflow() {
        let res = BinaryOperator::substract(Number(-2), Number(i32::MAX)).unwrap_err();
        assert_eq!(res, OperationError::Overflow);
    }

    #[test]
    fn divide_two_numbers_lossless() {
        let res = BinaryOperator::divide(Number(12), Number(3)).unwrap();
        assert_eq!(res, Number(4));
    }

    #[test]
    fn divide_two_numbers_truncates_decimal_part() {
        let res = BinaryOperator::divide(Number(100), Number(6)).unwrap();
        assert_eq!(res, Number(16));
    }

    #[test]
    fn divide_two_numbers_by_zero() {
        let res = BinaryOperator::divide(Number(-2), Number(0)).unwrap_err();
        assert_eq!(res, OperationError::DivisionByZero);
    }

    #[test]
    fn modulo_two_numbers_lossless() {
        let res = BinaryOperator::modulo(Number(10), Number(3)).unwrap();
        assert_eq!(res, Number(1));
    }

    #[test]
    fn modulo_two_numbers_by_zero() {
        let res = BinaryOperator::modulo(Number(-2), Number(0)).unwrap_err();
        assert_eq!(res, OperationError::DivisionByZero);
    }

    #[test]
    fn modulo_two_numbers_by_negative() {
        let res = BinaryOperator::modulo(Number(100), Number(-6)).unwrap();
        assert_eq!(res, Number(4));
    }

    #[test]
    fn equal_two_numbers_true_when_equal() {
        let res = BinaryOperator::equal(Number(10), Number(10)).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn equal_two_numbers_false_when_not_equal() {
        let res = BinaryOperator::equal(Number(10), Number(3)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn equal_two_lists_true_when_equal() {
        let res = BinaryOperator::equal(List(vec![Number(10), List(Vec::new()), List(vec![Number(1), Number(2)])]), List(vec![Number(10), List(Vec::new()), List(vec![Number(1), Number(2)])])).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn equal_two_lists_false_when_not_equal() {
        let res = BinaryOperator::equal(List(vec![Number(10), List(Vec::new()), List(vec![Number(1), Number(2), Number(3)])]), List(vec![Number(10), List(Vec::new()), List(vec![Number(1), Number(2)])])).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn inequal_two_numbers_false_when_equal() {
        let res = BinaryOperator::inequal(Number(10), Number(10)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn inequal_two_numbers_true_when_not_equal() {
        let res = BinaryOperator::inequal(Number(10), Number(3)).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn inequal_two_lists_false_when_equal() {
        let res = BinaryOperator::inequal(List(vec![Number(10), List(Vec::new()), List(vec![Number(1), Number(2)])]), List(vec![Number(10), List(Vec::new()), List(vec![Number(1), Number(2)])])).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn inequal_two_lists_true_when_not_equal() {
        let res = BinaryOperator::inequal(List(vec![Number(10), List(Vec::new()), List(vec![Number(1), Number(2), Number(3)])]), List(vec![Number(10), List(Vec::new()), List(vec![Number(1), Number(2)])])).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn greater_than_strictly_greater_true() {
        let res = BinaryOperator::greater_than(Number(4), Number(-1)).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn greater_than_strictly_lesser_false() {
        let res = BinaryOperator::greater_than(Number(-1), Number(4)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn greater_than_equal_false() {
        let res = BinaryOperator::greater_than(Number(4), Number(4)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn greater_than_equal_strictly_greater_true() {
        let res = BinaryOperator::greater_than_equal(Number(4), Number(-1)).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn greater_than_equal_strictly_lesser_false() {
        let res = BinaryOperator::greater_than_equal(Number(-1), Number(4)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn greater_than_equal_equal_true() {
        let res = BinaryOperator::greater_than_equal(Number(4), Number(4)).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn lesser_than_strictly_greater_false() {
        let res = BinaryOperator::lesser_than(Number(4), Number(-1)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn lesser_than_strictly_lesser_true() {
        let res = BinaryOperator::lesser_than(Number(-1), Number(4)).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn lesser_than_equal_false() {
        let res = BinaryOperator::lesser_than(Number(4), Number(4)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn lesser_than_equal_strictly_greater_false() {
        let res = BinaryOperator::lesser_than_equal(Number(4), Number(-1)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn lesser_than_equal_strictly_lesser_true() {
        let res = BinaryOperator::lesser_than_equal(Number(-1), Number(4)).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn lesser_than_equal_equal_true() {
        let res = BinaryOperator::lesser_than_equal(Number(4), Number(4)).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn logical_or_both_true_returns_true() {
        let res = BinaryOperator::logical_or(Number(2), Value::TRUE).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn logical_or_single_true_returns_true() {
        let res = BinaryOperator::logical_or(Number(-5), Value::FALSE).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn logical_or_both_false_returns_true() {
        let res = BinaryOperator::logical_or(Value::FALSE, Value::FALSE).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn logical_and_both_true_returns_true() {
        let res = BinaryOperator::logical_and(Number(2), Value::TRUE).unwrap();
        assert_eq!(res, Value::TRUE);
    }

    #[test]
    fn logical_and_single_true_returns_false() {
        let res = BinaryOperator::logical_and(Number(-5), Value::FALSE).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn logical_and_both_false_returns_true() {
        let res = BinaryOperator::logical_and(Value::FALSE, Value::FALSE).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn index_empty_list_errors() {
        let res = LeftAssociativeUnaryOperator::index(List(Vec::new()), Number(0)).unwrap_err();
        assert_matches!(res, OperationError::IndexOutOfBound { .. });
    }

    #[test]
    fn index_list_with_integer_in_bounds() {
        let res = LeftAssociativeUnaryOperator::index(List(vec![Number(1), List(Vec::new())]), Number(1)).unwrap();
        assert_eq!(res, List(Vec::new()));
    }

    #[test]
    fn index_list_with_negative_index_errors() {
        let res = LeftAssociativeUnaryOperator::index(List(vec![Number(1), List(Vec::new())]), Number(-1)).unwrap_err();
        assert_matches!(res, OperationError::IndexOutOfBound { .. });
    }

    #[test]
    fn index_list_with_index_over_bound_errors() {
        let res = LeftAssociativeUnaryOperator::index(List(vec![Number(1), List(Vec::new())]), Number(2)).unwrap_err();
        assert_matches!(res, OperationError::IndexOutOfBound { .. });
    }

    #[test]
    fn index_list_with_empty_list_returns_empty_list() {
        let res = LeftAssociativeUnaryOperator::index(List(vec![Number(1), List(Vec::new())]), List(Vec::new())).unwrap();
        assert_eq!(res, List(Vec::new()));
    }

    #[test]
    fn index_list_with_list_single_item_returns_single_item_list() {
        let res = LeftAssociativeUnaryOperator::index(List(vec![Number(1), List(Vec::new())]), List(vec![Number(1)])).unwrap();
        assert_eq!(res, List(vec![List(Vec::new())]));
    }

    #[test]
    fn index_list_with_list_many_items_returns_many_items_list() {
        let res = LeftAssociativeUnaryOperator::index(List(vec![Number(1), List(Vec::new()), Number(3)]), List(vec![Number(0), Number(2)])).unwrap();
        assert_eq!(res, List(vec![Number(1), Number(3)]));
    }

    #[test]
    fn index_list_with_list_many_items_out_of_bounds_errors() {
        let res = LeftAssociativeUnaryOperator::index(List(vec![Number(1), List(Vec::new()), Number(3)]), List(vec![Number(0), Number(4)])).unwrap_err();
        assert_matches!(res, OperationError::IndexOutOfBound { .. });
    }

    #[test]
    fn index_list_with_list_as_element_errors() {
        let res = LeftAssociativeUnaryOperator::index(List(vec![Number(1), List(Vec::new()), Number(3)]), List(vec![Number(0), List(Vec::new())])).unwrap_err();
        assert_matches!(res, OperationError::InvalidTypeBinary { .. });
    }

    #[test]
    fn not_non_zero_becomes_false() {
        let res = RightAssociativeUnaryOperator::not(Number(3)).unwrap();
        assert_eq!(res, Value::FALSE);
    }

    #[test]
    fn not_zero_becomes_true() {
        let res = RightAssociativeUnaryOperator::not(Number(0)).unwrap();
        assert_eq!(res, Value::TRUE);
    }
}
