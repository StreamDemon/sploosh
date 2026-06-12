# std::crypto

> Hashing, signing, and key generation.

**Available targets:** all (including onchain)

```sploosh
use std::crypto;

let hash = std::crypto::sha256(data);
```

Blake3 hashing is part of this module's planned surface — the `sploosh.lock` lockfile contract already depends on Blake3 checksums (32-byte digest, base32-no-pad, `"blake3:"` prefix; §14.3).

<!-- TODO: Document full API (hashing algorithms, signing schemes, key generation) once implemented -->
