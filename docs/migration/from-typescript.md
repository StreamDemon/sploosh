# Coming from TypeScript

> Key mindset shifts for TypeScript developers learning Sploosh.

## Biggest Differences

### No Null/Undefined
TypeScript has `null`, `undefined`, and optional chaining. Sploosh has `Option<T>` -- you must explicitly handle the absence of a value.

```sploosh
// Instead of: user?.email ?? "default"
let email = find_user(42)
    |> map(|u| u.email)
    |> unwrap_or("default@example.com".into());
```

### No Exceptions
TypeScript uses `try`/`catch`. Sploosh uses `Result<T, E>` with `?` propagation.

```sploosh
// Instead of: try { ... } catch (e) { ... }
fn load(path: &str) -> Result<Config, AppError> {
    let data = fs::read(path)?;    // returns Err early
    Ok(Config::from(data))
}
```

### Ownership (New Concept)
Variables are moved, not copied (except primitives). You can't use a value after giving it to another function unless you borrow it.

```sploosh
let name = String::from("Alice");
greet(&name);     // borrow -- name is still valid
consume(name);    // move -- name is no longer valid
```

### No Classes
Use `struct` + `impl` + `trait` instead of classes and interfaces.

### Concurrency
Instead of `Promise`/`async`/`await` for everything, Sploosh has actors for stateful concurrency and `async`/`await` for I/O.

## Familiar Concepts
- `let`/`const` bindings
- Arrow-like syntax in match arms (`=>`)
- Generics (`<T>`)
- Pattern matching (like switch but exhaustive and more powerful)
- Module imports (`use` instead of `import`)
