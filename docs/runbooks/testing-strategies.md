# Runbook: Testing Strategies

> Unit tests, integration tests, and on-chain test patterns.

## Unit Tests

Use `@test` attribute and `#[cfg(test)]` for test modules:

```sploosh
#[cfg(test)]
mod tests {
    use crate::math;

    @test
    fn test_add() {
        assert(math::add(2, 3) == 5);
    }

    @test
    fn test_division_error() {
        let result = math::divide(10, 0);
        match result {
            Err(MathError::DivisionByZero) => {},
            _ => assert(false),
        }
    }
}
```

## Running Tests

```bash
sploosh test                     # Run all tests
sploosh test --filter "test_add" # Run specific test (once supported)
```

## Integration Tests

Place in `tests/` directory:

```
tests/
└── integration.sp
```

## On-Chain Test Patterns

<!-- TODO: Document on-chain testing (mock ctx, storage setup, event assertions) once implemented -->

## Testing Actors

<!-- TODO: Document actor testing patterns (spawning test actors, asserting messages, timeout handling) once implemented -->
