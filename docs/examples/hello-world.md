# Example: Hello World

> The minimal Sploosh program.

## main.sp

```sploosh
fn main() -> Result<(), AppError> {
    print("Hello, Sploosh!");
    Ok(())
}
```

## Build and Run

```bash
sploosh run
```

## What's Happening

- `fn main()` -- entry point, must return `Result<(), AppError>`
- `print()` -- built-in function (prelude), not a macro
- `Ok(())` -- explicit success return with unit value
