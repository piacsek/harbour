use std::ffi::OsString;
use std::fmt;
use std::io::Read;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use ratatui::style::Color;
use serde::Deserialize;
use toml::{Spanned, Value};

use crate::icons::ClaudeIcon;
use crate::tui::{DIR_PICKER, EDITOR, SESSION_COMMAND};

const LONGEST: u64 = 64 * 1024;
const PROGRAM: &str = "expected a list of strings: the program, then its arguments";
const WIDTH: &str = "expected a whole number of columns above 0";
const ACCENT: &str = "expected one of the 16 colour names or #rrggbb";
const BOOLEAN: &str = "expected true or false";
const CLAUDE_ICON: &str = "expected \"sparkle\" or \"clawd\"";

#[derive(Debug, PartialEq)]
pub enum Source {
    Variable(&'static str),
    File,
    Default,
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
    picker: Setting<Vec<String>>,
    session_command: Setting<Vec<String>>,
    sidebar_width: Setting<Option<u16>>,
    accent: Setting<Color>,
    nerd_font: Setting<bool>,
    claude_icon: Setting<ClaudeIcon>,
    ignored: Vec<&'static str>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Written {
    editor: Option<Spanned<Value>>,
    picker: Option<Spanned<Value>>,
    session_command: Option<Spanned<Value>>,
    sidebar_width: Option<Spanned<Value>>,
    accent: Option<Spanned<Value>>,
    nerd_font: Option<Spanned<Value>>,
    claude_icon: Option<Spanned<Value>>,
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
        let ignored = [DIR_PICKER, SESSION_COMMAND]
            .into_iter()
            .filter(|name| set(name).is_some_and(|setting| setting.value.to_str().is_none()))
            .collect();
        let editor = reader.program("editor", written.editor)?;
        let picker = reader.program("picker", written.picker)?;
        let session_command = reader.program("session_command", written.session_command)?;
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
            path,
            ignored,
        })
    }

    pub fn from_env() -> Result<Config, Refusal> {
        Config::read(|name| std::env::var_os(name))
    }

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
        .map(|line| line + "\n")
        .concat()
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

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn editor(&self) -> &Setting<Vec<OsString>> {
        &self.editor
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

    pub fn nerd_font(&self) -> &Setting<bool> {
        &self.nerd_font
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

fn line(key: &str, value: &str, source: &Source) -> String {
    let source = match source {
        Source::Variable(name) => format!("variable {name}"),
        Source::File => "file".to_string(),
        Source::Default => "default".to_string(),
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
