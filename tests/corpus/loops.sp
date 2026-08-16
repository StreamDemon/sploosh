/// Loop shapes from §5.4/§5.5/§7.3: if let (with else), while let, for with
/// binding/tuple/struct destructuring patterns, range and pipe iterables,
/// while, and loop with break. Every construct sits in fn-tail position —
/// §16 `expr_stmt` wants a `;` when a block-like expression is followed by
/// more statements (that rule is #62's to revisit).
// §5.4: if let — executes block only if pattern matches.
fn find_and_process() {
    if let Some(user) = find_user(42) {
        process(user);
    } else {
        log("user not found");
    }
}

// §5.4: if let with enum variants.
fn load_and_start() {
    if let Ok(config) = load_config("app.toml") {
        start_server(config);
    }
}

// §5.4: nested if let.
fn nested_if_let() {
    if let Some(user) = find_user(42) {
        if let Role::Admin = user.role {
            grant_access();
        }
    }
}

// §5.4: while let — loops while pattern matches.
fn drain_queue() {
    while let Some(item) = queue.pop() {
        process(item);
    }
}

fn read_messages() {
    while let Ok(msg) = connection.read() {
        handle(msg);
    }
}

// §5.5: iterate (primary loop form).
fn iterate(collection: Vec<i64>) {
    for item in collection {
        process(item);
    }
}

// §5.5: destructuring in for loops.
fn with_index(items: Vec<String>) {
    for (index, value) in items.iter() |> enumerate() {
        print(format("{}: {}", index, value));
    }
}

fn destructure_users(users: Vec<User>) {
    for User { name, age, .. } in users {
        print(format("{} is {}", name, age));
    }
}

// §5.5: range iteration.
fn log_range() {
    for i in 0..10 {
        log(i);
    }
}

// §5.5: while.
fn serve() {
    while connection.is_alive() {
        let msg = connection.read()?;
        handle(msg);
    }
}

// §5.5: infinite loop with break.
fn event_loop() {
    loop {
        let event = poll();
        if event.is_shutdown() {
            break;
        }
    }
}

// §7.3: .iter() borrows, .iter_mut() borrows mutably, and bare for consumes
// the binding — the consuming form comes last since the value is moved.
fn iteration_modes(items: Vec<i64>) {
    for x in items.iter() {
        print(x);
    };
    for x in items.iter_mut() {
        *x = *x + 1;
    };
    for x in items {
        print(x);
    }
}
