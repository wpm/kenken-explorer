pub mod generate;
pub mod histogram;

use kenken::Uniqueness;

pub const DEFAULT_N: usize = 4;

pub fn uniqueness_str(u: Uniqueness) -> &'static str {
    match u {
        Uniqueness::None => "none",
        Uniqueness::Unique => "unique",
        Uniqueness::Multiple => "multiple",
    }
}
