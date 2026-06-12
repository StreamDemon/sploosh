# The Pipe Operator

> Sploosh's signature `|>` operator for readable data transformation chains.

## Basic Usage

The pipe operator passes the left-hand value as the **first argument** to the right-hand function:

```sploosh
let result = raw_input |> parse_json |> validate |> serialize;
// Equivalent to: serialize(validate(parse_json(raw_input)))
```

## Multi-Argument Functions

Piped value is always the first argument:

```sploosh
fn add(a: i64, b: i64) -> i64 { a + b }
let result = 10 |> add(5);     // add(10, 5) = 15
```

## Other Argument Positions

There is no placeholder syntax. Use a closure:

```sploosh
let result = 10 |> (|v| multiply(3, v));   // multiply(3, 10) = 30
```

## Pipe + Error Propagation

A trailing `?` on a pipe stage is part of the pipe grammar itself (spec §5.7,
§16 `pipe_stage`) and applies to the accumulated pipe result: `expr |> f?`
parses as `(expr |> f)?`, i.e. `f(expr)?`. Use `?` on each fallible stage:

```sploosh
let report = raw_input
    |> parse_json?        // parse_json(raw_input)? -- unwrap or return Err
    |> validate?          // validate(parsed)?
    |> transform?;        // transform(valid)?
```

Mixed chains (fallible and infallible):

```sploosh
let output = raw_input
    |> trim                 // infallible, no ?
    |> parse_json?          // fallible, needs ?
    |> extract_name;        // infallible, no ?
```

## Pipe with Methods

When the piped value's type has a method matching the stage name,
`expr |> method(args)` desugars to `expr.method(args)` — for **any** receiver
type, not just iterators (the method wins over a same-named free function;
otherwise the stage is a free-function call with the piped value as first
argument). Iterator chains are the most common case:

```sploosh
// These are identical:
let names = users.iter().filter(|u| u.active).map(|u| u.name.clone()).collect();
let names = users.iter() |> filter(|u| u.active) |> map(|u| u.name.clone()) |> collect();
```

## Rules Summary

| Expression | Desugars To |
|-----------|-------------|
| `x \|> f` | `f(x)` |
| `x \|> f(a, b)` | `f(x, a, b)` |
| `x \|> f?` | `f(x)?` |
| `x \|> method(args)` | `x.method(args)` (any receiver whose type has the method; method wins over a same-named free function) |

## Next Steps

- [Actors and Concurrency](actors-and-concurrency.md)
- [Error Handling](error-handling.md)
