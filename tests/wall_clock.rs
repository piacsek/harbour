use std::path::{Path, PathBuf};

const GUARD: &str = "tests/wall_clock.rs";
const SEAMS: &[&str] = &[
    "with_program(",
    "with_patience(",
    "with_hangup_grace(",
    "hand(",
    "finish(",
    "retrying_every(",
    "nvim::mode(",
    "nvim::eval(",
];

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

#[test]
fn a_constant_bound_that_allows_equality_is_found() {
    let source = "    assert!(elapsed <= Duration::from_millis(500));\n";

    assert_eq!(constant_upper_bounds(source), vec![1]);
}

#[test]
fn a_constant_bound_written_the_other_way_round_is_found() {
    let source = "    assert!(Duration::from_millis(500) > elapsed);\n";

    assert_eq!(constant_upper_bounds(source), vec![1]);
}

#[test]
fn a_constant_wait_for_something_that_must_arrive_is_found() {
    let source = "    let first = asked.recv_timeout(Duration::from_secs(5)).unwrap();\n";

    assert_eq!(constant_upper_bounds(source), vec![1]);
}

#[test]
fn a_constant_wait_for_something_that_must_not_arrive_is_not_found() {
    let source = "    assert!(asked.recv_timeout(Duration::from_millis(200)).is_err());\n";

    assert!(constant_upper_bounds(source).is_empty());
}

#[test]
fn a_literal_deadline_in_a_seam_call_that_opens_with_an_argument_is_found() {
    let source = "    let tmux = Tmux::with_program(vec![\n        program(),\n    ],\n        Duration::from_secs(5),\n    );\n";

    assert_eq!(literal_seam_deadlines(source), vec![4]);
}

#[test]
fn a_literal_deadline_after_a_script_with_a_lone_parenthesis_is_found() {
    let source = "    let tmux = Tmux::with_program(\n        program(\n            \"case $1 in a) sleep 9 ;; esac\",\n        ),\n        Duration::from_secs(5),\n    );\n";

    assert_eq!(literal_seam_deadlines(source), vec![5]);
}

#[test]
fn a_constant_bound_on_a_count_of_milliseconds_is_found() {
    let source = "    assert!(elapsed.as_millis() < 500);\n";

    assert_eq!(constant_upper_bounds(source), vec![1]);
}

#[test]
fn a_constant_bound_on_a_count_of_seconds_that_allows_equality_is_found() {
    let source = "    assert!(elapsed.as_secs() <= 2);\n";

    assert_eq!(constant_upper_bounds(source), vec![1]);
}

#[test]
fn a_literal_deadline_handed_to_the_app_is_found() {
    let source = "    let reply = screen.app.hand(\n        Verb::Open,\n        &file(scratch.path(), \"acme-api\"),\n        Instant::now() + Duration::from_millis(500),\n    );\n";

    assert_eq!(literal_seam_deadlines(source), vec![4]);
}

#[test]
fn a_literal_deadline_handed_to_finish_is_found() {
    let source = "    let elapsed = finish(Duration::from_millis(500));\n";

    assert_eq!(literal_seam_deadlines(source), vec![1]);
}

#[test]
fn a_literal_retry_interval_is_found_but_no_retry_is_not() {
    let source = "    Listener::retrying_every(socket, Duration::from_millis(5));\n    Listener::retrying_every(socket, Duration::ZERO);\n";

    assert_eq!(literal_seam_deadlines(source), vec![1]);
}

#[test]
fn a_literal_deadline_handed_to_neovim_is_found() {
    let source = "    nvim::mode(&socket, Instant::now() + Duration::from_secs(1));\n    nvim::eval(&socket, \"1\", Instant::now() + Duration::from_secs(1));\n";

    assert_eq!(literal_seam_deadlines(source), vec![1, 2]);
}

#[test]
fn a_literal_deadline_after_a_parenthesis_in_a_character_is_found() {
    let source =
        "    screen.app.hand(\n        key(')'),\n        Duration::from_secs(5),\n    );\n";

    assert_eq!(literal_seam_deadlines(source), vec![3]);
}

#[test]
fn a_literal_deadline_inside_the_leftmost_of_two_seams_on_a_line_is_found() {
    let source = "    screen.app.hand(Verb::Open, with_program(acme()),\n        Duration::from_secs(5),\n    );\n";

    assert_eq!(literal_seam_deadlines(source), vec![2]);
}

#[test]
fn a_call_whose_name_only_ends_like_a_seam_is_not_a_seam() {
    let source = "    let shown = beforehand(\n        Duration::from_secs(5),\n    );\n";

    assert!(literal_seam_deadlines(source).is_empty());
}

#[test]
fn a_bound_on_a_count_tied_to_the_shared_deadline_is_not_found() {
    let source = "    assert!(elapsed.as_millis() < deadline().as_millis());\n";

    assert!(constant_upper_bounds(source).is_empty());
}

#[test]
fn a_constant_wait_followed_by_an_error_check_of_something_else_is_found() {
    let source = "    let first = asked.recv_timeout(Duration::from_secs(5)).unwrap(); assert!(other.is_err());\n";

    assert_eq!(constant_upper_bounds(source), vec![1]);
}

#[test]
fn a_count_bound_scaled_from_the_shared_deadline_is_not_found() {
    let source = "    assert!(elapsed.as_millis() < 2 * deadline().as_millis());\n";

    assert!(constant_upper_bounds(source).is_empty());
}

#[test]
fn a_constant_bound_on_any_count_of_a_duration_is_found() {
    let source = "    assert!(elapsed.as_secs_f64() < 1.5);\n    assert!(elapsed.as_secs_f32() < 1.5);\n    assert!(elapsed.as_micros() < 500);\n    assert!(elapsed.as_nanos() <= 500);\n";

    assert_eq!(constant_upper_bounds(source), vec![1, 2, 3, 4]);
}

fn constant_upper_bounds(source: &str) -> Vec<usize> {
    numbered(source)
        .filter(|(_, line)| bounds_by_a_constant(line))
        .map(|(number, _)| number)
        .collect()
}

fn bounds_by_a_constant(line: &str) -> bool {
    line.contains("< Duration::from_")
        || line.contains("<= Duration::from_")
        || constant_wait_that_must_arrive(line)
        || constant_on_the_left(line)
        || constant_count_bound(line)
}

fn constant_wait_that_must_arrive(line: &str) -> bool {
    line.match_indices("recv_timeout(").any(|(start, call)| {
        let rest = &line[start + call.len()..];
        rest.starts_with("Duration::from_")
            && closing(rest).is_none_or(|end| !rest[end + 1..].starts_with(".is_err()"))
    })
}

fn closing(text: &str) -> Option<usize> {
    let mut depth = 1;
    text.char_indices().find_map(|(index, character)| {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        (depth == 0).then_some(index)
    })
}

const COUNTS: &[&str] = &[
    "as_millis() <",
    "as_secs() <",
    "as_secs_f64() <",
    "as_secs_f32() <",
    "as_micros() <",
    "as_nanos() <",
];

fn constant_count_bound(line: &str) -> bool {
    COUNTS.iter().any(|count| {
        line.match_indices(count).any(|(start, _)| {
            let bound = line[start + count.len()..]
                .trim_start_matches('=')
                .trim_start();
            bound.starts_with(|character: char| character.is_ascii_digit())
                && !bound.contains("deadline()")
        })
    })
}

fn constant_on_the_left(line: &str) -> bool {
    line.match_indices("Duration::from_").any(|(start, _)| {
        let rest = &line[start..];
        rest.find(')')
            .is_some_and(|close| rest[close + 1..].trim_start().starts_with('>'))
    })
}

fn literal_seam_deadlines(source: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut call = Call::default();
    for (number, line) in numbered(source) {
        let opened = leftmost_seam(line);
        let inside = call.depth > 0;
        if (inside || opened.is_some()) && line.contains("Duration::from_") {
            found.push(number);
        }
        match opened {
            Some(start) if !inside => call.read(&line[start..]),
            _ if inside => call.read(line),
            _ => {}
        }
    }
    found
}

fn leftmost_seam(line: &str) -> Option<usize> {
    SEAMS
        .iter()
        .flat_map(|seam| line.match_indices(seam).map(|(start, _)| start))
        .filter(|start| {
            !line[..*start]
                .ends_with(|character: char| character.is_alphanumeric() || character == '_')
        })
        .min()
}

fn skip_character_literal(characters: &mut std::iter::Peekable<std::str::Chars>) {
    let rest: String = characters.clone().take(3).collect();
    let literal = match rest.chars().next() {
        Some('\\') => rest
            .char_indices()
            .skip(2)
            .find(|(_, character)| *character == '\'')
            .map(|(end, _)| end + 1),
        Some(_) if rest.chars().nth(1) == Some('\'') => Some(2),
        _ => None,
    };
    for _ in 0..literal.unwrap_or(0) {
        characters.next();
    }
}

#[derive(Default)]
struct Call {
    depth: i32,
    quoted: bool,
}

impl Call {
    fn read(&mut self, text: &str) {
        let mut escaped = false;
        let mut characters = text.chars().peekable();
        while let Some(character) = characters.next() {
            match character {
                _ if escaped => escaped = false,
                '\'' if !self.quoted => skip_character_literal(&mut characters),
                '\\' if self.quoted => escaped = true,
                '"' => self.quoted = !self.quoted,
                '(' if !self.quoted => self.depth += 1,
                ')' if !self.quoted => self.depth -= 1,
                _ => {}
            }
        }
        if self.depth <= 0 {
            *self = Call::default();
        }
    }
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
