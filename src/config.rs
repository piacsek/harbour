use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::io::Read;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use ratatui::style::Color;
use serde::Deserialize;
use toml::{Spanned, Value};

use crate::editor::EditorDir;
use crate::facts::Colors;
use crate::icons::ClaudeIcon;
use crate::keymap::{self, ACTIONS, Action, Keymap};
use crate::tui::{DIR_PICKER, EDITOR, SESSION_COMMAND};

mod report;

const LONGEST: u64 = 64 * 1024;
const PROGRAM: &str = "expected a list of strings: the program, then its arguments";
const WIDTH: &str = "expected a whole number of columns above 0";
const ACCENT: &str = "expected one of the 16 colour names or #rrggbb";
const BOOLEAN: &str = "expected true or false";
const CLAUDE_ICON: &str = "expected \"sparkle\" or \"clawd\"";
const EDITOR_DIR: &str = "expected \"root\" or \"exact\"";
const TIDY: &str = "expected a whole number of hours above 0, as \"6h\", or \"off\"";
const KEYS: &str = "expected a table of action = \"key\"";
const COLORS: &str = "expected a table of fact = \"colour\"";
const FACTS: [(&str, Color); 3] = [
    ("repository", Color::Blue),
    ("branch", Color::Magenta),
    ("pull_request", Color::Cyan),
];
const PREFIX_VARIABLE: &str = "HARBOUR_PREFIX";
const TIDY_VARIABLE: &str = "HARBOUR_TIDY_AFTER";
pub(crate) const HOUR: u64 = 60 * 60;
const TIDY_AFTER: Duration = Duration::from_secs(6 * HOUR);

type Binding = (Action, Setting<Option<char>>);

#[derive(Debug, PartialEq)]
pub enum Source {
    Variable(&'static str),
    File,
    Default,
    Yielded(&'static str),
}

pub struct Setting<T> {
    pub value: T,
    pub source: Source,
}

#[derive(Debug)]
pub struct Refusal(String);

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub struct Config {
    path: Option<PathBuf>,
    editor: Setting<Vec<OsString>>,
    editor_dir: Setting<EditorDir>,
    picker: Setting<Vec<String>>,
    session_command: Setting<Vec<String>>,
    sidebar_width: Setting<Option<u16>>,
    accent: Setting<Color>,
    nerd_font: Setting<bool>,
    claude_icon: Setting<ClaudeIcon>,
    prefix: Setting<char>,
    tidy_after: Setting<Option<Duration>>,
    colors: [(&'static str, Setting<Color>); 3],
    bindings: Vec<Binding>,
    ignored: Vec<&'static str>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Written {
    editor: Option<Spanned<Value>>,
    editor_dir: Option<Spanned<Value>>,
    picker: Option<Spanned<Value>>,
    session_command: Option<Spanned<Value>>,
    sidebar_width: Option<Spanned<Value>>,
    accent: Option<Spanned<Value>>,
    nerd_font: Option<Spanned<Value>>,
    claude_icon: Option<Spanned<Value>>,
    prefix: Option<Spanned<Value>>,
    tidy_after: Option<Spanned<Value>>,
    colors: Option<Spanned<Value>>,
    keys: Option<Spanned<Value>>,
}

#[derive(Deserialize, Default)]
struct Bindings {
    #[serde(default)]
    keys: BTreeMap<Spanned<String>, Spanned<Value>>,
}

#[derive(Deserialize, Default)]
struct Palette {
    #[serde(default)]
    colors: BTreeMap<Spanned<String>, Spanned<Value>>,
}

struct Reader<'a> {
    path: &'a Path,
    text: &'a str,
    home: Option<&'a Path>,
}

impl Config {
    pub fn read(variable: impl Fn(&str) -> Option<OsString>) -> Result<Config, Refusal> {
        let home = absolute(variable("HOME"));
        let path = absolute(variable("XDG_CONFIG_HOME"))
            .or_else(|| home.as_ref().map(|home| home.join(".config")))
            .map(|folder| folder.join("harbour/config.toml"));
        let text = match &path {
            Some(path) => written(path)?,
            None => String::new(),
        };
        let file = path.as_deref().unwrap_or(Path::new(""));
        let reader = Reader {
            path: file,
            text: &text,
            home: home.as_deref(),
        };
        let written: Written = toml::from_str(&text).map_err(|error| reader.syntax(&error))?;
        let set = |name: &'static str| {
            variable(name)
                .filter(|value| !value.is_empty())
                .map(|value| Setting {
                    value,
                    source: Source::Variable(name),
                })
        };
        let set_text = |name| {
            set(name).and_then(|setting| {
                Some(Setting {
                    value: vec![setting.value.into_string().ok()?],
                    source: setting.source,
                })
            })
        };
        let ignored = [DIR_PICKER, SESSION_COMMAND, PREFIX_VARIABLE, TIDY_VARIABLE]
            .into_iter()
            .filter(|name| set(name).is_some_and(|setting| setting.value.to_str().is_none()))
            .collect();
        let editor = reader.program("editor", written.editor)?;
        let picker = reader.program("picker", written.picker)?;
        let session_command = reader.program("session_command", written.session_command)?;
        let written_prefix = reader.prefix(written.prefix)?;
        let prefix = match set_text(PREFIX_VARIABLE) {
            Some(setting) => Setting {
                value: keymap::parse_prefix(&setting.value[0])
                    .map_err(|what| Refusal(format!("{PREFIX_VARIABLE}: {what}")))?,
                source: setting.source,
            },
            None => written_prefix.map_or_else(|| default('/'), from_file),
        };
        let written_tidy = reader.tidy(written.tidy_after)?;
        let tidy_after = match set_text(TIDY_VARIABLE) {
            Some(setting) => Setting {
                value: parse_tidy(&setting.value[0])
                    .map_err(|what| Refusal(format!("{TIDY_VARIABLE}: {what}")))?,
                source: setting.source,
            },
            None => written_tidy.map_or_else(|| default(Some(TIDY_AFTER)), from_file),
        };
        let colors = reader.colors(written.colors)?;
        let bindings = reader.bindings(written.keys)?;
        Ok(Config {
            editor: match (set(EDITOR), editor) {
                (Some(setting), _) => Setting {
                    value: vec![setting.value],
                    source: setting.source,
                },
                (None, Some(command)) => {
                    from_file(command.into_iter().map(OsString::from).collect())
                }
                (None, None) => default(vec![OsString::from("nvim")]),
            },
            editor_dir: reader
                .editor_dir(written.editor_dir)?
                .map_or_else(|| default(EditorDir::Root), from_file),
            picker: set_text(DIR_PICKER).unwrap_or_else(|| {
                picker.map_or_else(|| default(vec!["fzf".to_string()]), from_file)
            }),
            session_command: set_text(SESSION_COMMAND).unwrap_or_else(|| {
                session_command.map_or_else(|| default(vec!["claude".to_string()]), from_file)
            }),
            sidebar_width: match reader.width(written.sidebar_width)? {
                Some(width) => from_file(Some(width)),
                None => default(None),
            },
            accent: reader
                .accent(written.accent)?
                .map_or_else(|| default(Color::Cyan), from_file),
            nerd_font: reader
                .boolean("nerd_font", written.nerd_font)?
                .map_or_else(|| default(false), from_file),
            claude_icon: reader
                .claude_icon(written.claude_icon)?
                .map_or_else(|| default(ClaudeIcon::Sparkle), from_file),
            prefix,
            tidy_after,
            colors,
            bindings,
            path,
            ignored,
        })
    }

    pub fn from_env() -> Result<Config, Refusal> {
        Config::read(|name| std::env::var_os(name))
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn editor(&self) -> &Setting<Vec<OsString>> {
        &self.editor
    }

    pub fn editor_dir(&self) -> &Setting<EditorDir> {
        &self.editor_dir
    }

    pub fn picker(&self) -> &Setting<Vec<String>> {
        &self.picker
    }

    pub fn session_command(&self) -> &Setting<Vec<String>> {
        &self.session_command
    }

    pub fn names_sessions(&self) -> bool {
        let command = &self.session_command;
        !matches!(command.source, Source::Variable(_))
            && Path::new(&command.value[0]).file_name() == Some("claude".as_ref())
            && !command.value[1..]
                .iter()
                .any(|word| chooses_a_conversation(word))
    }

    pub fn sidebar_width(&self) -> &Setting<Option<u16>> {
        &self.sidebar_width
    }

    pub fn accent(&self) -> &Setting<Color> {
        &self.accent
    }

    pub fn colors(&self) -> Colors {
        Colors {
            repository: self.colors[0].1.value,
            branch: self.colors[1].1.value,
            pull_request: self.colors[2].1.value,
        }
    }

    pub fn nerd_font(&self) -> &Setting<bool> {
        &self.nerd_font
    }

    pub fn keymap(&self) -> Keymap {
        self.bindings.iter().fold(
            Keymap::default().with_prefix(self.prefix.value),
            |keymap, (action, setting)| keymap.bind(*action, setting.value),
        )
    }

    pub fn tidy_after(&self) -> &Setting<Option<Duration>> {
        &self.tidy_after
    }

    pub fn claude_icon(&self) -> &Setting<ClaudeIcon> {
        &self.claude_icon
    }
}

impl Reader<'_> {
    fn syntax(&self, error: &toml::de::Error) -> Refusal {
        let span = error.span().unwrap_or(0..0);
        let message = error.message();
        if message.starts_with("unknown field") {
            return self.refusal(&span, &self.text[span.clone()], "unknown key");
        }
        let line = self.line_text(span.start);
        match line.split_once('=') {
            Some((key, _)) if !key.trim().is_empty() && !key.contains('[') => {
                self.refusal(&span, key.trim(), message)
            }
            _ => Refusal(format!(
                "{}:{}: {message}",
                self.path.display(),
                self.line(span.start)
            )),
        }
    }

    fn program(
        &self,
        key: &str,
        value: Option<Spanned<Value>>,
    ) -> Result<Option<Vec<String>>, Refusal> {
        let Some(value) = value else {
            return Ok(None);
        };
        let span = value.span();
        let words: Option<Vec<String>> = match value.into_inner() {
            Value::Array(items) => items
                .into_iter()
                .map(|item| match item {
                    Value::String(word) => Some(word),
                    _ => None,
                })
                .collect(),
            _ => None,
        };
        let mut words = match words {
            Some(words) if words.first().is_some_and(|program| !program.is_empty()) => words,
            _ => return Err(self.refusal(&span, key, PROGRAM)),
        };
        if self.home.is_none() && (words[0] == "~" || words[0].starts_with("~/")) {
            let what = format!("{:?} starts with ~ but HOME is not set", words[0]);
            return Err(self.refusal(&span, key, &what));
        }
        words[0] = self.expanded(&words[0]);
        if words[0].contains('/') && !Path::new(&words[0]).is_absolute() {
            let what = format!(
                "{:?} is a relative path; give an absolute path, ~/... or a bare name",
                words[0]
            );
            return Err(self.refusal(&span, key, &what));
        }
        Ok(Some(words))
    }

    fn expanded(&self, program: &str) -> String {
        match (self.home, program.strip_prefix('~')) {
            (Some(home), Some("")) => home.display().to_string(),
            (Some(home), Some(rest)) if rest.starts_with('/') => {
                format!("{}{rest}", home.display())
            }
            _ => program.to_string(),
        }
    }

    fn width(&self, value: Option<Spanned<Value>>) -> Result<Option<u16>, Refusal> {
        let Some(value) = value else {
            return Ok(None);
        };
        match value.get_ref() {
            Value::Integer(width) if *width > 0 => {
                Ok(Some(u16::try_from(*width).unwrap_or(u16::MAX)))
            }
            _ => Err(self.refusal(&value.span(), "sidebar_width", WIDTH)),
        }
    }

    fn accent(&self, value: Option<Spanned<Value>>) -> Result<Option<Color>, Refusal> {
        let Some(value) = value else {
            return Ok(None);
        };
        let color = match value.get_ref() {
            Value::String(name) => colour(name),
            _ => None,
        };
        match color {
            Some(color) => Ok(Some(color)),
            None => Err(self.refusal(&value.span(), "accent", ACCENT)),
        }
    }

    fn boolean(&self, key: &str, value: Option<Spanned<Value>>) -> Result<Option<bool>, Refusal> {
        let Some(value) = value else {
            return Ok(None);
        };
        match value.get_ref() {
            Value::Boolean(on) => Ok(Some(*on)),
            _ => Err(self.refusal(&value.span(), key, BOOLEAN)),
        }
    }

    fn claude_icon(&self, value: Option<Spanned<Value>>) -> Result<Option<ClaudeIcon>, Refusal> {
        let Some(value) = value else {
            return Ok(None);
        };
        match value.get_ref() {
            Value::String(name) if name == "sparkle" => Ok(Some(ClaudeIcon::Sparkle)),
            Value::String(name) if name == "clawd" => Ok(Some(ClaudeIcon::Clawd)),
            _ => Err(self.refusal(&value.span(), "claude_icon", CLAUDE_ICON)),
        }
    }

    fn editor_dir(&self, value: Option<Spanned<Value>>) -> Result<Option<EditorDir>, Refusal> {
        let Some(value) = value else {
            return Ok(None);
        };
        match value.get_ref() {
            Value::String(name) if name == "root" => Ok(Some(EditorDir::Root)),
            Value::String(name) if name == "exact" => Ok(Some(EditorDir::Exact)),
            _ => Err(self.refusal(&value.span(), "editor_dir", EDITOR_DIR)),
        }
    }

    fn prefix(&self, value: Option<Spanned<Value>>) -> Result<Option<char>, Refusal> {
        let Some(value) = value else {
            return Ok(None);
        };
        let parsed = match value.get_ref() {
            Value::String(name) => keymap::parse_prefix(name),
            _ => Err(keymap::PREFIX.to_string()),
        };
        parsed
            .map(Some)
            .map_err(|what| self.refusal(&value.span(), "prefix", &what))
    }

    fn tidy(&self, value: Option<Spanned<Value>>) -> Result<Option<Option<Duration>>, Refusal> {
        let Some(value) = value else {
            return Ok(None);
        };
        let parsed = match value.get_ref() {
            Value::String(text) => parse_tidy(text),
            _ => Err(TIDY.to_string()),
        };
        parsed
            .map(Some)
            .map_err(|what| self.refusal(&value.span(), "tidy_after", &what))
    }

    fn colors(
        &self,
        colors: Option<Spanned<Value>>,
    ) -> Result<[(&'static str, Setting<Color>); 3], Refusal> {
        let mut settings = FACTS.map(|(fact, color)| (fact, default(color)));
        let Some(colors) = colors else {
            return Ok(settings);
        };
        if !colors.get_ref().is_table() {
            return Err(self.refusal(&colors.span(), "colors", COLORS));
        }
        let read: Palette = toml::from_str(self.text).map_err(|error| self.syntax(&error))?;
        let mut entries: Vec<_> = read.colors.into_iter().collect();
        entries.sort_by_key(|(name, _)| name.span().start);
        for (name, value) in entries {
            let key = format!("colors.{}", name.get_ref());
            let Some((_, setting)) = settings.iter_mut().find(|(fact, _)| fact == name.get_ref())
            else {
                return Err(self.refusal(&name.span(), &key, "unknown key"));
            };
            let color = match value.get_ref() {
                Value::String(text) => colour(text),
                _ => None,
            };
            *setting = from_file(color.ok_or_else(|| self.refusal(&name.span(), &key, ACCENT))?);
        }
        Ok(settings)
    }

    fn bindings(&self, keys: Option<Spanned<Value>>) -> Result<Vec<Binding>, Refusal> {
        let mut bindings: Vec<Binding> = ACTIONS
            .iter()
            .map(|(action, _, key)| (*action, default(Some(*key))))
            .collect();
        let Some(keys) = keys else {
            return Ok(bindings);
        };
        if !keys.get_ref().is_table() {
            return Err(self.refusal(&keys.span(), "keys", KEYS));
        }
        let read: Bindings = toml::from_str(self.text).map_err(|error| self.syntax(&error))?;
        let mut written: Vec<(Range<usize>, String, Action, Option<char>)> = Vec::new();
        let mut entries: Vec<_> = read.keys.into_iter().collect();
        entries.sort_by_key(|(name, _)| name.span().start);
        for (name, value) in entries {
            let span = name.span();
            let key = format!("keys.{}", name.get_ref());
            let action = keymap::action_named(name.get_ref())
                .ok_or_else(|| self.refusal(&span, &key, &no_such_action()))?;
            let bound = match value.get_ref() {
                Value::String(text) => keymap::parse_key(text),
                _ => Err(keymap::KEY.to_string()),
            }
            .map_err(|what| self.refusal(&span, &key, &what))?;
            if let Some(c) = bound.filter(|c| {
                matches!(action, Action::New | Action::End | Action::Close)
                    && matches!(c, 'j' | 'J' | 'k' | 'K')
            }) {
                let what = format!("{:?} moves the selection in the sidebar", c.to_string());
                return Err(self.refusal(&span, &key, &what));
            }
            written.push((span, key, action, bound));
        }
        let mut keymap = written
            .iter()
            .fold(Keymap::default(), |keymap, (_, _, action, bound)| {
                keymap.bind(*action, *bound)
            });
        for yielding in [Action::Close, Action::Rename, Action::SendPrefix] {
            let unwritten = written.iter().all(|(_, _, action, _)| *action != yielding);
            let taker = match yielding {
                Action::SendPrefix => keymap.clash_in_any_case(yielding),
                _ => keymap.clash(yielding),
            };
            let Some(taker) = taker.filter(|_| unwritten) else {
                continue;
            };
            keymap = keymap.bind(yielding, None);
            for (each, setting) in &mut bindings {
                if *each == yielding {
                    *setting = Setting {
                        value: None,
                        source: Source::Yielded(keymap::name(taker)),
                    };
                }
            }
        }
        for (span, key, action, bound) in written.iter().rev() {
            if let (Some(other), Some(c)) = (keymap.clash(*action), bound) {
                let other = keymap::name(other);
                let what = format!(
                    "{:?} is {other}'s key too; bind {other} elsewhere",
                    c.to_string()
                );
                return Err(self.refusal(span, key, &what));
            }
        }
        for (_, _, action, bound) in written {
            for (each, setting) in &mut bindings {
                if *each == action {
                    *setting = from_file(bound);
                }
            }
        }
        Ok(bindings)
    }

    fn refusal(&self, span: &Range<usize>, key: &str, what: &str) -> Refusal {
        Refusal(format!(
            "{}:{}: {key}: {what}",
            self.path.display(),
            self.line(span.start)
        ))
    }

    fn line(&self, at: usize) -> usize {
        self.text[..at.min(self.text.len())].matches('\n').count() + 1
    }

    fn line_text(&self, at: usize) -> &str {
        let at = at.min(self.text.len());
        let start = self.text[..at].rfind('\n').map_or(0, |newline| newline + 1);
        let end = self.text[at..]
            .find('\n')
            .map_or(self.text.len(), |newline| at + newline);
        &self.text[start..end]
    }
}

fn colour(name: &str) -> Option<Color> {
    let hex = name.len() == 7 && name.starts_with('#');
    match Color::from_str(name).ok()? {
        Color::Reset | Color::Indexed(_) => None,
        Color::Rgb(..) if !hex => None,
        color => Some(color),
    }
}

fn written(path: &Path) -> Result<String, Refusal> {
    let refused = |reason: String| Refusal(format!("{}: {reason}", path.display()));
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            ) =>
        {
            return Ok(String::new());
        }
        Err(error) => return Err(refused(error.to_string())),
    };
    if !metadata.is_file() {
        return Err(refused("not a regular file".to_string()));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| file.take(LONGEST + 1).read_to_end(&mut bytes))
        .map_err(|error| refused(error.to_string()))?;
    if bytes.len() as u64 > LONGEST {
        return Err(refused("larger than 64 KiB".to_string()));
    }
    String::from_utf8(bytes).map_err(|_| refused("stream did not contain valid UTF-8".to_string()))
}

fn parse_tidy(text: &str) -> Result<Option<Duration>, String> {
    if text == "off" {
        return Ok(None);
    }
    let hours = text
        .strip_suffix('h')
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|digits| digits.parse::<u32>().ok())
        .filter(|hours| *hours > 0)
        .ok_or_else(|| TIDY.to_string())?;
    Ok(Some(Duration::from_secs(u64::from(hours) * HOUR)))
}

fn no_such_action() -> String {
    let names: Vec<&str> = ACTIONS.iter().map(|(_, name, _)| *name).collect();
    format!("no such action; the actions are {}", names.join(", "))
}

fn chooses_a_conversation(word: &str) -> bool {
    let flag = word.split_once('=').map_or(word, |(flag, _)| flag);
    matches!(flag, "--continue" | "--resume" | "--session-id") || matches!(word, "-c" | "-r")
}

fn absolute(value: Option<OsString>) -> Option<PathBuf> {
    value.map(PathBuf::from).filter(|path| path.is_absolute())
}

fn from_file<T>(value: T) -> Setting<T> {
    Setting {
        value,
        source: Source::File,
    }
}

fn default<T>(value: T) -> Setting<T> {
    Setting {
        value,
        source: Source::Default,
    }
}
