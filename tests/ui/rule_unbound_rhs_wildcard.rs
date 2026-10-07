use rmath::prelude::*;

fn main() {
    symbols!(f, g);
    let _ = rule!(f(a_) => g(a_, b_));
}
