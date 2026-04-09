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

`?` (precedence 12) binds tighter than `|>` (precedence 8). Use `?` on each fallible stage:

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

## Pipe with Iterators

Pipe and method chains are interchangeable for iterator operations:

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
| `iter \|> method(args)` | `iter.method(args)` |

## Next Steps

- [Actors and Concurrency](actors-and-concurrency.md)
- [Error Handling](error-handling.md)
