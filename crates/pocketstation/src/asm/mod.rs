//! A two-pass (with branch relaxation) assembler for ARM7TDMI ARM and Thumb code.
//!
//! The syntax follows the common ARM conventions: `label:` definitions,
//! numeric local labels (`1:` referenced as `1b`/`1f`), `;`, `@` and `//`
//! comments, condition and `s` suffixes in either order (`addseq`/`addeqs`),
//! `ldr rX, =value` literal pools flushed by `.pool`/`.ltorg` (and at the end),
//! and the directives listed in [`Assembler::directive`].
//!
//! Thumb conditional branches whose target is out of range are relaxed into an
//! inverted branch over an unconditional one, so generated code never has to
//! care about the short conditional range.

mod arm;
pub mod expr;
mod thumb;

use expr::{Scope, Symbol};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: String,
    pub line: u32,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}

/// Source position of an emitted address range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineInfo {
    pub address: u32,
    pub file: String,
    pub line: u32,
    pub thumb: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Assembly {
    pub base: u32,
    pub bytes: Vec<u8>,
    /// Every label and constant, with Thumb function labels carrying bit 0.
    pub symbols: BTreeMap<String, u32>,
    /// Labels only (address of the label, without the Thumb bit).
    pub labels: BTreeMap<String, u32>,
    /// One entry per emitted instruction or data statement.
    pub lines: Vec<LineInfo>,
}

impl Assembly {
    pub fn symbol(&self, name: &str) -> Option<u32> {
        self.symbols.get(name).copied()
    }
    pub fn end(&self) -> u32 {
        self.base + self.bytes.len() as u32
    }
}

pub type Include<'a> = &'a dyn Fn(&str) -> Option<String>;

pub struct Options<'a> {
    pub base: u32,
    pub thumb: bool,
    pub file: String,
    pub symbols: Vec<(String, i64)>,
    pub include: Option<Include<'a>>,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            base: 0,
            thumb: false,
            file: "source.s".into(),
            symbols: Vec::new(),
            include: None,
        }
    }
}

/// One source statement after include expansion.
#[derive(Clone, Debug)]
struct Statement {
    file: String,
    line: u32,
    labels: Vec<String>,
    mnemonic: String,
    operands: String,
}

#[derive(Clone, Debug, Default)]
struct Layout {
    addresses: Vec<u32>,
    sizes: Vec<u32>,
    thumb: Vec<bool>,
    /// For each `ldr =` statement, the pool slot address.
    literal_slots: HashMap<usize, u32>,
    /// Pool contents per flush statement: (statement index of flush, entries).
    pools: Vec<(usize, Vec<usize>)>,
}

pub(crate) struct Context<'a> {
    pub scope: Scope<'a>,
    pub thumb_funcs: &'a HashMap<String, Symbol>,
    pub final_pass: bool,
    pub literal: Option<u32>,
}

impl Context<'_> {
    pub fn eval(&self, text: &str) -> Result<i64, String> {
        let expr = expr::parse(text)?;
        let value = expr.eval(&self.scope)?;
        if !value.known && self.final_pass {
            return Err(format!("undefined symbol in `{}`", text.trim()));
        }
        Ok(value.value)
    }
    /// Evaluates an address expression, adding the Thumb bit for `.thumb_func` labels.
    pub fn eval_code_address(&self, text: &str) -> Result<i64, String> {
        let expr = expr::parse(text)?;
        let value = expr.eval(&self.scope)?;
        if !value.known && self.final_pass {
            return Err(format!("undefined symbol in `{}`", text.trim()));
        }
        if let Some(name) = expr.as_symbol()
            && self.thumb_funcs.get(name).is_some_and(|s| s.thumb_func)
        {
            return Ok(value.value | 1);
        }
        Ok(value.value)
    }
    pub fn here(&self) -> u32 {
        self.scope.here as u32
    }
}

pub fn assemble(source: &str, options: &Options) -> Result<Assembly, Vec<Diagnostic>> {
    let mut statements = Vec::new();
    let mut errors = Vec::new();
    let mut pending = Vec::new();
    let mut loc = None;
    expand(
        source,
        &options.file,
        options,
        0,
        &mut statements,
        &mut errors,
        &mut pending,
        &mut loc,
    );
    if !pending.is_empty() {
        statements.push(Statement {
            file: options.file.clone(),
            line: source.lines().count() as u32,
            labels: pending,
            mnemonic: String::new(),
            operands: String::new(),
        });
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Assembler {
        statements,
        options,
        relaxed: HashMap::new(),
        previous: Vec::new(),
    }
    .run()
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut quote = '"';
    let bytes: Vec<char> = line.chars().collect();
    let mut cut = line.len();
    let mut byte_index = 0;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if in_string {
            if c == '\\' {
                byte_index += c.len_utf8();
                i += 1;
                if i < bytes.len() {
                    byte_index += bytes[i].len_utf8();
                    i += 1;
                }
                continue;
            }
            if c == quote {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            quote = '"';
        } else if c == '\'' && i + 2 < bytes.len() && bytes[i + 2] == '\'' {
            byte_index += bytes[i].len_utf8() + bytes[i + 1].len_utf8() + bytes[i + 2].len_utf8();
            i += 3;
            continue;
        } else if c == ';' || c == '@' || (c == '/' && bytes.get(i + 1) == Some(&'/')) {
            cut = byte_index;
            break;
        }
        byte_index += c.len_utf8();
        i += 1;
    }
    &line[..cut]
}

#[allow(clippy::too_many_arguments)]
fn expand(
    source: &str,
    file: &str,
    options: &Options,
    depth: u32,
    out: &mut Vec<Statement>,
    errors: &mut Vec<Diagnostic>,
    pending: &mut Vec<String>,
    loc: &mut Option<(String, u32)>,
) {
    for (index, raw) in source.lines().enumerate() {
        let line = index as u32 + 1;
        let mut text = strip_comment(raw).trim();
        let mut labels = Vec::new();
        // Labels: `name:` possibly several, possibly followed by a statement.
        while let Some(colon) = text.find(':') {
            let candidate = text[..colon].trim();
            let valid = !candidate.is_empty()
                && !candidate.contains(char::is_whitespace)
                && !candidate.contains('"')
                && candidate
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '$');
            if !valid {
                break;
            }
            labels.push(candidate.to_string());
            text = text[colon + 1..].trim();
        }
        // `name = value` and `name equ value` constants.
        let (mnemonic, operands) = match text.split_once(char::is_whitespace) {
            Some((m, rest)) => (m.to_string(), rest.trim().to_string()),
            None => (text.to_string(), String::new()),
        };
        let (mnemonic, operands) = if let Some((name, value)) = text.split_once('=') {
            let name = name.trim();
            if !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
                && !name.starts_with('.')
                && !value.starts_with('=')
            {
                (".equ".to_string(), format!("{name}, {}", value.trim()))
            } else {
                (mnemonic, operands)
            }
        } else if operands.to_ascii_lowercase().starts_with("equ ") {
            (
                ".equ".to_string(),
                format!("{mnemonic}, {}", operands[4..].trim()),
            )
        } else {
            (mnemonic, operands)
        };
        if mnemonic.eq_ignore_ascii_case(".loc") {
            // `.loc line` or `.loc "file", line`: source position of what follows.
            let parts = split_operands(&operands);
            let (name, number) = match parts.as_slice() {
                [n] => (
                    loc.as_ref()
                        .map(|l| l.0.clone())
                        .unwrap_or_else(|| file.to_string()),
                    n.clone(),
                ),
                [f, n] => (f.trim_matches('"').to_string(), n.clone()),
                _ => (file.to_string(), "0".to_string()),
            };
            match number.trim().parse::<u32>() {
                Ok(n) => *loc = Some((name, n)),
                Err(_) => errors.push(Diagnostic {
                    file: file.into(),
                    line,
                    message: "`.loc` needs a line number".into(),
                }),
            }
            continue;
        }
        if mnemonic.eq_ignore_ascii_case(".include") {
            let name = operands.trim().trim_matches('"');
            if depth > 8 {
                errors.push(Diagnostic {
                    file: file.into(),
                    line,
                    message: "includes are nested too deeply".into(),
                });
                continue;
            }
            match options.include.and_then(|f| f(name)) {
                Some(text) => {
                    pending.extend(labels);
                    expand(&text, name, options, depth + 1, out, errors, pending, loc);
                }
                None => errors.push(Diagnostic {
                    file: file.into(),
                    line,
                    message: format!("cannot include `{name}`"),
                }),
            }
            continue;
        }
        if mnemonic.is_empty() {
            // Label-only lines attach to the next statement so the label takes
            // that statement's (possibly aligned) address.
            pending.extend(labels);
            continue;
        }
        let mut all = std::mem::take(pending);
        all.extend(labels);
        let (file_name, line) = match loc {
            Some((f, l)) => (f.clone(), *l),
            None => (file.to_string(), line),
        };
        out.push(Statement {
            file: file_name,
            line,
            labels: all,
            mnemonic,
            operands,
        });
    }
}

/// Splits operands at top-level commas (outside brackets, braces, quotes).
pub(crate) fn split_operands(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    let mut in_string = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            current.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                current.push(c);
            }
            '[' | '{' | '(' => {
                depth += 1;
                current.push(c);
            }
            ']' | '}' | ')' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() || !parts.is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

pub(crate) fn register(text: &str) -> Option<u32> {
    let t = text.trim().to_ascii_lowercase();
    Some(match t.as_str() {
        "sp" => 13,
        "lr" => 14,
        "pc" => 15,
        "ip" => 12,
        "fp" => 11,
        "sl" => 10,
        "sb" => 9,
        "a1" => 0,
        "a2" => 1,
        "a3" => 2,
        "a4" => 3,
        _ => {
            if let Some(n) = t.strip_prefix('r') {
                let n: u32 = n.parse().ok()?;
                if n > 15 {
                    return None;
                }
                n
            } else {
                let n = t.strip_prefix('v')?;
                let n: u32 = n.parse().ok()?;
                if !(1..=8).contains(&n) {
                    return None;
                }
                n + 3
            }
        }
    })
}

/// Parses `{r0-r3, lr}` into a bit mask.
pub(crate) fn register_list(text: &str) -> Result<u32, String> {
    let inner = text
        .trim()
        .strip_prefix('{')
        .and_then(|t| t.strip_suffix('}'))
        .ok_or_else(|| format!("expected a register list, found `{text}`"))?;
    let mut mask = 0;
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            let a = register(a).ok_or_else(|| format!("unknown register `{a}`"))?;
            let b = register(b).ok_or_else(|| format!("unknown register `{b}`"))?;
            if a > b {
                return Err(format!("register range `{part}` is reversed"));
            }
            for r in a..=b {
                mask |= 1 << r;
            }
        } else {
            let r = register(part).ok_or_else(|| format!("unknown register `{part}`"))?;
            mask |= 1 << r;
        }
    }
    Ok(mask)
}

pub(crate) fn immediate(text: &str) -> Option<&str> {
    let t = text.trim();
    t.strip_prefix('#').or_else(|| {
        // Bare numbers are accepted as immediates for convenience.
        t.starts_with(|c: char| c.is_ascii_digit() || c == '-')
            .then_some(t)
    })
}

pub(crate) const CONDITIONS: &[&str] = &[
    "eq", "ne", "cs", "cc", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt", "gt", "le", "al",
];

pub(crate) fn condition(text: &str) -> Option<u32> {
    match text {
        "hs" => Some(2),
        "lo" => Some(3),
        "" => Some(14),
        _ => CONDITIONS.iter().position(|c| *c == text).map(|p| p as u32),
    }
}

fn parse_string(text: &str) -> Result<Vec<u8>, String> {
    let t = text.trim();
    let inner = t
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .ok_or_else(|| format!("expected a quoted string, found `{t}`"))?;
    let mut out = Vec::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push(10),
                Some('r') => out.push(13),
                Some('t') => out.push(9),
                Some('0') => out.push(0),
                Some('"') => out.push(b'"'),
                Some('\\') => out.push(b'\\'),
                Some('x') => {
                    let hex: String = chars.by_ref().take(2).collect();
                    out.push(u8::from_str_radix(&hex, 16).map_err(|_| "bad \\x escape")?);
                }
                Some(other) => return Err(format!("unknown escape `\\{other}`")),
                None => return Err("string ends with a backslash".into()),
            }
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
    Ok(out)
}

struct Assembler<'a> {
    statements: Vec<Statement>,
    options: &'a Options<'a>,
    /// Thumb conditional branches (statement index) expanded to 4 bytes.
    relaxed: HashMap<usize, bool>,
    /// Statement addresses of the previous pass (for numeric local labels).
    previous: Vec<u32>,
}

impl Assembler<'_> {
    fn run(mut self) -> Result<Assembly, Vec<Diagnostic>> {
        let mut symbols: HashMap<String, Symbol> = HashMap::new();
        for (name, value) in &self.options.symbols {
            symbols.insert(
                name.clone(),
                Symbol {
                    value: *value,
                    ..Default::default()
                },
            );
        }
        let predefined = symbols.clone();
        let mut previous: Option<Vec<u32>> = None;
        let mut layout = Layout::default();
        for _ in 0..16 {
            let mut next = predefined.clone();
            layout = self.layout(&symbols, &mut next).map_err(|e| vec![e])?;
            self.previous = layout.addresses.clone();
            let stable =
                previous.as_ref() == Some(&layout.addresses) && symbols_equal(&symbols, &next);
            symbols = next;
            let relaxed_now = self.relax(&symbols, &layout);
            if stable && !relaxed_now {
                break;
            }
            previous = Some(layout.addresses.clone());
        }
        self.emit(&symbols, &layout)
    }

    fn locals(&self, layout: &Layout) -> Vec<(u32, usize, i64)> {
        let mut locals = Vec::new();
        for (i, s) in self.statements.iter().enumerate() {
            for label in &s.labels {
                if let Ok(n) = label.parse::<u32>() {
                    locals.push((n, i, layout.addresses.get(i).copied().unwrap_or(0) as i64));
                }
            }
        }
        locals
    }

    fn statement_size(
        &mut self,
        index: usize,
        thumb: bool,
        here: u32,
        symbols: &HashMap<String, Symbol>,
        locals: &[(u32, usize, i64)],
    ) -> Result<(u32, bool, Option<u32>), String> {
        let s = &self.statements[index];
        let mnemonic = s.mnemonic.to_ascii_lowercase();
        let scope = Scope {
            symbols,
            here: here as i64,
            locals,
            statement: index,
        };
        let ctx = Context {
            scope,
            thumb_funcs: symbols,
            final_pass: false,
            literal: None,
        };
        // Returns (size, new thumb flag, alignment requirement).
        if mnemonic.is_empty() {
            return Ok((0, thumb, None));
        }
        if mnemonic.starts_with('.') {
            let ops = split_operands(&s.operands);
            return Ok(match mnemonic.as_str() {
                ".arm" | ".code32" => (0, false, Some(4)),
                ".thumb" | ".code16" => (0, true, Some(2)),
                ".code" => {
                    let bits = ctx.eval(&s.operands)?;
                    (0, bits == 16, Some(if bits == 16 { 2 } else { 4 }))
                }
                ".word" | ".long" | ".int" | ".4byte" => (4 * ops.len() as u32, thumb, None),
                ".hword" | ".short" | ".half" | ".2byte" => (2 * ops.len() as u32, thumb, None),
                ".byte" => (ops.len() as u32, thumb, None),
                ".ascii" | ".asciz" | ".string" => {
                    let mut size = 0;
                    for op in &ops {
                        size += parse_string(op)?.len() as u32;
                        if mnemonic != ".ascii" {
                            size += 1;
                        }
                    }
                    (size, thumb, None)
                }
                ".space" | ".skip" | ".zero" => {
                    let n = ctx.eval(ops.first().ok_or("missing size")?)?;
                    if n < 0 {
                        return Err("negative size".into());
                    }
                    (n as u32, thumb, None)
                }
                ".fill" => {
                    let n = ctx.eval(ops.first().ok_or("missing count")?)?;
                    let size = match ops.get(1) {
                        Some(s) => ctx.eval(s)?,
                        None => 1,
                    };
                    ((n.max(0) * size.clamp(0, 8)) as u32, thumb, None)
                }
                ".align" | ".p2align" => {
                    let n = match ops.first() {
                        Some(op) if !op.is_empty() => ctx.eval(op)?,
                        _ => 2,
                    };
                    (0, thumb, Some(1u32 << n.clamp(0, 12)))
                }
                ".balign" => {
                    let n = ctx.eval(ops.first().ok_or("missing alignment")?)?;
                    (0, thumb, Some(n.clamp(1, 4096) as u32))
                }
                ".org" => {
                    let target = ctx.eval(ops.first().ok_or("missing address")?)? as u32;
                    if target < here {
                        return Err(format!(".org {target:#x} moves backwards from {here:#x}"));
                    }
                    (target - here, thumb, None)
                }
                ".pool" | ".ltorg" => (0, thumb, Some(4)),
                ".equ" | ".set" | ".global" | ".globl" | ".thumb_func" | ".type" | ".size"
                | ".text" | ".data" | ".section" | ".func" | ".endfunc" | ".syntax" | ".cpu"
                | ".end" | ".extern" => (0, thumb, None),
                _ => return Err(format!("unknown directive `{mnemonic}`")),
            });
        }
        if thumb {
            let relaxed = self.relaxed.get(&index).copied().unwrap_or(false);
            let size = thumb::size(&mnemonic, &s.operands, relaxed)?;
            Ok((size, thumb, Some(2)))
        } else {
            Ok((4, thumb, Some(4)))
        }
    }

    fn layout(
        &mut self,
        known: &HashMap<String, Symbol>,
        out: &mut HashMap<String, Symbol>,
    ) -> Result<Layout, Diagnostic> {
        let previous_layout = Layout {
            addresses: self.previous.clone(),
            ..Default::default()
        };
        let locals = self.locals(&previous_layout);
        let mut layout = Layout::default();
        let mut here = self.options.base;
        let mut thumb = self.options.thumb;
        let mut thumb_func_pending = false;
        let mut pending_literals: Vec<usize> = Vec::new();
        for index in 0..self.statements.len() {
            let mnemonic = self.statements[index].mnemonic.to_ascii_lowercase();
            let is_literal = mnemonic_is_literal_load(&mnemonic, &self.statements[index].operands);
            let (size, new_thumb, align) = self
                .statement_size(index, thumb, here, known, &locals)
                .map_err(|message| self.diagnostic(index, message))?;
            // Alignment applies before the labels attached to the statement.
            if let Some(a) = align {
                here = here.div_ceil(a) * a;
            }
            if mnemonic == ".pool" || mnemonic == ".ltorg" {
                let entries = std::mem::take(&mut pending_literals);
                for &entry in &entries {
                    layout.literal_slots.insert(entry, here);
                    here += 4;
                }
                layout.pools.push((index, entries));
            }
            for label in &self.statements[index].labels {
                if label.parse::<u32>().is_ok() {
                    continue;
                }
                if out.contains_key(label)
                    && !self.options.symbols.iter().any(|(n, _)| n == label)
                    && out.get(label).is_some_and(|s| s.label)
                {
                    return Err(self.diagnostic(index, format!("label `{label}` is defined twice")));
                }
                out.insert(
                    label.clone(),
                    Symbol {
                        value: here as i64,
                        thumb_func: thumb && thumb_func_pending,
                        label: true,
                    },
                );
                thumb_func_pending = false;
            }
            if mnemonic == ".thumb_func" {
                thumb_func_pending = true;
            }
            if mnemonic == ".equ" || mnemonic == ".set" {
                let ops = split_operands(&self.statements[index].operands);
                if ops.len() != 2 {
                    return Err(self.diagnostic(index, "expected `.equ name, value`".into()));
                }
                let scope = Scope {
                    symbols: out,
                    here: here as i64,
                    locals: &locals,
                    statement: index,
                };
                let value = expr::parse(&ops[1])
                    .and_then(|e| e.eval(&scope))
                    .map_err(|m| self.diagnostic(index, m))?;
                let value = if value.known {
                    value.value
                } else {
                    let scope = Scope {
                        symbols: known,
                        here: here as i64,
                        locals: &locals,
                        statement: index,
                    };
                    expr::parse(&ops[1])
                        .and_then(|e| e.eval(&scope))
                        .map_err(|m| self.diagnostic(index, m))?
                        .value
                };
                out.insert(
                    ops[0].trim().to_string(),
                    Symbol {
                        value,
                        thumb_func: false,
                        label: false,
                    },
                );
            }
            layout.addresses.push(here);
            layout.sizes.push(size);
            layout.thumb.push(new_thumb);
            if is_literal {
                pending_literals.push(index);
            }
            here = here.wrapping_add(size);
            thumb = new_thumb;
        }
        if !pending_literals.is_empty() {
            here = here.div_ceil(4) * 4;
            for &entry in &pending_literals {
                layout.literal_slots.insert(entry, here);
                here += 4;
            }
            layout.pools.push((usize::MAX, pending_literals));
        }
        layout.addresses.push(here);
        Ok(layout)
    }

    fn relax(&mut self, symbols: &HashMap<String, Symbol>, layout: &Layout) -> bool {
        let locals = self.locals(layout);
        let mut changed = false;
        for index in 0..self.statements.len() {
            if !layout.thumb.get(index).copied().unwrap_or(false) {
                continue;
            }
            let s = &self.statements[index];
            let mnemonic = s.mnemonic.to_ascii_lowercase();
            if !thumb::is_conditional_branch(&mnemonic) {
                continue;
            }
            let here = layout.addresses[index];
            let scope = Scope {
                symbols,
                here: here as i64,
                locals: &locals,
                statement: index,
            };
            let Ok(target) = expr::parse(&s.operands).and_then(|e| e.eval(&scope)) else {
                continue;
            };
            if !target.known {
                continue;
            }
            let offset = target.value - (here as i64 + 4);
            let fits = (-256..=254).contains(&offset);
            if !fits && !self.relaxed.get(&index).copied().unwrap_or(false) {
                self.relaxed.insert(index, true);
                changed = true;
            }
        }
        changed
    }

    fn diagnostic(&self, index: usize, message: String) -> Diagnostic {
        let s = &self.statements[index];
        Diagnostic {
            file: s.file.clone(),
            line: s.line,
            message,
        }
    }

    fn emit(
        &mut self,
        symbols: &HashMap<String, Symbol>,
        layout: &Layout,
    ) -> Result<Assembly, Vec<Diagnostic>> {
        let base = self.options.base;
        let end = *layout.addresses.last().unwrap_or(&base);
        let mut bytes = vec![0u8; (end - base) as usize];
        let mut errors = Vec::new();
        let mut lines = Vec::new();
        let locals = self.locals(layout);
        let mut literal_values: HashMap<usize, u32> = HashMap::new();
        for index in 0..self.statements.len() {
            let here = layout.addresses[index];
            let size = layout.sizes[index];
            let thumb_before = if index == 0 {
                self.options.thumb
            } else {
                layout.thumb[index - 1]
            };
            let s = self.statements[index].clone();
            let mnemonic = s.mnemonic.to_ascii_lowercase();
            if mnemonic.is_empty() {
                continue;
            }
            let scope = Scope {
                symbols,
                here: here as i64,
                locals: &locals,
                statement: index,
            };
            let ctx = Context {
                scope,
                thumb_funcs: symbols,
                final_pass: true,
                literal: layout.literal_slots.get(&index).copied(),
            };
            let offset = (here - base) as usize;
            let result: Result<Vec<u8>, String> = if mnemonic.starts_with('.') {
                self.directive(&mnemonic, &s.operands, &ctx, size)
            } else if thumb_before {
                let relaxed = self.relaxed.get(&index).copied().unwrap_or(false);
                thumb::encode(&mnemonic, &s.operands, &ctx, relaxed).map(|(code, literal)| {
                    if let Some(v) = literal {
                        literal_values.insert(index, v);
                    }
                    code
                })
            } else {
                arm::encode(&mnemonic, &s.operands, &ctx).map(|(word, literal)| {
                    if let Some(v) = literal {
                        literal_values.insert(index, v);
                    }
                    word.to_le_bytes().to_vec()
                })
            };
            match result {
                Ok(code) => {
                    if code.len() as u32 != size
                        && !(mnemonic == ".org" || mnemonic.contains("align"))
                    {
                        errors.push(self.diagnostic(
                            index,
                            format!("internal size mismatch ({} != {size})", code.len()),
                        ));
                        continue;
                    }
                    bytes[offset..offset + code.len()].copy_from_slice(&code);
                    if size > 0 {
                        lines.push(LineInfo {
                            address: here,
                            file: s.file.clone(),
                            line: s.line,
                            thumb: thumb_before && !mnemonic.starts_with('.'),
                        });
                    }
                }
                Err(message) => errors.push(self.diagnostic(index, message)),
            }
        }
        for (index, value) in &literal_values {
            if let Some(slot) = layout.literal_slots.get(index) {
                let at = (slot - base) as usize;
                bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        let mut out_symbols = BTreeMap::new();
        let mut labels = BTreeMap::new();
        for (name, symbol) in symbols {
            if name.starts_with('@') {
                continue;
            }
            let value = symbol.value as u32 | symbol.thumb_func as u32;
            out_symbols.insert(name.clone(), value);
            if symbol.label {
                labels.insert(name.clone(), symbol.value as u32);
            }
        }
        Ok(Assembly {
            base,
            bytes,
            symbols: out_symbols,
            labels,
            lines,
        })
    }

    /// Data and control directives:
    /// `.arm .thumb .code .word .hword .byte .ascii .asciz .space .fill .align
    /// .balign .org .pool .ltorg .equ .set .thumb_func .include` (and ignored
    /// section/type/global markers).
    fn directive(
        &self,
        mnemonic: &str,
        operands: &str,
        ctx: &Context,
        size: u32,
    ) -> Result<Vec<u8>, String> {
        let ops = split_operands(operands);
        let mut out = Vec::new();
        match mnemonic {
            ".word" | ".long" | ".int" | ".4byte" => {
                for op in &ops {
                    out.extend_from_slice(&(ctx.eval_code_address(op)? as u32).to_le_bytes());
                }
            }
            ".hword" | ".short" | ".half" | ".2byte" => {
                for op in &ops {
                    let v = ctx.eval(op)?;
                    if !(-0x8000..=0xFFFF).contains(&v) {
                        return Err(format!("value {v} does not fit in 16 bits"));
                    }
                    out.extend_from_slice(&(v as u16).to_le_bytes());
                }
            }
            ".byte" => {
                for op in &ops {
                    let v = ctx.eval(op)?;
                    if !(-0x80..=0xFF).contains(&v) {
                        return Err(format!("value {v} does not fit in 8 bits"));
                    }
                    out.push(v as u8);
                }
            }
            ".ascii" | ".asciz" | ".string" => {
                for op in &ops {
                    out.extend(parse_string(op)?);
                    if mnemonic != ".ascii" {
                        out.push(0);
                    }
                }
            }
            ".space" | ".skip" | ".zero" => {
                let fill = match ops.get(1) {
                    Some(f) => ctx.eval(f)? as u8,
                    None => 0,
                };
                out.resize(size as usize, fill);
            }
            ".fill" => {
                let n = ctx.eval(&ops[0])?.max(0) as usize;
                let width = match ops.get(1) {
                    Some(s) => ctx.eval(s)?.clamp(0, 8) as usize,
                    None => 1,
                };
                let value = match ops.get(2) {
                    Some(v) => ctx.eval(v)?,
                    None => 0,
                };
                for _ in 0..n {
                    out.extend_from_slice(&value.to_le_bytes()[..width]);
                }
            }
            ".org" => out.resize(size as usize, 0),
            _ => {}
        }
        Ok(out)
    }
}

fn symbols_equal(a: &HashMap<String, Symbol>, b: &HashMap<String, Symbol>) -> bool {
    a.len() == b.len()
        && a.iter().all(|(k, v)| {
            b.get(k)
                .is_some_and(|w| w.value == v.value && w.thumb_func == v.thumb_func)
        })
}

fn mnemonic_is_literal_load(mnemonic: &str, operands: &str) -> bool {
    mnemonic.starts_with("ldr")
        && split_operands(operands)
            .get(1)
            .is_some_and(|op| op.trim().starts_with('='))
}

#[cfg(test)]
mod tests;
