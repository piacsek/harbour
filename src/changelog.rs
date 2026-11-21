use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::text;

pub const CHANGELOG: &str = include_str!("../CHANGELOG.md");

const NO_NOTES: &str = "This build has no release notes.";

const UNRELEASED: &str = "0.0.0";

struct Section<'a> {
    heading: &'a str,
    version: &'a str,
    bullets: Vec<String>,
}

fn sections(text: &str) -> Vec<Section<'_>> {
    let mut sections: Vec<Section> = Vec::new();
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            let heading = heading.strip_prefix('v').unwrap_or(heading);
            let version = heading.split_whitespace().next().unwrap_or_default();
            sections.push(Section {
                heading: line,
                version,
                bullets: Vec::new(),
            });
        } else if let Some(section) = sections.last_mut() {
            if let Some(bullet) = line.strip_prefix("- ")
                && !bullet.trim().is_empty()
            {
                section.bullets.push(bullet.trim().to_string());
            } else if line.starts_with(char::is_whitespace)
                && !line.trim().is_empty()
                && let Some(bullet) = section.bullets.last_mut()
            {
                bullet.push(' ');
                bullet.push_str(line.trim());
            }
        }
    }
    sections
}

fn dated(heading: &str) -> bool {
    if !heading.starts_with("## v") {
        return false;
    }
    let Some(date) = heading
        .split_once(" (")
        .and_then(|(_, rest)| rest.strip_suffix(')'))
    else {
        return false;
    };
    date.len() == 10
        && date.char_indices().all(|(i, c)| match i {
            4 | 7 => c == '-',
            _ => c.is_ascii_digit(),
        })
        && on_the_calendar(&date[..4], &date[5..7], &date[8..])
}

fn on_the_calendar(year: &str, month: &str, day: &str) -> bool {
    let (Ok(year), Ok(month), Ok(day)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

fn numbers(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

pub fn check(text: &str, version: &str) -> Result<(), String> {
    let sections = sections(text);
    let newest = sections.first().map_or("none", |section| section.version);
    if version != UNRELEASED && newest != version {
        return Err(format!(
            "CHANGELOG.md has no entry for v{version} at the top (found v{newest})"
        ));
    }
    if let Some(section) = sections.iter().find(|section| !dated(section.heading)) {
        return Err(format!(
            "`{}` must read `## vX.Y.Z (YYYY-MM-DD)`",
            section.heading
        ));
    }
    for pair in sections.windows(2) {
        if numbers(pair[0].version) <= numbers(pair[1].version) {
            return Err(format!(
                "v{} is listed above v{}; newest first, once each",
                pair[0].version, pair[1].version
            ));
        }
    }
    for (i, section) in sections.iter().enumerate() {
        if section.bullets.is_empty() {
            return Err(format!("v{} has no `- ` bullet", section.version));
        }
        for older in &sections[i + 1..] {
            if let Some(bullet) = section.bullets.iter().find(|b| older.bullets.contains(b)) {
                return Err(format!(
                    "v{} repeats a bullet of v{}: {bullet}",
                    section.version, older.version
                ));
            }
        }
    }
    Ok(())
}

pub fn check_released(text: &str, released: &[&str], current: &str) -> Result<(), String> {
    let sections = sections(text);
    match released.iter().find(|version| {
        (current == UNRELEASED || numbers(version) <= numbers(current))
            && !sections.iter().any(|section| section.version == **version)
    }) {
        Some(version) => Err(format!(
            "v{version} was released but has no entry; restore it, or merge origin/main if another branch released it"
        )),
        None => Ok(()),
    }
}

pub fn shown(changelog: &str, width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut bullet: Option<String> = None;
    let mut released = false;
    for line in changelog.lines() {
        if let Some(more) = bullet.as_mut()
            && line.starts_with(char::is_whitespace)
            && !line.trim().is_empty()
        {
            more.push(' ');
            more.push_str(line.trim());
            continue;
        }
        if let Some(done) = bullet.take() {
            lines.extend(hanging(&done, width));
        }
        if let Some(rest) = line.strip_prefix("- ") {
            bullet = Some(rest.trim().to_string());
        } else if let Some(heading) = heading(line) {
            released |= line.starts_with("## ");
            lines.extend(
                text::wrap(&unquoted(heading), width)
                    .into_iter()
                    .map(|part| Line::from(part).bold()),
            );
        } else if line.trim().is_empty() {
            lines.push(Line::default());
        } else {
            lines.extend(
                text::wrap(&unquoted(line), width)
                    .into_iter()
                    .map(Line::from),
            );
        }
    }
    if let Some(done) = bullet {
        lines.extend(hanging(&done, width));
    }
    if !released {
        lines.push(Line::default());
        lines.push(Line::from(NO_NOTES).dim());
    }
    lines
}

fn heading(line: &str) -> Option<&str> {
    let rest = line.trim_start_matches('#');
    (rest.len() < line.len())
        .then(|| rest.strip_prefix(' '))
        .flatten()
}

fn unquoted(line: &str) -> String {
    let mut shown = String::new();
    let mut rest = line;
    while let Some((before, after)) = rest.split_once('`') {
        let Some((code, next)) = after.split_once('`') else {
            break;
        };
        shown.push_str(before);
        shown.push_str(code);
        rest = next;
    }
    shown.push_str(rest);
    shown
}

fn hanging(bullet: &str, width: usize) -> Vec<Line<'static>> {
    text::wrap(&unquoted(bullet), width.saturating_sub(2))
        .into_iter()
        .enumerate()
        .map(|(index, part)| {
            let lead = if index == 0 { "- " } else { "  " };
            Line::from(format!("{lead}{part}"))
        })
        .collect()
}
