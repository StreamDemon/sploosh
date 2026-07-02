/// Attribute shapes (§12, §16 attrs/attr_args): bare markers, derive lists,
/// named arguments, and actor-handler attributes.
@derive(Serialize, Clone, Debug)
struct Payload {
    pub body: String,
}

@error
enum AppError {
    NotFound,
    Denied,
}

@overflow(wrapping)
fn wrap_add(a: u8, b: u8) -> u8 {
    a + b
}

@fast_math(contract, afn)
fn mix(a: f64, b: f64) -> f64 {
    a * b + a
}

@supervisor(strategy: "one_for_one", max_restarts: 5, window_secs: 60)
actor Sup {
    children: i64,
}

actor Mailer {
    queued: i64,

    @mailbox(capacity: 2048)
    pub fn enqueue(&mut self, n: i64) {
        self.queued = self.queued + n;
    }
}

@test
fn smoke() {
    let ok = true;
}
