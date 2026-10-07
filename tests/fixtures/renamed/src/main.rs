//! Every macro must work when the dependency is called `rm` instead of `rmath`.

use rm::prelude::*;

fn main() {
    symbols!(x, y, f);
    let e = expr!(f(x, y) + sin(x) ^ 2);
    let drop_f = rule!(f(a_, b_) => a_ * b_);
    let rewritten = e.apply(&drop_f);
    let matches = find!(rewritten, sin(a_)).count();
    let sols = solve!([2 * x == 4] for x).expect("solvable");
    let mut g = func!(|x| x ^ 2).expect("compilable");
    println!(
        "{rewritten} | sin matches: {matches} | x = {} | g(3) = {}",
        sols[0].value(x).expect("assigned"),
        g(3.0)
    );
}
