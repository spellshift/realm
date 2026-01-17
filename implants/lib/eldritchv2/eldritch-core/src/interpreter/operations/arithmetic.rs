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
    use crate::token::{Span, TokenKind};

    fn dummy_span() -> Span {
        Span::new(0, 0, 1)
    }

    #[test]
    fn test_apply_arithmetic_op_int_basics() {
        let interp = Interpreter::new();
        let span = dummy_span();

        let a = Value::Int(10);
        let b = Value::Int(3);

        // +
        assert_eq!(
            apply_arithmetic_op(&interp, &a, &TokenKind::Plus, &b, span).unwrap(),
            Value::Int(13)
        );
        // -
        assert_eq!(
            apply_arithmetic_op(&interp, &a, &TokenKind::Minus, &b, span).unwrap(),
            Value::Int(7)
        );
        // *
        assert_eq!(
            apply_arithmetic_op(&interp, &a, &TokenKind::Star, &b, span).unwrap(),
            Value::Int(30)
        );
        // / -> Float
        assert!(matches!(
            apply_arithmetic_op(&interp, &a, &TokenKind::Slash, &b, span).unwrap(),
            Value::Float(v) if (v - 3.3333333333).abs() < 1e-9
        ));
    }

    #[test]
    fn test_apply_arithmetic_op_int_div_zero() {
        let interp = Interpreter::new();
        let span = dummy_span();

        let a = Value::Int(10);
        let b = Value::Int(0);

        let res = apply_arithmetic_op(&interp, &a, &TokenKind::Slash, &b, span);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind, EldritchErrorKind::ZeroDivisionError);

        let res = apply_arithmetic_op(&interp, &a, &TokenKind::SlashSlash, &b, span);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind, EldritchErrorKind::ZeroDivisionError);

        let res = apply_arithmetic_op(&interp, &a, &TokenKind::Percent, &b, span);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind, EldritchErrorKind::ZeroDivisionError);
    }

    #[test]
    fn test_apply_arithmetic_op_floor_div_python_semantics() {
        let interp = Interpreter::new();
        let span = dummy_span();

        // 10 // 3 = 3
        assert_eq!(
            apply_arithmetic_op(
                &interp,
                &Value::Int(10),
                &TokenKind::SlashSlash,
                &Value::Int(3),
                span
            )
            .unwrap(),
            Value::Int(3)
        );
        // -10 // 3 = -4
        assert_eq!(
            apply_arithmetic_op(
                &interp,
                &Value::Int(-10),
                &TokenKind::SlashSlash,
                &Value::Int(3),
                span
            )
            .unwrap(),
            Value::Int(-4)
        );
        // 10 // -3 = -4
        assert_eq!(
            apply_arithmetic_op(
                &interp,
                &Value::Int(10),
                &TokenKind::SlashSlash,
                &Value::Int(-3),
                span
            )
            .unwrap(),
            Value::Int(-4)
        );
        // -10 // -3 = 3
        assert_eq!(
            apply_arithmetic_op(
                &interp,
                &Value::Int(-10),
                &TokenKind::SlashSlash,
                &Value::Int(-3),
                span
            )
            .unwrap(),
            Value::Int(3)
        );
    }

    #[test]
    fn test_apply_arithmetic_op_modulo_python_semantics() {
        let interp = Interpreter::new();
        let span = dummy_span();

        // 10 % 3 = 1
        assert_eq!(
            apply_arithmetic_op(
                &interp,
                &Value::Int(10),
                &TokenKind::Percent,
                &Value::Int(3),
                span
            )
            .unwrap(),
            Value::Int(1)
        );
        // -10 % 3 = 2  (Python: -10 % 3 = 2, Rust: -10 % 3 = -1)
        assert_eq!(
            apply_arithmetic_op(
                &interp,
                &Value::Int(-10),
                &TokenKind::Percent,
                &Value::Int(3),
                span
            )
            .unwrap(),
            Value::Int(2)
        );
        // 10 % -3 = -2 (Python: 10 % -3 = -2)
        assert_eq!(
            apply_arithmetic_op(
                &interp,
                &Value::Int(10),
                &TokenKind::Percent,
                &Value::Int(-3),
                span
            )
            .unwrap(),
            Value::Int(-2)
        );
    }

    #[test]
    fn test_apply_arithmetic_op_mixed_types() {
        let interp = Interpreter::new();
        let span = dummy_span();

        // Int + Float
        match apply_arithmetic_op(
            &interp,
            &Value::Int(1),
            &TokenKind::Plus,
            &Value::Float(2.5),
            span,
        )
        .unwrap()
        {
            Value::Float(v) => assert!((v - 3.5).abs() < f64::EPSILON),
            _ => panic!("Expected Float"),
        }

        // Float + Int
        match apply_arithmetic_op(
            &interp,
            &Value::Float(2.5),
            &TokenKind::Plus,
            &Value::Int(1),
            span,
        )
        .unwrap()
        {
            Value::Float(v) => assert!((v - 3.5).abs() < f64::EPSILON),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_apply_arithmetic_op_mixed_div_zero() {
        let interp = Interpreter::new();
        let span = dummy_span();

        // Int / Float(0.0)
        let res = apply_arithmetic_op(
            &interp,
            &Value::Int(1),
            &TokenKind::Slash,
            &Value::Float(0.0),
            span,
        );
        assert!(res.is_err());

        // Float / Int(0)
        let res = apply_arithmetic_op(
            &interp,
            &Value::Float(1.0),
            &TokenKind::Slash,
            &Value::Int(0),
            span,
        );
        assert!(res.is_err());
    }
}
