use std::fs;
use std::path::Path;

#[test]
fn the_check_job_runs_the_formatter_the_linter_and_every_test() {
    assert!(ci().contains(CHECK_JOB), "{}", ci());
}

#[test]
fn the_check_job_installs_nothing_beyond_the_toolchain_and_tmux() {
    assert_eq!(ci().matches("apt-get install").count(), 1, "{}", ci());
}

#[test]
fn the_audit_job_checks_the_lock_against_the_advisory_database() {
    assert!(ci().contains(AUDIT_JOB), "{}", ci());
}

#[test]
fn the_workflow_grants_only_read_access_to_the_contents() {
    assert_eq!(ci().matches("permissions:").count(), 1, "{}", ci());
    assert!(
        ci().contains("permissions:\n  contents: read\n\n"),
        "{}",
        ci()
    );
}

fn ci() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml"))
        .unwrap()
}

const CHECK_JOB: &str = "  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09 # v5
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@d1031067263f94b142dd6c0ce24c5eb9d02d52a0 # master 2026-09
        with:
          toolchain: 1.98.1
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2
      - run: sudo apt-get update && sudo apt-get install -y --no-install-recommends tmux
      - run: cargo fmt --check
      - run: cargo clippy --locked --all-targets -- -D warnings
      - run: cargo test --locked -- --include-ignored
";

const AUDIT_JOB: &str = "  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09 # v5
";
