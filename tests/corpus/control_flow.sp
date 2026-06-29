/// Control-flow shapes: `if`/`else` conditions and division.
/// Conditions end in bare paths/comparisons whose trailing operand sits right
/// before the block head — the struct-literal restriction (§5.1) keeps the
/// block from being swallowed as a literal.
fn classify(n: i64) -> i64 {
    let label = if n < 0 { 0 } else { 1 };
    label
}

fn ratio(a: i64, b: i64) -> i64 {
    a / b
}

fn nested(x: i64, y: i64) -> i64 {
    if x < y {
        x / 2
    } else if y < x {
        y / 2
    } else {
        0
    }
}
