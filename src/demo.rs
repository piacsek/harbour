use crate::picker::Picker;

const WORKING: &str = r#"printf '\033[38;5;209m*\033[0m \033[1macme agent\033[0m in ~/%s\n\n' "${PWD##*/}"
printf '\033[2m>\033[0m fix the flaky retry test in acme/api\n\n'
printf '\033[38;5;114m*\033[0m Read src/retry.rs\n'
printf '\033[38;5;114m*\033[0m Bash cargo test retry\n'
printf '  \033[2m12 passed, 0 failed\033[0m\n\n'
printf 'The retry now waits on the clock it is given, so the test no longer races.\n'
exec sleep 86400"#;

const FAILED: &str = r#"printf 'acme agent: could not reach acme/web\n'
exit 1"#;

const PLACES: &str = r#"printf '%s\n' /home/acme/api /home/acme/web /home/acme/infra /home/acme/docs /home/acme/api/crates/retry"#;

pub fn picker() -> Picker {
    Picker::Fuzzy {
        fzf: vec!["fzf".to_string()],
        list: script(PLACES),
    }
}

pub fn working() -> Vec<String> {
    script(WORKING)
}

pub fn failed() -> Vec<String> {
    script(FAILED)
}

fn script(text: &str) -> Vec<String> {
    vec!["/bin/sh".to_string(), "-c".to_string(), text.to_string()]
}
