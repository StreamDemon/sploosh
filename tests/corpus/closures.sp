/// Closure shapes from §4.6/§5.6/§6.5/§7.3/§8.2: inferred/typed/wildcard
/// params, zero-arg `||`, `move` captures, closures as call arguments, and
/// the parenthesized closure pipe stage. Top-level or-pattern params must
/// parenthesize (`|(A | B)|`) — the bare form would collide with the closing
/// delimiter. §4.6's `spawn move || { ... }` example is written here without
/// `spawn`: spawn has no parse production yet.
fn typed_and_inferred() {
    let double = |n: i64| n * 2;
    let apply_result = apply(double, 21);
    let is_active = |u| u.active;
    let spawn_one = |_| spawn_worker();
}

// §4.6: zero-param closure with a block body; `move` takes ownership.
fn counters() {
    let mut inc = || { counter = counter + 1; };
    inc();
    inc();
    let handle = move || {
        process(data);
    };
}

// §4.6: typed param borrowing.
fn greeter(name: String) {
    let greet = |prefix: &str| format("{} {}", prefix, name);
    greet("Hello");
}

// §5.6: the piped value in a non-first position — a closure stage.
fn pipe_closure() {
    let result = 10 |> (|v| multiply(3, v));
}

// §6.5: chaining with map/unwrap_or.
fn find_email() {
    let email = find_user(42)
        |> map(|u| u.email)
        |> unwrap_or("unknown@example.com".into());
}

// §7.3: method-chain style and pipe style — same call sequence.
fn active_names(users: Vec<User>) -> Vec<String> {
    let names: Vec<String> = users.iter()
        .filter(|u| u.active)
        .map(|u| u.name.clone())
        .collect();

    let piped: Vec<String> = users.iter()
        |> filter(|u| u.active)
        |> map(|u| u.name.clone())
        |> collect();
    piped
}

// §8.2: filter over a field access, from the Logger actor.
fn count_matching(entries: Vec<String>, needle: &str) -> u64 {
    entries.iter()
        |> filter(|e| e.contains(needle))
        |> count
}
