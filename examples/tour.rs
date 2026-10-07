//! A tour of every rmath macro. Run with `cargo run --example tour`.

// Single Greek letters as symbol names trip rustc's confusables lint; that is intended here.
#![allow(mixed_script_confusables)]

use rmath::prelude::*;

fn main() {
    // symbols! — compiler-checked declarations, attributes like trait bounds.
    symbols!(x, y, θ: Real, κ: Real + Positive, f: Symmetric);

    // expr! — math syntax, Rust values by name, result is a Symbolica `Atom`.
    let k = 3;
    let e = expr!(k * x ^ 2 + sin(y) / 2 - f(y, x));
    println!("expr!    {e}");
    println!("expand   {}", expr!((x + 1) ^ 3).expand());
    println!("d/dx     {}", expr!(x ^ 3 + sin(x)).derivative(x));

    // rule! — rewrite rules with wildcards and guards; `.apply` from the prelude.
    let swap = rule!(f(a_, b_) => a_ - b_);
    println!("rule!    {}", e.apply(&swap));
    // `g` is not symmetric, so each call matches exactly once (a symmetric `f` would also
    // be tried with its arguments swapped).
    symbols!(g);
    let drop_unless_one = rule!(g(a_, b_) => 0, if a_ != expr!(1));
    println!("guard    {}", expr!(g(1, 2) + g(3, 4)).apply(&drop_unless_one));

    // find! — matches with wildcards as named fields.
    let sums = expr!(g(1, 2) + g(x, y) + 7);
    for m in find!(sums, g(a_, b_)) {
        println!("find!    a_ = {}, b_ = {}", m.a_, m.b_);
    }

    // solve! — systems written with `==`, optional domain.
    let sols = solve!([2 * x + y == 3, x - y == 0] for x, y).expect("linear system");
    println!(
        "solve!   x = {}, y = {}",
        sols[0].value(x).expect("x assigned"),
        sols[0].value(y).expect("y assigned")
    );
    let reals = solve!([x ^ 2 + 1 == 0] for x over Reals).expect("solvable");
    println!("over R   solutions: {}", reals.len());

    // func! — compile to a fast f64 closure (FnMut: bind with `let mut`).
    let torque = expr!(-κ * sin(θ));
    let mut tau = func!(|θ, κ| torque).expect("compilable");
    println!("func!    τ(0.1, 4.9) = {:.4}", tau(0.1, 4.9));
    println!("5! = {}, (x+1)! = {}", expr!(5!), expr!((x + 1)!));
}
