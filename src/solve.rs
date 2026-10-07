//! Runtime support for `solve!`.

use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    poly::PolyVariable,
    solve::{Solution, SolutionSet, SolveDomain, SolveError},
};

/// Solve `equations` (each implicitly `= 0`) for `unknowns`, optionally over a domain.
pub fn solve(
    equations: &[Atom],
    unknowns: &[Atom],
    domain: Option<SolveDomain>,
) -> Result<SolutionSet, SolveError> {
    let builder = Atom::solve(equations);
    let builder = match domain {
        Some(domain) => builder.over(domain),
        None => builder,
    };
    builder.wrt(unknowns)
}

/// Read a solution branch by symbol instead of by `PolyVariable`.
pub trait SolutionExt {
    /// The expression assigned to `unknown` in this branch, if it is assigned.
    fn value(&self, unknown: Symbol) -> Option<&Atom>;
}

impl SolutionExt for Solution {
    fn value(&self, unknown: Symbol) -> Option<&Atom> {
        self.get(&PolyVariable::from(unknown))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::symbol;

    #[test]
    fn a_linear_system_has_a_point_solution_readable_by_symbol() {
        let (x, y) = symbol!("rmath_solve_x", "rmath_solve_y");
        let equations = [
            Atom::var(x) + Atom::var(y) - 3,
            Atom::var(x) - Atom::var(y) - 1,
        ];
        let solutions = solve(&equations, &[Atom::var(x), Atom::var(y)], None).unwrap();
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0].value(x), Some(&Atom::num(2)));
        assert_eq!(solutions[0].value(y), Some(&Atom::num(1)));
    }

    #[test]
    fn restricting_the_domain_can_empty_the_solution_set() {
        let x = symbol!("rmath_solve_real_x");
        let equations = [Atom::var(x) * Atom::var(x) + 1];
        let complex = solve(&equations, &[Atom::var(x)], None).unwrap();
        assert!(!complex.is_empty().unwrap());
        let real = solve(&equations, &[Atom::var(x)], Some(SolveDomain::Reals)).unwrap();
        assert!(real.is_empty().unwrap());
    }
}
