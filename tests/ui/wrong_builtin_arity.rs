use rmath::prelude::*;

fn main() {
    symbols!(x, y);
    let _ = expr!(sin(x, y));
    let _ = expr!(sqrt());
}
