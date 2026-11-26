pub const NEOVIM: &str = "\u{E6AE}";
pub const REPOSITORY: &str = "\u{F401}";
pub const BRANCH: &str = "\u{F418}";
pub const PULL_REQUEST: &str = "\u{F407}";
pub const SUBAGENTS: &str = "\u{F160E}";
pub const TASKS_OPEN: &str = "\u{F08A8}";
pub const TASKS_DONE: &str = "\u{F014E}";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClaudeIcon {
    Sparkle,
    Clawd,
}

impl ClaudeIcon {
    pub fn name(self) -> &'static str {
        match self {
            ClaudeIcon::Sparkle => "sparkle",
            ClaudeIcon::Clawd => "clawd",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            ClaudeIcon::Sparkle => "\u{F0674}",
            ClaudeIcon::Clawd => "\u{100000} ",
        }
    }
}
