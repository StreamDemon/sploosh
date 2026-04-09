# Runbook: Debugging Ownership Errors

> Common borrow checker errors and how to fix them.

## "Value used after move"

**Problem:** You used a value after it was moved to another variable or function.

```sploosh
let name = String::from("Alice");
let greeting = format("Hello, {}", name);
print(name);    // ERROR: name was moved into format
```

**Fix:** Borrow instead of move, or clone:

```sploosh
let name = String::from("Alice");
let greeting = format("Hello, {}", &name);  // borrow
print(name);    // OK
```

## "Cannot borrow as mutable -- already borrowed as immutable"

**Problem:** You have an immutable borrow active and try to take a mutable borrow.

**Fix:** Ensure the immutable borrow's scope ends before the mutable borrow begins.

## "Cannot move out of borrowed content"

**Problem:** You tried to move a value out of a reference.

**Fix:** Use `clone()` to create an owned copy, or use `ref` in pattern matching.

```sploosh
match user.role {
    Role::Editor { ref level } => format("editor-{}", level),  // borrow, don't move
    _ => "other".into(),
}
```

## Actor Method References

**Problem:** Using `&str` or `&T` in a `pub` actor method.

**Fix:** Use owned types (`String`, `T`) for all public actor method parameters.

```sploosh
// WRONG: pub fn log(&mut self, msg: &str)
// RIGHT:
pub fn log(&mut self, msg: String) { /* ... */ }
```

<!-- TODO: Add more patterns with actual compiler error messages once the compiler exists -->
