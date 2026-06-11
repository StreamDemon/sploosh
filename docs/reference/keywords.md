# Keywords (44 Total: 36 Reserved + 8 Contextual)

> All keywords in Sploosh. Authoritative list: LANGUAGE_SPEC.md §2.3.

Sploosh distinguishes two keyword categories (§2.3, §2.7):

- A **reserved keyword** (§2.3.1) is always tokenized as a keyword and can never
  be used as an identifier, in any position.
- A **contextual keyword** (§2.3.2) is a keyword only in its defined syntactic
  position(s); everywhere else the same spelling is an ordinary identifier. This
  is what lets `tx.send(...)`, `rx.recv()`, and `storage::get(...)` work without
  renames.

## Reserved Keywords (36)

### Declarations (12)
| Keyword | Purpose |
|---------|---------|
| `fn` | Function definition |
| `let` | Variable binding |
| `const` | Compile-time constant |
| `type` | Type alias |
| `struct` | Struct definition |
| `enum` | Enum definition |
| `trait` | Trait definition |
| `impl` | Trait implementation |
| `mod` | Module definition |
| `use` | Import |
| `pub` | Public visibility |
| `extern` | Foreign function interface (`extern "C" { ... }`, §4.9) and on-chain cross-contract interface declarations (`extern onchain mod X { ... }`, §11.4a) |

### Control Flow (10)
| Keyword | Purpose |
|---------|---------|
| `if` | Conditional |
| `else` | Alternative branch |
| `match` | Pattern matching |
| `for` | Iteration |
| `in` | Iterator binding |
| `while` | Conditional loop |
| `loop` | Infinite loop |
| `break` | Exit loop |
| `continue` | Skip to next iteration |
| `return` | Early return |

### Types & Values (5)
| Keyword | Purpose |
|---------|---------|
| `self` | Current instance |
| `Self` | Current type |
| `true` | Boolean true |
| `false` | Boolean false |
| `as` | Numeric type cast |

### Concurrency (5)
| Keyword | Purpose |
|---------|---------|
| `actor` | Actor definition |
| `spawn` | Create actor instance |
| `async` | Async function |
| `await` | Await async result |
| `select` | Multiplexed receive |

### Closures (1)
| Keyword | Purpose |
|---------|---------|
| `move` | Move capture in closures |

### Web3 (3)
| Keyword | Purpose |
|---------|---------|
| `onchain` | On-chain module / event enum |
| `offchain` | Off-chain function |
| `emit` | Emit on-chain event |

## Contextual Keywords (8)

| Keyword | Keyword position(s) | Identifier everywhere else — e.g. |
|---------|---------------------|-----------------------------------|
| `send` | First token of a send-statement: `send handle.method(args);` (§8.2) | `tx.send(...)` (§8.5), `contract.send(...)` (§15) |
| `recv` | None in the current spec — reserved contextually for a possible future receive construct | `rx.recv()` (§8.5, §8.6 select arms) |
| `storage` | Block head of a storage block inside an `onchain mod`: `storage { ... }` (§11.1) | `storage::get(...)` / `storage::set(...)` paths (§11.1, §13.0) |
| `mut` | After `&` in a reference type or borrow (`&mut T`, `&mut self`), and immediately after `let` (`let mut x = ...`) | any other position |
| `dyn` | Type position, before a trait reference: `dyn Trait`, `Box<dyn Trait>` (§3.9) | any other position |
| `ref` | Pattern-binding position: `ref name` inside a pattern (§3.7) | any other position |
| `crate` | Path head: `crate::models::User` (§10) | any non-path-head position |
| `super` | Path head: `super::sibling` (§10) | any non-path-head position |

**Statement-head `send` disambiguation (§2.7):** `send` as the first token of a
statement, followed by any token that can begin an expression, always opens a
send-statement. A binding named `send` cannot appear bare at statement head —
parenthesize (`(send).method();`) or pick another name.
