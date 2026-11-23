use std::path::{Path, PathBuf};

const GUARD: &str = "tests/wall_clock.rs";
const SEAMS: &[&str] = &["with_program(", "with_patience("];

#[test]
fn no_test_bounds_elapsed_time_from_above_by_a_constant() {
    assert_eq!(found(constant_upper_bounds), "");
}

#[test]
fn no_test_hands_a_seam_a_literal_deadline() {
    assert_eq!(found(literal_seam_deadlines), "");
}

#[test]
fn a_constant_upper_bound_is_found() {
    let source = "    assert!(elapsed < Duration::from_millis(500));\n";

    assert_eq!(constant_upper_bounds(source), vec![1]);
}

#[test]
fn a_bound_tied_to_the_shared_deadline_is_not_found() {
    let source = "    assert!(elapsed < 2 * deadline());\n";

    assert!(constant_upper_bounds(source).is_empty());
}

#[test]
fn a_literal_deadline_across_the_lines_of_a_seam_call_is_found() {
    let source = "    let tmux = Tmux::with_program(\n        program(),\n        Duration::from_secs(5),\n    );\n";

    assert_eq!(literal_seam_deadlines(source), vec![3]);
}

#[test]
fn a_literal_deadline_after_a_nested_call_of_a_seam_call_is_found() {
    let source = "    let tmux = Tmux::with_program(\n        program(\n            scratch.path(),\n        ),\n        Duration::from_secs(5),\n    );\n";

    assert_eq!(literal_seam_deadlines(source), vec![5]);
}

#[test]
fn a_literal_deadline_on_the_line_of_a_seam_call_is_found() {
    let source = "    fake_neovim().with_patience(Duration::from_secs(5))\n";

    assert_eq!(literal_seam_deadlines(source), vec![1]);
}

#[test]
fn a_duration_after_a_seam_call_ends_is_not_found() {
    let source = "    Tmux::with_program(\n        program(),\n        5 * deadline(),\n    );\n    sleep(Duration::from_millis(50));\n";

    assert!(literal_seam_deadlines(source).is_empty());
}

fn constant_upper_bounds(source: &str) -> Vec<usize> {
    numbered(source)
        .filter(|(_, line)| line.contains("< Duration::from_"))
        .map(|(number, _)| number)
        .collect()
}

fn literal_seam_deadlines(source: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut opened_at: Option<usize> = None;
    for (number, line) in numbered(source) {
        let opens = SEAMS.iter().any(|seam| line.contains(seam));
        if (opened_at.is_some() || opens) && line.contains("Duration::from_") {
            found.push(number);
        }
        if opens && line.trim_end().ends_with('(') {
            opened_at = Some(indent(line));
        } else if line.trim_start().starts_with(')')
            && opened_at.is_some_and(|opened| indent(line) <= opened)
        {
            opened_at = None;
        }
    }
    found
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn numbered(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line))
}

fn found(scan: fn(&str) -> Vec<usize>) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect(&root.join("tests"), &mut files);
    files.sort();
    files
        .iter()
        .map(|path| path.strip_prefix(root).unwrap())
        .filter(|path| *path != Path::new(GUARD))
        .flat_map(|path| {
            let source = std::fs::read_to_string(root.join(path)).unwrap();
            scan(&source)
                .into_iter()
                .map(move |number| format!("{}:{number}", path.display()))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}
