/// Module trees and use declarations (§10): inline modules, file modules,
/// nested paths, brace imports, contextual `crate`/`super` heads, and
/// re-exports.
mod auth {
    pub mod login;
    pub mod token;

    pub fn is_enabled() -> bool {
        true
    }
}

mod models;

use std::collections::Map;
use crate::models::{User, Role};
pub use crate::models::User;
use super::shared;
use crate::api;

fn wire() -> bool {
    true
}
