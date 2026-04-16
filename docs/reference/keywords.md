# Keywords (40 Total)

> All reserved words in Sploosh.

## Declarations (12)
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

## Control Flow (10)
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

## Types & Values (6)
| Keyword | Purpose |
|---------|---------|
| `self` | Current instance |
| `Self` | Current type |
| `true` | Boolean true |
| `false` | Boolean false |
| `none` | No value |
| `as` | Numeric type cast |

## Concurrency (7)
| Keyword | Purpose |
|---------|---------|
| `actor` | Actor definition |
| `send` | Fire-and-forget message |
| `recv` | Receive from channel |
| `spawn` | Create actor instance |
| `async` | Async function |
| `await` | Await async result |
| `select` | Multiplexed receive |

## Closures (1)
| Keyword | Purpose |
|---------|---------|
| `move` | Move capture in closures |

## Web3 (4)
| Keyword | Purpose |
|---------|---------|
| `onchain` | On-chain module |
| `offchain` | Off-chain function |
| `storage` | On-chain persistent state |
| `emit` | Emit on-chain event |
