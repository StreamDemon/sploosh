# Strings and Formatting

> `str` vs `String`, the `format()` intrinsic, format specifiers, and string methods.

## str vs String

- `str` -- immutable string slice (borrowed reference, typically `&str`)
- `String` -- owned, heap-allocated, growable string

```sploosh
let slice: &str = "hello";                    // string literal is &str
let owned: String = String::from("hello");    // owned String
let also: String = "hello".into();            // &str -> String via Into
let back: &str = &owned;                      // String -> &str via auto-deref
```

## The format() Function

`format` is a compiler intrinsic. No f-strings, no template literals. One way to format strings.

```sploosh
let s = format("Hello, {}!", name);
let s = format("{} is {} years old", name, age);
let s = format("Pi is {:.4}", 3.14159);
```

## Format Specifiers

| Specifier | Meaning | Example |
|-----------|---------|---------|
| `{}` | Default display | `"42"` |
| `{:?}` | Debug representation | `"[1, 2, 3]"` |
| `{:.N}` | N decimal places | `"3.14"` |
| `{:x}` / `{:X}` | Hex lower/upper | `"ff"` |
| `{:b}` | Binary | `"1010"` |
| `{:o}` | Octal | `"10"` |
| `{:>N}` | Right-align, width N | `"        hi"` |
| `{:<N}` | Left-align, width N | `"hi        "` |
| `{:0N}` | Zero-pad, width N | `"000042"` |

Types used with `{}` must implement `Display`. Types used with `{:?}` must implement `Debug`.

## String Methods

**On `str`:** `len`, `is_empty`, `contains`, `starts_with`, `ends_with`, `find`, `trim`, `trim_start`, `trim_end`, `to_uppercase`, `to_lowercase`, `replace`, `split`, `chars`, `as_bytes`

**On `String` (additional):** `push_str`, `push`, `clear`, `into_bytes`

## No + Concatenation

No operator overloading means `+` is always arithmetic. Use `format` or `push_str`:

```sploosh
let full = format("{} {}", first_name, last_name);

let mut s = String::from("hello");
s.push_str(" world");
```

## Next Steps

- [Modules and Visibility](modules-and-visibility.md)
- [Getting Started](getting-started.md) -- revisit the basics
