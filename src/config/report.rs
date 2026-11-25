use std::time::Duration;

use ratatui::style::Color;

use super::{Config, HOUR, PREFIX_VARIABLE, Source, TIDY_VARIABLE};
use crate::keymap;
use crate::tui::{DIR_PICKER, SESSION_COMMAND};

impl Config {
    pub fn report(&self) -> String {
        let file = match &self.path {
            Some(path) if path.exists() => format!("file: {}", escaped(&path.to_string_lossy())),
            Some(path) => format!("file: {} (none there)", escaped(&path.to_string_lossy())),
            None => "file: none (HOME is not set)".to_string(),
        };
        let editor: Vec<String> = self
            .editor
            .value
            .iter()
            .map(|word| word.to_string_lossy().into_owned())
            .collect();
        let width = match self.sidebar_width.value {
            Some(width) => width.to_string(),
            None => "a third".to_string(),
        };
        [
            file,
            line("editor", &listed(&editor), &self.editor.source),
            line(
                "editor_dir",
                &quoted(self.editor_dir.value.name()),
                &self.editor_dir.source,
            ),
            self.line_of(
                DIR_PICKER,
                "picker",
                &listed(&self.picker.value),
                &self.picker.source,
            ),
            self.line_of(
                SESSION_COMMAND,
                "session_command",
                &listed(&self.session_command.value),
                &self.session_command.source,
            ),
            line("sidebar_width", &width, &self.sidebar_width.source),
            line(
                "accent",
                &quoted(&named(self.accent.value)),
                &self.accent.source,
            ),
            line(
                "nerd_font",
                &self.nerd_font.value.to_string(),
                &self.nerd_font.source,
            ),
            line(
                "claude_icon",
                &quoted(self.claude_icon.value.name()),
                &self.claude_icon.source,
            ),
        ]
        .into_iter()
        .chain([self.line_of(
            PREFIX_VARIABLE,
            "prefix",
            &quoted(&keymap::prefix_name(self.prefix.value)),
            &self.prefix.source,
        )])
        .chain([self.line_of(
            TIDY_VARIABLE,
            "tidy_after",
            &quoted(&tidy_name(self.tidy_after.value)),
            &self.tidy_after.source,
        )])
        .chain(self.colors.iter().map(|(fact, setting)| {
            line(
                &format!("colors.{fact}"),
                &quoted(&named(setting.value)),
                &setting.source,
            )
        }))
        .chain(self.bindings.iter().map(|(action, setting)| {
            let key = setting.value.map_or("none".to_string(), String::from);
            line(
                &format!("keys.{}", keymap::name(*action)),
                &quoted(&key),
                &setting.source,
            )
        }))
        .map(|line| line + "\n")
        .collect()
    }

    fn line_of(&self, variable: &str, key: &str, value: &str, source: &Source) -> String {
        let shown = line(key, value, source);
        if !self.ignored.contains(&variable) {
            return shown;
        }
        match shown.strip_suffix(')') {
            Some(start) => format!("{start}; variable {variable} ignored: not UTF-8)"),
            None => shown,
        }
    }
}

fn line(key: &str, value: &str, source: &Source) -> String {
    let source = match source {
        Source::Variable(name) => format!("variable {name}"),
        Source::File => "file".to_string(),
        Source::Default => "default".to_string(),
        Source::Yielded(taker) => format!("yields to {taker}"),
    };
    format!("{key} = {value}  ({source})")
}

fn listed(words: &[String]) -> String {
    let quoted: Vec<String> = words.iter().map(|word| quoted(word)).collect();
    format!("[{}]", quoted.join(", "))
}

fn quoted(word: &str) -> String {
    format!("\"{}\"", escaped(word))
}

fn escaped(word: &str) -> String {
    let mut text = String::new();
    for character in word.chars() {
        match character {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            '\n' => text.push_str("\\n"),
            '\t' => text.push_str("\\t"),
            '\r' => text.push_str("\\r"),
            character if character.is_control() => {
                text.push_str(&format!("\\u{:04X}", u32::from(character)));
            }
            character => text.push(character),
        }
    }
    text
}

fn named(color: Color) -> String {
    match color {
        Color::Rgb(red, green, blue) => format!("#{red:02x}{green:02x}{blue:02x}"),
        color => color.to_string().to_lowercase(),
    }
}

fn tidy_name(after: Option<Duration>) -> String {
    after.map_or("off".to_string(), |after| {
        format!("{}h", after.as_secs() / HOUR)
    })
}
