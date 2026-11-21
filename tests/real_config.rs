use std::path::Path;

const LAUNCH: &str = "CARGO_BIN_EXE_harbour";
const ITSELF: &str = "tests/real_config.rs";
const SCREENSHOTS: &str = "scripts/screenshots.sh";

#[test]
fn every_launch_of_the_binary_keeps_the_real_config_out() {
    assert_eq!(offenders(), "");
}

#[test]
fn the_screenshots_keep_the_real_config_out() {
    let script =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(SCREENSHOTS)).unwrap();

    assert_eq!(unisolated_screenshots(&script), "");
}

#[test]
fn the_scan_names_a_launch_that_sets_neither() {
    let found = scan_text(
        "tests/acme.rs",
        "fn acme() {\n    Command::new(env!(\"CARGO_BIN_EXE_harbour\"))\n        .arg(\"status\")\n        .output();\n}\n",
    );

    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("tests/acme.rs:2: "), "{found:?}");
}

#[test]
fn the_scan_names_a_launch_that_sets_only_home() {
    let found = scan_text(
        "tests/acme.rs",
        "fn acme(home: &Path) {\n    Command::new(env!(\"CARGO_BIN_EXE_harbour\"))\n        .env(\"HOME\", home)\n        .output();\n}\n",
    );

    assert_eq!(found.len(), 1);
}

#[test]
fn the_scan_names_a_launch_that_only_removes_xdg_config_home() {
    let found = scan_text(
        "tests/acme.rs",
        "fn acme() {\n    Command::new(env!(\"CARGO_BIN_EXE_harbour\"))\n        .env_remove(\"XDG_CONFIG_HOME\")\n        .output();\n}\n",
    );

    assert_eq!(found.len(), 1);
}

#[test]
fn the_scan_leaves_a_launch_with_a_scratch_home_alone() {
    let found = scan_text(
        "tests/acme.rs",
        "fn acme(home: &Path) {\n    Command::new(env!(\"CARGO_BIN_EXE_harbour\"))\n        .env(\"HOME\", home)\n        .env_remove(\"XDG_CONFIG_HOME\")\n        .output();\n}\n",
    );

    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn the_scan_leaves_a_launch_with_a_scratch_config_folder_alone() {
    let found = scan_text(
        "tests/acme.rs",
        "fn acme(home: &Path) {\n    Command::new(env!(\"CARGO_BIN_EXE_harbour\"))\n        .env(\"XDG_CONFIG_HOME\", home)\n        .output();\n}\n",
    );

    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn the_scan_reads_only_the_function_that_launches() {
    let found = scan_text(
        "tests/acme.rs",
        "fn acme() {\n    Command::new(env!(\"CARGO_BIN_EXE_harbour\")).output();\n}\n\nfn other(home: &Path) {\n    command.env(\"HOME\", home).env_remove(\"XDG_CONFIG_HOME\");\n}\n",
    );

    assert_eq!(found.len(), 1);
}

#[test]
fn the_screenshot_check_names_a_launch_that_keeps_xdg_config_home() {
    let found = unisolated_screenshots(
        "t respawn-pane -k -t \"$shot_window\" \"env HOME=$(printf %q \"$home\") $(printf %q \"$bin\") $args\"\n",
    );

    assert!(found.starts_with("scripts/screenshots.sh:1: "), "{found}");
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
    let lines: Vec<&str> = text.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains(LAUNCH))
        .filter(|(index, _)| !isolated(&function_around(&lines, *index)))
        .map(|(index, line)| {
            format!(
                "{name}:{}: launches harbour without setting both HOME and XDG_CONFIG_HOME, or XDG_CONFIG_HOME alone: {}",
                index + 1,
                line.trim()
            )
        })
        .collect()
}

fn function_around(lines: &[&str], at: usize) -> String {
    let start = (0..=at)
        .rev()
        .find(|index| is_function(lines[*index]))
        .unwrap_or(0);
    let indent = &lines[start][..lines[start].len() - lines[start].trim_start().len()];
    let closing = format!("{indent}}}");
    let end = (at..lines.len())
        .find(|index| lines[*index] == closing)
        .unwrap_or(lines.len() - 1);
    lines[start..=end].join("\n")
}

fn is_function(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("fn ") || line.starts_with("pub fn ") || line.starts_with("pub(crate) fn ")
}

fn isolated(function: &str) -> bool {
    let sets_home = function.contains("\"HOME\"") || function.contains("HOME=");
    function.contains(".env(\"XDG_CONFIG_HOME\"")
        || (function.contains("XDG_CONFIG_HOME") && sets_home)
}

fn unisolated_screenshots(script: &str) -> String {
    script
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("\"$bin\")") && line.contains("respawn-pane"))
        .filter(|(_, line)| !(line.contains("-u XDG_CONFIG_HOME") && line.contains("HOME=")))
        .map(|(index, line)| format!("{SCREENSHOTS}:{}: {}", index + 1, line.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}
