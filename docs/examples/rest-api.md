# Example: REST API Server

> A web server with routing, database access, and on-chain integration.

## main.sp

```sploosh
use std::web::{Server, Router, Request, Response, Status};
use std::json;
use std::db::Pool;
use crate::contracts::token;
use crate::models::User;

struct AppState {
    db: Pool,
    contract: Contract,
}

fn main() -> Result<(), AppError> {
    let state = AppState {
        db: Pool::connect("postgres://localhost/myapp")?,
        contract: Contract::connect("0xABC123...")?,
    };

    let router = Router::new()
        |> route("GET", "/users/:id", get_user)
        |> route("POST", "/transfer", transfer_tokens)
        |> middleware(auth::require_token);

    Server::bind("0.0.0.0:8080")
        |> serve(router, state)?;

    Ok(())
}

async fn get_user(req: &Request, state: &AppState) -> Result<Response, AppError> {
    let id: u64 = req.param("id")?.parse()?;

    let user = state.db
        .query("SELECT * FROM users WHERE id = $1", &[id])
        .await?
        |> map(User::from)
        |> first
        |> ok_or(AppError::NotFound { resource: "user".into() })?;

    let balance = state.contract
        .call(token::balance_of, user.wallet)
        .await?;

    let body = json::to_string(&UserResponse { user, token_balance: balance })?;
    Ok(Response::new(Status::Ok, body))
}

async fn transfer_tokens(req: &Request, state: &AppState) -> Result<Response, AppError> {
    let body: TransferRequest = json::from_reader(req.body())?;

    let tx = state.contract
        .send(token::transfer, body.to, body.amount)
        .await?;

    Ok(Response::new(Status::Ok, json::to_string(&tx)?))
}
```

## Key Patterns

- **Pipe operator** for router configuration and query processing
- **`?` propagation** at every fallible step
- **Async handlers** for I/O-bound operations
- **On-chain integration** via `Contract::connect` and `.call()`/`.send()`
