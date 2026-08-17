/// Spawn / select / emit shapes from §4.6, §8.2, §8.2a, §8.6, §8.7, §8.9,
/// §11.1, §11.3. Parse-only: actor semantics (§8.1a lifecycle, deterministic
/// select ordering, JoinHandle) and the on-chain-only emit restriction are
/// later milestones. §8.6's `=> return Err(AppError::Timeout),` arm is a
/// block body here — `return` in an expression arm is #89 item 1 (third
/// occurrence). Non-tail select / spawn-async statements carry their `;`
/// per §16 expr_stmt.
fn start_workers(table: Table) {
    let logger = spawn Logger::init();
    let worker = spawn Worker::init(logger.clone());
    let counter: Handle<Counter> = spawn Counter::init(0);
}

// §8.2a/§8.7: worker pools — spawn inside a closure argument, piped.
fn pool(logger: Handle<Logger>) {
    let workers: Vec<Handle<Worker>> = (0..4)
        |> map(|_| spawn Worker::init(logger.clone()))
        |> collect;
}

// §4.6: move capture into a spawned task — parseable verbatim now.
fn move_task(data: Vec<i64>) {
    let handle = spawn move || {
        process(data);
    };
}

// §8.9: non-actor async task with a JoinHandle.
fn fetch_task(url: String) {
    let handle: JoinHandle<String> = spawn async {
        fetch(url).await
    };
}

// §8.6: multiplexed receive; timeout(ms) is an intrinsic usable in arms.
fn multiplex(rx1: Receiver<Msg>, rx2: Receiver<Msg>) -> Result<(), AppError> {
    select {
        msg = rx1.recv() => handle_a(msg),
        msg = rx2.recv() => handle_b(msg),
        _ = timeout(5000) => { return Err(AppError::Timeout); }
    }
}

// §11.1/§11.3: event emission inside an onchain module — `to` is field
// shorthand.
onchain mod token {
    fn transfer(sender: Address, to: Address, amount: u256) -> Result<(), TokenError> {
        emit Transfer { from: sender, to, amount };
        Ok(())
    }

    fn deposit(sender: Address, amount: u256) -> Result<(), TokenError> {
        emit Deposit { sender, amount };
        Ok(())
    }
}
