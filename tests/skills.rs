const ORCHESTRATE: &str =
    include_str!("../.claude/skills/orchestrate-issue-execution-autonomously/SKILL.md");

const OWNER: &str = "--author <login>";

const SWEEP_WORKFLOW: &str = "--author app/github-actions";

#[test]
fn the_orchestrator_finds_the_owners_login_in_a_step_of_its_own() {
    assert!(
        ORCHESTRATE
            .lines()
            .any(|line| line.trim() == "gh repo view --json owner --jq .owner.login")
    );
}

#[test]
fn the_orchestrator_lists_only_the_owners_issues_to_build() {
    assert!(issue_listings()[0].contains(OWNER));
}

#[test]
fn the_orchestrator_lists_the_owners_issues_with_their_author() {
    assert!(issue_listings()[0].contains("--json number,title,author"));
}

#[test]
fn the_orchestrator_lists_the_sweep_workflows_issues_by_number_and_title() {
    assert!(ORCHESTRATE.contains(&format!(
        "gh issue list --state open {SWEEP_WORKFLOW} --json number,title"
    )));
}

#[test]
fn the_orchestrator_reads_no_text_of_an_issue_that_is_not_the_owners() {
    assert_eq!(reading_a_strangers_text(ORCHESTRATE), Vec::<&str>::new());
}

#[test]
fn a_listing_without_the_owner_filter_that_reads_a_title_is_caught() {
    let line = "gh issue list --state open --json number,title";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

#[test]
fn a_listing_without_the_owner_filter_that_reads_a_body_is_caught() {
    let line = "gh issue list --json number,body,author";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

#[test]
fn a_listing_without_the_owner_filter_that_reads_comments_is_caught() {
    let line = "gh issue list --json number,comments";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

#[test]
fn a_listing_with_quoted_fields_is_read_without_its_quotes() {
    let line = "gh issue list --json \"number,title\"";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

#[test]
fn a_listing_without_json_prints_titles_and_is_caught() {
    let line = "gh issue list --state open --limit 100";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

#[test]
fn a_view_of_an_issue_that_reads_its_body_is_caught() {
    let line = "gh issue view 7 --json body";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

#[test]
fn an_api_read_of_an_issue_without_a_filter_is_caught() {
    let line = "gh api repos/acme/widgets/issues/7";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

#[test]
fn an_api_read_of_an_issue_whose_filter_reads_its_body_is_caught() {
    let line = "gh api repos/acme/widgets/issues/7 --jq .body";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

#[test]
fn an_api_read_of_blocking_issue_numbers_passes() {
    let line = "gh api repos/acme/widgets/issues/7/dependencies/blocked_by --jq '.[].number'";
    assert_eq!(reading_a_strangers_text(line), Vec::<&str>::new());
}

#[test]
fn a_listing_of_the_owners_issues_may_read_their_text() {
    let line = "gh issue list --author <login> --json number,title,body,author";
    assert_eq!(reading_a_strangers_text(line), Vec::<&str>::new());
}

#[test]
fn a_listing_of_the_sweep_workflows_issues_may_read_their_title() {
    let line = "gh issue list --author app/github-actions --json number,title";
    assert_eq!(reading_a_strangers_text(line), Vec::<&str>::new());
}

#[test]
fn a_listing_of_the_sweep_workflows_issues_that_reads_a_body_is_caught() {
    let line = "gh issue list --author app/github-actions --json number,body";
    assert_eq!(reading_a_strangers_text(line), vec![line]);
}

fn issue_listings() -> Vec<&'static str> {
    ORCHESTRATE
        .lines()
        .filter(|line| line.contains("gh issue list"))
        .collect()
}

fn reading_a_strangers_text(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| reads_an_issue(line))
        .filter(|line| !line.contains(OWNER))
        .filter(|line| reads_text(line))
        .collect()
}

fn reads_an_issue(line: &str) -> bool {
    line.contains("gh issue list")
        || line.contains("gh issue view")
        || (line.contains("gh api") && line.contains("/issues/"))
}

fn reads_text(line: &str) -> bool {
    if line.contains("gh api") {
        return match after(line, "--jq") {
            Some(filter) => [".title", ".body", ".comments"]
                .iter()
                .any(|field| filter.contains(field)),
            None => true,
        };
    }
    let Some(fields) = after(line, "--json") else {
        return true;
    };
    let allowed_title = line.contains(SWEEP_WORKFLOW);
    fields
        .trim_matches(['"', '\''])
        .split(',')
        .any(|field| field == "body" || field == "comments" || (field == "title" && !allowed_title))
}

fn after<'a>(line: &'a str, flag: &str) -> Option<&'a str> {
    let mut words = line.split_whitespace();
    words.find(|word| *word == flag)?;
    words.next()
}
