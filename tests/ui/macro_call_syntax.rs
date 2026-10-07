use rmath::{expr, symbols};

fn main() {
    symbols!(x);
    let _ = expr!(f!(x) + 1);
}
