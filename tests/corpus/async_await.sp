/// Async functions, `.await`, and `?` error propagation outside pipes
/// (§6 errors, §8.9 async). `.context(...)` is an ordinary method call.
async fn fetch(url: &str) -> Result<Response, NetError> {
    let response = net::get(url).await?;
    Ok(response)
}

pub async fn retry(url: &str) -> Result<Response, NetError> {
    let first = fetch(url).await;
    let second = fetch(url).await?;
    Ok(second)
}

fn propagate(input: &str) -> Result<i64, ParseError> {
    let n = parse::<i64>(input)?;
    let checked = validate(n).context("validating input")?;
    Ok(checked)
}
