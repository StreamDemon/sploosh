/// Struct literals (§5.1 incl. the parenthesized block-head escape and
/// shorthand field init), nesting, and the boolean/comparison operator set.
struct Point {
    pub x: i64,
    pub y: i64,
}

struct Cfg {
    pub on: bool,
}

fn build(x: i64, y: i64) -> Point {
    let origin = Point { x: 0, y: 0 };
    let shorthand = Point { x, y };
    let nested = Wrap { inner: Point { x: 1, y: 2 } };
    origin
}

fn guarded() -> i64 {
    if (Cfg { on: true }).on {
        1
    } else {
        2
    }
}

fn logic(a: bool, b: bool, lo: i64, hi: i64) -> bool {
    let both = a && b;
    let either = a || b;
    let cmp = lo <= hi;
    let ne = lo != hi;
    let modulo = hi % 7 == 0;
    both && either && cmp && ne && modulo
}
