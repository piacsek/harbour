use harbour::help::KEYS;

const REVIEW: &str = include_str!("../.claude/skills/adversarial-review/SKILL.md");

const PREFIX: &str = "C-_";

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

#[test]
fn the_review_skill_sends_every_prefixed_key_of_the_help() {
    let missing = missing(&review_keys());
    assert!(missing.is_empty(), "{missing:?}");
}

#[test]
fn the_review_skill_sends_no_prefixed_key_of_the_help_without_the_prefix() {
    let unprefixed = unprefixed(&review_keys());
    assert!(unprefixed.is_empty(), "{unprefixed:?}");
}

#[test]
fn the_review_skill_ends_by_quitting_and_answering_yes() {
    let keys = review_keys();
    let last = keys.len() - 1;
    assert_eq!(keys[last - 2], PREFIX);
    assert_eq!(keys[last - 1], "q");
    assert_eq!(keys[last], "y");
}

#[test]
fn a_prefixed_key_sent_without_the_prefix_is_caught() {
    let unprefixed = unprefixed(&keys(&["q", "y"]));
    assert_eq!(unprefixed.len(), 1);
    assert_eq!(unprefixed[0], "q");
}

#[test]
fn a_prefixed_key_never_sent_is_caught() {
    assert!(missing(&keys(&[PREFIX, "q"])).contains(&"?".to_string()));
}

#[test]
fn the_prefix_sent_twice_sends_the_prefix_key() {
    assert!(!missing(&keys(&[PREFIX, PREFIX])).contains(&PREFIX.to_string()));
}

#[test]
fn the_prefix_sent_twice_is_not_a_prefixed_key_alone() {
    let unprefixed = unprefixed(&keys(&[PREFIX, PREFIX]));
    assert!(unprefixed.is_empty(), "{unprefixed:?}");
}

#[test]
fn a_jump_is_sent_as_its_first_digit() {
    assert!(!missing(&keys(&[PREFIX, "1"])).contains(&"1".to_string()));
}

#[test]
fn the_review_skill_sends_every_key_of_the_help_that_needs_no_prefix() {
    let missing = missing_alone(&review_keys());
    assert!(missing.is_empty(), "{missing:?}");
}

#[test]
fn a_key_that_needs_no_prefix_never_sent_is_caught() {
    assert!(missing_alone(&keys(&["j"])).contains(&"Enter".to_string()));
}

#[test]
fn a_key_that_needs_no_prefix_sent_after_the_prefix_is_caught() {
    assert!(missing_alone(&keys(&[PREFIX, "x"])).contains(&"x".to_string()));
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

fn review_keys() -> Vec<String> {
    let line = REVIEW
        .lines()
        .find(|line| line.trim_start().starts_with("keys=("))
        .expect("the review skill's keys=( line");
    let inside = line
        .trim()
        .trim_start_matches("keys=(")
        .trim_end_matches(')');
    inside
        .split_whitespace()
        .map(|key| key.trim_matches(['\'', '"']).to_string())
        .collect()
}

fn keys(sent: &[&str]) -> Vec<String> {
    sent.iter().map(|key| key.to_string()).collect()
}

fn prefixed_keys() -> Vec<String> {
    KEYS.iter()
        .filter_map(|(keys, _)| keys.strip_prefix("C-/ "))
        .flat_map(|keys| {
            if keys == "C-/" {
                return vec![PREFIX.to_string()];
            }
            keys.split([' ', '/'])
                .flat_map(|word| match word.split_once("..") {
                    Some((first, _)) => vec![first.to_string()],
                    None => word.chars().map(String::from).collect(),
                })
                .collect()
        })
        .collect()
}

fn after_the_prefix(sent: &[String]) -> (Vec<String>, Vec<String>) {
    let mut prefixed = Vec::new();
    let mut alone = Vec::new();
    let mut sent = sent.iter();
    while let Some(key) = sent.next() {
        if key == PREFIX {
            prefixed.extend(sent.next().cloned());
        } else {
            alone.push(key.clone());
        }
    }
    (prefixed, alone)
}

fn missing(sent: &[String]) -> Vec<String> {
    let (prefixed, _) = after_the_prefix(sent);
    prefixed_keys()
        .into_iter()
        .filter(|key| !prefixed.contains(key))
        .collect()
}

fn unprefixed(sent: &[String]) -> Vec<String> {
    let (_, alone) = after_the_prefix(sent);
    let needs_the_prefix = prefixed_keys();
    alone
        .into_iter()
        .filter(|key| needs_the_prefix.contains(key))
        .collect()
}

fn keys_alone() -> Vec<String> {
    KEYS.iter()
        .filter(|(keys, _)| !keys.starts_with("C-/ "))
        .flat_map(|(keys, _)| keys.split([' ', '/']))
        .map(String::from)
        .collect()
}

fn missing_alone(sent: &[String]) -> Vec<String> {
    let (_, alone) = after_the_prefix(sent);
    keys_alone()
        .into_iter()
        .filter(|key| !alone.contains(key))
        .collect()
}
