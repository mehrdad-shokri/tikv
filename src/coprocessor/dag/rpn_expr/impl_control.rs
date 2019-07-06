// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

use cop_codegen::rpn_fn;

use crate::coprocessor::codec::data_type::*;
use crate::coprocessor::Result;

#[rpn_fn]
#[inline]
fn if_null<T: Evaluable>(lhs: &Option<T>, rhs: &Option<T>) -> Result<Option<T>> {
    if lhs.is_some() {
        return Ok(lhs.clone());
    }
    Ok(rhs.clone())
}

#[rpn_fn(raw_varg)]
#[inline]
pub fn case_when<T: Evaluable>(args: &[ScalarValueRef<'_>]) -> Result<Option<T>> {
    for chunk in args.chunks(2) {
        if chunk.len() == 1 {
            // else statement
            // TODO: Must verify type
            let ret: &Option<T> = Evaluable::borrow_scalar_value_ref(&chunk[0]);
            return Ok(ret.clone());
        }
        let cond: &Option<Int> = Evaluable::borrow_scalar_value_ref(&chunk[0]);
        if cond.unwrap_or(0) != 0 {
            // TODO: Must verify type
            let ret: &Option<T> = Evaluable::borrow_scalar_value_ref(&chunk[1]);
            return Ok(ret.clone());
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    use tipb::expression::ScalarFuncSig;

    use crate::coprocessor::dag::rpn_expr::test_util::RpnFnScalarEvaluator;

    #[test]
    fn test_if_null() {
        let cases = vec![
            (None, None, None),
            (None, Some(1), Some(1)),
            (Some(2), None, Some(2)),
            (Some(2), Some(1), Some(2)),
        ];
        for (lhs, rhs, expected) in cases {
            let output = RpnFnScalarEvaluator::new()
                .push_param(lhs)
                .push_param(rhs)
                .evaluate(ScalarFuncSig::IfNullInt)
                .unwrap();
            assert_eq!(output, expected, "lhs={:?}, rhs={:?}", lhs, rhs);
        }
    }

    #[test]
    fn test_case_when() {
        let cases: Vec<(Vec<ScalarValue>, Option<Real>)> = vec![
            (
                vec![1.into(), (3.0).into(), 1.into(), (5.0).into()],
                Real::new(3.0).ok(),
            ),
            (
                vec![0.into(), (3.0).into(), 1.into(), (5.0).into()],
                Real::new(5.0).ok(),
            ),
            (
                vec![ScalarValue::Int(None), (2.0).into(), 1.into(), (6.0).into()],
                Real::new(6.0).ok(),
            ),
            (vec![(7.0).into()], Real::new(7.0).ok()),
            (vec![0.into(), ScalarValue::Real(None)], None),
            (vec![1.into(), ScalarValue::Real(None)], None),
            (vec![1.into(), (3.5).into()], Real::new(3.5).ok()),
            (vec![2.into(), (3.5).into()], Real::new(3.5).ok()),
            (
                vec![
                    0.into(),
                    ScalarValue::Real(None),
                    ScalarValue::Int(None),
                    ScalarValue::Real(None),
                    (5.5).into(),
                ],
                Real::new(5.5).ok(),
            ),
        ];

        for (args, expected) in cases {
            let mut evaluator = RpnFnScalarEvaluator::new();
            for arg in args {
                evaluator = evaluator.push_param(arg);
            }
            let output = evaluator.evaluate(ScalarFuncSig::CaseWhenReal).unwrap();
            assert_eq!(output, expected);
        }
    }
}