use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use tempfile::TempDir;

use crate::picker::Picker;

const WORKING: &str = r#"printf '\033[38;5;209m*\033[0m \033[1macme agent\033[0m in ~/%s\n\n' "${PWD##*/}"
printf '\033[2m>\033[0m fix the flaky retry test in acme/api\n\n'
printf '\033[38;5;114m*\033[0m Read src/retry.rs\n'
printf '\033[38;5;114m*\033[0m Bash cargo test retry\n'
printf '  \033[2m12 passed, 0 failed\033[0m\n\n'
printf 'The retry now waits on the clock it is given, so the test no longer races.\n'
exec sleep 86400"#;

const BLOCKED: &str = r#"printf '\033[2m>\033[0m publish the new acme-api crate\n\n'
printf '\033[38;5;209m*\033[0m Bash cargo publish -p acme-api\n\n'
printf 'Do you want to proceed?\n\033[1m> 1. Yes\033[0m\n  2. No\n'
exec sleep 86400"#;

const IDLE: &str = r#"printf '\033[2m>\033[0m bump acme/web to the new api client\n\n'
printf 'Done: acme/web builds against acme-api 0.4.1 and its tests pass.\n'
exec sleep 86400"#;

const FAILED: &str = r#"printf 'acme agent: could not reach acme/web\n'
exit 1"#;

const PLACES: &str = r#"printf '%s\n' /home/acme/api /home/acme/web /home/acme/infra /home/acme/docs /home/acme/api/crates/retry"#;

pub fn picker() -> Picker {
    Picker::Fuzzy {
        fzf: vec!["fzf".to_string()],
        list: script(PLACES),
        here_first: false,
    }
}

const UNKNOWN: &str = r#"printf 'acme shell in ~/%s\n' "${PWD##*/}"
exec sleep 86400"#;

pub fn fresh() -> Vec<String> {
    script(WORKING)
}

pub fn working(registry: &Path) -> Vec<String> {
    carded(
        registry,
        Card {
            id: "6f0c2a9e-31d4-4b7a-9e58-0d2c7f41a3b6",
            record: r#""status":"busy""#,
            minutes: 3,
            transcript: &[
                r#"{"type":"ai-title","aiTitle":"Fix the flaky retry test"}"#,
                r#"{"type":"user","cwd":"/home/acme/acme-api","gitBranch":"fix-retry","message":{}}"#,
                r#"{"type":"pr-link","prNumber":128}"#,
                r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"cargo test retry","description":"Run the retry tests"}}]}}"#,
            ],
            subagents: 2,
        },
        WORKING,
    )
}

pub fn blocked(registry: &Path) -> Vec<String> {
    carded(
        registry,
        Card {
            id: "b83e5d17-c0a2-4f69-8d14-7e9a2b6c05f1",
            record: r#""status":"waiting","waitingFor":"permission prompt""#,
            minutes: 1,
            transcript: &[
                r#"{"type":"ai-title","aiTitle":"Publish the acme-api crate"}"#,
                r#"{"type":"user","cwd":"/home/acme/acme-api","gitBranch":"release-0.4.1","message":{}}"#,
            ],
            subagents: 0,
        },
        BLOCKED,
    )
}

pub fn idle(registry: &Path) -> Vec<String> {
    carded(
        registry,
        Card {
            id: "1d9f4c62-8e07-4a3b-b5c1-f2a6e8d03c97",
            record: r#""status":"idle""#,
            minutes: 12,
            transcript: &[
                r#"{"type":"ai-title","aiTitle":"Bump acme-web to the new client"}"#,
                r#"{"type":"user","cwd":"/home/acme/acme-web","gitBranch":"main","message":{}}"#,
                r#"{"type":"last-prompt","lastPrompt":"bump acme/web to the new api client"}"#,
                r#"{"type":"pr-link","prNumber":131}"#,
            ],
            subagents: 0,
        },
        IDLE,
    )
}

pub fn checkout(registry: &Path, name: &str, branch: &str) -> PathBuf {
    let directory = registry.join(name);
    let git = directory.join(".git");
    let _ = fs::create_dir_all(&git)
        .and_then(|()| fs::write(git.join("HEAD"), format!("ref: refs/heads/{branch}\n")));
    directory
}

pub fn failed() -> Vec<String> {
    script(FAILED)
}

pub fn unknown() -> Vec<String> {
    script(UNKNOWN)
}

struct Card {
    id: &'static str,
    record: &'static str,
    minutes: u64,
    transcript: &'static [&'static str],
    subagents: usize,
}

fn carded(registry: &Path, card: Card, text: &str) -> Vec<String> {
    let dir = registry.display().to_string().replace('\'', r"'\''");
    let Card {
        id,
        record,
        minutes,
        transcript,
        subagents,
    } = card;
    let lines = transcript
        .iter()
        .map(|line| format!("'{line}'"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut setup = format!(
        "mkdir -p '{dir}'/projects/demo/{id}/subagents\nprintf '%s\\n' {lines} > '{dir}'/projects/demo/{id}.jsonl\n"
    );
    for agent in 1..=subagents {
        setup.push_str(&format!(
            "printf '%s' '{{\"agentType\":\"Explore\"}}' > '{dir}'/projects/demo/{id}/subagents/agent-{agent}.meta.json\n"
        ));
    }
    let since = SystemTime::now()
        .checked_sub(Duration::from_secs(minutes * 60))
        .and_then(|since| since.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |since| since.as_millis());
    script(&format!(
        "{setup}printf '%s' '{{{record},\"sessionId\":\"{id}\",\"statusUpdatedAt\":{since}}}' > '{dir}'/$$.json\n{text}"
    ))
}

fn script(text: &str) -> Vec<String> {
    vec!["/bin/sh".to_string(), "-c".to_string(), text.to_string()]
}

pub struct Registry(TempDir);

impl Registry {
    pub fn create_in(parent: &Path) -> io::Result<Registry> {
        tempfile::Builder::new()
            .prefix("harbour-demo-")
            .tempdir_in(parent)
            .map(Registry)
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }
}
