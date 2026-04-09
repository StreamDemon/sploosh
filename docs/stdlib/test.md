# std::test

> Test framework and assertions.

**Available targets:** native (test builds only)

## Writing Tests

```sploosh
@test
fn test_addition() {
    assert(add(2, 3) == 5);
}
```

Use `#[cfg(test)]` to include test-only code.

```sploosh
#[cfg(test)]
mod tests {
    @test
    fn test_example() {
        // ...
    }
}
```

<!-- TODO: Document assertion functions, test runner options, and async test support once implemented -->
