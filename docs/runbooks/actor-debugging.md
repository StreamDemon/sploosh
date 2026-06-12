# Runbook: Actor Debugging

> Diagnosing dead actors, mailbox issues, supervisor configuration, re-entrant deadlocks, and how to use the §8.12 observability surface to triage all of the above.

This runbook is the operational counterpart to §8.12 of `LANGUAGE_SPEC.md`. The full API lives in [`docs/stdlib/actor.md`](../stdlib/actor.md); the language guide is [`docs/guide/actors-and-concurrency.md`](../guide/actors-and-concurrency.md).

## Pre-conditions

- A working `sploosh` toolchain.
- A package with a `sploosh.toml` manifest.
- Code that spawns at least one actor — the `std::actor::observe` surface is `not onchain` and is a compile error inside `onchain` modules (§11.1, §12.3).

The observability surface is **always available, every build mode** — there is no `@observable` attribute, no debug-only gating, no feature flag. The recipes below work identically against `sploosh build` and `sploosh build --release`.

## Recipe: Is this actor stuck?

Symptom: a request/reply call you expect to return promptly is hanging, or `send` calls back up faster than they drain.

Diagnose with mailbox depth and liveness:

```sploosh
use std::actor::observe;

fn diagnose(target: &Handle<Worker>) {
    let len = target.mailbox_len();
    let cap = target.mailbox_capacity();
    let alive = target.alive();
    let id = target.actor_id();

    log::info(format("worker {:?} alive={} mailbox={}/{}", id, alive, len, cap));

    if !alive {
        // Dead-actor recipe below.
        return;
    }

    if len >= cap {
        log::warn("mailbox full — senders are blocked on backpressure");
    } else if len > cap * 8 / 10 {
        log::warn("mailbox at >80% — actor is falling behind");
    }
}
```

If `alive()` is `true` but `mailbox_len()` keeps climbing, the handler is taking longer than messages arrive — the actor isn't *stuck*, it's *slow*. Profile the handler.

If `alive()` is `true`, `mailbox_len()` is non-trivial, *and* the count never decreases, the handler is wedged. Common causes:

- A request/reply call into another actor that is itself blocked (multi-actor cycle — see "Diagnosing multi-actor deadlock" below).
- A direct self-call that should have surfaced as `ActorError::SelfCall` but didn't because the handler is using fire-and-forget `send` to itself in a way that re-enqueues faster than it drains.
- Blocking FFI through an `extern "C"` (not `extern "C" async`) that the compile-time check missed because the call is transitive through a non-actor crate. The compile-time check is not a runtime bypass — file a bug.

## Recipe: Why did this actor die?

Symptom: request/reply against a handle returns `Err(ActorError::Dead)` and `send` silently drops.

Each dead actor retains a snapshot — including its cause of death — for as long as any `Handle<T>` clone remains live (§8.12.4). Query it:

```sploosh
use std::actor::observe;

fn triage(handle: &Handle<Worker>) -> String {
    match observe::actor_info(handle) {
        None => "snapshot already gc'd (no live handle clones remain)".into(),
        Some(info) => match info.death_cause {
            None => format("worker {:?} is {:?}", info.id, info.lifecycle_state),
            Some(DeathCause::RuntimeFailure { panic }) =>
                format("worker {:?} died: {}", info.id, panic),
            Some(DeathCause::Stopped) =>
                format("worker {:?} was stopped cooperatively", info.id),
            Some(DeathCause::Killed) =>
                format("worker {:?} was killed", info.id),
            Some(DeathCause::Supervised { restart_pending }) =>
                format("worker {:?} terminated by supervisor (restart_pending={})",
                       info.id, restart_pending),
            Some(DeathCause::RuntimeShutdown) =>
                format("worker {:?} dropped on runtime shutdown", info.id),
        },
    }
}
```

`DeathCause::RuntimeFailure { panic }` is the smoking gun for a runtime check tripping (bounds, overflow, failed `assert`). `DeathCause::Supervised { restart_pending: true }` means the supervisor is mid-restart — the *next* spawn produces a new handle, and the old handle stays dead (§8.7a).

To preserve the snapshot for post-mortem analysis, **keep at least one `Handle<T>` clone alive** until you've finished querying. Once the last clone drops, the snapshot is GC'd and `actor_info` returns `None`.

## Recipe: Which actors are pinning memory right now?

Symptom: process RSS climbs over time and you suspect an actor leak.

```sploosh
use std::actor::observe;

fn report_top_mailboxes(n: usize) {
    let mut all: Vec<ActorInfo> = observe::actors().collect();
    all.sort_by(|a, b| b.mailbox_len.cmp(&a.mailbox_len));
    for info in all.iter().take(n) {
        log::info(format(
            "{} ({:?}): mailbox={}/{} state={:?}",
            info.name, info.id, info.mailbox_len, info.mailbox_capacity, info.lifecycle_state
        ));
    }
}
```

`observe::actors()` walks the runtime registry and is O(N_actors) — fine for triage, not for hot paths. Filter further with `.by_supervisor(&sup)` or `.by_name("Worker")` when you already know the suspect class.

Holding the iterator across an `.await` inside an actor handler is permitted but inadvisable: it pins snapshots and observably delays GC of dead-actor entries.

## Recipe: What does our supervisor tree look like right now?

A real supervisor tree is heterogeneous — a `RootSupervisor` whose children include a `WorkerPoolSupervisor`, a `LoggingSupervisor`, and so on. There is no `dyn Supervisor` trait object to recurse through typed handles, so `children()` (which lives only on `@supervisor`-decorated handles, §8.12.3) is the right tool for one-shot listing of a *known* supervisor's direct children but not for a generic tree walk.

For the tree walk, recurse via `ActorId` against the registry instead. `observe::actors()` plus `ActorInfo.supervisor` (§8.12.2) does the job without needing a typed handle for each intermediate node:

```sploosh
use std::actor::observe;

fn dump_tree(root: &Handle<RootSupervisor>) {
    let root_info = observe::actor_info(root)
        .expect("root supervisor must have a live snapshot");
    log::info(format("{} ({:?})", root_info.name, root_info.id));
    walk_by_supervisor(root_info.id, 1);
}

fn walk_by_supervisor(parent_id: ActorId, depth: usize) {
    let indent: String = " ".repeat(depth * 2);
    for info in observe::actors().filter(|i| i.supervisor == Some(parent_id)) {
        log::info(format("{}- {} ({:?}) state={:?}", indent, info.name, info.id, info.lifecycle_state));
        walk_by_supervisor(info.id, depth + 1);
    }
}
```

`children()` on a `@supervisor`-decorated handle still enumerates its current children in supervisor-order (the same order `rest_for_one` uses; §8.7a) when you already hold a typed handle to that specific supervisor. Combine with `restart_count(&child)` to spot a child that is restarting in a tight loop:

```sploosh
fn restart_storm_audit(sup: &Handle<WorkerPool>) {
    for child_info in sup.children() {
        // Reconstruct a handle for the child — typically via the supervisor's pub API.
        // Here we assume `sup.child_handle(id)` is exposed by the supervisor:
        let child = sup.child_handle(child_info.id);
        match sup.restart_count(&child) {
            Ok(n) if n > 10 => log::warn(format("child {:?} restarted {} times", child_info.id, n)),
            Ok(n) => log::info(format("child {:?} restarted {} times", child_info.id, n)),
            Err(ObserveError::NotASupervisedChild) => log::error("child not supervised here"),
        }
    }
}
```

For a richer post-mortem, walk `restart_history(&child)` — each `RestartEvent` carries `timestamp_ms_since_spawn` and the `cause: DeathCause` of the death that triggered the restart. Default cap is 16 events; tune per-supervisor with `@supervisor(restart_history: N)`.

## Detecting init-loop supervisor kill

Symptom: a supervised actor dies immediately after spawn, the supervisor restarts it, and the cycle repeats until the supervisor itself dies and cascades.

Cause: `init` is panicking on every restart (typically a misconfigured field, or an `assert` against an impossible precondition).

Diagnose by walking `restart_history`:

```sploosh
match sup.restart_history(&child) {
    Ok(events) if events.len() >= 2 => {
        let first = &events[0];
        let last  = &events[events.len() - 1];
        let span_ms = last.timestamp_ms_since_spawn - first.timestamp_ms_since_spawn;
        log::warn(format("{} restarts in {}ms — likely init loop", events.len(), span_ms));
        for e in events {
            if let DeathCause::RuntimeFailure { panic } = &e.cause {
                log::warn(format("  t={}ms panic={}", e.timestamp_ms_since_spawn, panic));
            }
        }
    }
    _ => {}
}
```

Because init failures count toward `max_restarts` (§8.7a), a bad config can exhaust the window quickly. Fix:

- Audit `init` for runtime checks that depend on supervisor-passed arguments.
- The supervisor replays the **same arguments** verbatim on every restart (§8.7a), so a misconfiguration is permanent until the supervisor itself respawns.
- Convert recoverable initialization into a post-`init` handshake message: store an `Option<T>` field, run a `pub fn ready(&mut self, ...)` to populate it, and let `init` always succeed.

## Diagnosing `ActorError::SelfCall`

`SelfCall` means a handler issued a synchronous request/reply call on its own actor's handle (directly or via a cloned copy stored in a field). The runtime detects this in O(1) and returns the error instead of letting the actor hang on itself.

Typical shape:

```sploosh
pub async fn process(&mut self, job: Job) -> Result<Report, ActorError> {
    // SelfCall: request/reply on own handle via a field.
    let n = self.self_handle.as_ref().unwrap().count()?;
    // ...
}
```

**Fix:** call the local method directly on `self`, or — if you genuinely want re-entrant behavior — use a fire-and-forget self-send that enqueues to the actor's own mailbox for the next handler turn:

```sploosh
send self.self_handle.as_ref().unwrap().retry(job);
```

Self-stop and self-kill via `self.handle.stop()` / `.kill()` are **not** `SelfCall` — the signal is observed after the current handler returns (§8.2a, §8.10.1).

## Diagnosing multi-actor deadlock

Symptom: actors A and B appear healthy (`alive() == true` for both), but a user-level timeout eventually fires and neither actor processes new mail. Cause: a synchronous call chain A → B → A (or longer) has closed a cycle.

The current runtime does **not** detect multi-actor cycles — only direct self-calls surface as `SelfCall`. Diagnose with the observability surface:

```sploosh
let a_len = a.mailbox_len();
let b_len = b.mailbox_len();
log::info(format("a alive={} mailbox={}/{}; b alive={} mailbox={}/{}",
    a.alive(), a_len, a.mailbox_capacity(),
    b.alive(), b_len, b.mailbox_capacity()));
```

If both mailboxes have backed-up messages and neither actor is making progress, you're in a cycle. Resolution:

1. Restructure the call graph as a DAG — one side of the cycle should fetch state proactively and pass it as an argument rather than calling back.
2. Break the cycle with fire-and-forget `send`: replace the synchronous back-call with an enqueued message that completes after the outer handler returns.
3. Wrap cross-actor calls in an outer `send_timeout` so the deadlock surfaces as `SendError::Timeout` rather than an indefinite hang.

## Diagnosing `SendError::Dead` on a blocked sender

When `send_timeout` on a full mailbox returns `Err(SendError::Dead)` before the timeout elapses, the destination actor died *while the sender was blocked waiting* (§8.11). This is **not retryable against the same handle** — the supervisor may have restarted the actor with a new handle, but blocked senders are never transparently redirected.

Resolution:

1. Re-fetch the current handle from the supervisor's public API.
2. Retry the operation against the new handle.
3. If the actor is unsupervised, treat `Dead` as terminal — spawn a new instance or propagate the error.

To confirm a restart actually happened, query `sup.restart_count(&child)` before and after; an increment confirms a restart, no increment means the child is permanently dead.

## Supervisor strategy

```sploosh
@supervisor(strategy: "one_for_one", max_restarts: 3, window_secs: 60, restart_history: 32)
actor WorkerPool { /* ... */ }
```

- `one_for_one` — only restart the failed child.
- `max_restarts` — maximum restarts within the sliding `window_secs` window.
- `window_secs` — sliding wall-clock window over each restart's timestamp.
- `restart_history` — per-child cap on retained `RestartEvent` ring buffer (default 16).

On restart: the child is always recreated via a **fresh `init`** with the supervisor's stored arguments. Old handles become permanently dead and are not transparently redirected (§8.7a).

## If something goes wrong

| Symptom | Likely cause | Fix |
|---|---|---|
| `observe::actor_info(handle)` returns `None` for a recently-dead actor | Last `Handle<T>` clone has dropped; snapshot was GC'd | Hold a clone of the handle until you finish querying; store handles in your audit code |
| `restart_count(&child)` returns `Err(ObserveError::NotASupervisedChild)` (`E1210` reserved) | The child handle was spawned by a different supervisor, or by an unsupervised path | Verify the supervisor that spawned the child; restart history is rooted on the spawning supervisor only |
| `mailbox_len()` keeps climbing on a healthy actor | Handler is slower than message arrival rate | Profile the handler; consider a larger mailbox via `@mailbox(capacity: N)` or split the work across more actors |
| `mailbox_len()` is high and never decreases | Handler wedged (multi-actor cycle, blocking FFI, or external sync resource) | Use the multi-actor-deadlock recipe above; check for `extern "C"` calls in the handler chain |
| `alive()` flips from `true` to `false` between two calls | Actor died between observations — race is expected | Use `actor_info` and `death_cause` to learn *why* it died |
| `children()` enumerates a child not in the strategy's restart set | The child was added dynamically and the supervisor uses an unordered collection | `rest_for_one` falls back to `one_for_one` for unordered children (compile-time warning, §8.7a); verify the supervisor's child collection is `Vec<Handle<T>>` |
| `restart_history` truncates events you expected to see | Default cap is 16; older events drop FIFO | Bump the cap with `@supervisor(restart_history: 64)` (or higher) |

## Related

- Spec: `docs/spec-plans/LANGUAGE_SPEC.md` §8.1 (definition), §8.1a (lifecycle), §8.2 (handles), §8.2a (`stop`/`kill`), §8.7 / §8.7a (supervision), §8.8 (failure), §8.10.1 (re-entrant calls), §8.11 (runtime), §8.12 (observability).
- Stdlib: [`docs/stdlib/actor.md`](../stdlib/actor.md) — `std::actor::observe` API.
- Guide: [`docs/guide/actors-and-concurrency.md`](../guide/actors-and-concurrency.md).
- Diagnostics: [`docs/reference/compiler-errors.md`](../reference/compiler-errors.md) — `E1210`, `E1211` (reserved).
- Build flags: [`docs/tooling/build-system.md`](../tooling/build-system.md).
