use std::path::Path;

const READS_THE_PROCESS_DIRECTORY: &str = "current_dir()";
const SCREEN_LITERAL: &str = "Screen {";
const SCREEN_HARNESS: &str = "tests/app/main.rs";
const ITSELF: &str = "tests/checkout_name.rs";

const ALLOWED: &[(&str, &str)] = &[
    (
        "tests/app/moving.rs",
        "let before = std::env::current_dir().unwrap();",
    ),
    (
        "tests/app/moving.rs",
        "assert_eq!(std::env::current_dir().unwrap(), before);",
    ),
];

#[test]
fn no_test_reads_the_directory_cargo_runs_it_from() {
    assert_eq!(offenders(), "");
}

#[test]
fn the_scan_names_a_planted_offender_with_its_line() {
    let found = scan_text(
        "tests/acme.rs",
        "fn acme() {\n    let here = std::env::current_dir().unwrap();\n}\n",
    );

    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("tests/acme.rs:2: "), "{found:?}");
}

#[test]
fn the_scan_names_a_screen_built_outside_the_harness() {
    let found = scan_text(
        "tests/app/acme.rs",
        "fn acme() -> Screen {\n    let screen = Screen {\n        app,\n    };\n}\n",
    );

    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("tests/app/acme.rs:2: "), "{found:?}");
}

#[test]
fn the_scan_leaves_a_command_given_its_own_directory_alone() {
    let found = scan_text(
        "tests/acme.rs",
        "    command.current_dir(scratch.path());\n",
    );

    assert!(found.is_empty(), "{found:?}");
}

fn offenders() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found = Vec::new();
    collect(&root.join("tests"), root, &mut found);
    found.sort();
    found.join("\n")
}

fn collect(directory: &Path, root: &Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, root, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let name = path.strip_prefix(root).unwrap().display().to_string();
            if name != ITSELF {
                let text = std::fs::read_to_string(&path).unwrap();
                found.extend(scan_text(&name, &text));
            }
        }
    }
}

fn scan_text(name: &str, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let code = line.trim();
        if ALLOWED.contains(&(name, code)) {
            continue;
        }
        if code.contains(READS_THE_PROCESS_DIRECTORY) {
            found.push(format!(
                "{name}:{}: reads the directory cargo runs the test from, the checkout; use a scratch folder: {code}",
                index + 1
            ));
        }
        if builds_a_screen(name, code) {
            found.push(format!(
                "{name}:{}: builds a Screen outside {SCREEN_HARNESS}, so its app may start sessions in the checkout; use Screen::of or Screen::at: {code}",
                index + 1
            ));
        }
    }
    found
}

fn builds_a_screen(name: &str, code: &str) -> bool {
    name.starts_with("tests/app/")
        && name != SCREEN_HARNESS
        && (code.starts_with(SCREEN_LITERAL) || code.contains(&format!("= {SCREEN_LITERAL}")))
}
