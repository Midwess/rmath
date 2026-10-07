//! Rewrite rules produced by `rule!` and applied with [`ApplyRule::apply`].

use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::{Condition, ConditionResult, MatchStack, Pattern, PatternRestriction, Replacement},
};

/// One or more rewrites, applied together in a single pass.
///
/// `rule!(lhs => rhs)` holds one replacement; `rule! { a => b, c => d }` holds several.
pub struct Rule(Vec<Replacement>);

impl Rule {
    pub fn new(replacements: Vec<Replacement>) -> Self {
        Rule(replacements)
    }

    pub fn replacements(&self) -> &[Replacement] {
        &self.0
    }
}

/// Apply a [`Rule`] to any expression.
pub trait ApplyRule {
    fn apply(&self, rule: &Rule) -> Atom;
}

impl<T: AtomCore<Output = Atom>> ApplyRule for T {
    fn apply(&self, rule: &Rule) -> Atom {
        self.replace_multiple(rule.replacements())
    }
}

/// A rule guard: `pred` receives the atoms matched by `wildcards` (in that order) once all of
/// them are bound, and decides whether the match counts.
///
/// Until every listed wildcard has a value the restriction reports `Inconclusive`, so the
/// matcher keeps assigning instead of rejecting the partial match.
pub fn guard<const N: usize>(
    wildcards: [Symbol; N],
    pred: impl Fn([&Atom; N]) -> bool + Clone + Send + Sync + 'static,
) -> Condition<PatternRestriction> {
    let check = move |stack: &MatchStack<'_>| -> ConditionResult {
        let mut bound = Vec::with_capacity(N);
        for wildcard in &wildcards {
            match stack.get(*wildcard) {
                Some(m) => bound.push(m.to_atom()),
                None => return ConditionResult::Inconclusive,
            }
        }
        let refs: [&Atom; N] = std::array::from_fn(|i| &bound[i]);
        pred(refs).into()
    };
    Condition::from(PatternRestriction::MatchStack(Box::new(check)))
}

/// Every match of `pattern` inside `target`, as the atoms bound to `wildcards` in that order.
///
/// Collected eagerly: the pattern usually lives in the `find!` expansion's block and could
/// not be borrowed by a returned iterator.
pub fn find_all<T: AtomCore, const N: usize>(
    target: &T,
    pattern: &Pattern,
    wildcards: [Symbol; N],
) -> Vec<[Atom; N]> {
    target
        .pattern_match(pattern, None, None)
        .map(|mut bindings| {
            std::array::from_fn(|i| {
                bindings
                    .remove(&wildcards[i])
                    // `find!` passes exactly the wildcards the pattern binds.
                    .expect("pattern binds every listed wildcard")
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::{function, symbol};

    #[test]
    fn a_rule_applies_all_its_replacements_in_one_pass() {
        let (f, g, h, a_) = symbol!(
            "rmath_rule_f",
            "rmath_rule_g",
            "rmath_rule_h",
            "rmath_rule_a_"
        );
        let swap = Replacement::new(
            function!(f, Atom::var(a_)).to_pattern(),
            function!(g, Atom::var(a_)).to_pattern(),
        );
        let zero = Replacement::new(function!(h, Atom::var(a_)).to_pattern(), Atom::num(0));
        let rule = Rule::new(vec![swap, zero]);

        let e = function!(f, Atom::num(1)) + function!(h, Atom::num(2)) + Atom::num(7);
        assert_eq!(e.apply(&rule), function!(g, Atom::num(1)) + Atom::num(7));
    }

    #[test]
    fn a_guard_sees_the_bound_wildcards_and_filters_matches() {
        let (f, g, a_) = symbol!("rmath_guard_f", "rmath_guard_g", "rmath_guard_a_");
        let rule = Rule::new(vec![
            Replacement::new(
                function!(f, Atom::var(a_)).to_pattern(),
                function!(g, Atom::var(a_)).to_pattern(),
            )
            .when(guard([a_], |[a]| *a != Atom::num(1))),
        ]);

        let e = function!(f, Atom::num(1)) + function!(f, Atom::num(2));
        assert_eq!(
            e.apply(&rule),
            function!(f, Atom::num(1)) + function!(g, Atom::num(2))
        );
    }

    #[test]
    fn find_all_returns_the_bound_atoms_of_every_match() {
        let (f, x, y, a_, b_) = symbol!(
            "rmath_find_f",
            "rmath_find_x",
            "rmath_find_y",
            "rmath_find_a_",
            "rmath_find_b_"
        );
        let e = function!(f, Atom::num(1), Atom::num(2)) + function!(f, Atom::var(x), Atom::var(y));
        let pattern = function!(f, Atom::var(a_), Atom::var(b_)).to_pattern();

        let mut found: Vec<String> = find_all(&e, &pattern, [a_, b_])
            .into_iter()
            .map(|[a, b]| format!("{a},{b}"))
            .collect();
        found.sort();
        let mut expected = vec![
            "1,2".to_string(),
            format!("{},{}", Atom::var(x), Atom::var(y)),
        ];
        expected.sort();
        assert_eq!(found, expected);
    }
}
