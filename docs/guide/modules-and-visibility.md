# Modules and Visibility

> Organizing code into modules, imports, and the two-level visibility model.

## Module Declaration

```sploosh
// File: src/auth/mod.sp
mod auth {
    pub mod login;
    pub mod token;
    mod internal;   // private submodule
}
```

## Imports

```sploosh
use std::collections::Map;
use crate::auth::token::verify;
use crate::models::{User, Role, Permission};   // multiple imports
```

## Visibility

Sploosh has exactly two visibility levels:

- `pub` -- visible outside the module
- (no modifier) -- private to the module

No `protected`, no `internal`, no `pub(crate)`. One way to do it.

## File Layout

```
src/
├── main.sp              // Entry point (fn main)
├── lib.sp               // Library root
├── models/
│   ├── mod.sp           // Module declaration
│   ├── user.sp
│   └── role.sp
└── handlers/
    ├── mod.sp
    └── api.sp
```

## Next Steps

- [Generics and Advanced Types](generics-and-advanced-types.md)
- [Strings and Formatting](strings-and-formatting.md)
