use rmath::{expr, symbols};

fn main() {
    symbols!(x, y);
    let _ = expr!(2x);
    let _ = expr!(x y);
    let _ = expr!(2 (x + y));
}
