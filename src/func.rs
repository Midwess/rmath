//! Runtime support for `func!`: compile an expression to a fast `f64` evaluator.

use symbolica::{
    atom::{Atom, AtomCore},
    evaluate::{EvaluationError, ExpressionEvaluator},
};

/// Build an `f64` evaluator of `body` over `params` (positional).
///
/// Symbolica compiles with exact complex-rational coefficients first; they are then mapped to
/// their real `f64` value, so an `f64` closure can call `evaluate_single` directly.
pub fn compile_f64(
    body: &Atom,
    params: &[Atom],
) -> Result<ExpressionEvaluator<f64>, EvaluationError> {
    let exact = body.evaluator(params).build()?;
    Ok(exact.map_coeff(&|c| c.re.to_f64()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::{atom::AtomCore, symbol};

    #[test]
    fn a_compiled_expression_evaluates_positionally() {
        let (x, y) = symbol!("rmath_func_x", "rmath_func_y");
        let body = Atom::var(x).pow(Atom::num(2)) + Atom::var(y).sin();
        let mut f = compile_f64(&body, &[Atom::var(x), Atom::var(y)]).unwrap();
        assert_eq!(f.evaluate_single(&[3.0, 0.0]), 9.0);
        assert!((f.evaluate_single(&[0.0, std::f64::consts::FRAC_PI_2]) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_free_symbol_in_the_body_is_an_error_not_a_panic() {
        let (x, y) = symbol!("rmath_func_free_x", "rmath_func_free_y");
        let body = Atom::var(x) + Atom::var(y);
        assert!(compile_f64(&body, &[Atom::var(x)]).is_err());
    }
}
