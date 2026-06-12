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

**Problem:** Using `&str` or `&T` as a parameter of a `pub fn (&mut self, ...)` actor method.

**Fix:** Use owned types (`String`, `T`) for the parameters of `&mut self` public actor methods (§8.2). `&self` request/reply methods *may* take references — the caller blocks for the reply, so the borrow is sound.

```sploosh
// WRONG: pub fn log(&mut self, msg: &str)
// RIGHT:
pub fn log(&mut self, msg: String) { /* ... */ }

// Also fine — &self request/reply may borrow (caller blocks):
pub fn lookup(&self, key: &str) -> Option<String> { /* ... */ }
```

If the fix-by-cloning feels expensive for large read-mostly data, reach for `Shared<T>` (§4.4a): the wrapper itself is an owned value (satisfying the `&mut self` owned-parameter rule) while the inner data is shared by an O(1) refcount bump instead of a deep clone.

<!-- TODO: Add more patterns with actual compiler error messages once the compiler exists -->
