use rmath::prelude::*;

fn pick<A, B>(a: A, _b: B) -> A {
    a
}

fn main() {
    symbols!(f);
    // A top-level comma inside the guard needs parentheses: `if (pick::<Atom, i32>(a_, 1) ...)`.
    let _ = rule!(f(a_) => 0, if pick::<Atom, i32>(a_, 1) != expr!(1));
}
