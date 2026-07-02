/// Range expressions (`..`/`..=`, non-associative per the precedence table)
/// and §16 item-modifier placement: `pub` on declarations, `offchain`/`async`
/// on functions only.
pub offchain async fn window(lo: i64, hi: i64) -> i64 {
    let half_open = lo..hi;
    let inclusive = lo..=hi;
    lo + hi
}

pub struct Config {
    pub retries: i64,
}

pub mod api;

pub use crate::api;

pub const LIMIT: i64 = 8;

pub type Pair = (i64, i64);

pub trait Sink {}
