/// Adds two integers.
@inline(always)
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

struct User {
    pub id: u64,
    name: String,
}

enum Message {
    Quit,
    Move(i64, i64),
    Write { text: String },
}

const MAX_USERS: u64 = 1_000u64;

type Users = Vec<User>;

fn demo_parse() {
    let mut value = parse::<i64>("42");
    let items = vec![1, 2, 3];
    let zeros = vec![0; 4];
    value = value + items[0] + zeros[1];
}
