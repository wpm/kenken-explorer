//! End-to-end checks that JSON output is byte-identical for the same inputs.
//!
//! The whole "scriptable, reproducible" design rests on per-trial seeding being
//! stable across runs and across thread counts. These tests pin that contract.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_kenken-explorer")
}

/// Run the binary with `args`, expect exit 0, return stdout with the timing-dependent
/// `meta` field stripped so equality compares only the experiment data.
fn run_strip_meta(args: &[&str]) -> String {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("spawn binary");
    assert!(
        out.status.success(),
        "binary failed: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    let mut value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    if let Some(obj) = value.as_object_mut() {
        obj.remove("meta");
    }
    serde_json::to_string(&value).expect("re-serialize")
}

#[test]
fn histogram_is_byte_identical_across_runs() {
    let args = &[
        "histogram",
        "--n",
        "3",
        "--trials",
        "100",
        "--seed",
        "99",
        "--max-solutions",
        "10",
    ];
    let a = run_strip_meta(args);
    let b = run_strip_meta(args);
    assert_eq!(a, b, "same args produced different output across runs");
}

#[test]
fn histogram_is_thread_count_independent() {
    let base = &[
        "histogram",
        "--n",
        "3",
        "--trials",
        "100",
        "--seed",
        "42",
        "--max-solutions",
        "10",
    ];
    let mut single = vec!["--threads", "1"];
    single.extend_from_slice(base);
    let mut multi = vec!["--threads", "4"];
    multi.extend_from_slice(base);
    assert_eq!(
        run_strip_meta(&single),
        run_strip_meta(&multi),
        "histogram differs between thread counts; per-trial seeding is broken"
    );
}

#[test]
fn generate_is_byte_identical_across_runs() {
    let args = &["generate", "--n", "4", "--seed", "7"];
    assert_eq!(run_strip_meta(args), run_strip_meta(args));
}

#[test]
fn max_solutions_zero_is_rejected() {
    let out = Command::new(bin())
        .args(["histogram", "--max-solutions", "0", "--trials", "1"])
        .output()
        .expect("spawn binary");
    assert!(!out.status.success(), "expected non-zero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("max_solutions"),
        "stderr did not mention max_solutions: {stderr}"
    );
}

#[test]
fn invalid_op_policy_is_rejected_at_parse_time() {
    let out = Command::new(bin())
        .args(["histogram", "--op-policy", "no-such-policy"])
        .output()
        .expect("spawn binary");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid value"),
        "stderr should reflect a clap parse error: {stderr}"
    );
}
