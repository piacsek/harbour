use std::io;
use std::path::Path;

use tempfile::TempDir;

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

const UNKNOWN: &str = r#"printf 'acme shell in ~/%s\n' "${PWD##*/}"
exec sleep 86400"#;

pub fn working(registry: &Path) -> Vec<String> {
    registered(registry, r#"{"status":"busy"}"#, WORKING)
}

pub fn blocked(registry: &Path) -> Vec<String> {
    registered(
        registry,
        r#"{"status":"waiting","waitingFor":"permission prompt"}"#,
        BLOCKED,
    )
}

pub fn idle(registry: &Path) -> Vec<String> {
    registered(registry, r#"{"status":"idle"}"#, IDLE)
}

pub fn failed() -> Vec<String> {
    script(FAILED)
}

pub fn unknown() -> Vec<String> {
    script(UNKNOWN)
}

fn registered(registry: &Path, record: &str, text: &str) -> Vec<String> {
    let dir = registry.display().to_string().replace('\'', r"'\''");
    script(&format!("printf '%s' '{record}' > '{dir}'/$$.json\n{text}"))
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
