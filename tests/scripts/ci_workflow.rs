use std::fs;
use std::path::Path;

#[test]
fn one_shard_runs_the_app_tests_and_the_other_the_formatter_the_linter_and_every_other_test() {
    assert!(ci().contains(SHARD_JOB), "{}", ci());
}

#[test]
fn the_required_check_passes_only_when_both_shards_passed() {
    assert!(ci().contains(CHECK_JOB), "{}", ci());
}

#[test]
fn only_one_job_installs_a_package_with_apt_get() {
    assert_eq!(ci().matches("apt-get install").count(), 1, "{}", ci());
}

#[test]
fn nextest_runs_the_end_to_end_tests_two_at_a_time_never_retries_a_failure_and_writes_junit() {
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
fn a_job_header_with_a_trailing_comment_counts_as_a_job() {
    assert_eq!(job_header("  shard: # the test legs"), Some("shard"));
}

#[test]
fn a_job_header_ending_in_a_carriage_return_counts_as_a_job() {
    assert_eq!(job_header("  shard:\r"), Some("shard"));
}

#[test]
fn a_step_key_is_not_a_job_header() {
    assert_eq!(job_header("    steps:"), None);
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
            if let Some(name) = job_header(line) {
                if let Some(name) = job.take().filter(|_| !timed) {
                    missing.push(name);
                }
                job = Some(format!(
                    "{} {}:",
                    path.file_name().unwrap().to_string_lossy(),
                    name
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

fn job_header(line: &str) -> Option<&str> {
    let line = line.trim_end_matches('\r');
    let line = line.split(" #").next().unwrap().trim_end();
    let name = line.strip_prefix("  ")?.strip_suffix(':')?;
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    valid.then_some(name)
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

[profile.default.junit]
path = \"junit.xml\"
";

const SHARD_JOB: &str = "  shard:
    name: shard (${{ matrix.name }})
    strategy:
      fail-fast: false
      matrix:
        include:
          - name: app
            tests: --test app
          - name: rest
            tests: -E 'not binary(app)'
            rest: true
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
        with:
          key: ${{ matrix.name }}
      - if: matrix.rest
        run: sudo apt-get update && sudo apt-get install -y --no-install-recommends tmux
      - uses: taiki-e/install-action@3f74d7c16a4242f1c95561e98edc25d36adb4375 # v2.87.12
        with:
          tool: cargo-nextest@0.9.144
          fallback: none
      - if: matrix.rest
        run: cargo fmt --check
      - if: matrix.rest
        run: cargo clippy --locked --all-targets -- -D warnings
      - run: cargo nextest run --locked --config-file .github/nextest.toml --run-ignored all ${{ matrix.tests }}
      - if: matrix.rest
        run: cargo test --doc --locked
      - if: ${{ !cancelled() }}
        continue-on-error: true
        uses: codecov/codecov-action@303a32d7a59b442fa8d48b6a1cc6825c09c847a5 # v7.1.1
        with:
          report_type: test_results
          files: target/nextest/default/junit.xml
          disable_search: true
          flags: ${{ matrix.name }}
          token: ${{ secrets.CODECOV_TOKEN }}
    timeout-minutes: 10
";

const CHECK_JOB: &str = "  check:
    needs: shard
    if: ${{ !cancelled() }}
    runs-on: ubuntu-latest
    steps:
      - env:
          SHARDS: ${{ needs.shard.result }}
        run: test \"$SHARDS\" = success
    timeout-minutes: 5
";

const AUDIT_JOB: &str = "  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09 # v5
";
