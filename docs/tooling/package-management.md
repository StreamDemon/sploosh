# Package Management

> Adding dependencies, publishing packages, and versioning.

## Adding Dependencies

Edit `sploosh.toml`:

```toml
[dependencies]
sploosh_web = "0.3"
sploosh_db = "0.2"
```

## Importing

```sploosh
use sploosh_web::{Server, Router};
```

<!-- TODO: Document package registry, publishing workflow, version resolution, and lockfile once implemented -->
