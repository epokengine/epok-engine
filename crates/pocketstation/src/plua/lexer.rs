//! Pocket Lua tokens.

use super::Diagnostic;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Name(String),
    Int(i64),
    Str(String),
    /// Keywords and punctuation.
    Sym(&'static str),
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub column: u32,
}

const KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

const SYMBOLS: &[&str] = &[
    "...", "..", "==", "~=", "<=", ">=", "<<", ">>", "//", "::", "+", "-", "*", "/", "%", "^", "#",
    "&", "~", "|", "<", ">", "=", "(", ")", "{", "}", "[", "]", ";", ":", ",", ".",
];

pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, Diagnostic> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    let mut line = 1u32;
    let mut line_start = 0usize;
    let error = |line: u32, column: u32, message: String| Diagnostic {
        file: file.to_string(),
        line,
        column,
        message,
    };
    while i < chars.len() {
        let c = chars[i];
        let column = (i - line_start) as u32 + 1;
        if c == '\n' {
            line += 1;
            i += 1;
            line_start = i;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // Comments: -- line, --[[ block ]] / --[==[ ]==].
        if c == '-' && chars.get(i + 1) == Some(&'-') {
            i += 2;
            if let Some(level) = long_bracket(&chars, i) {
                let (end, lines, _) = long_string(&chars, i, level)
                    .ok_or_else(|| error(line, column, "unfinished long comment".into()))?;
                for &ch in &chars[i..end] {
                    if ch == '\n' {
                        line += 1;
                    }
                }
                let _ = lines;
                i = end;
                if let Some(pos) = chars[..i].iter().rposition(|c| *c == '\n') {
                    line_start = pos + 1;
                }
                continue;
            }
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            let mut text = String::new();
            if c == '0' && matches!(chars.get(i + 1), Some('x') | Some('X')) {
                i += 2;
                while i < chars.len() && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                    text.push(chars[i]);
                    i += 1;
                }
                let value = i64::from_str_radix(&text.replace('_', ""), 16)
                    .map_err(|_| error(line, column, "malformed hexadecimal number".into()))?;
                tokens.push(Token {
                    tok: Tok::Int(value),
                    line,
                    column,
                });
            } else if c == '0' && matches!(chars.get(i + 1), Some('b') | Some('B')) {
                i += 2;
                while i < chars.len() && matches!(chars[i], '0' | '1' | '_') {
                    text.push(chars[i]);
                    i += 1;
                }
                let value = i64::from_str_radix(&text.replace('_', ""), 2)
                    .map_err(|_| error(line, column, "malformed binary number".into()))?;
                tokens.push(Token {
                    tok: Tok::Int(value),
                    line,
                    column,
                });
            } else {
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                    text.push(chars[i]);
                    i += 1;
                }
                if i < chars.len()
                    && (chars[i] == '.' && chars.get(i + 1).is_some_and(|c| c.is_ascii_digit())
                        || chars[i] == 'e'
                        || chars[i] == 'E')
                {
                    return Err(error(
                        line,
                        column,
                        "Pocket Lua numbers are whole numbers; the PocketStation has no floating point".into(),
                    ));
                }
                let value: i64 = text
                    .replace('_', "")
                    .parse()
                    .map_err(|_| error(line, column, "number is too large".into()))?;
                tokens.push(Token {
                    tok: Tok::Int(value),
                    line,
                    column,
                });
            }
            if i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                let _ = start;
                return Err(error(line, column, "malformed number".into()));
            }
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let tok = match KEYWORDS.iter().find(|k| **k == word) {
                Some(k) => Tok::Sym(k),
                None => Tok::Name(word),
            };
            tokens.push(Token { tok, line, column });
            continue;
        }
        if c == '"' || c == '\'' {
            let quote = c;
            i += 1;
            let mut text = String::new();
            loop {
                let Some(&ch) = chars.get(i) else {
                    return Err(error(line, column, "unfinished string".into()));
                };
                if ch == '\n' {
                    return Err(error(line, column, "unfinished string".into()));
                }
                i += 1;
                if ch == quote {
                    break;
                }
                if ch == '\\' {
                    let esc = chars.get(i).copied().unwrap_or(' ');
                    i += 1;
                    text.push(match esc {
                        'n' => '\n',
                        't' => '\t',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        _ => return Err(error(line, column, format!("unknown escape `\\{esc}`"))),
                    });
                } else {
                    text.push(ch);
                }
            }
            tokens.push(Token {
                tok: Tok::Str(text),
                line,
                column,
            });
            continue;
        }
        if c == '['
            && let Some(level) = long_bracket(&chars, i)
        {
            let (end, _, content) = long_string(&chars, i, level)
                .ok_or_else(|| error(line, column, "unfinished long string".into()))?;
            tokens.push(Token {
                tok: Tok::Str(content),
                line,
                column,
            });
            for &ch in &chars[i..end] {
                if ch == '\n' {
                    line += 1;
                }
            }
            i = end;
            if let Some(pos) = chars[..i].iter().rposition(|c| *c == '\n') {
                line_start = pos + 1;
            }
            continue;
        }
        let rest: String = chars[i..chars.len().min(i + 3)].iter().collect();
        match SYMBOLS.iter().find(|s| rest.starts_with(**s)) {
            Some(s) => {
                tokens.push(Token {
                    tok: Tok::Sym(s),
                    line,
                    column,
                });
                i += s.len();
            }
            None => return Err(error(line, column, format!("unexpected character `{c}`"))),
        }
    }
    tokens.push(Token {
        tok: Tok::Eof,
        line,
        column: 1,
    });
    Ok(tokens)
}

/// Level of a long bracket `[[` / `[==[` starting at `i`, if any.
fn long_bracket(chars: &[char], i: usize) -> Option<usize> {
    if chars.get(i) != Some(&'[') {
        return None;
    }
    let mut j = i + 1;
    while chars.get(j) == Some(&'=') {
        j += 1;
    }
    (chars.get(j) == Some(&'[')).then_some(j - i - 1)
}

/// Returns (end index after the closing bracket, newline count, content).
fn long_string(chars: &[char], i: usize, level: usize) -> Option<(usize, u32, String)> {
    let open_len = level + 2;
    let mut j = i + open_len;
    // A newline right after the opening bracket is skipped.
    if chars.get(j) == Some(&'\n') {
        j += 1;
    }
    let start = j;
    while j < chars.len() {
        if chars[j] == ']' {
            let mut k = j + 1;
            let mut eq = 0;
            while chars.get(k) == Some(&'=') {
                eq += 1;
                k += 1;
            }
            if eq == level && chars.get(k) == Some(&']') {
                let content: String = chars[start..j].iter().collect();
                let lines = content.matches('\n').count() as u32;
                return Some((k + 1, lines, content));
            }
        }
        j += 1;
    }
    None
}
