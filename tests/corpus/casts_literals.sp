/// Numeric casts (§3.2 `as`, numeric-only) and the §16.1 literal zoo: based
/// integers, separators, suffixes, float exponents, string escapes, and
/// character literals. Lifetimes appear in generic params and reference types.
fn convert(n: i64, ratio: f64) -> u32 {
    let small = n as i32;
    let wide = small as i64;
    let scaled = ratio as f32;
    let back = scaled as f64;
    let total = wide + n;
    total as u32
}

fn literals() -> f64 {
    let hex = 0xFF_FFu32;
    let oct = 0o777;
    let bin = 0b1010_1010u8;
    let million = 1_000_000;
    let big = 340_282_366_920_938u128;
    let chain_cap = 115_792u256;
    let pi = 3.14159f64;
    let tiny = 2.5e-3;
    let large = 1e10f64;
    let greeting = "hi\n\t\\\"\x41\u{1F600}";
    let letter = 'a';
    let newline = '\n';
    let quote = '\'';
    let escaped = '\u{41}';
    let yes = true;
    let no = false;
    3.0
}

fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    a
}
