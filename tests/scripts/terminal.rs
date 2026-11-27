use crate::helpers::on_a_terminal;
use crate::pty_slots;

#[test]
fn a_command_on_a_terminal_holds_a_pty_slot() {
    on_a_terminal(portable_pty::CommandBuilder::new("true"));

    assert!(pty_slots::holding());
}
