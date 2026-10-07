use rmath::prelude::*;

fn main() {
    symbols!(f);
    let _ = rule!(f(_) => 0);
}
