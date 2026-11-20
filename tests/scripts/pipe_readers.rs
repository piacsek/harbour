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
            .collect();
        paths.sort();
        for path in paths {
            let name = path.strip_prefix(root).unwrap().display().to_string();
            let text = fs::read_to_string(&path).unwrap();
            let shell = match path.extension().and_then(|extension| extension.to_str()) {
                Some("sh") => text,
                Some("yml") => workflow_shell(&text),
                _ => continue,
            };
            for finding in early_readers(&shell) {
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
fn a_pipe_into_head_named_through_command_is_found() {
    assert_eq!(
        early_readers("printf '%s\\n' \"$a\" | command head -n 1\n"),
        ["1: a pipe into head"]
    );
}

#[test]
fn a_reader_on_the_line_after_a_trailing_pipe_is_found_on_its_line() {
    assert_eq!(
        early_readers("printf '%s\\n' \"$a\" |\n  head -n 1\nprintf b | \\\n  read -r c\n"),
        ["2: a pipe into head", "4: a pipe into read"]
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
fn a_pipe_into_a_grep_with_a_long_option_that_stops_early_is_found() {
    assert_eq!(
        early_readers(
            "printf a | grep --max-count=1 b\nprintf a | grep --silent b\nprintf a | grep --quiet b\n"
        ),
        [
            "1: a pipe into grep --max-count=1",
            "2: a pipe into grep --silent",
            "3: a pipe into grep --quiet"
        ]
    );
}

#[test]
fn a_pipe_into_grep_q_in_an_until_condition_is_found() {
    assert_eq!(
        early_readers(
            "wait_for() {\n  local tries=0\n  until t capture-pane -p -t \"$w\" | grep -qF \"$1\" || [ \"$tries\" -ge 40 ]; do\n    tries=$((tries + 1))\n  done\n}\n"
        ),
        ["3: a pipe into grep -qF"]
    );
}

#[test]
fn a_quote_inside_a_parameter_expansion_does_not_hide_what_follows() {
    assert_eq!(
        early_readers(
            "freeze ${font[@]+\"${font[@]}\"} --font.family \"Fira\"\nprintf a | grep -qF b\n"
        ),
        ["2: a pipe into grep -qF"]
    );
}

#[test]
fn a_pipe_that_carries_stderr_too_is_a_pipe() {
    assert_eq!(
        early_readers("printf a |& head -n 1\n"),
        ["1: a pipe into head"]
    );
}

#[test]
fn an_escaped_quote_in_an_ansi_c_string_does_not_hide_what_follows() {
    assert_eq!(
        early_readers("echo $'a\\'b'\nprintf a | head -n 1\necho 'c'\n"),
        ["2: a pipe into head"]
    );
}

#[test]
fn a_character_outside_ascii_in_awk_code_is_read_past() {
    assert_eq!(
        early_readers("awk '{ x = 1 \u{20ac} 2; exit }'\n"),
        ["1: an awk exit outside BEGIN and END"]
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
fn a_pipe_inside_quotes_or_a_comment_is_text() {
    assert_eq!(
        early_readers(
            "echo \"pipe it | head -n 1\"\necho 'a | grep -q b'\n# printf a | head -n 1\n"
        ),
        Vec::<String>::new()
    );
}

#[test]
fn a_here_document_is_text() {
    assert_eq!(
        early_readers("cat <<EOF\nit's a | head\nEOF\nprintf a | head -n 1\n"),
        ["4: a pipe into head"]
    );
}

#[test]
fn a_comment_naming_awk_with_an_apostrophe_is_skipped_and_the_scan_goes_on() {
    assert_eq!(
        early_readers("# do not pipe into awk when it won't read\nprintf a | head -n 1\n"),
        ["2: a pipe into head"]
    );
}

#[test]
fn an_unclosed_quote_or_a_backslash_at_the_end_is_no_program() {
    assert_eq!(
        early_readers("awk it's\necho \"a\nawk '{ exit\necho a \\"),
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
fn an_exit_in_gawk_or_mawk_named_explicitly_is_found() {
    assert_eq!(
        early_readers("printf a | gawk '{ exit }'\nprintf a | mawk '{ exit }'\n"),
        [
            "1: an awk exit outside BEGIN and END",
            "2: an awk exit outside BEGIN and END"
        ]
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

#[test]
fn a_workflow_is_read_only_in_its_run_steps() {
    let workflow = "name: it's\non: push\njobs:\n  a:\n    steps:\n      - run: printf a | head -n 1\n      - name: b\n        run: |\n          echo \"it's\"\n          printf a | grep -q b\n      - name: printf a | head -n 1\n";

    assert_eq!(
        early_readers(&workflow_shell(workflow)),
        ["6: a pipe into head", "10: a pipe into grep -q"]
    );
}

enum Token {
    Word {
        at: usize,
        end: usize,
        program: Option<(usize, usize)>,
    },
    Pipe,
    Break,
}

type Word<'a> = (usize, &'a str, Option<(usize, usize)>);

const SKIPPED: [&str; 10] = [
    "if", "!", "then", "do", "elif", "else", "{", "time", "command", "exec",
];

fn early_readers(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    lex(text.as_bytes(), 0, false, &mut tokens);
    tokens.push(Token::Break);
    let mut findings = Vec::new();
    let mut words = Vec::new();
    let mut piped = false;
    for token in tokens {
        match token {
            Token::Word { at, end, program } => words.push((at, &text[at..end], program)),
            Token::Pipe | Token::Break => {
                findings.extend(command_findings(text, piped, &words));
                words.clear();
                piped = matches!(token, Token::Pipe);
            }
        }
    }
    findings.sort_by_key(|(line, _)| *line);
    findings
        .into_iter()
        .map(|(line, finding)| format!("{line}: {finding}"))
        .collect()
}

fn command_findings(text: &str, piped: bool, words: &[Word]) -> Vec<(usize, String)> {
    let start = words
        .iter()
        .position(|(_, word, _)| !SKIPPED.contains(word) && !is_assignment(word));
    let Some(start) = start else {
        return Vec::new();
    };
    let (at, word, _) = words[start];
    let name = word.rsplit('/').next().unwrap_or(word);
    let rest = &words[start + 1..];
    let line = line_of(text, at);
    match name {
        "head" | "read" if piped => vec![(line, format!("a pipe into {name}"))],
        "grep" if piped => rest
            .iter()
            .find(|(_, word, _)| stops_grep(word))
            .map(|(_, word, _)| vec![(line, format!("a pipe into grep {word}"))])
            .unwrap_or_default(),
        "awk" | "gawk" | "mawk" | "nawk" => awk_program(rest)
            .map(|(from, to)| {
                exits_in_rules(&text[from..to])
                    .into_iter()
                    .map(|exit| {
                        (
                            line_of(text, from + exit),
                            "an awk exit outside BEGIN and END".to_string(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn stops_grep(word: &str) -> bool {
    let long = word.split('=').next().unwrap_or(word);
    matches!(
        long,
        "--quiet" | "--silent" | "--max-count" | "--files-with-matches" | "--files-without-match"
    ) || (word.starts_with('-') && !word.starts_with("--") && word.contains(['q', 'm', 'l', 'L']))
}

fn awk_program(words: &[Word]) -> Option<(usize, usize)> {
    let mut words = words.iter();
    while let Some((_, word, program)) = words.next() {
        match *word {
            "-v" | "-F" => {
                words.next();
            }
            "--" => return words.next().and_then(|(_, _, program)| *program),
            _ if word.starts_with("-f") || word.starts_with("--file") => return None,
            _ if word.starts_with('-') => {}
            _ => return *program,
        }
    }
    None
}

fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        !name.is_empty() && name.chars().all(is_word) && !name.starts_with(|c: char| c.is_numeric())
    })
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count() + 1
}

fn lex(bytes: &[u8], mut at: usize, substitution: bool, tokens: &mut Vec<Token>) -> usize {
    let mut depth = 0;
    let mut heredocs = Vec::new();
    while at < bytes.len() {
        match bytes[at] {
            b' ' | b'\t' | b'\r' => at += 1,
            b'\\' if bytes.get(at + 1) == Some(&b'\n') => at += 2,
            b'\\' if at + 1 == bytes.len() => at += 1,
            b'\n' => {
                at = past_heredocs(bytes, at, &mut heredocs);
                if !matches!(tokens.last(), Some(Token::Pipe)) {
                    tokens.push(Token::Break);
                }
            }
            b'#' => at = line_end(bytes, at),
            b'|' if bytes.get(at + 1) == Some(&b'|') => {
                tokens.push(Token::Break);
                at += 2;
            }
            b'|' if bytes.get(at + 1) == Some(&b'&') => {
                tokens.push(Token::Pipe);
                at += 2;
            }
            b'|' => {
                tokens.push(Token::Pipe);
                at += 1;
            }
            b'&' | b';' => {
                tokens.push(Token::Break);
                at += 1;
            }
            b'(' => {
                depth += 1;
                tokens.push(Token::Break);
                at += 1;
            }
            b')' => {
                tokens.push(Token::Break);
                at += 1;
                if substitution && depth == 0 {
                    return at;
                }
                depth = (depth - 1).max(0);
            }
            _ => at = word(bytes, at, tokens, &mut heredocs),
        }
    }
    at
}

fn word(bytes: &[u8], start: usize, tokens: &mut Vec<Token>, heredocs: &mut Vec<Vec<u8>>) -> usize {
    if bytes[start..].starts_with(b"<<") && !bytes[start..].starts_with(b"<<<") {
        return heredoc(bytes, start + 2, heredocs);
    }
    let mut at = start;
    let mut single = None;
    let mut parts = 0;
    while at < bytes.len() && !b" \t\r\n|&;()".contains(&bytes[at]) {
        parts += 1;
        match bytes[at] {
            b'\'' => match bytes[at + 1..].iter().position(|&c| c == b'\'') {
                Some(length) => {
                    single = Some((at + 1, at + 1 + length));
                    at += length + 2;
                }
                None => at += 1,
            },
            b'"' => at = double_quoted(bytes, at + 1, tokens),
            b'\\' => at = (at + 2).min(bytes.len()),
            b'$' if bytes.get(at + 1) == Some(&b'\'') => at = ansi_c_quoted(bytes, at + 2),
            b'$' | b'<' | b'>' if bytes.get(at + 1) == Some(&b'(') => {
                at = substituted(bytes, at + 1, tokens)
            }
            b'$' if bytes.get(at + 1) == Some(&b'{') => at = braced(bytes, at + 2, tokens),
            _ => at += 1,
        }
    }
    let program = single.filter(|&(from, to)| parts == 1 && from == start + 1 && to + 1 == at);
    tokens.push(Token::Word {
        at: start,
        end: at,
        program,
    });
    at
}

fn ansi_c_quoted(bytes: &[u8], mut at: usize) -> usize {
    while at < bytes.len() {
        match bytes[at] {
            b'\'' => return at + 1,
            b'\\' => at += 2,
            _ => at += 1,
        }
    }
    bytes.len()
}

fn double_quoted(bytes: &[u8], mut at: usize, tokens: &mut Vec<Token>) -> usize {
    while at < bytes.len() {
        match bytes[at] {
            b'"' => return at + 1,
            b'\\' => at += 2,
            b'$' if bytes.get(at + 1) == Some(&b'(') => at = substituted(bytes, at + 1, tokens),
            b'$' if bytes.get(at + 1) == Some(&b'{') => at = braced(bytes, at + 2, tokens),
            _ => at += 1,
        }
    }
    bytes.len()
}

fn braced(bytes: &[u8], mut at: usize, tokens: &mut Vec<Token>) -> usize {
    while at < bytes.len() {
        match bytes[at] {
            b'}' => return at + 1,
            b'"' => at = double_quoted(bytes, at + 1, tokens),
            b'\'' => {
                at = bytes[at + 1..]
                    .iter()
                    .position(|&c| c == b'\'')
                    .map_or(bytes.len(), |length| at + length + 2)
            }
            b'\\' => at += 2,
            b'$' if bytes.get(at + 1) == Some(&b'(') => at = substituted(bytes, at + 1, tokens),
            b'$' if bytes.get(at + 1) == Some(&b'{') => at = braced(bytes, at + 2, tokens),
            _ => at += 1,
        }
    }
    bytes.len()
}

fn substituted(bytes: &[u8], open: usize, tokens: &mut Vec<Token>) -> usize {
    if bytes.get(open + 1) == Some(&b'(') {
        let mut depth = 0;
        for (at, &byte) in bytes.iter().enumerate().skip(open) {
            match byte {
                b'(' => depth += 1,
                b')' if depth == 1 => return at + 1,
                b')' => depth -= 1,
                _ => {}
            }
        }
        return bytes.len();
    }
    tokens.push(Token::Break);
    lex(bytes, open + 1, true, tokens)
}

fn heredoc(bytes: &[u8], mut at: usize, heredocs: &mut Vec<Vec<u8>>) -> usize {
    if bytes.get(at) == Some(&b'-') {
        at += 1;
    }
    while bytes.get(at).is_some_and(|&c| c == b' ' || c == b'\t') {
        at += 1;
    }
    let start = at;
    while at < bytes.len() && !b" \t\r\n|&;()<>".contains(&bytes[at]) {
        at += 1;
    }
    let delimiter: Vec<u8> = bytes[start..at]
        .iter()
        .copied()
        .filter(|c| !b"'\"\\".contains(c))
        .collect();
    if !delimiter.is_empty() {
        heredocs.push(delimiter);
    }
    at
}

fn past_heredocs(bytes: &[u8], newline: usize, heredocs: &mut Vec<Vec<u8>>) -> usize {
    let mut at = newline;
    for delimiter in heredocs.drain(..) {
        loop {
            let start = at + 1;
            if start >= bytes.len() {
                return bytes.len();
            }
            at = line_end(bytes, start);
            if bytes[start..at].trim_ascii() == delimiter.as_slice() {
                break;
            }
        }
    }
    at + 1
}

fn line_end(bytes: &[u8], at: usize) -> usize {
    bytes[at..]
        .iter()
        .position(|&c| c == b'\n')
        .map_or(bytes.len(), |length| at + length)
}

fn workflow_shell(text: &str) -> String {
    let mut lines = Vec::new();
    let mut block = None;
    for line in text.lines() {
        let indent = line.len() - line.trim_start().len();
        if let Some(key) = block {
            if line.trim().is_empty() || indent > key {
                lines.push(line);
                continue;
            }
            block = None;
        }
        let item = line.trim_start().strip_prefix("- ");
        let key = if item.is_some() { indent + 2 } else { indent };
        match item.unwrap_or(line.trim_start()).strip_prefix("run:") {
            Some(rest) if matches!(rest.trim(), "|" | "|-" | ">" | ">-") => {
                block = Some(key);
                lines.push("");
            }
            Some(rest) => lines.push(rest),
            None => lines.push(""),
        }
    }
    lines.join("\n") + "\n"
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
                depth = (depth - 1).max(0);
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
            && bytes[at..].starts_with(b"exit")
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
            b'"' => Some(awk_string_end(bytes, at + 1)),
            b'/' if b"\n({},;!~&|".contains(&previous) => Some(awk_regex_end(bytes, at + 1)),
            b'#' => Some(line_end(bytes, at) - 1),
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
    String::from_utf8_lossy(&code).into_owned()
}

fn awk_string_end(bytes: &[u8], mut at: usize) -> usize {
    while at < bytes.len() {
        match bytes[at] {
            b'"' => return at,
            b'\\' => at += 2,
            _ => at += 1,
        }
    }
    bytes.len() - 1
}

fn awk_regex_end(bytes: &[u8], mut at: usize) -> usize {
    let mut bracket = false;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 1,
            b'[' => bracket = true,
            b']' => bracket = false,
            b'/' if !bracket => return at,
            _ => {}
        }
        at += 1;
    }
    bytes.len() - 1
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
