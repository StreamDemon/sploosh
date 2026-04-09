# Operator Precedence

> Complete precedence table from highest to lowest.

| Prec | Operator | Meaning | Assoc |
|------|----------|---------|-------|
| 14 | `.` | Field/method access | Left |
| 13 | `()` `[]` | Call, Index | Left |
| 12 | `?` | Error propagation | Post |
| 12 | `as` | Numeric cast | Left |
| 11 | `!` | Logical NOT | Pre |
| 10 | `*` `/` `%` | Multiply, Divide, Modulo | Left |
| 9 | `+` `-` | Add, Subtract | Left |
| 8 | `\|>` | Pipe | Left |
| 7 | `<` `>` `<=` `>=` | Comparison | Left |
| 6 | `==` `!=` | Equality | Left |
| 5 | `&&` | Logical AND | Left |
| 4 | `\|\|` | Logical OR | Left |
| 3 | `..` `..=` | Range, Inclusive Range | None |
| 2 | `=` | Assignment | Right |
| 1 | `=>` | Match arm / Lambda | Right |
| 0 | `->` | Return type annotation | Right |

## Key Interaction: Pipe + Error Propagation

Since `?` (12) binds tighter than `|>` (8):

```sploosh
expr |> f?    // parsed as (expr |> f)?  =  f(expr)?
```

## Sigils

| Sigil | Meaning |
|-------|---------|
| `&` | Immutable reference (borrow) |
| `&mut` | Mutable reference (borrow) |
| `@` | Attribute / decorator |
| `#` | Compiler directive |
| `::` | Path separator / type access |
| `:` | Type annotation |
