//! Constant expressions used by the assembler.

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Number(i64),
    Symbol(String),
    /// The address of the current statement (`.` or `$`).
    Here,
    /// A numeric local label reference such as `1b` or `2f`.
    Local(u32, bool),
    Unary(char, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, Default)]
pub struct Symbol {
    pub value: i64,
    /// Defined as a label in Thumb code following `.thumb_func`.
    pub thumb_func: bool,
    pub label: bool,
}

pub struct Scope<'a> {
    pub symbols: &'a HashMap<String, Symbol>,
    pub here: i64,
    /// Addresses of numeric local labels, in source order: (number, statement index, address).
    pub locals: &'a [(u32, usize, i64)],
    pub statement: usize,
}

/// The result of evaluating an expression. `known` is false when a symbol is
/// not (yet) defined; the value is then a placeholder of zero.
#[derive(Clone, Copy, Debug)]
pub struct Value {
    pub value: i64,
    pub known: bool,
}

impl Expr {
    pub fn eval(&self, scope: &Scope) -> Result<Value, String> {
        Ok(match self {
            Expr::Number(n) => Value {
                value: *n,
                known: true,
            },
            Expr::Here => Value {
                value: scope.here,
                known: true,
            },
            Expr::Symbol(name) => match scope.symbols.get(name) {
                Some(symbol) => Value {
                    value: symbol.value,
                    known: true,
                },
                None => Value {
                    value: 0,
                    known: false,
                },
            },
            Expr::Local(number, forward) => {
                let found = if *forward {
                    scope
                        .locals
                        .iter()
                        .find(|(n, at, _)| n == number && *at > scope.statement)
                } else {
                    scope
                        .locals
                        .iter()
                        .rev()
                        .find(|(n, at, _)| n == number && *at <= scope.statement)
                };
                match found {
                    Some((_, _, address)) => Value {
                        value: *address,
                        known: true,
                    },
                    None => Value {
                        value: 0,
                        known: false,
                    },
                }
            }
            Expr::Unary(op, inner) => {
                let v = inner.eval(scope)?;
                let value = match op {
                    '-' => v.value.wrapping_neg(),
                    '~' => !v.value,
                    '!' => (v.value == 0) as i64,
                    _ => v.value,
                };
                Value {
                    value,
                    known: v.known,
                }
            }
            Expr::Binary(op, a, b) => {
                let a = a.eval(scope)?;
                let b = b.eval(scope)?;
                let known = a.known && b.known;
                let (x, y) = (a.value, b.value);
                let value = match *op {
                    "+" => x.wrapping_add(y),
                    "-" => x.wrapping_sub(y),
                    "*" => x.wrapping_mul(y),
                    "/" | "%" => {
                        if y == 0 {
                            if known {
                                return Err("division by zero in expression".into());
                            }
                            0
                        } else if *op == "/" {
                            x.wrapping_div(y)
                        } else {
                            x.wrapping_rem(y)
                        }
                    }
                    "<<" => x.wrapping_shl(y as u32),
                    ">>" => ((x as u64 & 0xFFFF_FFFF) >> (y as u32 & 63)) as i64,
                    "&" => x & y,
                    "|" => x | y,
                    "^" => x ^ y,
                    "==" => (x == y) as i64,
                    "!=" => (x != y) as i64,
                    "<" => (x < y) as i64,
                    ">" => (x > y) as i64,
                    "<=" => (x <= y) as i64,
                    ">=" => (x >= y) as i64,
                    "&&" => (x != 0 && y != 0) as i64,
                    "||" => (x != 0 || y != 0) as i64,
                    _ => return Err(format!("unknown operator {op}")),
                };
                Value { value, known }
            }
        })
    }

    /// The single symbol this expression names, if it is just a symbol.
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Expr::Symbol(name) => Some(name),
            _ => None,
        }
    }
}

pub fn parse(text: &str) -> Result<Expr, String> {
    let tokens = tokenize(text)?;
    if tokens.is_empty() {
        return Err("expected an expression".into());
    }
    let mut parser = Parser { tokens, at: 0 };
    let expr = parser.binary(0)?;
    if parser.at != parser.tokens.len() {
        return Err(format!(
            "unexpected `{}` in expression",
            parser.tokens[parser.at]
        ));
    }
    Ok(expr)
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(i64),
    Name(String),
    Op(&'static str),
    Open,
    Close,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{n}"),
            Token::Name(n) => write!(f, "{n}"),
            Token::Op(o) => write!(f, "{o}"),
            Token::Open => write!(f, "("),
            Token::Close => write!(f, ")"),
        }
    }
}

const OPERATORS: &[&str] = &[
    "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "+", "-", "*", "/", "%", "&", "|", "^", "~",
    "!", "<", ">",
];

pub fn parse_number(text: &str) -> Option<i64> {
    let t = text.replace('_', "");
    let lower = t.to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix("0x") {
        return i64::from_str_radix(hex, 16).ok();
    }
    if let Some(bin) = lower.strip_prefix("0b") {
        return i64::from_str_radix(bin, 2).ok();
    }
    if let Some(hex) = lower.strip_suffix('h')
        && hex.starts_with(|c: char| c.is_ascii_digit())
    {
        return i64::from_str_radix(hex, 16).ok();
    }
    lower.parse::<i64>().ok()
}

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '(' {
            tokens.push(Token::Open);
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(Token::Close);
            i += 1;
            continue;
        }
        if c == '\'' {
            // Character literal 'A' or '\n'.
            let (value, used) = match (chars.get(i + 1), chars.get(i + 2), chars.get(i + 3)) {
                (Some('\\'), Some(e), Some('\'')) => {
                    let v = match e {
                        'n' => 10,
                        'r' => 13,
                        't' => 9,
                        '0' => 0,
                        other => *other as i64,
                    };
                    (v, 4)
                }
                (Some(ch), Some('\''), _) => (*ch as i64, 3),
                _ => return Err("malformed character literal".into()),
            };
            tokens.push(Token::Number(value));
            i += used;
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            // Numeric local label reference: 1b / 1f.
            if word.len() >= 2 && (word.ends_with('b') || word.ends_with('f')) {
                let digits = &word[..word.len() - 1];
                if digits.chars().all(|d| d.is_ascii_digit()) && !word.starts_with("0b") {
                    tokens.push(Token::Name(format!("@local:{word}")));
                    continue;
                }
            }
            match parse_number(&word) {
                Some(n) => tokens.push(Token::Number(n)),
                None => return Err(format!("invalid number `{word}`")),
            }
            continue;
        }
        if c.is_alphabetic() || c == '_' || c == '.' || c == '$' {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_alphanumeric()
                    || chars[i] == '_'
                    || chars[i] == '.'
                    || chars[i] == '$')
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if word == "." || word == "$" {
                tokens.push(Token::Name("@here".into()));
            } else if let Some(hex) = word.strip_prefix('$') {
                match i64::from_str_radix(hex, 16) {
                    Ok(n) => tokens.push(Token::Number(n)),
                    Err(_) => return Err(format!("invalid hexadecimal number `{word}`")),
                }
            } else {
                tokens.push(Token::Name(word));
            }
            continue;
        }
        let rest: String = chars[i..].iter().take(2).collect();
        match OPERATORS.iter().find(|op| rest.starts_with(**op)) {
            Some(op) => {
                tokens.push(Token::Op(op));
                i += op.len();
            }
            None => return Err(format!("unexpected character `{c}` in expression")),
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

fn precedence(op: &str) -> Option<u8> {
    Some(match op {
        "||" => 1,
        "&&" => 2,
        "|" => 3,
        "^" => 4,
        "&" => 5,
        "==" | "!=" => 6,
        "<" | ">" | "<=" | ">=" => 7,
        "<<" | ">>" => 8,
        "+" | "-" => 9,
        "*" | "/" | "%" => 10,
        _ => return None,
    })
}

impl Parser {
    fn binary(&mut self, min: u8) -> Result<Expr, String> {
        let mut left = self.unary()?;
        while let Some(Token::Op(op)) = self.tokens.get(self.at).cloned() {
            let Some(p) = precedence(op) else { break };
            if p <= min {
                break;
            }
            self.at += 1;
            let right = self.binary(p)?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        match self.tokens.get(self.at).cloned() {
            Some(Token::Op(op)) if op == "-" || op == "~" || op == "!" || op == "+" => {
                self.at += 1;
                let inner = self.unary()?;
                Ok(Expr::Unary(op.chars().next().unwrap(), Box::new(inner)))
            }
            Some(Token::Open) => {
                self.at += 1;
                let inner = self.binary(0)?;
                if self.tokens.get(self.at) != Some(&Token::Close) {
                    return Err("missing `)` in expression".into());
                }
                self.at += 1;
                Ok(inner)
            }
            Some(Token::Number(n)) => {
                self.at += 1;
                Ok(Expr::Number(n))
            }
            Some(Token::Name(name)) => {
                self.at += 1;
                if name == "@here" {
                    return Ok(Expr::Here);
                }
                if let Some(local) = name.strip_prefix("@local:") {
                    let forward = local.ends_with('f');
                    let number = local[..local.len() - 1].parse().unwrap_or(0);
                    return Ok(Expr::Local(number, forward));
                }
                Ok(Expr::Symbol(name))
            }
            Some(other) => Err(format!("unexpected `{other}` in expression")),
            None => Err("expression ends unexpectedly".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(text: &str) -> i64 {
        let symbols = HashMap::from([(
            "five".to_string(),
            Symbol {
                value: 5,
                ..Default::default()
            },
        )]);
        let scope = Scope {
            symbols: &symbols,
            here: 0x100,
            locals: &[],
            statement: 0,
        };
        parse(text).unwrap().eval(&scope).unwrap().value
    }

    #[test]
    fn precedence_and_literals() {
        assert_eq!(value("1 + 2 * 3"), 7);
        assert_eq!(value("(1 + 2) * 3"), 9);
        assert_eq!(value("0x10 | 0b1"), 17);
        assert_eq!(value("1 << 4 + 1"), 32);
        assert_eq!(value("-five + 'A'"), 60);
        assert_eq!(value(". + 4"), 0x104);
        assert_eq!(value("0FFh"), 255);
        assert_eq!(value("10 - 3 - 2"), 5);
        assert_eq!(value("~0 & 0xFF"), 255);
    }
}
