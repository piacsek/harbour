use std::fs;
use std::path::Path;

#[test]
fn the_check_job_runs_the_formatter_the_linter_and_every_test() {
    assert!(ci().contains(CHECK_JOB), "{}", ci());
}

#[test]
fn the_check_job_installs_nothing_beyond_the_toolchain_tmux_and_nextest() {
    assert_eq!(ci().matches("apt-get install").count(), 1, "{}", ci());
}

#[test]
fn nextest_runs_the_end_to_end_tests_two_at_a_time_and_never_retries_a_failure() {
    assert_eq!(nextest_config(), NEXTEST_CONFIG);
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

#[test]
fn every_job_of_every_workflow_has_a_timeout() {
    assert_eq!(jobs_without_a_timeout(), Vec::<String>::new());
}

fn jobs_without_a_timeout() -> Vec<String> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows");
    let mut missing = Vec::new();
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        let jobs = &text[text.find("\njobs:\n").unwrap()..];
        let mut job: Option<String> = None;
        let mut timed = false;
        for line in jobs.lines().skip(2) {
            let header = line.starts_with("  ") && !line.starts_with("   ") && line.ends_with(':');
            if header {
                if let Some(name) = job.take().filter(|_| !timed) {
                    missing.push(name);
                }
                job = Some(format!(
                    "{} {}",
                    path.file_name().unwrap().to_string_lossy(),
                    line.trim()
                ));
                timed = false;
            }
            timed |= line.starts_with("    timeout-minutes: ");
        }
        if let Some(name) = job.filter(|_| !timed) {
            missing.push(name);
        }
    }
    missing.sort();
    missing
}

fn nextest_config() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/nextest.toml"))
        .unwrap_or_default()
}

fn ci() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml"))
        .unwrap()
}

const NEXTEST_CONFIG: &str = "[profile.default]
retries = 0
fail-fast = false
slow-timeout = { period = \"60s\", terminate-after = 3 }

[test-groups]
e2e = { max-threads = 2 }

[[profile.default.overrides]]
filter = \"binary(e2e)\"
test-group = \"e2e\"
priority = 100
";

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
      - uses: taiki-e/install-action@3f74d7c16a4242f1c95561e98edc25d36adb4375 # v2.87.12
        with:
          tool: cargo-nextest@0.9.144
          fallback: none
      - run: cargo fmt --check
      - run: cargo clippy --locked --all-targets -- -D warnings
      - run: cargo nextest run --locked --config-file .github/nextest.toml --run-ignored all
    timeout-minutes: 15
";

const AUDIT_JOB: &str = "  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09 # v5
";
