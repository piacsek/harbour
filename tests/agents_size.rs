use std::path::Path;

const ROOT_LIMIT: usize = 12 * 1024;
const LAYOUT_LINE_LIMIT: usize = 120;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn root_agents() -> String {
    std::fs::read_to_string(root().join("AGENTS.md")).unwrap()
}

#[test]
fn the_root_agents_file_stays_under_its_limit() {
    let bytes = root_agents().len();
    assert!(
        bytes <= ROOT_LIMIT,
        "AGENTS.md is {bytes} bytes, over {ROOT_LIMIT}: move detail into the AGENTS.md of the directory it is about, or into docs/"
    );
}

#[test]
fn every_layout_line_is_short() {
    let text = root_agents();
    let layout = text
        .split("\n## Layout\n")
        .nth(1)
        .expect("AGENTS.md has a ## Layout section");
    let block = layout
        .split("```")
        .nth(1)
        .expect("the Layout section holds a fenced block");
    let long: Vec<String> = block
        .lines()
        .filter(|line| line.chars().count() > LAYOUT_LINE_LIMIT)
        .map(|line| line.split_whitespace().next().unwrap_or("").to_string())
        .collect();
    assert_eq!(
        long,
        Vec::<String>::new(),
        "Layout lines over {LAYOUT_LINE_LIMIT} characters"
    );
}

#[test]
fn each_directory_guide_loads_as_claude_md() {
    let missing: Vec<&str> = ["tests", "scripts", "docs", ".github"]
        .into_iter()
        .filter(|directory| {
            let link = root().join(directory).join("CLAUDE.md");
            std::fs::read_link(&link).ok().as_deref() != Some(Path::new("AGENTS.md"))
        })
        .collect();
    assert_eq!(
        missing,
        Vec::<&str>::new(),
        "no CLAUDE.md symlink to AGENTS.md"
    );
}
