# std::actor

> Actor observability and introspection. Direct introspection methods on `Handle<T>`, the `std::actor::observe` query module, supervisor-rooted restart history, and the `ActorId` / `ActorInfo` / `DeathCause` / `RestartEvent` / `LifecycleState` / `ObserveError` type reference.

**Targets:** native ✅ · wasm ✅ · evm ❌ · svm ❌

`std::actor` is a **compile error inside `onchain` modules** (§11.1, §12.3 of the language spec). The entire actor-runtime surface — `Handle<T>`, `spawn`, `send`, supervisors, channels — is already an on-chain compile error; observability rides on the same prohibition. There is no on-chain analog in v0.5.6.

This page mirrors §8.12 of `docs/spec-plans/LANGUAGE_SPEC.md`. The spec is the source of truth.

---

## Overview

The observability surface is **always available, every build mode** — there is no `@observable` attribute, no debug-only gating, and no feature flag. Bookkeeping is paid on every spawn (§8.12.6). The cost trade is intentional: production triage is when you most need observability, and a feature flag fails exactly when it matters.

The surface is split into two layers by cost:

- **Constant-time reads** are direct methods on `Handle<T>`. Available on dead handles. Infallible.
- **Richer queries** that walk the runtime registry live in `std::actor::observe`.
- **Restart history** is rooted on the **supervisor's** handle, because only `@supervisor`-decorated actors run a restart loop in the first place.

---

## `Handle<T>` introspection methods

```sploosh
impl<A: Actor> Handle<A> {
    pub fn mailbox_len(&self) -> usize;
    pub fn mailbox_capacity(&self) -> usize;
    pub fn alive(&self) -> bool;
    pub fn actor_id(&self) -> ActorId;
}
```

| Method | Purpose | Notes |
|---|---|---|
| `mailbox_len()` | Current queued message count | Atomic snapshot; may be stale by one increment. Reads the same atomic that `send` and `send_timeout` consult for backpressure. |
| `mailbox_capacity()` | Configured mailbox capacity | The value passed to `@mailbox(capacity: N)` or the runtime default 1024. Constant for the actor's lifetime. |
| `alive()` | `true` if not `DEAD` | `true` for `INITIALIZING`, `READY`, `DRAINING`. `false` is final; `true` is an instant — the actor may transition to `DEAD` between the read and any subsequent call. |
| `actor_id()` | Opaque per-spawn `ActorId` | Same value for every clone of the handle and after the actor has died. |

All four methods are `&self`, infallible, and **available on dead handles**. Constant time except `mailbox_len`, which is an atomic load.

```sploosh
let worker = spawn Worker::init();
log::info(format("worker {}: alive={}, mailbox={}/{}",
    worker.actor_id(),
    worker.alive(),
    worker.mailbox_len(),
    worker.mailbox_capacity()));
```

---

## `std::actor::observe` module

Richer queries that walk runtime state. Import explicitly:

```sploosh
use std::actor::observe;
```

### `observe::actor_info`

```sploosh
pub fn actor_info<A: Actor>(handle: &Handle<A>) -> Option<ActorInfo>;
```

Returns the full `ActorInfo` snapshot for the actor the handle targets. Returns `Some(info)` whenever the runtime still retains an entry — i.e., whenever any `Handle<T>` clone targeting that actor is live. Returns `None` only for stale handles whose snapshot has been GC'd.

For a dead actor, `info.lifecycle_state == LifecycleState::Dead` and `info.death_cause` is populated.

### `observe::actors`

```sploosh
pub fn actors() -> Iter<ActorInfo>;
```

Enumerates every live actor in the runtime. Iteration order is **unspecified but deterministic** for a given runtime instance and observation point — two back-to-back calls within one runtime yield the same order for the same population. The spec does not commit to ordering across runtime instances or releases.

`O(N_actors)`. Intended for diagnostics and triage, not hot paths.

### Filtered iteration

```sploosh
impl Iter<ActorInfo> {
    pub fn by_supervisor<S: Actor>(self, sup: &Handle<S>) -> Iter<ActorInfo>;
    pub fn by_name(self, name: &str) -> Iter<ActorInfo>;
}
```

`by_supervisor` filters to actors whose `supervisor` field matches the given handle's `actor_id`. Returns an empty iterator if the handle is not a `@supervisor`-decorated actor. `by_name` filters by the unqualified actor type name (e.g., `"Worker"`).

```sploosh
let pool: Iter<ActorInfo> = observe::actors().by_supervisor(&pool_handle);
let workers: Iter<ActorInfo> = observe::actors().by_name("Worker");
```

---

## Supervisor-rooted restart history

When `S` is `@supervisor`-decorated, `Handle<S>` exposes three additional methods:

```sploosh
impl<S: Actor> Handle<S> {
    pub fn restart_count<C: Actor>(&self, child: &Handle<C>) -> Result<u32, ObserveError>;
    pub fn restart_history<C: Actor>(&self, child: &Handle<C>) -> Result<Vec<RestartEvent>, ObserveError>;
    pub fn children(&self) -> Iter<ActorInfo>;
}
```

| Method | Purpose | Error |
|---|---|---|
| `restart_count(&child)` | Total restart count for that child since the supervisor first spawned it | `Err(ObserveError::NotASupervisedChild)` if `child` is not supervised by `self` |
| `restart_history(&child)` | Retained `RestartEvent`s in chronological order (oldest first) | Same as above |
| `children()` | Currently-supervised children, in supervisor-order (matches `rest_for_one` order) | Infallible |

**`restart_count` returns the *total* count**, not the current sliding-window count. For window-aware diagnostics, walk `restart_history` and filter by `timestamp_ms_since_spawn`.

**Retained history is capped** — default 16 events per child, tunable via `@supervisor(restart_history: N)`. Older events drop FIFO. A child that has never died has an empty history vector and a `restart_count` of `0`.

A child terminated via `handle.stop()` or `handle.kill()` (§8.2a) is **intentional termination**, not a restart — the supervisor does not restart it (§8.7) and such terminations do **not** appear in `restart_history`.

```sploosh
@supervisor(strategy: "one_for_one", max_restarts: 5, window_secs: 60, restart_history: 32)
actor WorkerPool {
    children: Vec<Handle<Worker>>,
    fn init(size: u32) -> Self { /* spawn children */ }
    pub fn child_at(&self, idx: usize) -> Handle<Worker> { self.children[idx].clone() }
}

let pool: Handle<WorkerPool> = spawn WorkerPool::init(8);
let target = pool.child_at(0);

match pool.restart_count(&target) {
    Ok(n)  => log::info(format("worker has restarted {} times", n)),
    Err(ObserveError::NotASupervisedChild) => log::warn("not supervised here"),
}

for event in pool.restart_history(&target).unwrap() {
    log::info(format("t={}ms cause={:?}", event.timestamp_ms_since_spawn, event.cause));
}
```

---

## Types

### `ActorId`

```sploosh
struct ActorId( /* opaque */ );

impl Copy for ActorId {}
impl Eq   for ActorId {}
impl Hash for ActorId {}
```

Opaque, monotonically assigned at `spawn`. Two distinct actors never share an `ActorId`; an ID is **never reused** even after the actor it identified has died and its snapshot has been GC'd. The runtime assigns IDs from a non-zero counter; `ActorId(0)` is reserved as a sentinel.

`ActorId` is **not** `Send` across runtime instances. v0.5.6 has one runtime per process. Comparing `ActorId`s produced by different runtime instances is a compile error (`E1211`, reserved).

`ActorId` is exported from the prelude.

### `ActorInfo`

```sploosh
@derive(Debug, Clone)
struct ActorInfo {
    pub id:                ActorId,
    pub name:              String,
    pub spawn_location:    String,           // file:line, "<unknown>" without debug info
    pub supervisor:        Option<ActorId>,
    pub lifecycle_state:   LifecycleState,
    pub mailbox_len:       usize,
    pub mailbox_capacity:  usize,
    pub death_cause:       Option<DeathCause>,
}
```

`name` is the unqualified actor type name (e.g., `"Worker"`). `spawn_location` is best-effort — the runtime captures the call-site file and line at `spawn` when debug info is available; otherwise the field is `"<unknown>"`. `supervisor` is `Some(parent_id)` when the actor was spawned from inside a `@supervisor`-decorated actor's `init` or handler, and `None` otherwise. `death_cause` is `Some(...)` iff `lifecycle_state == LifecycleState::Dead`.

### `LifecycleState`

```sploosh
@derive(Debug, Clone, Copy, Eq)
enum LifecycleState {
    Initializing,
    Ready,
    Draining,
    Dead,
}
```

Mirrors §8.1a. `Initializing` covers the `INITIALIZING` state; `Ready` covers `READY`; `Draining` covers `DRAINING` (entered via `handle.stop()`, §8.2a); `Dead` covers `DEAD`.

### `DeathCause`

```sploosh
@derive(Debug, Clone)
enum DeathCause {
    RuntimeFailure { panic: String },        // bounds, overflow, assert (§8.8)
    Stopped,                                  // handle.stop() drained the mailbox (§8.2a)
    Killed,                                   // handle.kill() (§8.2a)
    Supervised { restart_pending: bool },     // supervisor terminated the child (§8.7)
    RuntimeShutdown,                          // main() returned (§8.11)
}
```

Captured when an actor reaches `DEAD`. `Supervised { restart_pending: true }` means the supervisor is mid-restart — the next spawn produces a new `Handle<T>` and the old handle remains permanently dead (§8.7a).

### `RestartEvent`

```sploosh
@derive(Debug, Clone)
struct RestartEvent {
    pub timestamp_ms_since_spawn: u64,
    pub cause:                    DeathCause,
}
```

One entry per restart, returned by `Handle<S>.restart_history(&child)` in chronological order (oldest first). For `one_for_all` and `rest_for_one` strategies, `cause` is propagated from the failed sibling.

### `ObserveError`

```sploosh
@error
enum ObserveError {
    NotASupervisedChild,    // see E1210 (reserved)
}
```

Returned by `restart_count` and `restart_history` when the child handle is not supervised by `self`.

---

## Cost model

The bookkeeping is paid on every spawn, whether or not anything ever calls `observe::*`:

| Surface | Per-instance cost |
|---|---|
| Registry entry per actor | ~24 bytes (the `ActorId`, a pointer to the actor's runtime cell, the supervisor's `ActorId` if any) |
| Mailbox counter | One atomic `usize` per actor (reused from existing backpressure machinery) |
| Restart-history ring buffer | ~24 bytes × *N* per supervised child (default *N* = 16 → ~384 bytes) |
| Dead-actor snapshot | ~256 bytes per `ActorInfo` until the last `Handle<T>` clone drops |

`observe::actors()` walks the registry and is **O(N_actors)**. `observe::actor_info(handle)` is **O(1)**. `Handle<T>` introspection methods are **O(1)** (atomic load for `mailbox_len`, plain reads for the rest).

Holding an `Iter<ActorInfo>` across an `.await` inside an actor handler is permitted but inadvisable — it pins snapshots and observably delays GC of dead-actor entries whose only remaining reference is the iterator.

---

## Dead-handle semantics

All four `Handle<T>` introspection methods remain callable on a dead handle. They return last-known values. The behavior of every other method on a dead handle is **unchanged from the existing actor model**:

- `send` silently drops (§8.2).
- `send_timeout` returns `Err(SendError::Dead)` (§8.5).
- Request/reply returns `Err(ActorError::Dead)` (§8.8).
- `stop()` / `kill()` return `Err(StopError::AlreadyDead)` (§8.2a).

The §8.12 surface adds new observation methods; it does not alter dead-handle messaging behavior.

---

## Snapshot retention contract

When an actor reaches `DEAD`, the runtime captures a final `ActorInfo` snapshot and stores it in a side-table keyed by `ActorId`. The snapshot is **retained as long as any `Handle<T>` clone targeting the actor remains live**.

This is a **refcount-driven retention on the snapshot side-table**, *not* on the actor itself. The contrast with §8.2 is deliberate: handle drop has no effect on actor *lifetime* (§8.2 retains that property unchanged), but handle drop *does* affect snapshot *retention*. Once the last handle clone drops, the snapshot is GC'd and `observe::actor_info(handle)` against a stale handle returns `None`.

This is the **only** refcount in the actor model.

---

## Cross-references

- `docs/spec-plans/LANGUAGE_SPEC.md` §8.12 — full specification.
- §8.1a — lifecycle states.
- §8.2 — `Handle<T>` semantics (handle drop does not kill the actor).
- §8.2a — `stop()` / `kill()` — termination paths that produce `DeathCause::Stopped` / `DeathCause::Killed`.
- §8.7 / §8.7a — supervision and restart semantics — what `restart_count` and `restart_history` reflect.
- §8.8 — `ActorError`, runtime failure → `DeathCause::RuntimeFailure`.
- §8.11 — runtime architecture and lifecycle (`DeathCause::RuntimeShutdown`).
- §11.1 / §12.3 — on-chain prohibition.
- §13.0 — handle introspection intrinsics (`mailbox_len` / `mailbox_capacity` / `alive` / `actor_id`).
- §13.1 — prelude (`ActorId` is auto-imported).
- §12.1 — `@supervisor(restart_history: N)` parameter.
- `docs/guide/actors-and-concurrency.md` — Observability section, worked examples.
- `docs/runbooks/actor-debugging.md` — operational recipes.
- `docs/reference/compiler-errors.md` — `E1210`, `E1211` (reserved).
