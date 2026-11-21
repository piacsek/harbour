use std::panic;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::session::Session;
use harbour::tui::{leave_terminal, panic_hook};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

static RESTORED: AtomicUsize = AtomicUsize::new(0);

#[test]
fn leaving_the_terminal_turns_off_mouse_capture_and_bracketed_paste() {
    let mut written = Vec::new();

    leave_terminal(&mut written).unwrap();

    let written = String::from_utf8(written).unwrap();
    assert!(written.contains("\x1b[?1000l"), "{written:?}");
    assert!(written.contains("\x1b[?2004l"), "{written:?}");
}

#[test]
fn leaving_the_terminal_gives_back_the_default_cursor_shape() {
    let mut written = Vec::new();

    leave_terminal(&mut written).unwrap();

    let written = String::from_utf8(written).unwrap();
    assert!(written.contains("\x1b[0 q"), "{written:?}");
}

#[test]
fn the_panic_hook_is_silent_for_a_contained_panic_and_restores_for_another() {
    panic::set_hook(panic_hook(Box::new(|_| {
        RESTORED.fetch_add(1, Ordering::SeqCst);
    })));
    let mut session = Session::start(&wide_at_the_edge(), &std::env::temp_dir(), 10, 160).unwrap();
    wait_for(&session, "ready");
    session.resize(10, 159);
    session.send(b"\r");
    sleep(Duration::from_millis(300));
    session.send(b"\r");
    wait_for(&session, "redrew 159");
    let after_the_contained_panic = RESTORED.load(Ordering::SeqCst);

    let _ = panic::catch_unwind(|| panic!("acme agent fault"));

    let _ = panic::take_hook();
    assert_eq!(after_the_contained_panic, 0);
    assert_eq!(RESTORED.load(Ordering::SeqCst), 1);
}

fn wide_at_the_edge() -> [String; 3] {
    let script = "cols=$(stty size | cut -d' ' -f2); \
                  printf '\\033[1;%dH\\343\\201\\202' $((cols-1)); \
                  printf '\\033[3;1Hready'; \
                  while read -r line; do \
                  cols=$(stty size | cut -d' ' -f2); \
                  printf '\\033[1;1H%*s' \"$cols\" ''; \
                  printf '\\033[4;1Hredrew %s' \"$cols\"; \
                  done";
    ["/bin/sh".to_string(), "-c".into(), script.into()]
}

fn wait_for(session: &Session, text: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let screen = rendered(session);
        if screen.contains(text) {
            return screen;
        }
        assert!(
            Instant::now() < deadline,
            "waited 5 s for {text:?}, last screen:\n{screen}"
        );
        sleep(Duration::from_millis(20));
    }
}

fn rendered(session: &Session) -> String {
    let area = Rect::new(0, 0, 160, 10);
    let mut buffer = Buffer::empty(area);
    session.render(area, &mut buffer);
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
