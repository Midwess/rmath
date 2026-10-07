use rmath::prelude::*;

fn main() {
    symbols!(f);
    let _ = rule!(f(a_) => 0, if b_ != expr!(1));
}
