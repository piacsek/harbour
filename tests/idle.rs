#![cfg(target_os = "linux")]

use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use harbour::handoff::Listener;

#[test]
fn an_idle_listener_sleeps_until_it_has_work() {
    let scratch = tempfile::tempdir().unwrap();
    let mut listener = Listener::new(&scratch.path().join("edit"));
    listener.incoming();
    let thread = listener_thread();
    let woken = switches(&thread);

    sleep(Duration::from_secs(1));

    assert!(switches(&thread) - woken < 10);
}

fn listener_thread() -> PathBuf {
    for _ in 0..200 {
        let named = fs::read_dir("/proc/self/task")
            .unwrap()
            .map(|task| task.unwrap().path())
            .find(|task| {
                fs::read_to_string(task.join("comm"))
                    .is_ok_and(|name| name.trim() == "harbour-handoff")
            });
        if let Some(thread) = named {
            return thread;
        }
        sleep(Duration::from_millis(5));
    }
    panic!("no listener thread");
}

fn switches(thread: &Path) -> u64 {
    fs::read_to_string(thread.join("status"))
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("voluntary_ctxt_switches:"))
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}
