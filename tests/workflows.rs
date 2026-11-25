use std::fs;
use std::path::PathBuf;

const RELEASE: &str = include_str!("../.github/workflows/release.yml");

#[test]
fn one_release_job_reads_the_tap_token() {
    assert_eq!(jobs_reading_the_tap_token().len(), 1);
}

#[test]
fn the_job_reading_the_tap_token_runs_in_the_homebrew_tap_environment() {
    assert!(
        tap_job()
            .lines()
            .any(|line| line == "    environment: homebrew-tap")
    );
}

#[test]
fn the_job_reading_the_tap_token_checks_something_out() {
    assert!(!checkouts(&tap_job()).is_empty());
}

#[test]
fn every_checkout_of_the_job_reading_the_tap_token_pins_a_ref() {
    assert_eq!(unpinned(&checkouts(&tap_job())), Vec::<String>::new());
}

#[test]
fn no_workflow_but_release_names_the_tap_token() {
    assert_eq!(workflows_naming_the_tap_token(), vec!["release.yml"]);
}

fn tap_job() -> String {
    jobs_reading_the_tap_token().remove(0)
}

fn jobs_reading_the_tap_token() -> Vec<String> {
    jobs()
        .into_iter()
        .filter(|job| job.contains("secrets.TAP_TOKEN"))
        .collect()
}

fn jobs() -> Vec<String> {
    let body = RELEASE.split_once("\njobs:\n").unwrap().1;
    split_before(body, |line| {
        line.starts_with("  ") && !line.starts_with("   ") && line.ends_with(':')
    })
}

fn checkouts(job: &str) -> Vec<String> {
    split_before(job, |line| line.starts_with("      - "))
        .into_iter()
        .filter(|step| step.contains("uses: actions/checkout@"))
        .collect()
}

fn unpinned(steps: &[String]) -> Vec<String> {
    steps
        .iter()
        .filter(|step| {
            !step
                .lines()
                .any(|line| line.trim_start().starts_with("ref: "))
        })
        .cloned()
        .collect()
}

fn split_before(text: &str, starts: impl Fn(&str) -> bool) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    for line in text.lines() {
        if starts(line) {
            blocks.push(String::new());
        }
        if let Some(block) = blocks.last_mut() {
            block.push_str(line);
            block.push('\n');
        }
    }
    blocks
}

fn workflows_naming_the_tap_token() -> Vec<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows");
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| fs::read_to_string(path).unwrap().contains("TAP_TOKEN"))
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
