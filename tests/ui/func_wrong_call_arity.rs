use rmath::prelude::*;

fn main() {
    symbols!(x, y);
    let mut f = func!(|x, y| x + y).unwrap();
    let _ = f(1.0);
}
