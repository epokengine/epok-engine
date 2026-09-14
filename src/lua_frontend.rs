//! `epok-lua` v1 frontend: lexer, recursive-descent parser, profile enforcement,
//! scopes and type inference. It produces `script_ir`, never source text, and its
//! diagnostics describe the language profile only - never an execution backend.
#![allow(dead_code)] // M2 (`lua_compile`) drives this frontend; M1 only builds and tests it.
use crate::{
    blueprint::Registry,
    lua_asset::{Declaration, Diagnostic, LuaFile},
    reflection_schema::{self as schema, Type},
    script_ir::{self as ir, Span},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Profile diagnostics. The text never mentions an execution mode: the same
/// source is rejected identically whichever backend the project selects.
pub mod profile {
    pub const VARARGS: &str = "Varargs are not supported by the epok-lua profile";
    pub const CLOSURE: &str =
        "Anonymous functions and closures are not supported by the epok-lua profile";
    pub const NESTED_FUNCTION: &str =
        "Nested function definitions are not supported by the epok-lua profile";
    pub const MULTI_ASSIGN: &str = "Multiple assignment is not supported by the epok-lua profile";
    pub const MULTI_RETURN: &str =
        "Multiple return values are not supported by the epok-lua profile";
    pub const CONCAT: &str = "String concatenation is not supported by the epok-lua profile";
    pub const LENGTH: &str = "The length operator is not supported by the epok-lua profile";
    pub const POWER: &str = "The power operator is not supported by the epok-lua profile";
    pub const FLOOR_DIV: &str = "Floor division is not supported by the epok-lua profile";
    pub const MOD_TYPE: &str = "The modulo operator requires Int32 or UInt32 operands";
    pub const WHILE: &str =
        "while loops are not supported by the epok-lua profile; use a constant-bounded numeric for";
    pub const REPEAT: &str = "repeat loops are not supported by the epok-lua profile; use a constant-bounded numeric for";
    pub const GENERIC_FOR: &str = "Generic for loops are not supported by the epok-lua profile";
    pub const BREAK: &str = "break is not supported by the epok-lua profile";
    pub const GOTO: &str = "goto and labels are not supported by the epok-lua profile";
    pub const METATABLE: &str = "Metatables are not supported by the epok-lua profile";
    pub const DYNAMIC_LOAD: &str =
        "load, loadstring, dofile and require are not supported by the epok-lua profile";
    pub const STRING_VALUE: &str = "String values are not supported by the epok-lua profile";
    pub const NIL_VALUE: &str = "nil is not supported by the epok-lua profile";
    pub const TABLE_BODY: &str = "Table constructors are not supported by the epok-lua profile";
    pub const INDEXING: &str = "Indexed access is not supported by the epok-lua profile";
    pub const LOGICAL_TYPE: &str = "and/or require Bool operands";
    pub const CONDITION_TYPE: &str = "Conditions must be Bool";
    pub const RECURSION: &str = "Recursive calls are not supported by the epok-lua profile";
    pub const FOR_BOUNDS: &str = "Numeric for bounds must be constant integers";
    pub const FOR_BUDGET: &str = "Numeric for exceeds the 65536 iteration limit";
    pub const DEPTH: &str = "Nesting depth exceeds the 32 level limit";
    pub const REFERENCE_VALUE: &str = "Asset and class references are not readable or writable from a body in the epok-lua profile";
    pub const VECTOR_VALUE: &str = "Whole vector values are not supported by the epok-lua profile; use the .x, .y and .z components";
    pub const AMBIGUOUS_LITERAL: &str =
        "Numeric literal has no contextual type; annotate the target or use an explicit conversion";
}
pub const MAX_DEPTH: usize = 32;
pub const MAX_ITERATIONS: i64 = 65536;

const BANNED_GLOBALS: &[(&str, &str)] = &[
    ("load", profile::DYNAMIC_LOAD),
    ("loadstring", profile::DYNAMIC_LOAD),
    ("dofile", profile::DYNAMIC_LOAD),
    ("loadfile", profile::DYNAMIC_LOAD),
    ("require", profile::DYNAMIC_LOAD),
    ("setmetatable", profile::METATABLE),
    ("getmetatable", profile::METATABLE),
    ("rawget", profile::METATABLE),
    ("rawset", profile::METATABLE),
    ("rawequal", profile::METATABLE),
];

// ---------------------------------------------------------------- lexer ----

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Name(String),
    Number { value: f64, integer: bool },
    Str(String),
    Keyword(&'static str),
    Sym(&'static str),
    Eof,
}
#[derive(Clone, Debug)]
struct Token {
    tok: Tok,
    span: Span,
}
const KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];
/// Longest first: the scanner takes the first match.
const SYMBOLS: &[&str] = &[
    "...", "..", "::", "==", "~=", "<=", ">=", "//", "+", "-", "*", "/", "%", "^", "#", "<", ">",
    "=", "(", ")", "{", "}", "[", "]", ";", ":", ",", ".",
];

struct Lexer<'a> {
    bytes: &'a [u8],
    index: usize,
    line: u32,
    column: u32,
    file: &'a Path,
}
impl<'a> Lexer<'a> {
    fn new(file: &'a Path, source: &'a str) -> Self {
        Self {
            bytes: source.as_bytes(),
            index: 0,
            line: 1,
            column: 1,
            file,
        }
    }
    fn span(&self) -> Span {
        Span {
            line: self.line,
            column: self.column,
        }
    }
    fn error(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.file, span, message)
    }
    fn peek(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.index + offset).copied()
    }
    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek(0)?;
        self.index += 1;
        if byte == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(byte)
    }
    fn starts_with(&self, text: &str) -> bool {
        self.bytes[self.index..].starts_with(text.as_bytes())
    }
    /// `[[ ... ]]` with any number of `=` signs, shared by long strings and
    /// long comments. Returns the contents.
    fn long_bracket(&mut self) -> Option<Result<String, Diagnostic>> {
        let span = self.span();
        if self.peek(0) != Some(b'[') {
            return None;
        }
        let mut level = 0;
        while self.peek(1 + level) == Some(b'=') {
            level += 1;
        }
        if self.peek(1 + level) != Some(b'[') {
            return None;
        }
        for _ in 0..level + 2 {
            self.bump();
        }
        let close = format!("]{}]", "=".repeat(level));
        let start = self.index;
        while !self.starts_with(&close) {
            if self.bump().is_none() {
                return Some(Err(self.error(span, "Unterminated long bracket")));
            }
        }
        let text = String::from_utf8_lossy(&self.bytes[start..self.index]).into_owned();
        for _ in 0..close.len() {
            self.bump();
        }
        Some(Ok(text))
    }
    fn skip_trivia(&mut self) -> Result<(), Diagnostic> {
        loop {
            match self.peek(0) {
                Some(b' ' | b'\t' | b'\r' | b'\n') => {
                    self.bump();
                }
                Some(b'-') if self.peek(1) == Some(b'-') => {
                    self.bump();
                    self.bump();
                    if let Some(result) = self.long_bracket() {
                        result?;
                        continue;
                    }
                    while !matches!(self.peek(0), None | Some(b'\n')) {
                        self.bump();
                    }
                }
                _ => return Ok(()),
            }
        }
    }
    fn string(&mut self, quote: u8) -> Result<Token, Diagnostic> {
        let span = self.span();
        self.bump();
        let mut text = String::new();
        loop {
            match self.bump() {
                None | Some(b'\n') => return Err(self.error(span, "Unterminated string")),
                Some(byte) if byte == quote => break,
                Some(b'\\') => {
                    let escape = self
                        .bump()
                        .ok_or_else(|| self.error(span, "Unterminated string"))?;
                    text.push(match escape {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'0' => '\0',
                        b'\\' => '\\',
                        b'"' => '"',
                        b'\'' => '\'',
                        other => {
                            return Err(self.error(
                                span,
                                format!("Unsupported string escape \\{}", other as char),
                            ));
                        }
                    });
                }
                Some(byte) => text.push(byte as char),
            }
        }
        Ok(Token {
            tok: Tok::Str(text),
            span,
        })
    }
    fn number(&mut self) -> Result<Token, Diagnostic> {
        let span = self.span();
        let start = self.index;
        if self.peek(0) == Some(b'0') && matches!(self.peek(1), Some(b'x' | b'X')) {
            self.bump();
            self.bump();
            let digits = self.index;
            while self.peek(0).is_some_and(|b| b.is_ascii_hexdigit()) {
                self.bump();
            }
            if self.index == digits {
                return Err(self.error(span, "Malformed hexadecimal literal"));
            }
            let text = String::from_utf8_lossy(&self.bytes[digits..self.index]).into_owned();
            let value = u64::from_str_radix(&text, 16)
                .map_err(|_| self.error(span, "Hexadecimal literal is out of range"))?;
            return Ok(Token {
                tok: Tok::Number {
                    value: value as f64,
                    integer: true,
                },
                span,
            });
        }
        while self.peek(0).is_some_and(|b| b.is_ascii_digit()) {
            self.bump();
        }
        let mut integer = true;
        if self.peek(0) == Some(b'.') && self.peek(1).is_some_and(|b| b.is_ascii_digit()) {
            integer = false;
            self.bump();
            while self.peek(0).is_some_and(|b| b.is_ascii_digit()) {
                self.bump();
            }
        }
        let text = String::from_utf8_lossy(&self.bytes[start..self.index]).into_owned();
        let value = text
            .parse::<f64>()
            .map_err(|_| self.error(span, "Malformed numeric literal"))?;
        Ok(Token {
            tok: Tok::Number { value, integer },
            span,
        })
    }
    fn tokens(mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut out = vec![];
        loop {
            self.skip_trivia()?;
            let span = self.span();
            let Some(byte) = self.peek(0) else {
                out.push(Token {
                    tok: Tok::Eof,
                    span,
                });
                return Ok(out);
            };
            if byte == b'['
                && matches!(self.peek(1), Some(b'[' | b'='))
                && let Some(result) = self.long_bracket()
            {
                out.push(Token {
                    tok: Tok::Str(result?),
                    span,
                });
                continue;
            }
            if byte == b'"' || byte == b'\'' {
                out.push(self.string(byte)?);
                continue;
            }
            if byte.is_ascii_digit() {
                out.push(self.number()?);
                continue;
            }
            if byte.is_ascii_alphabetic() || byte == b'_' {
                let start = self.index;
                while self
                    .peek(0)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    self.bump();
                }
                let text = String::from_utf8_lossy(&self.bytes[start..self.index]).into_owned();
                let tok = match KEYWORDS.iter().find(|k| **k == text) {
                    Some(keyword) => Tok::Keyword(keyword),
                    None => Tok::Name(text),
                };
                out.push(Token { tok, span });
                continue;
            }
            match SYMBOLS.iter().find(|s| self.starts_with(s)) {
                Some(symbol) => {
                    for _ in 0..symbol.len() {
                        self.bump();
                    }
                    out.push(Token {
                        tok: Tok::Sym(symbol),
                        span,
                    });
                }
                None => {
                    return Err(self.error(span, format!("Unexpected character {}", byte as char)));
                }
            }
        }
    }
}

// ------------------------------------------------------------------ ast ----

pub mod ast {
    use crate::script_ir::Span;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BinOp {
        Add,
        Sub,
        Mul,
        Div,
        Mod,
        Eq,
        Ne,
        Lt,
        Le,
        Gt,
        Ge,
        And,
        Or,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum UnOp {
        Neg,
        Not,
    }
    #[derive(Clone, Debug)]
    pub struct Field {
        pub key: Option<String>,
        pub value: Expr,
        pub span: Span,
    }
    #[derive(Clone, Debug)]
    pub enum Expr {
        Nil(Span),
        Bool(bool, Span),
        Number {
            value: f64,
            integer: bool,
            span: Span,
        },
        Str {
            value: String,
            span: Span,
        },
        Name {
            name: String,
            span: Span,
        },
        /// `base.name`
        Field {
            base: Box<Expr>,
            name: String,
            span: Span,
        },
        Call {
            base: Box<Expr>,
            args: Vec<Expr>,
            span: Span,
        },
        /// `base:name(args)`
        MethodCall {
            base: Box<Expr>,
            name: String,
            args: Vec<Expr>,
            span: Span,
        },
        Table {
            fields: Vec<Field>,
            span: Span,
        },
        Unary {
            op: UnOp,
            operand: Box<Expr>,
            span: Span,
        },
        Binary {
            op: BinOp,
            left: Box<Expr>,
            right: Box<Expr>,
            span: Span,
        },
    }
    impl Expr {
        pub fn span(&self) -> Span {
            match self {
                Self::Nil(span) | Self::Bool(_, span) => *span,
                Self::Number { span, .. }
                | Self::Str { span, .. }
                | Self::Name { span, .. }
                | Self::Field { span, .. }
                | Self::Call { span, .. }
                | Self::MethodCall { span, .. }
                | Self::Table { span, .. }
                | Self::Unary { span, .. }
                | Self::Binary { span, .. } => *span,
            }
        }
    }
    pub type Block = Vec<Stat>;
    #[derive(Clone, Debug)]
    pub enum Stat {
        Local {
            name: String,
            value: Option<Expr>,
            span: Span,
        },
        Assign {
            target: Expr,
            value: Expr,
            span: Span,
        },
        If {
            arms: Vec<(Expr, Block)>,
            otherwise: Option<Block>,
            span: Span,
        },
        NumericFor {
            var: String,
            start: Expr,
            limit: Expr,
            step: Option<Expr>,
            body: Block,
            span: Span,
        },
        Return {
            value: Option<Expr>,
            span: Span,
        },
        Call {
            call: Expr,
            span: Span,
        },
        Do {
            body: Block,
            span: Span,
        },
    }
    impl Stat {
        pub fn span(&self) -> Span {
            match self {
                Self::Local { span, .. }
                | Self::Assign { span, .. }
                | Self::If { span, .. }
                | Self::NumericFor { span, .. }
                | Self::Return { span, .. }
                | Self::Call { span, .. }
                | Self::Do { span, .. } => *span,
            }
        }
    }
    /// `function <object>:<name>(<parameters>) ... end`
    #[derive(Clone, Debug)]
    pub struct Method {
        pub object: String,
        pub name: String,
        pub parameters: Vec<(String, Span)>,
        pub body: Block,
        pub span: Span,
    }
    /// A whole `.lua` file in the profile's fixed shape.
    #[derive(Clone, Debug)]
    pub struct Chunk {
        /// Top-level `local <name> = <value>` bindings, in source order.
        pub locals: Vec<(String, Expr, Span)>,
        pub methods: Vec<Method>,
        pub returns: Option<(String, Span)>,
    }
}
use ast::{BinOp, UnOp};

// --------------------------------------------------------------- parser ----

struct Parser<'a> {
    tokens: Vec<Token>,
    index: usize,
    file: &'a Path,
    depth: usize,
    /// Method bodies reject every construct the metadata table is allowed to use.
    in_body: bool,
}
impl<'a> Parser<'a> {
    fn error(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.file, span, message)
    }
    fn peek(&self) -> &Tok {
        &self.tokens[self.index.min(self.tokens.len() - 1)].tok
    }
    fn span(&self) -> Span {
        self.tokens[self.index.min(self.tokens.len() - 1)].span
    }
    fn advance(&mut self) -> Token {
        let token = self.tokens[self.index.min(self.tokens.len() - 1)].clone();
        self.index = (self.index + 1).min(self.tokens.len() - 1);
        token
    }
    fn at_sym(&self, symbol: &str) -> bool {
        matches!(self.peek(), Tok::Sym(s) if *s == symbol)
    }
    fn at_keyword(&self, keyword: &str) -> bool {
        matches!(self.peek(), Tok::Keyword(k) if *k == keyword)
    }
    fn eat_sym(&mut self, symbol: &str) -> bool {
        let hit = self.at_sym(symbol);
        if hit {
            self.advance();
        }
        hit
    }
    fn eat_keyword(&mut self, keyword: &str) -> bool {
        let hit = self.at_keyword(keyword);
        if hit {
            self.advance();
        }
        hit
    }
    fn expect_sym(&mut self, symbol: &str) -> Result<Span, Diagnostic> {
        let span = self.span();
        if self.eat_sym(symbol) {
            Ok(span)
        } else {
            Err(self.error(span, format!("Expected `{symbol}`")))
        }
    }
    fn expect_keyword(&mut self, keyword: &str) -> Result<Span, Diagnostic> {
        let span = self.span();
        if self.eat_keyword(keyword) {
            Ok(span)
        } else {
            Err(self.error(span, format!("Expected `{keyword}`")))
        }
    }
    fn expect_name(&mut self) -> Result<(String, Span), Diagnostic> {
        let span = self.span();
        match self.advance().tok {
            Tok::Name(name) => Ok((name, span)),
            _ => Err(self.error(span, "Expected a name")),
        }
    }

    fn chunk(&mut self) -> Result<ast::Chunk, Diagnostic> {
        let mut chunk = ast::Chunk {
            locals: vec![],
            methods: vec![],
            returns: None,
        };
        loop {
            if self.eat_sym(";") {
                continue;
            }
            let span = self.span();
            match self.peek().clone() {
                Tok::Eof => break,
                Tok::Keyword("local") => {
                    self.advance();
                    if self.at_keyword("function") {
                        return Err(self.error(span, profile::CLOSURE));
                    }
                    let (name, _) = self.expect_name()?;
                    if self.at_sym(",") {
                        return Err(self.error(span, profile::MULTI_ASSIGN));
                    }
                    self.expect_sym("=")?;
                    let value = self.expression(0)?;
                    if self.at_sym(",") {
                        return Err(self.error(span, profile::MULTI_ASSIGN));
                    }
                    chunk.locals.push((name, value, span));
                }
                Tok::Keyword("function") => chunk.methods.push(self.method()?),
                Tok::Keyword("return") => {
                    self.advance();
                    let (name, _) = self.expect_name()?;
                    if self.at_sym(",") {
                        return Err(self.error(span, profile::MULTI_RETURN));
                    }
                    self.eat_sym(";");
                    chunk.returns = Some((name, span));
                    if !matches!(self.peek(), Tok::Eof) {
                        return Err(self.error(self.span(), "`return` must be the last statement"));
                    }
                    break;
                }
                _ => {
                    return Err(self.error(
                        span,
                        "Only `local <Class> = epok.class{...}`, method definitions and a final `return` are allowed at file scope",
                    ));
                }
            }
        }
        Ok(chunk)
    }

    fn method(&mut self) -> Result<ast::Method, Diagnostic> {
        let span = self.expect_keyword("function")?;
        let (object, _) = self.expect_name()?;
        if self.eat_sym(".") {
            let (name, dot) = self.expect_name()?;
            return Err(self.error(
                dot,
                format!(
                    "`function {object}.{name}` declares a static function; instance methods must be declared as `function {object}:{name}`"
                ),
            ));
        }
        self.expect_sym(":")?;
        let (name, _) = self.expect_name()?;
        self.expect_sym("(")?;
        let mut parameters = vec![];
        if !self.at_sym(")") {
            loop {
                let parameter = self.span();
                if self.at_sym("...") {
                    return Err(self.error(parameter, profile::VARARGS));
                }
                let (name, _) = self.expect_name()?;
                parameters.push((name, parameter));
                if !self.eat_sym(",") {
                    break;
                }
            }
        }
        self.expect_sym(")")?;
        self.in_body = true;
        let body = self.block()?;
        self.in_body = false;
        self.expect_keyword("end")?;
        Ok(ast::Method {
            object,
            name,
            parameters,
            body,
            span,
        })
    }

    fn block(&mut self) -> Result<ast::Block, Diagnostic> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.error(self.span(), profile::DEPTH));
        }
        let mut out = vec![];
        loop {
            if self.eat_sym(";") {
                continue;
            }
            if matches!(
                self.peek(),
                Tok::Eof | Tok::Keyword("end" | "else" | "elseif" | "until")
            ) {
                break;
            }
            out.push(self.statement()?);
            if matches!(out.last(), Some(ast::Stat::Return { .. })) {
                break;
            }
        }
        self.depth -= 1;
        Ok(out)
    }

    fn statement(&mut self) -> Result<ast::Stat, Diagnostic> {
        let span = self.span();
        match self.peek().clone() {
            Tok::Keyword("while") => Err(self.error(span, profile::WHILE)),
            Tok::Keyword("repeat") => Err(self.error(span, profile::REPEAT)),
            Tok::Keyword("goto") => Err(self.error(span, profile::GOTO)),
            Tok::Keyword("break") => Err(self.error(span, profile::BREAK)),
            Tok::Sym("::") => Err(self.error(span, profile::GOTO)),
            Tok::Keyword("function") => Err(self.error(span, profile::NESTED_FUNCTION)),
            Tok::Keyword("local") => {
                self.advance();
                if self.at_keyword("function") {
                    return Err(self.error(span, profile::NESTED_FUNCTION));
                }
                let (name, _) = self.expect_name()?;
                if self.at_sym(",") {
                    return Err(self.error(span, profile::MULTI_ASSIGN));
                }
                let value = if self.eat_sym("=") {
                    let value = self.expression(0)?;
                    if self.at_sym(",") {
                        return Err(self.error(span, profile::MULTI_ASSIGN));
                    }
                    Some(value)
                } else {
                    None
                };
                Ok(ast::Stat::Local { name, value, span })
            }
            Tok::Keyword("do") => {
                self.advance();
                let body = self.block()?;
                self.expect_keyword("end")?;
                Ok(ast::Stat::Do { body, span })
            }
            Tok::Keyword("if") => {
                self.advance();
                let mut arms = vec![];
                let condition = self.expression(0)?;
                self.expect_keyword("then")?;
                arms.push((condition, self.block()?));
                let mut otherwise = None;
                loop {
                    if self.eat_keyword("elseif") {
                        let condition = self.expression(0)?;
                        self.expect_keyword("then")?;
                        arms.push((condition, self.block()?));
                        continue;
                    }
                    if self.eat_keyword("else") {
                        otherwise = Some(self.block()?);
                    }
                    break;
                }
                self.expect_keyword("end")?;
                Ok(ast::Stat::If {
                    arms,
                    otherwise,
                    span,
                })
            }
            Tok::Keyword("for") => {
                self.advance();
                let (var, _) = self.expect_name()?;
                if self.at_sym(",") || self.at_keyword("in") {
                    return Err(self.error(span, profile::GENERIC_FOR));
                }
                self.expect_sym("=")?;
                let start = self.expression(0)?;
                self.expect_sym(",")?;
                let limit = self.expression(0)?;
                let step = if self.eat_sym(",") {
                    Some(self.expression(0)?)
                } else {
                    None
                };
                self.expect_keyword("do")?;
                let body = self.block()?;
                self.expect_keyword("end")?;
                Ok(ast::Stat::NumericFor {
                    var,
                    start,
                    limit,
                    step,
                    body,
                    span,
                })
            }
            Tok::Keyword("return") => {
                self.advance();
                let value = if matches!(
                    self.peek(),
                    Tok::Eof | Tok::Keyword("end" | "else" | "elseif" | "until")
                ) || self.at_sym(";")
                {
                    None
                } else {
                    Some(self.expression(0)?)
                };
                if self.at_sym(",") {
                    return Err(self.error(span, profile::MULTI_RETURN));
                }
                self.eat_sym(";");
                Ok(ast::Stat::Return { value, span })
            }
            _ => {
                let target = self.suffixed()?;
                if self.at_sym(",") {
                    return Err(self.error(span, profile::MULTI_ASSIGN));
                }
                if self.eat_sym("=") {
                    let value = self.expression(0)?;
                    if self.at_sym(",") {
                        return Err(self.error(span, profile::MULTI_ASSIGN));
                    }
                    return Ok(ast::Stat::Assign {
                        target,
                        value,
                        span,
                    });
                }
                if !matches!(
                    target,
                    ast::Expr::Call { .. } | ast::Expr::MethodCall { .. }
                ) {
                    return Err(self.error(span, "Expected a statement"));
                }
                Ok(ast::Stat::Call { call: target, span })
            }
        }
    }

    /// `(left binding power, right binding power)`, in Lua 5.2 order.
    fn binary_op(&self) -> Option<(BinOp, u8, u8)> {
        let (op, power) = match self.peek() {
            Tok::Keyword("or") => (BinOp::Or, 1),
            Tok::Keyword("and") => (BinOp::And, 2),
            Tok::Sym("<") => (BinOp::Lt, 3),
            Tok::Sym(">") => (BinOp::Gt, 3),
            Tok::Sym("<=") => (BinOp::Le, 3),
            Tok::Sym(">=") => (BinOp::Ge, 3),
            Tok::Sym("~=") => (BinOp::Ne, 3),
            Tok::Sym("==") => (BinOp::Eq, 3),
            Tok::Sym("+") => (BinOp::Add, 5),
            Tok::Sym("-") => (BinOp::Sub, 5),
            Tok::Sym("*") => (BinOp::Mul, 6),
            Tok::Sym("/") => (BinOp::Div, 6),
            Tok::Sym("%") => (BinOp::Mod, 6),
            _ => return None,
        };
        Some((op, power, power + 1))
    }
    fn expression(&mut self, limit: u8) -> Result<ast::Expr, Diagnostic> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.error(self.span(), profile::DEPTH));
        }
        let span = self.span();
        let mut left = if self.at_sym("-") {
            self.advance();
            ast::Expr::Unary {
                op: UnOp::Neg,
                operand: Box::new(self.expression(7)?),
                span,
            }
        } else if self.at_keyword("not") {
            self.advance();
            ast::Expr::Unary {
                op: UnOp::Not,
                operand: Box::new(self.expression(7)?),
                span,
            }
        } else if self.at_sym("#") {
            return Err(self.error(span, profile::LENGTH));
        } else {
            self.simple()?
        };
        loop {
            if self.at_sym("..") {
                return Err(self.error(self.span(), profile::CONCAT));
            }
            if self.at_sym("^") {
                return Err(self.error(self.span(), profile::POWER));
            }
            if self.at_sym("//") {
                return Err(self.error(self.span(), profile::FLOOR_DIV));
            }
            let Some((op, left_power, right_power)) = self.binary_op() else {
                break;
            };
            if left_power <= limit {
                break;
            }
            let operator = self.span();
            self.advance();
            let right = self.expression(right_power - 1)?;
            left = ast::Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: operator,
            };
        }
        self.depth -= 1;
        Ok(left)
    }
    fn simple(&mut self) -> Result<ast::Expr, Diagnostic> {
        let span = self.span();
        match self.peek().clone() {
            Tok::Keyword("nil") => {
                if self.in_body {
                    return Err(self.error(span, profile::NIL_VALUE));
                }
                self.advance();
                Ok(ast::Expr::Nil(span))
            }
            Tok::Keyword("true") | Tok::Keyword("false") => {
                let value = self.at_keyword("true");
                self.advance();
                Ok(ast::Expr::Bool(value, span))
            }
            Tok::Keyword("function") => Err(self.error(span, profile::CLOSURE)),
            Tok::Sym("...") => Err(self.error(span, profile::VARARGS)),
            Tok::Number { value, integer } => {
                self.advance();
                Ok(ast::Expr::Number {
                    value,
                    integer,
                    span,
                })
            }
            Tok::Str(value) => {
                if self.in_body {
                    return Err(self.error(span, profile::STRING_VALUE));
                }
                self.advance();
                Ok(ast::Expr::Str { value, span })
            }
            Tok::Sym("{") => {
                if self.in_body {
                    return Err(self.error(span, profile::TABLE_BODY));
                }
                self.table()
            }
            _ => self.suffixed(),
        }
    }
    fn table(&mut self) -> Result<ast::Expr, Diagnostic> {
        let span = self.expect_sym("{")?;
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.error(span, profile::DEPTH));
        }
        let mut fields = vec![];
        while !self.at_sym("}") {
            let field = self.span();
            let key = match (self.peek().clone(), &self.tokens[self.index + 1].tok) {
                (Tok::Name(name), Tok::Sym("=")) => {
                    self.advance();
                    self.advance();
                    Some(name)
                }
                _ => None,
            };
            fields.push(ast::Field {
                key,
                value: self.expression(0)?,
                span: field,
            });
            if !self.eat_sym(",") && !self.eat_sym(";") {
                break;
            }
        }
        self.expect_sym("}")?;
        self.depth -= 1;
        Ok(ast::Expr::Table { fields, span })
    }
    fn suffixed(&mut self) -> Result<ast::Expr, Diagnostic> {
        let span = self.span();
        let mut base = match self.peek().clone() {
            Tok::Name(name) => {
                if let Some((_, message)) = BANNED_GLOBALS.iter().find(|(g, _)| *g == name) {
                    return Err(self.error(span, *message));
                }
                self.advance();
                ast::Expr::Name { name, span }
            }
            Tok::Sym("(") => {
                self.advance();
                let inner = self.expression(0)?;
                self.expect_sym(")")?;
                inner
            }
            _ => return Err(self.error(span, "Expected an expression")),
        };
        loop {
            let suffix = self.span();
            if self.eat_sym(".") {
                let (name, _) = self.expect_name()?;
                base = ast::Expr::Field {
                    base: Box::new(base),
                    name,
                    span: suffix,
                };
                continue;
            }
            if self.at_sym("[") {
                return Err(self.error(suffix, profile::INDEXING));
            }
            if self.eat_sym(":") {
                let (name, _) = self.expect_name()?;
                let args = self.arguments()?;
                base = ast::Expr::MethodCall {
                    base: Box::new(base),
                    name,
                    args,
                    span: suffix,
                };
                continue;
            }
            if self.at_sym("(") || self.at_sym("{") || matches!(self.peek(), Tok::Str(_)) {
                let args = self.arguments()?;
                base = ast::Expr::Call {
                    base: Box::new(base),
                    args,
                    span: suffix,
                };
                continue;
            }
            break;
        }
        Ok(base)
    }
    fn arguments(&mut self) -> Result<Vec<ast::Expr>, Diagnostic> {
        let span = self.span();
        if self.at_sym("{") {
            if self.in_body {
                return Err(self.error(span, profile::TABLE_BODY));
            }
            return Ok(vec![self.table()?]);
        }
        if let Tok::Str(value) = self.peek().clone() {
            if self.in_body {
                return Err(self.error(span, profile::STRING_VALUE));
            }
            self.advance();
            return Ok(vec![ast::Expr::Str { value, span }]);
        }
        self.expect_sym("(")?;
        let mut args = vec![];
        if !self.at_sym(")") {
            loop {
                args.push(self.expression(0)?);
                if !self.eat_sym(",") {
                    break;
                }
            }
        }
        self.expect_sym(")")?;
        Ok(args)
    }
}

pub fn parse(file: &LuaFile) -> Result<ast::Chunk, Diagnostic> {
    let tokens = Lexer::new(&file.path, &file.source).tokens()?;
    let mut parser = Parser {
        tokens,
        index: 0,
        file: &file.path,
        depth: 0,
        in_body: false,
    };
    parser.chunk()
}

// ------------------------------------------------------- semantic pass ----

/// Inferred type (fixed by the first assignment) and definite-assignment flag.
type State = (Option<Type>, bool);
struct Method<'a> {
    declared: &'a ast::Method,
    function: schema::Function,
    is_override: bool,
}
struct Lower<'a> {
    file: &'a Path,
    class: &'a schema::Class,
    registry: &'a Registry,
    /// Every reachable property, by authored name, with its reflected member id.
    properties: BTreeMap<String, schema::Property>,
    /// Every method callable on `self`, by name.
    methods: BTreeMap<String, schema::Function>,
    /// Every parent method reachable through `epok.super`, by name.
    parent_methods: BTreeMap<String, schema::Function>,
    /// Method names declared by this chunk; edges between them detect recursion.
    own: BTreeSet<String>,
    diagnostics: Vec<Diagnostic>,
    // per-method state
    locals: Vec<ir::Local>,
    slots: Vec<BTreeMap<String, ir::LocalId>>,
    states: BTreeMap<ir::LocalId, State>,
    current: String,
    current_returns: Type,
    calls: BTreeSet<(String, String)>,
}

/// Name-keyed view of everything the parent hierarchy exposes, with inherited
/// exposure flags folded in exactly as `blueprint::Registry::normalize_functions`
/// does for native overrides.
fn inherited_by_name(registry: &Registry, parent: &str) -> BTreeMap<String, schema::Function> {
    let mut by_id = BTreeMap::<String, schema::Function>::new();
    let mut by_name = BTreeMap::new();
    for ancestor in registry.ancestry(parent) {
        for original in &ancestor.functions {
            let mut resolved = original.clone();
            for id in &original.overrides {
                if let Some(parent) = by_id.get(id) {
                    resolved.callable |= parent.callable;
                    resolved.event |= parent.event;
                    resolved.pure |= parent.pure;
                }
            }
            for id in resolved
                .overrides
                .iter()
                .chain(std::iter::once(&resolved.id))
            {
                by_id.insert(id.clone(), resolved.clone());
            }
            by_name.insert(resolved.name.clone(), resolved);
        }
    }
    by_name
}

impl<'a> Lower<'a> {
    fn report(&mut self, span: Span, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::new(self.file, span, message));
    }
    fn slot(&self, name: &str) -> Option<ir::LocalId> {
        self.slots
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .copied()
    }
    fn state(&self, id: ir::LocalId) -> State {
        self.states.get(&id).cloned().unwrap_or((None, false))
    }
    fn declare(&mut self, name: &str, value_type: Option<Type>, assigned: bool) -> ir::LocalId {
        let id = ir::LocalId(self.locals.len() as u32);
        self.locals.push(ir::Local {
            id,
            name: name.into(),
            value_type: value_type.clone().unwrap_or(Type::Void),
        });
        self.states.insert(id, (value_type, assigned));
        if let Some(scope) = self.slots.last_mut() {
            scope.insert(name.into(), id);
        }
        id
    }
    /// The first assignment fixes a local's type for the rest of the method.
    fn fix_type(&mut self, id: ir::LocalId, value_type: &Type) {
        if let Some(local) = self.locals.iter_mut().find(|l| l.id == id) {
            local.value_type = value_type.clone();
        }
        self.states.insert(id, (Some(value_type.clone()), true));
    }
    fn temporary(&mut self, value_type: &Type) -> ir::LocalId {
        let id = ir::LocalId(self.locals.len() as u32);
        self.locals.push(ir::Local {
            id,
            name: format!("epok_tmp{}", id.0),
            value_type: value_type.clone(),
        });
        self.states.insert(id, (Some(value_type.clone()), true));
        id
    }

    // --- type hints ------------------------------------------------------
    /// A best-effort type for an expression that does not depend on literal
    /// context. Used to give bare numeric literals the other operand's type.
    fn hint(&self, expr: &ast::Expr) -> Option<Type> {
        match expr {
            ast::Expr::Bool(..) => Some(Type::Bool),
            ast::Expr::Number { integer, .. } => (!integer).then_some(Type::Fixed),
            ast::Expr::Name { name, .. } => self.slot(name).and_then(|id| self.state(id).0),
            ast::Expr::Field { base, name, .. } => self
                .place_property(base, name)
                .map(|p| p.value_type.clone()),
            ast::Expr::Unary { op, operand, .. } => match op {
                UnOp::Not => Some(Type::Bool),
                UnOp::Neg => self.hint(operand),
            },
            ast::Expr::Binary {
                op, left, right, ..
            } => {
                if matches!(
                    op,
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
                ) || matches!(op, BinOp::And | BinOp::Or)
                {
                    Some(Type::Bool)
                } else {
                    self.hint(left).or_else(|| self.hint(right))
                }
            }
            ast::Expr::MethodCall { name, base, .. } => {
                if matches!(&**base, ast::Expr::Name { name, .. } if name == "self") {
                    self.methods.get(name).map(|f| f.returns.clone())
                } else {
                    self.parent_methods.get(name).map(|f| f.returns.clone())
                }
            }
            ast::Expr::Call { base, .. } => match builtin(base) {
                Some("to_fixed") => Some(Type::Fixed),
                Some("to_int") => Some(Type::Int32),
                _ => None,
            },
            _ => None,
        }
    }
    /// `self.<name>` resolved against the class hierarchy.
    fn place_property(&self, base: &ast::Expr, name: &str) -> Option<&schema::Property> {
        match base {
            ast::Expr::Name { name: base, .. } if base == "self" => self.properties.get(name),
            _ => None,
        }
    }

    // --- expressions -----------------------------------------------------
    fn expr(
        &mut self,
        expr: &ast::Expr,
        expected: Option<&Type>,
        pending: &mut ir::Block,
        root: bool,
    ) -> Option<ir::Expr> {
        match expr {
            ast::Expr::Nil(span) => {
                self.report(*span, profile::NIL_VALUE);
                None
            }
            ast::Expr::Str { span, .. } => {
                self.report(*span, profile::STRING_VALUE);
                None
            }
            ast::Expr::Table { span, .. } => {
                self.report(*span, profile::TABLE_BODY);
                None
            }
            ast::Expr::Bool(value, span) => {
                if expected.is_some_and(|t| t != &Type::Bool) {
                    self.report(*span, "Boolean literal is not the expected type");
                    return None;
                }
                Some(ir::Expr::Literal {
                    value: serde_json::json!(value),
                    value_type: Type::Bool,
                })
            }
            ast::Expr::Number {
                value,
                integer,
                span,
            } => self.number(*value, *integer, expected, *span),
            ast::Expr::Name { name, span } => {
                if name == "self" {
                    self.report(*span, "`self` is only valid as a receiver");
                    return None;
                }
                let Some(id) = self.slot(name) else {
                    self.report(*span, format!("Unknown name {name}"));
                    return None;
                };
                let (value_type, assigned) = self.state(id);
                if !assigned || value_type.is_none() {
                    self.report(*span, format!("Local {name} is used before it is assigned"));
                    return None;
                }
                let value_type = value_type?;
                if !self.scalar(&value_type, *span) {
                    return None;
                }
                Some(ir::Expr::Read {
                    place: ir::Place::Local(id),
                    value_type,
                })
            }
            ast::Expr::Field { .. } => {
                let place = self.place(expr)?;
                let value_type = place.value_type(&self.locals)?;
                if !self.scalar(&value_type, expr.span()) {
                    return None;
                }
                Some(ir::Expr::Read { place, value_type })
            }
            ast::Expr::Unary { op, operand, span } => {
                let value = self.expr(operand, expected, pending, false)?;
                match op {
                    UnOp::Not => {
                        if value.value_type() != &Type::Bool {
                            self.report(*span, "`not` requires a Bool operand");
                            return None;
                        }
                        Some(ir::Expr::Unary {
                            op: ir::UnaryOp::Not,
                            operand: Box::new(value),
                            value_type: Type::Bool,
                        })
                    }
                    UnOp::Neg => {
                        let value_type = value.value_type().clone();
                        if !matches!(value_type, Type::Int32 | Type::Fixed) {
                            self.report(*span, "Negation requires an Int32 or Fixed operand");
                            return None;
                        }
                        Some(ir::Expr::Unary {
                            op: ir::UnaryOp::Negate,
                            operand: Box::new(value),
                            value_type,
                        })
                    }
                }
            }
            ast::Expr::Binary {
                op,
                left,
                right,
                span,
            } => self.binary(*op, left, right, *span, expected, pending),
            ast::Expr::Call { base, args, span } => self.builtin_call(base, args, *span, pending),
            ast::Expr::MethodCall {
                base,
                name,
                args,
                span,
            } => self.method_call(base, name, args, *span, pending, root),
        }
    }

    fn number(
        &mut self,
        value: f64,
        integer: bool,
        expected: Option<&Type>,
        span: Span,
    ) -> Option<ir::Expr> {
        let value_type = match (expected, integer) {
            (Some(Type::Fixed), _) => Type::Fixed,
            (Some(ty @ (Type::Int32 | Type::UInt32)), true) => ty.clone(),
            (Some(ty), _) => {
                self.report(span, format!("Numeric literal is not {}", ty.label()));
                return None;
            }
            (None, false) => Type::Fixed,
            (None, true) => {
                self.report(span, profile::AMBIGUOUS_LITERAL);
                return None;
            }
        };
        let literal = if value_type == Type::Fixed {
            serde_json::json!(value)
        } else if value_type == Type::UInt32 {
            serde_json::json!(value as u64)
        } else {
            serde_json::json!(value as i64)
        };
        if !crate::script_values::valid(&literal, &value_type) {
            self.report(
                span,
                format!(
                    "Literal {literal} is out of range for {}",
                    value_type.label()
                ),
            );
            return None;
        }
        Some(ir::Expr::Literal {
            value: literal,
            value_type,
        })
    }

    fn binary(
        &mut self,
        op: BinOp,
        left: &ast::Expr,
        right: &ast::Expr,
        span: Span,
        expected: Option<&Type>,
        pending: &mut ir::Block,
    ) -> Option<ir::Expr> {
        let logical = matches!(op, BinOp::And | BinOp::Or);
        let comparison = matches!(
            op,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
        );
        let operand_hint = if logical {
            Some(Type::Bool)
        } else if comparison {
            self.hint(left).or_else(|| self.hint(right))
        } else {
            expected
                .cloned()
                .or_else(|| self.hint(left))
                .or_else(|| self.hint(right))
        };
        let a = self.expr(left, operand_hint.as_ref(), pending, false)?;
        // `and`/`or` short-circuit: the right operand may not run when the left
        // already decides the result. A call on the right is hoisted into a
        // statement, so it is lowered into its own block and, when that block
        // is not empty, guarded by an `if` instead of being emitted eagerly.
        let mut branch = ir::Block::new();
        let target: &mut ir::Block = if logical { &mut branch } else { pending };
        let b = self.expr(right, Some(a.value_type()), target, false)?;
        if a.value_type() != b.value_type() {
            self.report(span, "Binary inputs must have exactly the same type");
            return None;
        }
        let operand = a.value_type().clone();
        let (ir_op, value_type) = match op {
            BinOp::And | BinOp::Or => {
                if operand != Type::Bool {
                    self.report(span, profile::LOGICAL_TYPE);
                    return None;
                }
                if !branch.is_empty() {
                    let temporary = self.temporary(&Type::Bool);
                    pending.push(ir::Statement::new(
                        ir::StatementKind::Local {
                            target: temporary,
                            value: a,
                        },
                        span,
                    ));
                    let read = ir::Expr::Read {
                        place: ir::Place::Local(temporary),
                        value_type: Type::Bool,
                    };
                    // `and` runs the right operand when the left is true, `or`
                    // when it is false; either way the temporary keeps the
                    // result and nothing to the right runs otherwise.
                    let cond = if op == BinOp::And {
                        read.clone()
                    } else {
                        ir::Expr::Unary {
                            op: ir::UnaryOp::Not,
                            operand: Box::new(read.clone()),
                            value_type: Type::Bool,
                        }
                    };
                    branch.push(ir::Statement::new(
                        ir::StatementKind::Assign {
                            target: ir::Place::Local(temporary),
                            value: b,
                        },
                        span,
                    ));
                    pending.push(ir::Statement::new(
                        ir::StatementKind::If {
                            cond,
                            then: branch,
                            otherwise: ir::Block::new(),
                        },
                        span,
                    ));
                    return Some(read);
                }
                (
                    if op == BinOp::And {
                        ir::BinaryOp::And
                    } else {
                        ir::BinaryOp::Or
                    },
                    Type::Bool,
                )
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                let ordered = !matches!(op, BinOp::Eq | BinOp::Ne);
                let comparable = matches!(
                    operand,
                    Type::Int32 | Type::UInt32 | Type::Fixed | Type::Enum { .. }
                ) || (!ordered && operand == Type::Bool);
                if !comparable {
                    self.report(span, "Unsupported operator for the declared type");
                    return None;
                }
                (
                    match op {
                        BinOp::Eq => ir::BinaryOp::Eq,
                        BinOp::Ne => ir::BinaryOp::Ne,
                        BinOp::Lt => ir::BinaryOp::Lt,
                        BinOp::Le => ir::BinaryOp::Le,
                        BinOp::Gt => ir::BinaryOp::Gt,
                        _ => ir::BinaryOp::Ge,
                    },
                    Type::Bool,
                )
            }
            BinOp::Mod => {
                if !matches!(operand, Type::Int32 | Type::UInt32) {
                    self.report(span, profile::MOD_TYPE);
                    return None;
                }
                (ir::BinaryOp::Mod, operand.clone())
            }
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if !matches!(
                    operand,
                    Type::Int32 | Type::UInt32 | Type::Fixed | Type::Vector { .. }
                ) {
                    self.report(span, "Unsupported operator for the declared type");
                    return None;
                }
                (
                    match op {
                        BinOp::Add => ir::BinaryOp::Add,
                        BinOp::Sub => ir::BinaryOp::Sub,
                        BinOp::Mul => ir::BinaryOp::Mul,
                        _ => ir::BinaryOp::Div,
                    },
                    operand.clone(),
                )
            }
        };
        Some(ir::Expr::Binary {
            op: ir_op,
            left: Box::new(a),
            right: Box::new(b),
            value_type,
        })
    }

    fn builtin_call(
        &mut self,
        base: &ast::Expr,
        args: &[ast::Expr],
        span: Span,
        pending: &mut ir::Block,
    ) -> Option<ir::Expr> {
        let name = builtin(base);
        let (kind, operand_type) = match name {
            Some("to_fixed") => (ir::Conversion::IntToFixed, Type::Int32),
            Some("to_int") => (ir::Conversion::FixedToInt, Type::Fixed),
            Some("super") => {
                self.report(
                    span,
                    "`epok.super(Class, self)` must be followed by a method call",
                );
                return None;
            }
            _ => {
                self.report(
                    span,
                    format!("Unknown global function {}", global_name(base)),
                );
                return None;
            }
        };
        if args.len() != 1 {
            self.report(span, "Conversion builtins take exactly one argument");
            return None;
        }
        let operand = self.expr(&args[0], Some(&operand_type), pending, false)?;
        if operand.value_type() != &operand_type {
            self.report(span, format!("Conversion expects {}", operand_type.label()));
            return None;
        }
        Some(ir::Expr::Convert {
            kind,
            operand: Box::new(operand),
        })
    }

    fn method_call(
        &mut self,
        base: &ast::Expr,
        name: &str,
        args: &[ast::Expr],
        span: Span,
        pending: &mut ir::Block,
        root: bool,
    ) -> Option<ir::Expr> {
        let parent = match base {
            ast::Expr::Name { name: base, .. } if base == "self" => false,
            ast::Expr::Call {
                base: callee, args, ..
            } if builtin(callee) == Some("super") => {
                if args.len() != 2
                    || !matches!(&args[1], ast::Expr::Name { name, .. } if name == "self")
                {
                    self.report(span, "`epok.super` takes the enclosing class and `self`");
                    return None;
                }
                let class = match &args[0] {
                    ast::Expr::Name { name, .. } => name.clone(),
                    other => {
                        self.report(other.span(), "`epok.super` takes the enclosing class name");
                        return None;
                    }
                };
                if class != self.class.cpp_name {
                    self.report(
                        span,
                        format!(
                            "`epok.super` must name the enclosing class {}",
                            self.class.cpp_name
                        ),
                    );
                    return None;
                }
                true
            }
            other => {
                self.report(
                    other.span(),
                    "Only `self` and `epok.super(Class, self)` receivers are supported",
                );
                return None;
            }
        };
        let function = if parent {
            let Some(function) = self.parent_methods.get(name).cloned() else {
                self.report(span, format!("Unknown parent method {name}"));
                return None;
            };
            let overridden = self
                .class
                .functions
                .iter()
                .find(|f| f.name == self.current)
                .is_some_and(|f| f.overrides.contains(&function.id));
            if !overridden && !(function.callable && function.access == "public") {
                self.report(
                    span,
                    format!("Parent method {name} is not callable from this method"),
                );
                return None;
            }
            function
        } else {
            let Some(function) = self.methods.get(name).cloned() else {
                self.report(span, format!("Unknown method {name} on this class"));
                return None;
            };
            if self.own.contains(name) {
                self.calls.insert((self.current.clone(), name.into()));
            }
            function
        };
        if args.len() != function.parameters.len() {
            self.report(
                span,
                format!("{name} expects {} argument(s)", function.parameters.len()),
            );
            return None;
        }
        let mut lowered = vec![];
        for (arg, parameter) in args.iter().zip(&function.parameters) {
            let value = self.expr(arg, Some(&parameter.value_type), pending, false)?;
            if !crate::blueprint_ir::assignable(
                value.value_type(),
                &parameter.value_type,
                self.registry,
            ) {
                self.report(
                    arg.span(),
                    format!(
                        "Argument {} is not {}",
                        parameter.name,
                        parameter.value_type.label()
                    ),
                );
                return None;
            }
            lowered.push(value);
        }
        let returns = function.returns.clone();
        let call = if parent {
            ir::Expr::CallParent {
                function_id: function.id.clone(),
                name: function.name.clone(),
                args: lowered,
                returns: returns.clone(),
            }
        } else {
            ir::Expr::CallSelf {
                function_id: function.id.clone(),
                name: function.name.clone(),
                args: lowered,
                returns: returns.clone(),
            }
        };
        if root || returns == Type::Void {
            return Some(call);
        }
        // Evaluation order is fixed left-to-right: a call inside a larger
        // expression is materialised before anything to its right runs.
        let temporary = self.temporary(&returns);
        pending.push(ir::Statement::new(
            ir::StatementKind::Local {
                target: temporary,
                value: call,
            },
            span,
        ));
        Some(ir::Expr::Read {
            place: ir::Place::Local(temporary),
            value_type: returns,
        })
    }

    /// Profile v1 moves one 32-bit value at a time (contract §10.1). A whole
    /// vector is not one of them in any mode, so the diagnostic belongs here
    /// rather than in a backend: `self.position.x` is the supported spelling.
    fn scalar(&mut self, value_type: &Type, span: Span) -> bool {
        if matches!(value_type, Type::Vector { .. }) {
            self.report(span, profile::VECTOR_VALUE);
            return false;
        }
        true
    }

    fn place(&mut self, expr: &ast::Expr) -> Option<ir::Place> {
        match expr {
            ast::Expr::Name { name, span } => match self.slot(name) {
                Some(id) => Some(ir::Place::Local(id)),
                None => {
                    self.report(*span, format!("Unknown name {name}"));
                    None
                }
            },
            ast::Expr::Field { base, name, span } => {
                if let Some(property) = self.place_property(base, name) {
                    let (id, value_type) = (property.id.clone(), property.value_type.clone());
                    // Asset and class handles are 64-bit native fields. They stay
                    // Inspector-editable, but no profile v1 body reads or writes one.
                    if matches!(value_type, Type::AssetRef { .. } | Type::ClassRef { .. }) {
                        self.report(*span, profile::REFERENCE_VALUE);
                        return None;
                    }
                    return Some(ir::Place::Property {
                        member_id: id,
                        name: name.clone(),
                        value_type,
                    });
                }
                if matches!(&**base, ast::Expr::Name { name, .. } if name == "self") {
                    self.report(*span, format!("Unknown property {name} on this class"));
                    return None;
                }
                // `self.<vector>.x`
                let inner = self.place(base)?;
                let index = match (inner.value_type(&self.locals), name.as_str()) {
                    (Some(Type::Vector { length }), "x") if length >= 1 => 0,
                    (Some(Type::Vector { length }), "y") if length >= 2 => 1,
                    (Some(Type::Vector { length }), "z") if length >= 3 => 2,
                    _ => {
                        self.report(*span, format!("Unknown member {name}"));
                        return None;
                    }
                };
                Some(ir::Place::VectorComponent {
                    base: Box::new(inner),
                    index,
                })
            }
            other => {
                self.report(other.span(), "Expression is not assignable");
                None
            }
        }
    }

    // --- statements -------------------------------------------------------
    fn block(&mut self, block: &ast::Block) -> ir::Block {
        self.slots.push(BTreeMap::new());
        let mut out = vec![];
        for statement in block {
            self.statement(statement, &mut out);
        }
        self.slots.pop();
        out
    }
    fn statement(&mut self, statement: &ast::Stat, out: &mut ir::Block) {
        match statement {
            ast::Stat::Do { body, span } => {
                let _ = span;
                let inner = self.block(body);
                out.extend(inner);
            }
            ast::Stat::Local { name, value, span } => {
                if !crate::scripts::identifier(name) || name.starts_with("epok_") {
                    self.report(*span, format!("{name} is not a valid local name"));
                    return;
                }
                match value {
                    None => {
                        self.declare(name, None, false);
                    }
                    Some(value) => {
                        let mut pending = vec![];
                        let hint = self.hint(value);
                        let Some(lowered) = self.expr(value, hint.as_ref(), &mut pending, true)
                        else {
                            return;
                        };
                        if lowered.value_type() == &Type::Void {
                            self.report(*span, "A void call produces no value to bind");
                            return;
                        }
                        out.extend(pending);
                        let value_type = lowered.value_type().clone();
                        let id = self.declare(name, Some(value_type), true);
                        out.push(ir::Statement::new(
                            ir::StatementKind::Local {
                                target: id,
                                value: lowered,
                            },
                            *span,
                        ));
                    }
                }
            }
            ast::Stat::Assign {
                target,
                value,
                span,
            } => {
                let Some(place) = self.place(target) else {
                    return;
                };
                let untyped = matches!(&place, ir::Place::Local(id) if self.state(*id).0.is_none());
                let declared = match &place {
                    ir::Place::Local(id) => self.state(*id).0,
                    other => other.value_type(&self.locals),
                };
                let expected = if untyped {
                    self.hint(value)
                } else {
                    declared.clone()
                };
                let mut pending = vec![];
                let Some(lowered) = self.expr(value, expected.as_ref(), &mut pending, true) else {
                    return;
                };
                let actual = lowered.value_type().clone();
                if actual == Type::Void {
                    self.report(*span, "A void call produces no value to assign");
                    return;
                }
                let vector_target = declared
                    .as_ref()
                    .is_some_and(|ty| matches!(ty, Type::Vector { .. }));
                if vector_target {
                    self.report(*span, profile::VECTOR_VALUE);
                }
                if vector_target || !self.scalar(&actual, *span) {
                    return;
                }
                match (&place, declared) {
                    (ir::Place::Local(id), None) => self.fix_type(*id, &actual),
                    (_, None) => {
                        self.report(*span, "Assignment target has no resolved type");
                        return;
                    }
                    (_, Some(declared)) => {
                        if declared != actual {
                            self.report(
                                *span,
                                format!(
                                    "Target already has type {}; it cannot hold {}",
                                    declared.label(),
                                    actual.label()
                                ),
                            );
                            return;
                        }
                        if let ir::Place::Local(id) = &place {
                            self.states.insert(*id, (Some(declared), true));
                        }
                    }
                }
                out.extend(pending);
                out.push(ir::Statement::new(
                    ir::StatementKind::Assign {
                        target: place,
                        value: lowered,
                    },
                    *span,
                ));
            }
            ast::Stat::Call { call, span } => {
                let mut pending = vec![];
                let Some(lowered) = self.expr(call, None, &mut pending, true) else {
                    return;
                };
                out.extend(pending);
                out.push(ir::Statement::new(
                    ir::StatementKind::Evaluate(lowered),
                    *span,
                ));
            }
            ast::Stat::If {
                arms,
                otherwise,
                span,
            } => self.conditional(arms, otherwise, *span, out),
            ast::Stat::NumericFor {
                var,
                start,
                limit,
                step,
                body,
                span,
            } => {
                let Some(start) = self.constant(start) else {
                    return;
                };
                let Some(limit) = self.constant(limit) else {
                    return;
                };
                let step = match step {
                    None => 1,
                    Some(step) => match self.constant(step) {
                        Some(step) => step,
                        None => return,
                    },
                };
                if step == 0 {
                    self.report(*span, "Numeric for step must not be zero");
                    return;
                }
                let iterations = if (limit - start).signum() == step.signum() || start == limit {
                    (limit - start) / step + 1
                } else {
                    0
                };
                if !(0..=MAX_ITERATIONS).contains(&iterations) {
                    self.report(*span, profile::FOR_BUDGET);
                    return;
                }
                if i32::try_from(start).is_err() || i32::try_from(limit).is_err() {
                    self.report(*span, "Numeric for bounds must fit in Int32");
                    return;
                }
                self.slots.push(BTreeMap::new());
                let id = self.declare(var, Some(Type::Int32), true);
                let mut body = self.block(body);
                self.slots.pop();
                let inner = std::mem::take(&mut body);
                out.push(ir::Statement::new(
                    ir::StatementKind::For {
                        var: id,
                        start,
                        limit,
                        step,
                        body: inner,
                    },
                    *span,
                ));
            }
            ast::Stat::Return { value, span } => {
                let returns = self.current_returns.clone();
                match (value, &returns) {
                    (None, Type::Void) => {
                        out.push(ir::Statement::new(ir::StatementKind::Return(None), *span))
                    }
                    (Some(value), expected) if expected != &Type::Void => {
                        let mut pending = vec![];
                        let Some(lowered) = self.expr(value, Some(expected), &mut pending, true)
                        else {
                            return;
                        };
                        if lowered.value_type() != expected {
                            self.report(*span, format!("Return value is not {}", expected.label()));
                            return;
                        }
                        out.extend(pending);
                        out.push(ir::Statement::new(
                            ir::StatementKind::Return(Some(lowered)),
                            *span,
                        ));
                    }
                    _ => self.report(
                        *span,
                        format!(
                            "{} returns {}; the return statement does not match",
                            self.current,
                            returns.label()
                        ),
                    ),
                }
            }
        }
    }

    /// `elseif` chains lower to nested `If` statements. A local is definitely
    /// assigned after the statement only when every arm assigns it.
    fn conditional(
        &mut self,
        arms: &[(ast::Expr, ast::Block)],
        otherwise: &Option<ast::Block>,
        span: Span,
        out: &mut ir::Block,
    ) {
        let Some((condition, body)) = arms.first() else {
            if let Some(block) = otherwise {
                let block = self.block(block);
                out.extend(block);
            }
            return;
        };
        let mut pending = vec![];
        let Some(cond) = self.expr(condition, Some(&Type::Bool), &mut pending, true) else {
            return;
        };
        if cond.value_type() != &Type::Bool {
            self.report(condition.span(), profile::CONDITION_TYPE);
            return;
        }
        out.extend(pending);
        let before = self.states.clone();
        let then = self.block(body);
        let after_then = std::mem::replace(&mut self.states, before.clone());
        let mut alternative = vec![];
        self.conditional(&arms[1..], otherwise, span, &mut alternative);
        let after_else = std::mem::replace(&mut self.states, before);
        let exhaustive = arms.len() > 1 || otherwise.is_some();
        for (id, state) in self.states.iter_mut() {
            let then_state = after_then.get(id).cloned().unwrap_or_else(|| state.clone());
            let else_state = after_else.get(id).cloned().unwrap_or_else(|| state.clone());
            state.1 = state.1 || (then_state.1 && (exhaustive && else_state.1));
            if state.0.is_none() {
                state.0 = then_state.0.or(else_state.0);
            }
        }
        out.push(ir::Statement::new(
            ir::StatementKind::If {
                cond,
                then,
                otherwise: alternative,
            },
            span,
        ));
    }

    /// Constant folding limited to integer literals and `+ - *` over them.
    fn constant(&mut self, expr: &ast::Expr) -> Option<i64> {
        fn fold(expr: &ast::Expr) -> Option<i64> {
            match expr {
                ast::Expr::Number {
                    value,
                    integer: true,
                    ..
                } if value.fract() == 0. => Some(*value as i64),
                ast::Expr::Unary {
                    op: UnOp::Neg,
                    operand,
                    ..
                } => fold(operand).map(|v| -v),
                ast::Expr::Binary {
                    op, left, right, ..
                } => {
                    let (a, b) = (fold(left)?, fold(right)?);
                    match op {
                        BinOp::Add => a.checked_add(b),
                        BinOp::Sub => a.checked_sub(b),
                        BinOp::Mul => a.checked_mul(b),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        match fold(expr) {
            Some(value) => Some(value),
            None => {
                self.report(expr.span(), profile::FOR_BOUNDS);
                None
            }
        }
    }
}

fn global_name(expr: &ast::Expr) -> String {
    match expr {
        ast::Expr::Name { name, .. } => name.clone(),
        ast::Expr::Field { base, name, .. } => format!("{}.{name}", global_name(base)),
        _ => "<expression>".into(),
    }
}
/// `epok.<name>` builtins. Nothing else is a callable global.
fn builtin(expr: &ast::Expr) -> Option<&str> {
    match expr {
        ast::Expr::Field { base, name, .. } if matches!(&**base, ast::Expr::Name { name, .. } if name == "epok") => {
            Some(name)
        }
        _ => None,
    }
}

/// Detects a cycle in the call graph restricted to methods this chunk declares.
fn recursive(edges: &BTreeSet<(String, String)>) -> Option<String> {
    let mut nodes = BTreeSet::new();
    for (from, to) in edges {
        nodes.insert(from.clone());
        nodes.insert(to.clone());
    }
    fn visit(
        node: &str,
        edges: &BTreeSet<(String, String)>,
        stack: &mut Vec<String>,
        done: &mut BTreeSet<String>,
    ) -> Option<String> {
        if stack.iter().any(|n| n == node) {
            return Some(node.into());
        }
        if !done.insert(node.into()) {
            return None;
        }
        stack.push(node.into());
        for (_, to) in edges.iter().filter(|(from, _)| from == node) {
            if let Some(cycle) = visit(to, edges, stack, done) {
                stack.pop();
                return Some(cycle);
            }
        }
        stack.pop();
        None
    }
    let mut done = BTreeSet::new();
    for node in &nodes {
        if let Some(cycle) = visit(node, edges, &mut vec![], &mut done) {
            return Some(cycle);
        }
    }
    None
}

pub fn lower_class(
    decl: &Declaration,
    chunk: &ast::Chunk,
    class: &schema::Class,
    registry: &Registry,
) -> Result<ir::ClassIr, Vec<Diagnostic>> {
    let parent_cpp_name = decl.extends.clone();
    let mut properties = BTreeMap::new();
    for ancestor in registry.ancestry(&parent_cpp_name) {
        for property in &ancestor.properties {
            properties.insert(property.name.clone(), property.clone());
        }
    }
    for property in &class.properties {
        properties.insert(property.name.clone(), property.clone());
    }
    let parent_methods = inherited_by_name(registry, &parent_cpp_name);
    let mut methods = parent_methods
        .iter()
        .filter(|(_, f)| f.callable && f.access == "public")
        .map(|(name, f)| (name.clone(), f.clone()))
        .collect::<BTreeMap<_, _>>();
    for function in &class.functions {
        methods.insert(function.name.clone(), function.clone());
    }
    let own = chunk
        .methods
        .iter()
        .map(|m| m.name.clone())
        .collect::<BTreeSet<_>>();
    let mut lower = Lower {
        file: &decl.file,
        class,
        registry,
        properties,
        methods,
        parent_methods,
        own,
        diagnostics: vec![],
        locals: vec![],
        slots: vec![],
        states: BTreeMap::new(),
        current: String::new(),
        current_returns: Type::Void,
        calls: BTreeSet::new(),
    };

    let mut resolved = vec![];
    for declared in &chunk.methods {
        if declared.object != decl.binding {
            lower.report(
                declared.span,
                format!(
                    "Methods must be declared on {}, the local bound to `epok.class`",
                    decl.binding
                ),
            );
            continue;
        }
        let Some(function) = class.functions.iter().find(|f| f.name == declared.name) else {
            lower.report(
                declared.span,
                format!(
                    "{} is neither declared in `functions` nor an inherited event",
                    declared.name
                ),
            );
            continue;
        };
        resolved.push(Method {
            declared,
            function: function.clone(),
            is_override: !function.overrides.is_empty(),
        });
    }

    let mut out = vec![];
    for method in &resolved {
        lower.locals.clear();
        lower.slots = vec![BTreeMap::new()];
        lower.states.clear();
        lower.current = method.function.name.clone();
        lower.current_returns = method.function.returns.clone();
        if method.declared.parameters.len() != method.function.parameters.len() {
            lower.report(
                method.declared.span,
                format!(
                    "{} takes {} parameter(s) in its declaration",
                    method.function.name,
                    method.function.parameters.len()
                ),
            );
            continue;
        }
        let mut parameters = vec![];
        let mut mismatch = false;
        for ((name, span), declared) in method
            .declared
            .parameters
            .iter()
            .zip(&method.function.parameters)
        {
            if name != &declared.name {
                lower.report(
                    *span,
                    format!("Parameter {name} is declared as {}", declared.name),
                );
                mismatch = true;
                break;
            }
            let id = lower.declare(name, Some(declared.value_type.clone()), true);
            parameters.push(ir::Local {
                id,
                name: name.clone(),
                value_type: declared.value_type.clone(),
            });
        }
        if mismatch {
            continue;
        }
        let parameter_count = parameters.len();
        let body = lower.block(&method.declared.body);
        let locals = lower.locals.split_off(parameter_count);
        for local in &locals {
            if local.value_type == Type::Void {
                lower.report(
                    method.declared.span,
                    format!("Local {} is never assigned, so it has no type", local.name),
                );
            }
        }
        out.push(ir::MethodIr {
            function: method.function.clone(),
            is_override: method.is_override,
            parameters,
            locals,
            body,
            span: method.declared.span,
        });
    }

    if let Some(cycle) = recursive(&lower.calls) {
        let span = resolved
            .iter()
            .find(|m| m.function.name == cycle)
            .map(|m| m.declared.span)
            .unwrap_or_default();
        lower.report(span, format!("{}: {}", profile::RECURSION, cycle));
    }
    if !lower.diagnostics.is_empty() {
        return Err(lower.diagnostics);
    }
    let result = ir::ClassIr {
        class_id: class.id.clone(),
        cpp_name: class.cpp_name.clone(),
        parent_cpp_name,
        methods: out,
    };
    ir::validate(&result).map_err(|e| {
        vec![Diagnostic::new(
            &decl.file,
            decl.span,
            format!("Internal IR check: {e}"),
        )]
    })?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_asset;

    fn file(source: &str) -> LuaFile {
        LuaFile {
            path: "assets/scripts/EnemyLogic.lua".into(),
            source: source.into(),
        }
    }
    fn body(source: &str) -> LuaFile {
        file(&format!(
            "{}\nfunction EnemyLogic:damage(amount)\n{source}\nend\nreturn EnemyLogic\n",
            lua_asset::tests::HEADER
        ))
    }
    fn lower(source: &str) -> Result<ir::ClassIr, Vec<Diagnostic>> {
        let file = body(source);
        let registry = lua_asset::tests::registry();
        let chunk = parse(&file).map_err(|e| vec![e])?;
        let decl = lua_asset::extract(&file).map_err(|e| vec![e])?;
        let class = lua_asset::declarations(&decl, &file, &registry).map_err(|e| vec![e])?;
        lower_class(&decl, &chunk, &class, &registry)
    }
    fn message(source: &str) -> String {
        match lower(source) {
            Ok(_) => panic!("expected a diagnostic for:\n{source}"),
            Err(errors) => errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
    fn parse_error(source: &str) -> String {
        let file = body(source);
        match parse(&file) {
            Ok(_) => panic!("expected a parse diagnostic for:\n{source}"),
            Err(error) => error.to_string(),
        }
    }

    /// `and`/`or` short-circuit. A call on the right needs a statement, and a
    /// statement emitted next to the expression would run unconditionally, so
    /// it has to be lowered inside a guard instead.
    #[test]
    fn lua_frontend_guards_short_circuit_operands_that_need_a_statement() {
        fn calls(block: &[ir::Statement]) -> bool {
            block.iter().any(|s| match &s.kind {
                ir::StatementKind::Local { value, .. }
                | ir::StatementKind::Assign { value, .. } => value.has_call(),
                ir::StatementKind::Evaluate(value) => value.has_call(),
                _ => false,
            })
        }
        let ir = lower("self.ready = self.ready and self:absorb(amount) > 0.0").unwrap();
        let body = &ir.methods[0].body;
        let ir::StatementKind::If {
            then, otherwise, ..
        } = &body[1].kind
        else {
            panic!("expected a short-circuit guard, got {body:?}");
        };
        assert!(otherwise.is_empty());
        assert!(calls(then), "the guarded operand lost its call: {then:?}");
        assert!(
            !calls(&body[..1]),
            "the call ran before the guard: {body:?}"
        );
        assert!(!calls(&body[2..]), "the call ran after the guard: {body:?}");

        // Without a hoisted statement the operator stays a plain expression.
        let ir = lower("self.ready = self.ready and self.ready").unwrap();
        let ir::StatementKind::Assign { value, .. } = &ir.methods[0].body[0].kind else {
            panic!("expected a single assignment");
        };
        assert!(matches!(
            value,
            ir::Expr::Binary {
                op: ir::BinaryOp::And,
                ..
            }
        ));
    }

    /// Profile v1 moves one 32-bit value across the boundary. A whole vector
    /// is rejected by the COMMON frontend, so Native and the VM modes agree;
    /// a backend-only restriction would compile here and fail only in the VM,
    /// and a silent narrowing to component 0 would change the program.
    #[test]
    fn lua_frontend_rejects_whole_vector_values_but_keeps_components() {
        for source in [
            "self.offset = self.offset",
            "local v = self.offset",
            "self.health = self.offset",
        ] {
            assert!(
                message(source).contains(profile::VECTOR_VALUE),
                "{source}: {}",
                message(source)
            );
        }
        // Components remain first-class in both directions.
        let ir = lower("self.offset.y = self.offset.x + 1.0").unwrap();
        let ir::StatementKind::Assign { target, .. } = &ir.methods[0].body[0].kind else {
            panic!("expected an assignment");
        };
        assert!(matches!(
            target,
            ir::Place::VectorComponent { index: 1, .. }
        ));
    }

    #[test]
    fn lua_frontend_lowers_the_documented_enemy_example() {
        let ir = lower("self.health = self.health - amount").unwrap();
        assert_eq!(ir.cpp_name, "EnemyLogic");
        assert_eq!(ir.parent_cpp_name, "epok::ActorComponent");
        let method = &ir.methods[0];
        assert_eq!(method.function.name, "damage");
        assert!(!method.is_override);
        let ir::StatementKind::Assign { target, value } = &method.body[0].kind else {
            panic!("expected an assignment, got {:?}", method.body[0]);
        };
        let ir::Place::Property {
            member_id,
            name,
            value_type,
        } = target
        else {
            panic!("expected a property place");
        };
        assert_eq!(member_id, "3d352b2b-c2d7-4b99-9ba1-a003d648e897");
        assert_eq!(name, "health");
        assert_eq!(value_type, &Type::Fixed);
        let ir::Expr::Binary {
            op, left, right, ..
        } = value
        else {
            panic!("expected a binary expression");
        };
        assert_eq!(*op, ir::BinaryOp::Sub);
        assert!(matches!(
            **left,
            ir::Expr::Read {
                place: ir::Place::Property { .. },
                ..
            }
        ));
        assert!(matches!(
            **right,
            ir::Expr::Read {
                place: ir::Place::Local(_),
                ..
            }
        ));
    }

    #[test]
    fn lua_frontend_types_literals_from_context() {
        let ir = lower("self.health = 0.5").unwrap();
        let ir::StatementKind::Assign { value, .. } = &ir.methods[0].body[0].kind else {
            panic!("expected an assignment");
        };
        let ir::Expr::Literal { value, value_type } = value else {
            panic!("expected a literal");
        };
        assert_eq!(value_type, &Type::Fixed);
        assert_eq!(ir::fixed_raw(value.as_f64().unwrap()), 2048);

        let ir = lower("self.health = 100").unwrap();
        let ir::StatementKind::Assign { value, .. } = &ir.methods[0].body[0].kind else {
            panic!("expected an assignment");
        };
        let ir::Expr::Literal { value, value_type } = value else {
            panic!("expected a literal");
        };
        assert_eq!(value_type, &Type::Fixed);
        assert_eq!(ir::fixed_raw(value.as_f64().unwrap()), 409600);

        // An integer literal with a UInt32 context stays UInt32.
        let ir = lower("self.charges = self.charges + 2").unwrap();
        let ir::StatementKind::Assign { value, .. } = &ir.methods[0].body[0].kind else {
            panic!("expected an assignment");
        };
        assert_eq!(value.value_type(), &Type::UInt32);

        assert!(message("local x = 1").contains(profile::AMBIGUOUS_LITERAL));
    }

    #[test]
    fn lua_frontend_rejects_constructs_outside_the_profile() {
        // Parse-time profile rejections.
        for (source, expected) in [
            ("while true do end", profile::WHILE),
            ("repeat until true", profile::REPEAT),
            ("local f = function() end", profile::CLOSURE),
            ("function inner() end", profile::NESTED_FUNCTION),
            ("local a, b = 1, 2", profile::MULTI_ASSIGN),
            ("return 1, 2", profile::MULTI_RETURN),
            ("local s = amount .. amount", profile::CONCAT),
            ("local n = #self", profile::LENGTH),
            ("local n = 2 ^ 3", profile::POWER),
            ("local n = 7 // 2", profile::FLOOR_DIV),
            ("goto done", profile::GOTO),
            ("::done::", profile::GOTO),
            ("setmetatable(self, self)", profile::METATABLE),
            ("require(\"other\")", profile::DYNAMIC_LOAD),
            ("local s = \"text\"", profile::STRING_VALUE),
            ("local t = nil", profile::NIL_VALUE),
            ("local t = { 1 }", profile::TABLE_BODY),
            ("for k, v in pairs(self) do end", profile::GENERIC_FOR),
        ] {
            let actual = parse_error(source);
            assert!(
                actual.contains(expected),
                "`{source}` reported `{actual}`, expected `{expected}`"
            );
            // Profile diagnostics never name an execution backend.
            for forbidden in ["Native", "VM", "bytecode", "interpreter", "mode"] {
                assert!(
                    !actual.contains(forbidden),
                    "`{actual}` mentions {forbidden}"
                );
            }
        }
        // Semantic profile rejections.
        // `%` is `umod`/`imod` on the integer types and nothing else.
        lower("self.charges = self.charges % 2").unwrap();
        assert!(message("self.health = self.health % self.health").contains(profile::MOD_TYPE));
        assert!(
            message("if self.charges and self.charges then end").contains(profile::LOGICAL_TYPE)
        );
        assert!(message("if self.charges then end").contains(profile::CONDITION_TYPE));
        assert!(message("unknown_helper()").contains("Unknown global function"));
        assert!(message("self.health = self.charges").contains("already has type"));
    }

    #[test]
    fn lua_frontend_enforces_loop_bounds_and_rejects_recursion() {
        lower("for i = 1, 4 do self.health = self.health - amount end").unwrap();
        assert!(message("for i = 1, self.charges do end").contains(profile::FOR_BOUNDS));
        assert!(message("for i = 1, 100000 do end").contains(profile::FOR_BUDGET));
        assert!(message("self:damage(amount)").contains(profile::RECURSION));
    }

    #[test]
    fn lua_frontend_checks_definite_assignment_and_single_local_types() {
        assert!(message("local x\nself.health = x").contains("used before it is assigned"));
        assert!(message("local x = self.health\nx = self.charges").contains("already has type"));
        lower("local x = self.health\nx = x - amount\nself.health = x").unwrap();
    }

    #[test]
    fn lua_frontend_resolves_inherited_members_and_parent_calls() {
        // `armour` is declared on the reflected native parent.
        let ir = lower("self.health = self.health - self.armour").unwrap();
        let ir::StatementKind::Assign { value, .. } = &ir.methods[0].body[0].kind else {
            panic!("expected an assignment");
        };
        let ir::Expr::Binary { right, .. } = value else {
            panic!("expected a binary expression");
        };
        let ir::Expr::Read {
            place: ir::Place::Property { member_id, .. },
            ..
        } = &**right
        else {
            panic!("expected an inherited property read");
        };
        assert_eq!(member_id, lua_asset::tests::ARMOUR_ID);

        // Calling an inherited callable resolves against the parent hierarchy.
        lower("self:hit(amount)").unwrap();
        assert!(message("self:sealed()").contains("Unknown method"));
    }

    #[test]
    fn lua_frontend_rejects_super_misuse() {
        assert!(
            message("epok.super(Wrong, self):hit(amount)")
                .contains("must name the enclosing class")
        );
        // `damage` is a new function, not an override: it has no parent slot.
        assert!(message("epok.super(EnemyLogic, self):damage(amount)").contains("Unknown parent"));
        lower("epok.super(EnemyLogic, self):hit(amount)").unwrap();
    }

    #[test]
    fn lua_frontend_hoists_calls_out_of_larger_expressions() {
        let ir = lower("self.health = self.health - self:absorb(amount)").unwrap();
        let body = &ir.methods[0].body;
        assert!(
            matches!(&body[0].kind, ir::StatementKind::Local { value, .. }
                if matches!(value, ir::Expr::CallSelf { .. })),
            "the call was not hoisted: {body:?}"
        );
        assert!(matches!(&body[1].kind, ir::StatementKind::Assign { .. }));
        ir::validate(&ir).unwrap();
    }

    #[test]
    fn lua_frontend_rejects_nesting_beyond_the_profile_limit() {
        let mut source = String::new();
        for _ in 0..40 {
            source.push_str("if self.ready then\n");
        }
        for _ in 0..40 {
            source.push_str("end\n");
        }
        assert!(parse_error(&source).contains(profile::DEPTH));
    }
}
