# std::web

> HTTP server, routing, and middleware.

**Available targets:** native, wasm

```sploosh
use std::web::{Server, Router, Request, Response, Status};

let router = Router::new()
    |> route("GET", "/users/:id", get_user)
    |> middleware(auth::require_token);

Server::bind("0.0.0.0:8080")
    |> serve(router, state)?;
```

Not available inside `onchain` modules (compile-time error).

<!-- TODO: Document full API (routing, middleware, request/response, WebSocket) once implemented -->
