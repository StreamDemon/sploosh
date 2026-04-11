# Coming from Elixir

> Actor model similarities, pattern matching, and pipe operator -- plus key differences.

## What's Familiar
- **Pipe operator** (`|>`) -- works the same way
- **Pattern matching** -- `match` with exhaustive arms
- **Actor model** -- message passing, isolated state, supervisors
- **No shared mutable state** -- same philosophy

## Key Differences

| Elixir | Sploosh | Notes |
|--------|---------|-------|
| `GenServer` | `actor` keyword | First-class syntax, not a behaviour |
| PID | `Handle<T>` | Typed handles, not opaque PIDs |
| `send(pid, msg)` | `send handle.method(args)` | Method-based messaging |
| `GenServer.call` | Direct method call on handle | `handle.get()` blocks |
| Dynamic typing | Static typing | All types known at compile time |
| BEAM GC | Ownership model | No garbage collector |
| `{:ok, value}` / `{:error, reason}` | `Result<T, E>` / `Option<T>` | Typed enums instead of tuples |
| `defmodule` | `mod` / `struct` + `impl` | Separate type and behavior |
| `Supervisor` | `@supervisor` attribute | Attribute on actor definition |
| `case`/`cond` | `match`/`if` | Similar but with type checking |
| Untyped process mailbox | Typed actor mailbox | Messages arrive via `send handle.method(args)` with type-checked params. `Channel<T>` is a separate primitive for explicit data flow. |
| Unbounded mailboxes | Bounded actor mailboxes | Fixed capacity; senders block when full |
| `:one_for_one`, `:one_for_all`, `:rest_for_one` | `one_for_one`, `one_for_all`, `rest_for_one` | Same three supervision strategies as OTP |
| `init/1` may return `{:stop, reason}` | `init` is infallible and non-async | `init` returns `Self` with no error channel. Panics in `init` transition the actor directly to `DEAD`; supervisor decides. Model recoverable init with a handshake message after spawn. |
| `GenServer.call` to self hangs | Direct self request/reply → `Err(ActorError::SelfCall)` | Sploosh runtime detects direct self-deadlocks in O(1) and returns an error instead of hanging. Multi-actor cycles (A → B → A) are still undetected — keep communication a DAG. |
| PID lifetime is independent of references/monitors | `Handle<T>` is not reference-counted either | Both systems share this non-refcounted model: a BEAM process lives until it exits or crashes regardless of who holds its PID, and `Process.monitor/1` is a `:DOWN` notifier rather than a lifetime control. The new information for Sploosh is the explicit concept of **orphaned actors** — a non-supervised actor with no live handle keeps running until runtime shutdown. Use supervisors (as in OTP) for clean shutdown. |
| Restart preserves no state (default) | Same — fresh `init` with stored args on every restart | Sploosh matches OTP default behavior. Old handles become permanently dead after restart; callers must re-fetch the new handle from the supervisor. |

## Actor Comparison

```elixir
# Elixir GenServer
defmodule Counter do
  use GenServer
  def init(n), do: {:ok, n}
  def handle_call(:get, _from, state), do: {:reply, state, state}
  def handle_cast({:inc, n}, state), do: {:noreply, state + n}
end

{:ok, pid} = GenServer.start_link(Counter, 0)
GenServer.cast(pid, {:inc, 5})
GenServer.call(pid, :get)  # => 5
```

```sploosh
// Sploosh actor
actor Counter {
    state: i64,
    fn init(n: i64) -> Self { Counter { state: n } }
    pub fn inc(&mut self, n: i64) { self.state = self.state + n; }
    pub fn get(&self) -> i64 { self.state }
}

let c = spawn Counter::init(0);
send c.inc(5);          // fire-and-forget (like cast)
let val = c.get();      // request/reply (like call)
```

## New Concepts for Elixir Developers
- **Ownership and borrowing** -- no GC, you manage memory through ownership
- **Static types** -- every variable has a known type at compile time
- **`onchain` modules** -- smart contract targeting (no Elixir equivalent)
- **Explicit lifetimes** -- when returning references
