use rmath::expr;

fn main() {
    let s = String::from("x");
    let _ = expr!(s + 1);
}
