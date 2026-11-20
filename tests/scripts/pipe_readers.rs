use std::fs;
use std::path::Path;

#[test]
fn no_script_or_workflow_pipes_into_a_reader_that_stops_before_its_input_ends() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut findings = Vec::new();
    for dir in ["scripts", ".github/workflows"] {
        let mut paths: Vec<_> = fs::read_dir(root.join(dir))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "sh" || extension == "yml")
            })
            .collect();
        paths.sort();
        for path in paths {
            let name = path.strip_prefix(root).unwrap().display().to_string();
            for finding in early_readers(&fs::read_to_string(&path).unwrap()) {
                findings.push(format!("{name}:{finding}"));
            }
        }
    }

    assert_eq!(findings, Vec::<String>::new());
}

#[test]
fn a_pipe_into_head_is_found() {
    assert_eq!(
        early_readers("x=\"$(printf '%s\\n' \"$a\" | head -n 1)\"\n"),
        ["1: a pipe into head"]
    );
}

#[test]
fn a_pipe_into_a_grep_that_stops_at_its_first_match_is_found() {
    assert_eq!(
        early_readers("printf '%s\\n' \"$a\" | grep -qx b && echo yes\n"),
        ["1: a pipe into grep -qx"]
    );
}

#[test]
fn a_grep_that_reads_to_the_end_is_not_found() {
    assert_eq!(
        early_readers("printf '%s\\n' \"$a\" | grep -vxF b | sed -n 1p\n"),
        Vec::<String>::new()
    );
}

#[test]
fn an_or_is_not_a_pipe() {
    assert_eq!(
        early_readers("grep -q b file || head -n 1 file\n"),
        Vec::<String>::new()
    );
}

#[test]
fn an_awk_exit_in_a_rule_is_found_on_its_line() {
    assert_eq!(
        early_readers(
            "printf '%s\\n' \"$a\" \\\n  | awk -v q=\"'\" '\n    /^b/ { print; exit }\n  '\n"
        ),
        ["3: an awk exit outside BEGIN and END"]
    );
}

#[test]
fn an_awk_exit_in_a_function_is_found() {
    assert_eq!(
        early_readers("awk 'function stop() { exit 1 } { stop() }'\n"),
        ["1: an awk exit outside BEGIN and END"]
    );
}

#[test]
fn an_awk_exit_in_begin_or_end_is_not_found() {
    assert_eq!(
        early_readers("awk 'BEGIN { if (!x) exit 2 } { n++ } END { exit !n }'\n"),
        Vec::<String>::new()
    );
}

#[test]
fn an_exit_inside_an_awk_string_regex_or_comment_is_not_found() {
    assert_eq!(
        early_readers("awk '/exit/ { print \"exit\" } # exit\n'\n"),
        Vec::<String>::new()
    );
}

fn early_readers(text: &str) -> Vec<String> {
    let mut findings = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for program in piped_programs(line) {
            if let Some(reader) = stops_early(&program) {
                findings.push(format!("{}: a pipe into {reader}", index + 1));
            }
        }
    }
    for (offset, program) in awk_programs(text) {
        for at in exits_in_rules(program) {
            let line = text[..offset + at].matches('\n').count() + 1;
            findings.push(format!("{line}: an awk exit outside BEGIN and END"));
        }
    }
    findings
}

fn piped_programs(line: &str) -> Vec<Vec<&str>> {
    let bytes = line.as_bytes();
    (0..bytes.len())
        .filter(|&at| bytes[at] == b'|')
        .filter(|&at| at == 0 || bytes[at - 1] != b'|')
        .filter(|&at| bytes.get(at + 1) != Some(&b'|'))
        .map(|at| {
            line[at + 1..]
                .split('|')
                .next()
                .unwrap()
                .split_whitespace()
                .collect()
        })
        .collect()
}

fn stops_early(program: &[&str]) -> Option<String> {
    match program.first() {
        Some(&"head") | Some(&"read") => Some(program[0].to_string()),
        Some(&"grep") => program[1..]
            .iter()
            .find(|word| {
                matches!(**word, "--quiet" | "--max-count" | "--files-with-matches")
                    || (word.starts_with('-')
                        && !word.starts_with("--")
                        && word.contains(['q', 'm', 'l']))
            })
            .map(|word| format!("grep {word}")),
        _ => None,
    }
}

fn awk_programs(text: &str) -> Vec<(usize, &str)> {
    let mut programs = Vec::new();
    let mut search = 0;
    while let Some(found) = text[search..].find("awk") {
        let start = search + found;
        search = start + 3;
        let before = text[..start].chars().next_back();
        let after = text[search..].chars().next();
        if before.is_some_and(is_word) || after.is_some_and(|c| !c.is_whitespace()) {
            continue;
        }
        if let Some(program) = program_word(text, search) {
            programs.push(program);
        }
    }
    programs
}

fn program_word(text: &str, from: usize) -> Option<(usize, &str)> {
    let mut words = shell_words(text, from).into_iter();
    while let Some((offset, word, quoted)) = words.next() {
        if word == "-v" || word == "-F" || word == "-f" {
            words.next();
        } else if !word.starts_with('-') {
            return quoted.then_some((offset, word));
        }
    }
    None
}

fn shell_words(text: &str, from: usize) -> Vec<(usize, &str, bool)> {
    let bytes = text.as_bytes();
    let mut words = Vec::new();
    let mut at = from;
    loop {
        while at < bytes.len() && (bytes[at] == b' ' || bytes[at] == b'\t') {
            at += 1;
        }
        if bytes[at..].starts_with(b"\\\n") {
            at += 2;
            continue;
        }
        if at >= bytes.len() || b"|;&)<>\n".contains(&bytes[at]) {
            return words;
        }
        let start = at;
        while at < bytes.len() && !b" \t|;&)<>\n".contains(&bytes[at]) {
            match bytes[at] {
                b'\'' => at += 1 + text[at + 1..].find('\'').unwrap() + 1,
                b'"' => at = closing_double_quote(bytes, at + 1) + 1,
                b'\\' => at += 2,
                _ => at += 1,
            }
        }
        let word = &text[start..at];
        let single = word.len() > 1 && word.starts_with('\'') && word.ends_with('\'');
        if single {
            words.push((start + 1, &word[1..word.len() - 1], true));
        } else {
            words.push((start, word, false));
        }
    }
}

fn closing_double_quote(bytes: &[u8], mut at: usize) -> usize {
    while bytes[at] != b'"' {
        at += if bytes[at] == b'\\' { 2 } else { 1 };
    }
    at
}

fn exits_in_rules(program: &str) -> Vec<usize> {
    let code = without_literals(program);
    let bytes = code.as_bytes();
    let mut exits = Vec::new();
    let mut depth = 0;
    let mut pattern = String::new();
    let mut kind = String::new();
    for (at, &byte) in bytes.iter().enumerate() {
        match byte {
            b'{' => {
                if depth == 0 {
                    kind = pattern.trim().to_string();
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    pattern.clear();
                }
            }
            b'\n' if depth == 0 => pattern.clear(),
            _ if depth == 0 => pattern.push(byte as char),
            _ => {}
        }
        let word_starts = at == 0 || !is_word(bytes[at - 1] as char);
        let word_ends = bytes.get(at + 4).is_none_or(|&next| !is_word(next as char));
        if depth > 0
            && word_starts
            && word_ends
            && code[at..].starts_with("exit")
            && kind != "BEGIN"
            && kind != "END"
        {
            exits.push(at);
        }
    }
    exits
}

fn without_literals(program: &str) -> String {
    let bytes = program.as_bytes();
    let mut code = bytes.to_vec();
    let mut at = 0;
    let mut previous = b'\n';
    while at < bytes.len() {
        let byte = bytes[at];
        let end = match byte {
            b'"' => Some(closing_double_quote(bytes, at + 1)),
            b'/' if b"\n({},;!~&|".contains(&previous) => Some(closing_slash(bytes, at + 1)),
            b'#' => Some(
                at + bytes[at..]
                    .iter()
                    .position(|&c| c == b'\n')
                    .unwrap_or(bytes.len() - at)
                    - 1,
            ),
            _ => None,
        };
        if let Some(end) = end {
            for blank in &mut code[at..=end] {
                if *blank != b'\n' {
                    *blank = b' ';
                }
            }
            previous = b'"';
            at = end + 1;
            continue;
        }
        if byte == b'\n' || !byte.is_ascii_whitespace() {
            previous = byte;
        }
        at += 1;
    }
    String::from_utf8(code).unwrap()
}

fn closing_slash(bytes: &[u8], mut at: usize) -> usize {
    let mut bracket = false;
    loop {
        match bytes[at] {
            b'\\' => at += 1,
            b'[' => bracket = true,
            b']' => bracket = false,
            b'/' if !bracket => return at,
            _ => {}
        }
        at += 1;
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
