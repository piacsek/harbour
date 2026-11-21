use std::io;
use std::sync::atomic::AtomicUsize;

use harbour::app::App;
use harbour::tui::settled;
use nix::sys::signal::Signal;

use crate::logging::{logged, read};
use crate::{Screen, reading_agent};

#[test]
fn a_step_with_nothing_caught_goes_on() {
    let mut screen = Screen::new(reading_agent());

    let ended = settled(&mut screen.app, Ok(()), &nothing());

    assert_eq!(ended.unwrap(), None);
    assert!(screen.app.running());
}

#[test]
fn a_caught_signal_stops_harbour_and_ends_by_it() {
    let mut screen = Screen::new(reading_agent());

    let ended = settled(&mut screen.app, Ok(()), &caught(Signal::SIGTERM));

    assert_eq!(ended.unwrap(), Some(Signal::SIGTERM as i32));
    assert!(!screen.app.running());
}

#[test]
fn an_end_of_input_stops_harbour_and_ends_by_hup() {
    let mut screen = Screen::new(reading_agent());
    let error = io::Error::from(io::ErrorKind::UnexpectedEof);

    let ended = settled(&mut screen.app, Err(error), &nothing());

    assert_eq!(ended.unwrap(), Some(Signal::SIGHUP as i32));
    assert!(!screen.app.running());
}

#[test]
fn another_error_stops_harbour_then_goes_on() {
    let mut screen = Screen::new(reading_agent());
    let error = io::Error::from(io::ErrorKind::PermissionDenied);

    let ended = settled(&mut screen.app, Err(error), &nothing());

    assert_eq!(ended.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    assert!(!screen.app.running());
}

#[test]
fn another_error_logs_the_stop_before_the_quit() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);
    read(&lines);
    let error = io::Error::from(io::ErrorKind::PermissionDenied);

    let _ = settled(&mut screen.app, Err(error), &nothing());

    let lines = read(&lines);
    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert!(lines[0].ends_with(" error\n"), "{lines:#?}");
    assert!(lines[1].ends_with(" quit sessions=0\n"), "{lines:#?}");
}

#[test]
fn a_signal_caught_after_a_confirmed_quit_ends_harbour_by_it() {
    let mut screen = Screen::new(reading_agent());
    screen.press('q');
    assert!(!screen.app.running());

    let ended = settled(&mut screen.app, Ok(()), &caught(Signal::SIGINT));

    assert_eq!(ended.unwrap(), Some(Signal::SIGINT as i32));
}

#[test]
fn a_signal_caught_after_a_confirmed_quit_logs_nothing_more() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);
    screen.press('q');
    let quit = read(&lines);
    assert!(
        quit.last().unwrap().ends_with(" quit sessions=0\n"),
        "{quit:#?}"
    );

    let _ = settled(&mut screen.app, Ok(()), &caught(Signal::SIGTERM));

    assert_eq!(read(&lines), Vec::<String>::new());
}

fn nothing() -> AtomicUsize {
    AtomicUsize::new(0)
}

fn caught(signal: Signal) -> AtomicUsize {
    AtomicUsize::new(signal as usize)
}
