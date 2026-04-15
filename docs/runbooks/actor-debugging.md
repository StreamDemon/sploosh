# Runbook: Actor Debugging

> Diagnosing dead actors, mailbox issues, supervisor configuration, and re-entrant deadlocks.

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
- Panic inside `init` (actor transitions `INITIALIZING → DEAD` without ever reaching `READY`)

## Diagnosing `ActorError::SelfCall`

`SelfCall` means a handler issued a synchronous request/reply call on its own actor's handle (directly or via a cloned copy). The runtime detects this in O(1) and returns the error instead of letting the actor hang forever on itself.

Typical shape:

```sploosh
pub async fn process(&mut self, job: Job) -> Result<Report, ActorError> {
    // SelfCall: request/reply on own handle via a field.
    let n = self.self_handle.as_ref().unwrap().count()?;
    // ...
}
```

**Fix:** call the local method directly on `self`, or if you genuinely want re-entrant behavior, use a fire-and-forget self-send that enqueues to the actor's own mailbox for the next handler turn:

```sploosh
send self.self_handle.as_ref().unwrap().retry(job);
```

## Diagnosing Multi-Actor Deadlock

Symptom: actor A and actor B appear healthy but a user-level timeout eventually fires; neither actor processes new mail. Cause: a synchronous call chain A → B → A (or longer) has closed a cycle, and each actor is waiting for the next to return.

The current runtime does **not** detect multi-actor cycles — only direct self-calls surface as `SelfCall`. Resolution:

1. Identify the handler that's blocked. Under a debug build, each handler logs its current request/reply target.
2. Restructure the call graph as a DAG — one side of the cycle should fetch state proactively and pass it as an argument rather than calling back.
3. Alternatively, break the cycle with fire-and-forget `send`: replace the synchronous back-call with an enqueued message that completes after the outer handler returns.
4. Wrap cross-actor calls in an outer `send_timeout` so the deadlock surfaces as `SendError::Timeout` rather than an indefinite hang.

## Diagnosing `SendError::Dead` on a Blocked Sender

When `send_timeout` on a full mailbox returns `Err(SendError::Dead)` before the timeout elapses, the destination actor died *while the sender was blocked waiting*. This is not a retry-able condition against the same handle — the supervisor may have restarted the actor with a new handle, but blocked senders are **never** transparently redirected.

Resolution:

1. Re-fetch the current handle from the supervisor's public API.
2. Retry the operation against the new handle.
3. If the actor is unsupervised, treat `Dead` as terminal — spawn a new instance or propagate the error.

## Detecting Init-Loop Supervisor Kill

Symptom: a supervised actor dies immediately after spawn, the supervisor restarts it, and the cycle repeats until the supervisor itself dies and cascades. Cause: `init` is panicking on every restart (for example, reading a misconfigured field, or asserting against an impossible precondition).

Because init failures count toward `max_restarts`, a bad config can exhaust the window quickly. Check:

- `init` implementations for runtime checks that depend on arguments passed in by the supervisor.
- The arguments the supervisor stores at spawn time — they are replayed verbatim on every restart, so a misconfiguration is permanent until the supervisor itself is respawned.
- Runtime logs for `PanicMessage` variants attached to the `ActorError` (see §8.8 of the spec).

## Supervisor Strategy

```sploosh
@supervisor(strategy: "one_for_one", max_restarts: 3, window_secs: 60)
actor WorkerPool { /* ... */ }
```

- `one_for_one` -- only restart the failed child
- `max_restarts` -- maximum restarts within the sliding `window_secs` window
- `window_secs` -- sliding wall-clock window over each restart's timestamp

On restart: the child is always recreated via a **fresh `init`** with the supervisor's stored arguments. Old handles become permanently dead and are not transparently redirected.

<!-- TODO: Add supervisor log shapes and a step-by-step incident playbook once the runtime is implemented -->
