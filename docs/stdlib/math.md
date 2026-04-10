# std::math

> Numeric operations and mathematical constants.

**Available targets:**

- **Integer math** (abs, min, max, clamp, pow, isqrt, bit ops) -- all targets including `onchain`.
- **Floating-point math** (sin, cos, sqrt, exp, log, ...) -- native, wasm. **Forbidden in `onchain`** (see §12.3).

All methods in this module are compiler intrinsics that lower directly to LLVM intrinsics (see §13.0). The default correctness contract is within 1 ULP of the correctly-rounded IEEE 754 value; `@fast_math(afn)` relaxes this (see §12.1).

## Floating-Point Methods

Defined on `f32` and `f64`. Signatures below use `f64` for brevity; `f32` versions are identical with `f32` in place of `f64`.

### Classification

Inspect the IEEE 754 category of a value.

**Methods:** `is_nan`, `is_finite`, `is_infinite`, `is_normal`, `is_sign_positive`, `is_sign_negative`, `classify`

```sploosh
let x = 1.0f64 / 0.0f64;
if x.is_infinite() { /* ... */ }
match y.classify() {
    FpCategory::Nan      => "nan",
    FpCategory::Infinite => "infinite",
    FpCategory::Zero     => "zero",
    FpCategory::Subnormal => "subnormal",
    FpCategory::Normal   => "normal",
}
```

`FpCategory` is an enum in the prelude with the five IEEE 754 classes.

### Sign and Absolute Value

**Methods:** `abs`, `signum`, `copysign`

```sploosh
let a = (-3.5f64).abs();          // 3.5
let s = (-42.0f64).signum();      // -1.0
let c = 3.0f64.copysign(-1.0);    // -3.0
```

`signum` returns `1.0`, `-1.0`, or `NaN` (for NaN input).

### Rounding

**Methods:** `floor`, `ceil`, `round`, `trunc`, `fract`

```sploosh
let x = 3.7f64;
x.floor();   //  3.0
x.ceil();    //  4.0
x.round();   //  4.0  (ties-away-from-zero)
x.trunc();   //  3.0  (toward zero)
x.fract();   //  0.7  (x - x.trunc())
```

### Min, Max, Clamp

**Methods:** `min`, `max`, `clamp`

```sploosh
1.0f64.min(2.0);                 // 1.0
1.0f64.max(2.0);                 // 2.0
3.5f64.clamp(0.0, 1.0);          // 1.0
```

Follows IEEE 754-2019 `minimumNumber`/`maximumNumber` semantics: if one operand is NaN, the non-NaN operand is returned.

### Power and Root

**Methods:** `sqrt`, `cbrt`, `powi`, `powf`, `hypot`, `recip`

```sploosh
16.0f64.sqrt();              // 4.0 (correctly rounded)
27.0f64.cbrt();              // 3.0
2.0f64.powi(10);             // 1024.0 (integer exponent)
2.0f64.powf(0.5);            // 1.4142135623730951 (float exponent)
3.0f64.hypot(4.0);           // 5.0 (sqrt(3*3 + 4*4), overflow-safe)
4.0f64.recip();              // 0.25
```

`sqrt` and `mul_add` are correctly rounded on all targets. `hypot` is overflow-safe — it does not produce infinity for `(1e200).hypot(1e200)`.

### Exponential and Logarithm

**Methods:** `exp`, `exp2`, `exp_m1`, `ln`, `ln_1p`, `log`, `log2`, `log10`

```sploosh
1.0f64.exp();                // e ≈ 2.718
10.0f64.exp2();              // 1024.0
1.0f64.exp_m1();             // e - 1  (accurate near zero)
f64::E.ln();                 // 1.0
0.001f64.ln_1p();            // ln(1.001)  (accurate near zero)
100.0f64.log(10.0);          // 2.0  (log base 10)
1024.0f64.log2();            // 10.0
1000.0f64.log10();           // 3.0
```

Use `exp_m1` / `ln_1p` when the argument is close to 0 — they avoid catastrophic cancellation that plain `exp(x) - 1` and `ln(1 + x)` would suffer from.

### Trigonometry

**Methods:** `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sin_cos`

```sploosh
let theta = f64::PI / 4.0;
theta.sin();                 // 0.7071...
theta.cos();                 // 0.7071...
theta.tan();                 // 1.0

let (s, c) = theta.sin_cos();  // fuses to llvm.sincos

0.5f64.asin();               // π/6
(1.0f64).atan();             // π/4
(1.0f64).atan2(1.0);         // π/4 (quadrant-correct)
```

Use `sin_cos` whenever you need both — the compiler lowers it to `llvm.sincos`, which is roughly 2x faster than computing sin and cos separately. Adjacent `.sin()` and `.cos()` calls on the same input are also auto-fused by the optimizer.

### Hyperbolic

**Methods:** `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`

```sploosh
1.0f64.sinh();               // 1.1752...
1.0f64.cosh();               // 1.5430...
1.0f64.tanh();               // 0.7615...
```

### Fused Multiply-Add

**Method:** `mul_add`

```sploosh
// Returns a * b + c, computed with a single rounding step.
let r = a.mul_add(b, c);
```

`mul_add` is correctly rounded on all targets (uses `llvm.fma` when the target has hardware FMA; otherwise a software FMA). It is the preferred form for any `a*b + c` expression in numerically-sensitive code.

### Angle Conversion

**Methods:** `to_degrees`, `to_radians`

```sploosh
f64::PI.to_degrees();        // 180.0
180.0f64.to_radians();       // π
```

## Constants

Associated constants on `f32` and `f64`. Values below are for `f64`; `f32` has the same names.

### Mathematical

| Constant | Value | Description |
|---|---|---|
| `f64::PI` | 3.141592653589793 | π |
| `f64::TAU` | 6.283185307179586 | 2π |
| `f64::E` | 2.718281828459045 | Euler's number |
| `f64::SQRT_2` | 1.4142135623730951 | √2 |
| `f64::FRAC_1_SQRT_2` | 0.7071067811865476 | 1/√2 |
| `f64::LN_2` | 0.6931471805599453 | ln(2) |
| `f64::LN_10` | 2.302585092994046 | ln(10) |
| `f64::LOG2_E` | 1.4426950408889634 | log₂(e) |
| `f64::LOG10_E` | 0.4342944819032518 | log₁₀(e) |
| `f64::LOG2_10` | 3.321928094887362 | log₂(10) |
| `f64::LOG10_2` | 0.3010299956639812 | log₁₀(2) |

### IEEE 754 limits

| Constant | Description |
|---|---|
| `f64::INFINITY` | Positive infinity |
| `f64::NEG_INFINITY` | Negative infinity |
| `f64::NAN` | A quiet NaN |
| `f64::MAX` | Largest finite positive value (≈ 1.798e308) |
| `f64::MIN` | Most-negative finite value (= -MAX) |
| `f64::MIN_POSITIVE` | Smallest positive normal value (≈ 2.225e-308) |
| `f64::EPSILON` | Difference between 1.0 and the next representable value (≈ 2.22e-16) |
| `f64::MANTISSA_DIGITS` | Mantissa bit count (53 for f64) |
| `f64::DIGITS` | Approximate decimal digits of precision (15 for f64) |
| `f64::RADIX` | Floating-point radix (always 2) |

## Integer Math (on-chain safe)

All methods below are available on every integer type (`i8`..`i128`, `u8`..`u128`, `u256`) and on every target, **including `onchain`**. See §4.10 for the full catalog and §4.8 for checked/wrapping/saturating variants.

### Arithmetic

**Methods:** `abs` (signed types only), `min`, `max`, `clamp`, `pow`

```sploosh
(-5i64).abs();                   // 5   (aborts on i64::MIN)
10u64.min(20);                   // 10
10u64.max(20);                   // 20
15i32.clamp(0, 10);              // 10
2u64.pow(10);                    // 1024  (checked exponentiation)
```

`abs` on signed integer types aborts when called on the type's minimum value (`i64::MIN.abs()` cannot fit in `i64` under two's complement). To allow the wraparound (returning `i64::MIN` unchanged), place the call inside a function marked with `@overflow(wrapping)` — see §4.8.

### Roots and Logarithms

**Methods:** `isqrt`, `ilog2`, `ilog10`

```sploosh
144u64.isqrt();                  // 12   (floor of √144)
1000u64.ilog2();                 // 9    (floor of log₂(1000))
1000u64.ilog10();                // 3    (floor of log₁₀(1000))
```

`ilog2` and `ilog10` abort on zero input (the logarithm of zero is undefined).

### Bit Counting

**Methods:** `count_ones`, `count_zeros`, `leading_zeros`, `trailing_zeros`

```sploosh
0b10110100u8.count_ones();       // 4
0b10110100u8.count_zeros();      // 4
0b00010000u8.leading_zeros();    // 3
0b00010000u8.trailing_zeros();   // 4
```

### Bit Rotation and Byte Order

**Methods:** `rotate_left`, `rotate_right`, `swap_bytes`, `to_be`, `to_le`, `from_be`, `from_le`

```sploosh
0x12345678u32.rotate_left(8);    // 0x34567812
0x12345678u32.swap_bytes();      // 0x78563412
0x1234u16.to_be();               // 0x3412 on little-endian
```

Use these for on-chain hash and cryptography primitives operating on `u256`.

## Examples

### Euclidean distance (off-chain)

```sploosh
fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    dx.hypot(dy)  // overflow-safe
}
```

### Polar to Cartesian (off-chain)

```sploosh
fn polar_to_cartesian(r: f64, theta: f64) -> (f64, f64) {
    let (s, c) = theta.sin_cos();  // fused llvm.sincos
    (r * c, r * s)
}
```

### Square root of a u256 (on-chain safe)

```sploosh
onchain module Pool {
    pub fn price_sqrt(reserve_a: u256, reserve_b: u256) -> u256 {
        (reserve_a * reserve_b).isqrt()
    }
}
```

### Vector length with fast-math (off-chain)

```sploosh
@fast_math(contract, afn)
fn length(v: &[f64]) -> f64 {
    let mut sum = 0.0f64;
    for x in v { sum = sum + x * x; }  // compiler fuses to FMA
    sum.sqrt()
}
```

## See also

- §4.10 (Floating-Point and Math Operations) — language-level spec
- §4.8 (Integer Overflow) — checked/wrapping/saturating integer methods
- §12.1 (Fast-math flags) — `@fast_math` attribute semantics
- §12.3 (Conditional Compilation) — on-chain stdlib restrictions
- §13.0 (Compiler Intrinsics) — LLVM lowering table for all math methods
