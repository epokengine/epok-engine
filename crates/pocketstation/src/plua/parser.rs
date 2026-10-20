//! Pocket Lua syntax tree and parser.

use super::Diagnostic;
use super::lexer::{Tok, Token};

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Bool(bool),
    Str(String),
    Name(String),
    /// `namespace.member` (library constants such as `button.LEFT`).
    Member(String, String),
    Index(Box<Expr>, Box<Expr>),
    /// Call of a function or library routine (`update`, `screen.pixel`).
    Call(String, Vec<Expr>),
    Unary(&'static str, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    Table(Vec<Expr>),
    Nil,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Name(String),
    Index(Expr, Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    Local(Vec<String>, Vec<Expr>),
    Assign(Target, Expr),
    Call(Expr),
    If(Vec<(Expr, Vec<Stmt>)>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    Repeat(Vec<Stmt>, Expr),
    For(String, Expr, Expr, Option<Expr>, Vec<Stmt>),
    Break,
    Return(Option<Expr>),
    Do(Vec<Stmt>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<String>,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Option<String>,
    pub body: Vec<Stmt>,
    pub line: u32,
    pub end_line: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Global {
    pub name: String,
    pub value: Expr,
    pub constant: bool,
    pub line: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Program {
    pub globals: Vec<Global>,
    pub functions: Vec<Function>,
}

struct Parser<'a> {
    tokens: &'a [Token],
    at: usize,
    file: &'a str,
}

type Result<T> = std::result::Result<T, Diagnostic>;

pub fn parse(tokens: &[Token], file: &str) -> Result<Program> {
    let mut p = Parser {
        tokens,
        at: 0,
        file,
    };
    p.program()
}

impl Parser<'_> {
    fn peek(&self) -> &Tok {
        &self.tokens[self.at].tok
    }
    fn token(&self) -> &Token {
        &self.tokens[self.at]
    }
    fn next(&mut self) -> Token {
        let t = self.tokens[self.at].clone();
        if self.at + 1 < self.tokens.len() {
            self.at += 1;
        }
        t
    }
    fn is(&self, sym: &str) -> bool {
        matches!(self.peek(), Tok::Sym(s) if *s == sym)
    }
    fn accept(&mut self, sym: &str) -> bool {
        if self.is(sym) {
            self.next();
            true
        } else {
            false
        }
    }
    fn error_at(&self, token: &Token, message: String) -> Diagnostic {
        Diagnostic {
            file: self.file.to_string(),
            line: token.line,
            column: token.column,
            message,
        }
    }
    fn error(&self, message: String) -> Diagnostic {
        self.error_at(self.token(), message)
    }
    fn describe(&self) -> String {
        match self.peek() {
            Tok::Name(n) => format!("`{n}`"),
            Tok::Int(n) => format!("`{n}`"),
            Tok::Str(_) => "a string".into(),
            Tok::Sym(s) => format!("`{s}`"),
            Tok::Eof => "the end of the file".into(),
        }
    }
    fn expect(&mut self, sym: &str, context: &str) -> Result<Token> {
        if self.is(sym) {
            Ok(self.next())
        } else {
            Err(self.error(format!(
                "expected `{sym}` {context}, found {}",
                self.describe()
            )))
        }
    }
    fn name(&mut self, context: &str) -> Result<(String, Token)> {
        match self.peek().clone() {
            Tok::Name(n) => {
                let t = self.next();
                Ok((n, t))
            }
            _ => Err(self.error(format!(
                "expected a name {context}, found {}",
                self.describe()
            ))),
        }
    }

    fn program(&mut self) -> Result<Program> {
        let mut program = Program::default();
        loop {
            if matches!(self.peek(), Tok::Eof) {
                break;
            }
            if self.accept(";") {
                continue;
            }
            let line = self.token().line;
            if self.is("function")
                || (self.is("local")
                    && matches!(
                        self.tokens.get(self.at + 1).map(|t| &t.tok),
                        Some(Tok::Sym("function"))
                    ))
            {
                self.accept("local");
                self.next();
                program.functions.push(self.function(line)?);
                continue;
            }
            if self.accept("local") {
                let (name, _) = self.name("after `local`")?;
                let constant = self.attribute()?;
                let mut names = vec![(name, constant)];
                while self.accept(",") {
                    let (n, _) = self.name("in the declaration")?;
                    let c = self.attribute()?;
                    names.push((n, c));
                }
                self.expect(
                    "=",
                    "after the variable name (top-level variables need a starting value)",
                )?;
                let mut values = vec![self.expr()?];
                while self.accept(",") {
                    values.push(self.expr()?);
                }
                if values.len() != names.len() {
                    return Err(self.error(format!(
                        "{} names but {} values in the declaration",
                        names.len(),
                        values.len()
                    )));
                }
                for ((name, constant), value) in names.into_iter().zip(values) {
                    program.globals.push(Global {
                        name,
                        value,
                        constant,
                        line,
                    });
                }
                continue;
            }
            return Err(self.error(format!(
                "only `local` declarations and functions may appear at the top of a Pocket Lua file (found {}); put code that runs at start in `function init()`",
                self.describe()
            )));
        }
        Ok(program)
    }

    /// `<const>` attribute.
    fn attribute(&mut self) -> Result<bool> {
        if self.accept("<") {
            let (attr, token) = self.name("as attribute")?;
            if attr != "const" {
                return Err(self.error_at(
                    &token,
                    format!("unknown attribute `<{attr}>` (only `<const>` is supported)"),
                ));
            }
            self.expect(">", "after the attribute")?;
            return Ok(true);
        }
        Ok(false)
    }

    fn function(&mut self, line: u32) -> Result<Function> {
        let (name, name_token) = self.name("after `function`")?;
        if self.is(".") || self.is(":") {
            return Err(self.error_at(
                &name_token,
                "Pocket Lua functions are plain names; tables of functions are not supported"
                    .into(),
            ));
        }
        self.expect("(", "after the function name")?;
        let mut params = Vec::new();
        if !self.is(")") {
            loop {
                let (p, t) = self.name("as a parameter")?;
                let ty = if self.accept(":") {
                    Some(self.name("as the parameter type")?.0)
                } else {
                    None
                };
                params.push(Param {
                    name: p,
                    ty,
                    line: t.line,
                });
                if !self.accept(",") {
                    break;
                }
            }
        }
        self.expect(")", "after the parameters")?;
        let ret = if self.accept(":") {
            Some(self.name("as the return type")?.0)
        } else {
            None
        };
        let body = self.block()?;
        let end = self.expect(
            "end",
            &format!("to close function `{name}` (opened on line {line})"),
        )?;
        Ok(Function {
            name,
            params,
            ret,
            body,
            line,
            end_line: end.line,
        })
    }

    fn block_end(&self) -> bool {
        matches!(self.peek(), Tok::Eof)
            || self.is("end")
            || self.is("else")
            || self.is("elseif")
            || self.is("until")
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut out = Vec::new();
        while !self.block_end() {
            if self.accept(";") {
                continue;
            }
            let returned = self.is("return");
            out.push(self.statement()?);
            if returned {
                self.accept(";");
                if !self.block_end() {
                    return Err(self.error("`return` must be the last statement of a block".into()));
                }
            }
        }
        Ok(out)
    }

    fn statement(&mut self) -> Result<Stmt> {
        let token = self.token().clone();
        let line = token.line;
        let kind = match self.peek().clone() {
            Tok::Sym("local") => {
                self.next();
                if self.is("function") {
                    return Err(
                        self.error("functions can only be declared at the top of the file".into())
                    );
                }
                let (name, _) = self.name("after `local`")?;
                if self.is("<") {
                    return Err(
                        self.error("`<const>` is only allowed at the top of the file".into())
                    );
                }
                let mut names = vec![name];
                while self.accept(",") {
                    names.push(self.name("in the declaration")?.0);
                }
                let mut values = Vec::new();
                if self.accept("=") {
                    values.push(self.expr()?);
                    while self.accept(",") {
                        values.push(self.expr()?);
                    }
                }
                StmtKind::Local(names, values)
            }
            Tok::Sym("if") => {
                self.next();
                let mut branches = Vec::new();
                let cond = self.expr()?;
                self.expect("then", "after the `if` condition")?;
                let body = self.block()?;
                branches.push((cond, body));
                let mut otherwise = None;
                loop {
                    if self.accept("elseif") {
                        let cond = self.expr()?;
                        self.expect("then", "after the `elseif` condition")?;
                        branches.push((cond, self.block()?));
                    } else if self.accept("else") {
                        otherwise = Some(self.block()?);
                        self.expect("end", &format!("to close `if` (opened on line {line})"))?;
                        break;
                    } else {
                        self.expect("end", &format!("to close `if` (opened on line {line})"))?;
                        break;
                    }
                }
                StmtKind::If(branches, otherwise)
            }
            Tok::Sym("while") => {
                self.next();
                let cond = self.expr()?;
                self.expect("do", "after the `while` condition")?;
                let body = self.block()?;
                self.expect("end", &format!("to close `while` (opened on line {line})"))?;
                StmtKind::While(cond, body)
            }
            Tok::Sym("repeat") => {
                self.next();
                let body = self.block()?;
                self.expect(
                    "until",
                    &format!("to close `repeat` (opened on line {line})"),
                )?;
                StmtKind::Repeat(body, self.expr()?)
            }
            Tok::Sym("for") => {
                self.next();
                let (var, _) = self.name("after `for`")?;
                if self.is(",") || self.is("in") {
                    return Err(self.error(
                        "Pocket Lua supports numeric loops: `for i = first, last[, step] do`"
                            .into(),
                    ));
                }
                self.expect("=", "in the `for` loop")?;
                let start = self.expr()?;
                self.expect(",", "between the first and last value")?;
                let end = self.expr()?;
                let step = if self.accept(",") {
                    Some(self.expr()?)
                } else {
                    None
                };
                self.expect("do", "after the loop range")?;
                let body = self.block()?;
                self.expect("end", &format!("to close `for` (opened on line {line})"))?;
                StmtKind::For(var, start, end, step, body)
            }
            Tok::Sym("do") => {
                self.next();
                let body = self.block()?;
                self.expect("end", &format!("to close `do` (opened on line {line})"))?;
                StmtKind::Do(body)
            }
            Tok::Sym("break") => {
                self.next();
                StmtKind::Break
            }
            Tok::Sym("return") => {
                self.next();
                if self.block_end() || self.is(";") {
                    StmtKind::Return(None)
                } else {
                    StmtKind::Return(Some(self.expr()?))
                }
            }
            Tok::Sym("function") => {
                return Err(
                    self.error("functions can only be declared at the top of the file".into())
                );
            }
            Tok::Sym("goto") => return Err(self.error("`goto` is not supported".into())),
            _ => {
                let expr = self.suffixed()?;
                if self.accept("=") {
                    let value = self.expr()?;
                    let target = match expr.kind {
                        ExprKind::Name(n) => Target::Name(n),
                        ExprKind::Index(base, index) => Target::Index(*base, *index),
                        ExprKind::Member(ns, field) => {
                            return Err(
                                self.error_at(&token, format!("`{ns}.{field}` cannot be assigned"))
                            );
                        }
                        _ => return Err(self.error_at(&token, "this cannot be assigned".into())),
                    };
                    StmtKind::Assign(target, value)
                } else if self.is(",") {
                    return Err(self.error("assign one variable per statement".into()));
                } else {
                    match expr.kind {
                        ExprKind::Call(..) => StmtKind::Call(expr),
                        _ => {
                            return Err(self.error_at(
                                &token,
                                "expected a statement (an assignment or a function call)".into(),
                            ));
                        }
                    }
                }
            }
        };
        Ok(Stmt { kind, line })
    }

    fn expr(&mut self) -> Result<Expr> {
        self.binary(0)
    }

    fn binary(&mut self, min: u8) -> Result<Expr> {
        let mut left = self.unary()?;
        while let Tok::Sym(op) = self.peek().clone() {
            let Some((precedence, right_assoc)) = binary_precedence(op) else {
                break;
            };
            if precedence <= min {
                break;
            }
            let token = self.next();
            if op == ".." {
                return Err(self.error_at(
                    &token,
                    "strings cannot be joined with `..`; draw each part with screen.text".into(),
                ));
            }
            let right = self.binary(if right_assoc {
                precedence - 1
            } else {
                precedence
            })?;
            left = Expr {
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
                line: token.line,
                column: token.column,
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr> {
        let token = self.token().clone();
        let op = match self.peek() {
            Tok::Sym("not") => "not",
            Tok::Sym("-") => "-",
            Tok::Sym("#") => "#",
            Tok::Sym("~") => "~",
            _ => return self.power(),
        };
        self.next();
        let inner = self.binary(11)?;
        // Fold negative literals.
        if op == "-"
            && let ExprKind::Int(n) = inner.kind
        {
            return Ok(Expr {
                kind: ExprKind::Int(-n),
                line: token.line,
                column: token.column,
            });
        }
        Ok(Expr {
            kind: ExprKind::Unary(op, Box::new(inner)),
            line: token.line,
            column: token.column,
        })
    }

    fn power(&mut self) -> Result<Expr> {
        let base = self.suffixed()?;
        if self.is("^") {
            return Err(self.error(
                "`^` (power) is not supported; multiply, or shift with `<<` for powers of two"
                    .into(),
            ));
        }
        Ok(base)
    }

    fn suffixed(&mut self) -> Result<Expr> {
        let token = self.token().clone();
        let mut expr = self.primary()?;
        loop {
            if self.is(".") {
                self.next();
                let (field, _) = self.name("after `.`")?;
                let ExprKind::Name(ns) = &expr.kind else {
                    return Err(
                        self.error_at(&token, "only library names can be followed by `.`".into())
                    );
                };
                expr = Expr {
                    kind: ExprKind::Member(ns.clone(), field),
                    line: token.line,
                    column: token.column,
                };
            } else if self.is("[") {
                self.next();
                let index = self.expr()?;
                self.expect("]", "after the index")?;
                expr = Expr {
                    kind: ExprKind::Index(Box::new(expr), Box::new(index)),
                    line: token.line,
                    column: token.column,
                };
            } else if self.is("(") || matches!(self.peek(), Tok::Str(_)) || self.is("{") {
                let name = match &expr.kind {
                    ExprKind::Name(n) => n.clone(),
                    ExprKind::Member(ns, f) => format!("{ns}.{f}"),
                    _ => {
                        return Err(
                            self.error_at(&token, "only named functions can be called".into())
                        );
                    }
                };
                let args = if self.accept("(") {
                    let mut args = Vec::new();
                    if !self.is(")") {
                        loop {
                            args.push(self.expr()?);
                            if !self.accept(",") {
                                break;
                            }
                        }
                    }
                    self.expect(")", "after the arguments")?;
                    args
                } else if let Tok::Str(s) = self.peek().clone() {
                    let t = self.next();
                    vec![Expr {
                        kind: ExprKind::Str(s),
                        line: t.line,
                        column: t.column,
                    }]
                } else {
                    vec![self.primary()?]
                };
                expr = Expr {
                    kind: ExprKind::Call(name, args),
                    line: token.line,
                    column: token.column,
                };
            } else if self.is(":")
                && matches!(
                    self.tokens.get(self.at + 1).map(|t| &t.tok),
                    Some(Tok::Name(_))
                )
            {
                return Err(self.error(
                    "method calls with `:` are not supported; call `library.function(...)`".into(),
                ));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.token().clone();
        let kind = match self.peek().clone() {
            Tok::Int(n) => {
                self.next();
                ExprKind::Int(n)
            }
            Tok::Str(s) => {
                self.next();
                ExprKind::Str(s)
            }
            Tok::Name(n) => {
                self.next();
                ExprKind::Name(n)
            }
            Tok::Sym("true") => {
                self.next();
                ExprKind::Bool(true)
            }
            Tok::Sym("false") => {
                self.next();
                ExprKind::Bool(false)
            }
            Tok::Sym("nil") => {
                self.next();
                ExprKind::Nil
            }
            Tok::Sym("(") => {
                self.next();
                let inner = self.expr()?;
                self.expect(")", "to close `(`")?;
                return Ok(inner);
            }
            Tok::Sym("{") => {
                self.next();
                let mut items = Vec::new();
                while !self.is("}") {
                    if matches!(self.peek(), Tok::Name(_))
                        && matches!(
                            self.tokens.get(self.at + 1).map(|t| &t.tok),
                            Some(Tok::Sym("="))
                        )
                    {
                        return Err(self.error(
                            "tables hold numbers only; named fields are not supported".into(),
                        ));
                    }
                    items.push(self.expr()?);
                    if !self.accept(",") && !self.accept(";") {
                        break;
                    }
                }
                self.expect("}", "to close the table")?;
                ExprKind::Table(items)
            }
            Tok::Sym("function") => {
                return Err(self.error("anonymous functions are not supported; declare `function name() ... end` at the top".into()));
            }
            Tok::Sym("...") => return Err(self.error("`...` is not supported".into())),
            _ => return Err(self.error(format!("expected a value, found {}", self.describe()))),
        };
        Ok(Expr {
            kind,
            line: token.line,
            column: token.column,
        })
    }
}

fn binary_precedence(op: &str) -> Option<(u8, bool)> {
    Some(match op {
        "or" => (1, false),
        "and" => (2, false),
        "<" | ">" | "<=" | ">=" | "~=" | "==" => (3, false),
        "|" => (4, false),
        "~" => (5, false),
        "&" => (6, false),
        "<<" | ">>" => (7, false),
        ".." => (8, true),
        "+" | "-" => (9, false),
        "*" | "/" | "//" | "%" => (10, false),
        _ => return None,
    })
}
