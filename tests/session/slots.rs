use crate::{pty_slots, start};

#[test]
fn a_started_session_holds_a_pty_slot() {
    let _session = start("sleep 5");

    assert!(pty_slots::holding());
}
