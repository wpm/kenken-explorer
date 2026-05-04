pub mod generate;
pub mod histogram;

use anyhow::{Result, anyhow};
use kenken::{Index, N, Operation, default_op_policy};

pub type OpPolicyFn = fn(&[N], Index) -> Operation;

pub fn resolve_op_policy(name: &str) -> Result<OpPolicyFn> {
    match name {
        "default" => Ok(default_op_policy),
        other => Err(anyhow!("unknown op_policy: {other}")),
    }
}
