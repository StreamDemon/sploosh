# std::collections

> Vec, Map, Set -- methods and usage patterns.

## Vec\<T\>

Growable, heap-allocated list.

**Methods:** `push`, `pop`, `insert`, `remove`, `len`, `is_empty`, `contains`, `sort`, `sort_by`, `reverse`, `dedup`, `clear`, `iter`, `iter_mut`, `get`, `first`, `last`

```sploosh
let mut items: Vec<i64> = vec![1, 2, 3];
items.push(4);
let first = items.first();  // Option<&i64>
```

## Map\<K, V\>

Hash map.

**Methods:** `insert`, `remove`, `get`, `contains_key`, `len`, `is_empty`, `keys`, `values`, `iter`, `entry`, `clear`

```sploosh
let mut scores: Map<String, i64> = Map::new();
scores.insert("alice".into(), 100);
let val = scores.get("alice");  // Option<&i64>
```

## Set\<T\>

Hash set.

**Methods:** `insert`, `remove`, `contains`, `len`, `is_empty`, `union`, `intersection`, `difference`, `iter`, `clear`

## Channel\<T\>

Bounded MPSC (multi-producer, single-consumer) channel.

**Constructor:** `Channel::new(capacity)` returns `(Sender<T>, Receiver<T>)`.

- `Sender<T>` is `Clone + Send` -- multiple producers can hold clones of the same sender.
- `Receiver<T>` is **not** `Clone` -- only a single consumer can own the receiver.
- `tx.send(val)?` blocks if the channel is full.
- `rx.recv()?` blocks until a value is available.

```sploosh
let (tx, rx) = Channel::new(16);  // bounded queue, capacity 16

// Producer side (tx is Clone, so it can be shared)
let tx2 = tx.clone();
tx.send(42)?;
tx2.send(43)?;

// Consumer side (rx cannot be cloned)
let val = rx.recv()?;  // 42
```

<!-- TODO: Add detailed API signatures and examples as stdlib is implemented -->
