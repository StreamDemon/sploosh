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

<!-- TODO: Add detailed API signatures and examples as stdlib is implemented -->
