# Runbook: Actor Debugging

> Diagnosing dead actors, mailbox issues, and supervisor configuration.

## Dead Actor Detection

When an actor dies, request/reply calls return `Err(ActorError::Dead)`. Fire-and-forget `send` silently drops messages.

```sploosh
match worker.process(data) {
    Ok(result) => use_result(result),
    Err(ActorError::Dead) => {
        log::warn("Worker died, spawning replacement");
        let worker = spawn Worker::init();
        worker.process(data)?
    }
    Err(e) => return Err(AppError::from(e)),
}
```

## Common Causes of Actor Death

- Out-of-bounds array/vector access
- Integer overflow (in debug builds)
- Failed `assert` calls
- Unhandled errors in message handlers

## Supervisor Strategy

```sploosh
@supervisor(strategy: "one_for_one", max_restarts: 3)
actor WorkerPool { /* ... */ }
```

- `one_for_one` -- only restart the failed child
- `max_restarts` -- maximum restarts within the restart window

<!-- TODO: Add more strategies, logging patterns, and debugging tools once the runtime is implemented -->
