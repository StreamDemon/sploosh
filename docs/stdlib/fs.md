# std::fs

> Filesystem operations.

**Available targets:** native only

```sploosh
use std::fs;

let content = fs::read("config.json")?;
fs::write("output.txt", data)?;
```

Not available inside `onchain` modules (compile-time error).

<!-- TODO: Document full API once implemented -->
