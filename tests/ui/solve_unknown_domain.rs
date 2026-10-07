use rmath::prelude::*;

fn main() {
    symbols!(x);
    let _ = solve!([x == 1] for x over Quaternions);
}
