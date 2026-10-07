use rmath::prelude::*;

fn main() {
    symbols!(x);
    let _ = solve!([x ^ 2 - 1] for x);
}
