use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::helpers::{stderr, stdout, which};

#[test]
fn the_slowest_tests_are_drawn_slowest_first_with_their_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let report = junit(
        dir.path(),
        "app.xml",
        &[
            ("harbour::app", "focus::quick", "0.012"),
            ("harbour::app", "picker::slow", "4.500"),
            ("harbour::e2e", "cards::middling", "1.250"),
        ],
    );

    let output = test_speed(&[&report]);

    let svg = stdout(&output);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(svg.contains(">app picker::slow</text>"), "{svg}");
    assert!(svg.contains(">4.50 s</text>"), "{svg}");
    assert!(svg.contains(">e2e cards::middling</text>"), "{svg}");
    assert!(svg.contains(">1.25 s</text>"), "{svg}");
    let slow = svg.find("picker::slow").unwrap();
    let middling = svg.find("cards::middling").unwrap();
    let quick = svg.find("focus::quick").unwrap();
    assert!(slow < middling && middling < quick, "{svg}");
}

#[test]
fn every_report_given_counts_and_only_the_fifteen_slowest_are_drawn() {
    let dir = tempfile::tempdir().unwrap();
    let names: Vec<String> = (1..=20).map(|n| format!("t{n:02}")).collect();
    let times: Vec<String> = (1..=20).map(|n| format!("{n}.0")).collect();
    let cases: Vec<(&str, &str, &str)> = names
        .iter()
        .zip(&times)
        .map(|(name, time)| ("harbour::app", name.as_str(), time.as_str()))
        .collect();
    let first = junit(dir.path(), "app.xml", &cases[..10]);
    let second = junit(dir.path(), "rest.xml", &cases[10..]);

    let output = test_speed(&[&first, &second]);

    let svg = stdout(&output);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(svg.contains(">app t20</text>"), "{svg}");
    assert!(svg.contains(">app t06</text>"), "{svg}");
    assert!(!svg.contains(">app t05</text>"), "{svg}");
    assert!(svg.contains(">20 tests, 210.0 s in all</text>"), "{svg}");
}

#[test]
fn the_chart_is_one_svg_document() {
    let dir = tempfile::tempdir().unwrap();
    let report = junit(dir.path(), "app.xml", &[("harbour", "a::b", "0.5")]);

    let output = test_speed(&[&report]);

    let svg = stdout(&output);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""),
        "{svg}"
    );
    assert!(svg.ends_with("</svg>\n"), "{svg}");
    assert!(svg.contains(">harbour a::b</text>"), "{svg}");
}

#[test]
fn a_report_without_tests_draws_nothing_and_fails() {
    let dir = tempfile::tempdir().unwrap();
    let report = junit(dir.path(), "empty.xml", &[]);

    let output = test_speed(&[&report]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("no test"), "{}", stderr(&output));
}

fn junit(dir: &Path, name: &str, cases: &[(&str, &str, &str)]) -> PathBuf {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites name=\"nextest-run\">\n<testsuite name=\"harbour::app\">\n",
    );
    for (class, test, time) in cases {
        xml.push_str(&format!(
            "<testcase name=\"{test}\" classname=\"{class}\" timestamp=\"2026-11-24T10:00:00Z\" time=\"{time}\">\n</testcase>\n"
        ));
    }
    xml.push_str("</testsuite>\n</testsuites>\n");
    let path = dir.join(name);
    fs::write(&path, xml).unwrap();
    path
}

fn test_speed(reports: &[&Path]) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/test-speed.py");
    Command::new(which("python3"))
        .arg(script)
        .args(reports)
        .output()
        .unwrap()
}
