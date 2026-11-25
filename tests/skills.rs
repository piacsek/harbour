const ORCHESTRATE: &str =
    include_str!("../.claude/skills/orchestrate-issue-execution-autonomously/SKILL.md");

const OWNER: &str = "--author \"$(gh repo view --json owner --jq .owner.login)\"";

#[test]
fn the_orchestrator_lists_only_the_owners_issues_to_build() {
    assert!(issue_listings()[0].contains(OWNER));
}

#[test]
fn the_orchestrator_lists_the_owners_issues_with_their_author() {
    assert!(issue_listings()[0].contains("--json number,title,author"));
}

#[test]
fn a_listing_of_anyone_elses_issues_reads_no_title_or_body() {
    assert_eq!(listings_reading_a_strangers_text(), Vec::<&str>::new());
}

#[test]
fn a_listing_without_the_owner_filter_that_reads_a_title_is_caught() {
    assert_eq!(
        reading_a_strangers_text(&["gh issue list --state open --json number,title"]),
        vec!["gh issue list --state open --json number,title"]
    );
}

#[test]
fn a_listing_without_the_owner_filter_that_reads_a_body_is_caught() {
    assert_eq!(
        reading_a_strangers_text(&["gh issue list --json number,body,author"]),
        vec!["gh issue list --json number,body,author"]
    );
}

fn issue_listings() -> Vec<&'static str> {
    ORCHESTRATE
        .lines()
        .filter(|line| line.contains("gh issue list"))
        .collect()
}

fn listings_reading_a_strangers_text() -> Vec<&'static str> {
    reading_a_strangers_text(&issue_listings())
}

fn reading_a_strangers_text<'a>(listings: &[&'a str]) -> Vec<&'a str> {
    listings
        .iter()
        .copied()
        .filter(|line| !line.contains(OWNER))
        .filter(|line| {
            line.split(['=', ' ', ','])
                .any(|word| word == "title" || word == "body")
        })
        .collect()
}
