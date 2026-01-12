use crate::ast::Value;
use crate::interpreter::core::Interpreter;
use crate::interpreter::error::{EldritchError, EldritchErrorKind};
use crate::interpreter::introspection::get_type_name;
use crate::token::{Span, TokenKind};
use alloc::format;

#[cfg(feature = "std")]
extern crate std;

pub(crate) fn apply_arithmetic_op(
    interp: &Interpreter,
    a: &Value,
    op: &TokenKind,
    b: &Value,
    span: Span,
) -> Result<Value, EldritchError> {
    match (a, op, b) {
        (Value::Int(a), TokenKind::Plus, Value::Int(b)) => Ok(Value::Int(a + b)),
        (Value::Int(a), TokenKind::Minus, Value::Int(b)) => Ok(Value::Int(a - b)),
        (Value::Int(a), TokenKind::Star, Value::Int(b)) => Ok(Value::Int(a * b)),
        (Value::Int(a), TokenKind::Slash, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            Ok(Value::Float((*a as f64) / (*b as f64)))
        }

        (Value::Float(a), TokenKind::Plus, Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Float(a), TokenKind::Minus, Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Float(a), TokenKind::Star, Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Float(a), TokenKind::Slash, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            Ok(Value::Float(a / b))
        }

        // Mixed
        (Value::Int(a), TokenKind::Plus, Value::Float(b)) => Ok(Value::Float((*a as f64) + b)),
        (Value::Int(a), TokenKind::Minus, Value::Float(b)) => Ok(Value::Float((*a as f64) - b)),
        (Value::Int(a), TokenKind::Star, Value::Float(b)) => Ok(Value::Float((*a as f64) * b)),
        (Value::Int(a), TokenKind::Slash, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            Ok(Value::Float((*a as f64) / b))
        }

        (Value::Float(a), TokenKind::Plus, Value::Int(b)) => Ok(Value::Float(a + (*b as f64))),
        (Value::Float(a), TokenKind::Minus, Value::Int(b)) => Ok(Value::Float(a - (*b as f64))),
        (Value::Float(a), TokenKind::Star, Value::Int(b)) => Ok(Value::Float(a * (*b as f64))),
        (Value::Float(a), TokenKind::Slash, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            Ok(Value::Float(a / (*b as f64)))
        }

        // Floor Div and Modulo
        (Value::Float(a), TokenKind::SlashSlash, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.div_euclid(*b)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(a / b)))
            }
        }
        (Value::Int(a), TokenKind::SlashSlash, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float((*a as f64).div_euclid(*b)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(*a as f64 / b)))
            }
        }
        (Value::Float(a), TokenKind::SlashSlash, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.div_euclid(*b as f64)))
            }
            #[cfg(not(feature = "std"))]
            {
                Ok(Value::Float(libm::floor(a / *b as f64)))
            }
        }
        (Value::Float(a), TokenKind::Percent, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "modulo by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.rem_euclid(*b)))
            }
            #[cfg(not(feature = "std"))]
            {
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
        }
        (Value::Int(a), TokenKind::Percent, Value::Float(b)) => {
            if *b == 0.0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "modulo by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float((*a as f64).rem_euclid(*b)))
            }
            #[cfg(not(feature = "std"))]
            {
                let a = *a as f64;
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
        }
        (Value::Float(a), TokenKind::Percent, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "modulo by zero", span);
            }
            #[cfg(feature = "std")]
            {
                Ok(Value::Float(a.rem_euclid(*b as f64)))
            }
            #[cfg(not(feature = "std"))]
            {
                let b = *b as f64;
                let div = libm::floor(a / b);
                Ok(Value::Float(a - b * div))
            }
        }

        (Value::Int(a), TokenKind::SlashSlash, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "divide by zero", span);
            }
            let mut res = a / b;
            if (a % b != 0) && ((*a < 0) ^ (*b < 0)) {
                res -= 1;
            }
            Ok(Value::Int(res))
        }
        (Value::Int(a), TokenKind::Percent, Value::Int(b)) => {
            if *b == 0 {
                return interp.error(EldritchErrorKind::ZeroDivisionError, "modulo by zero", span);
            }
            let res = ((a % b) + b) % b;
            Ok(Value::Int(res))
        }

        _ => interp.error(
            EldritchErrorKind::TypeError,
            &format!(
                "unsupported operand type(s) for {}: '{}' and '{}'",
                match op {
                    TokenKind::Plus => "+",
                    TokenKind::Minus => "-",
                    TokenKind::Star => "*",
                    TokenKind::Slash => "/",
                    TokenKind::SlashSlash => "//",
                    TokenKind::Percent => "%",
                    _ => "?",
                },
                get_type_name(a),
                get_type_name(b)
            ),
            span,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::core::Interpreter;
    use crate::token::Span;

    // Helper to run ops
    fn run_op(a: Value, op: TokenKind, b: Value) -> Result<Value, EldritchError> {
        let interp = Interpreter::new();
        let span = Span::new(0, 0, 1);
        apply_arithmetic_op(&interp, &a, &op, &b, span)
    }

    #[test]
    fn test_int_arithmetic() {
        assert_eq!(
            run_op(Value::Int(1), TokenKind::Plus, Value::Int(2)).unwrap(),
            Value::Int(3)
        );
        assert_eq!(
            run_op(Value::Int(5), TokenKind::Minus, Value::Int(2)).unwrap(),
            Value::Int(3)
        );
        assert_eq!(
            run_op(Value::Int(3), TokenKind::Star, Value::Int(2)).unwrap(),
            Value::Int(6)
        );
        assert_eq!(
            run_op(Value::Int(6), TokenKind::Slash, Value::Int(2)).unwrap(),
            Value::Float(3.0)
        );
    }

    #[test]
    fn test_float_arithmetic() {
        assert_eq!(
            run_op(Value::Float(1.5), TokenKind::Plus, Value::Float(2.5)).unwrap(),
            Value::Float(4.0)
        );
        assert_eq!(
            run_op(Value::Float(5.5), TokenKind::Minus, Value::Float(2.0)).unwrap(),
            Value::Float(3.5)
        );
    }

    #[test]
    fn test_mixed_arithmetic() {
        assert_eq!(
            run_op(Value::Int(1), TokenKind::Plus, Value::Float(2.5)).unwrap(),
            Value::Float(3.5)
        );
        assert_eq!(
            run_op(Value::Float(1.5), TokenKind::Plus, Value::Int(2)).unwrap(),
            Value::Float(3.5)
        );
        assert_eq!(
            run_op(Value::Int(3), TokenKind::Slash, Value::Float(2.0)).unwrap(),
            Value::Float(1.5)
        );
    }

    #[test]
    fn test_zero_division() {
        let err = run_op(Value::Int(1), TokenKind::Slash, Value::Int(0));
        assert!(matches!(
            err.unwrap_err().kind,
            EldritchErrorKind::ZeroDivisionError
        ));

        let err = run_op(Value::Float(1.0), TokenKind::Slash, Value::Float(0.0));
        assert!(matches!(
            err.unwrap_err().kind,
            EldritchErrorKind::ZeroDivisionError
        ));

        let err = run_op(Value::Int(1), TokenKind::SlashSlash, Value::Int(0));
        assert!(matches!(
            err.unwrap_err().kind,
            EldritchErrorKind::ZeroDivisionError
        ));
    }

    #[test]
    fn test_modulo_python_behavior() {
        // 5 % 3 = 2
        assert_eq!(
            run_op(Value::Int(5), TokenKind::Percent, Value::Int(3)).unwrap(),
            Value::Int(2)
        );
        // -5 % 3 = 1
        assert_eq!(
            run_op(Value::Int(-5), TokenKind::Percent, Value::Int(3)).unwrap(),
            Value::Int(1)
        );
        // 5 % -3 = -1
        assert_eq!(
            run_op(Value::Int(5), TokenKind::Percent, Value::Int(-3)).unwrap(),
            Value::Int(-1)
        );
        // -5 % -3 = -2
        assert_eq!(
            run_op(Value::Int(-5), TokenKind::Percent, Value::Int(-3)).unwrap(),
            Value::Int(-2)
        );
    }

    #[test]
    fn test_floor_div_python_behavior() {
        // 5 // 2 = 2
        assert_eq!(
            run_op(Value::Int(5), TokenKind::SlashSlash, Value::Int(2)).unwrap(),
            Value::Int(2)
        );
        // -5 // 2 = -3
        assert_eq!(
            run_op(Value::Int(-5), TokenKind::SlashSlash, Value::Int(2)).unwrap(),
            Value::Int(-3)
        );
        // 5.0 // 2.0 = 2.0
        assert_eq!(
            run_op(Value::Float(5.0), TokenKind::SlashSlash, Value::Float(2.0)).unwrap(),
            Value::Float(2.0)
        );
        // -5.0 // 2.0 = -3.0
        assert_eq!(
            run_op(Value::Float(-5.0), TokenKind::SlashSlash, Value::Float(2.0)).unwrap(),
            Value::Float(-3.0)
        );
    }
}
