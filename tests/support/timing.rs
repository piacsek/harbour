use std::time::Duration;

pub fn deadline() -> Duration {
    let slowdown = std::env::var("HARBOUR_TEST_SLOWDOWN")
        .ok()
        .and_then(|factor| factor.parse::<u32>().ok())
        .filter(|factor| *factor > 0)
        .unwrap_or(1);
    Duration::from_secs(2) * slowdown
}
