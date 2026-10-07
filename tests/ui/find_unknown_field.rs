use rmath::prelude::*;

fn main() {
    symbols!(f, x);
    let e = expr!(f(x, 1));
    for m in find!(e, f(a_, b_)) {
        let _ = m.c_;
    }
}
