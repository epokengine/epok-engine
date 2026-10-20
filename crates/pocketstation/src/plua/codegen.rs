//! Type checking and Thumb code generation for Pocket Lua.

use super::parser::{Expr, ExprKind, Function, Program, Stmt, StmtKind, Target};
use super::runtime;
use super::{Diagnostic, Sprite};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    Bool,
    Sprite,
    Tune,
    Table,
    Array,
    Str,
    Void,
}

impl Ty {
    pub fn name(self) -> &'static str {
        match self {
            Ty::Int => "number",
            Ty::Bool => "boolean",
            Ty::Sprite => "sprite",
            Ty::Tune => "tune",
            Ty::Table => "table",
            Ty::Array => "array",
            Ty::Str => "string",
            Ty::Void => "nothing",
        }
    }
    fn from_annotation(name: &str) -> Option<Ty> {
        Some(match name {
            "int" | "integer" | "number" => Ty::Int,
            "bool" | "boolean" => Ty::Bool,
            "sprite" => Ty::Sprite,
            "tune" => Ty::Tune,
            "table" => Ty::Table,
            "array" => Ty::Array,
            "string" => Ty::Str,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
enum GlobalKind {
    Ram(u32),
    Const(i64),
    Label(String),
}

#[derive(Clone, Debug)]
struct Global {
    ty: Ty,
    kind: GlobalKind,
}

#[derive(Clone, Debug)]
struct Signature {
    params: Vec<Ty>,
    ret: Ty,
    label: String,
}

/// One library routine available to programs.
struct Builtin {
    name: &'static str,
    routine: &'static str,
    include: &'static str,
    params: &'static [Ty],
    optional: &'static [(Ty, i64)],
    ret: Ty,
    /// Pass the source line as the last argument (for runtime errors).
    line: bool,
}

const fn b(
    name: &'static str,
    routine: &'static str,
    include: &'static str,
    params: &'static [Ty],
    ret: Ty,
) -> Builtin {
    Builtin {
        name,
        routine,
        include,
        params,
        optional: &[],
        ret,
        line: false,
    }
}

use Ty::*;

const BUILTINS: &[Builtin] = &[
    b(
        "screen.clear",
        "rt_screen_clear",
        "rt_screen_clear",
        &[],
        Void,
    ),
    b(
        "screen.fill",
        "rt_screen_fill",
        "rt_screen_clear",
        &[],
        Void,
    ),
    b(
        "screen.invert",
        "rt_screen_invert",
        "rt_screen_invert",
        &[],
        Void,
    ),
    Builtin {
        optional: &[(Bool, 1)],
        ..b(
            "screen.pixel",
            "rt_screen_pixel",
            "rt_screen_pixel",
            &[Int, Int],
            Void,
        )
    },
    b(
        "screen.get",
        "rt_screen_get",
        "rt_screen_get",
        &[Int, Int],
        Bool,
    ),
    b(
        "screen.line",
        "rt_screen_line",
        "rt_screen_line",
        &[Int, Int, Int, Int],
        Void,
    ),
    b(
        "screen.rect",
        "rt_screen_rect",
        "rt_screen_rect",
        &[Int, Int, Int, Int],
        Void,
    ),
    b(
        "screen.box",
        "rt_screen_box",
        "rt_screen_box",
        &[Int, Int, Int, Int],
        Void,
    ),
    Builtin {
        optional: &[(Int, 1)],
        ..b(
            "screen.sprite",
            "rt_screen_sprite",
            "rt_screen_sprite",
            &[Sprite, Int, Int],
            Void,
        )
    },
    Builtin {
        optional: &[(Int, 1)],
        ..b(
            "screen.erase",
            "rt_screen_erase",
            "rt_screen_sprite",
            &[Sprite, Int, Int],
            Void,
        )
    },
    b(
        "screen.text",
        "rt_screen_text",
        "rt_screen_text",
        &[Str, Int, Int],
        Int,
    ),
    b(
        "screen.number",
        "rt_screen_number",
        "rt_screen_number",
        &[Int, Int, Int],
        Int,
    ),
    b(
        "button.down",
        "rt_button_down",
        "rt_button_down",
        &[Int],
        Bool,
    ),
    b(
        "button.pressed",
        "rt_button_pressed",
        "rt_button_down",
        &[Int],
        Bool,
    ),
    b(
        "button.released",
        "rt_button_released",
        "rt_button_down",
        &[Int],
        Bool,
    ),
    b("sound.tone", "rt_sound_tone", "rt_sound", &[Int, Int], Void),
    b("sound.play", "rt_sound_play", "rt_sound", &[Tune], Void),
    b("sound.stop", "rt_sound_stop", "rt_sound", &[], Void),
    b("sound.playing", "rt_sound_playing", "rt_sound", &[], Bool),
    b("clock.hour", "rt_clock_hour", "rt_clock", &[], Int),
    b("clock.minute", "rt_clock_minute", "rt_clock", &[], Int),
    b("clock.second", "rt_clock_second", "rt_clock", &[], Int),
    b("clock.day", "rt_clock_day", "rt_clock", &[], Int),
    b("clock.month", "rt_clock_month", "rt_clock", &[], Int),
    b("clock.year", "rt_clock_year", "rt_clock", &[], Int),
    b("clock.weekday", "rt_clock_weekday", "rt_clock", &[], Int),
    Builtin {
        line: true,
        ..b("save.get", "rt_save_get", "rt_save", &[Int], Int)
    },
    Builtin {
        line: true,
        ..b("save.set", "rt_save_set", "rt_save", &[Int, Int], Void)
    },
    b("save.write", "rt_save_write", "rt_save", &[], Bool),
    b("link.connected", "rt_link_connected", "rt_link", &[], Bool),
    b("link.received", "rt_link_received", "rt_link", &[], Int),
    b("link.byte", "rt_link_byte", "rt_link", &[Int], Int),
    b("link.clear", "rt_link_clear", "rt_link", &[], Void),
    b("link.put", "rt_link_put", "rt_link", &[Int, Int], Void),
    b("link.send", "rt_link_publish", "rt_link", &[Int], Void),
    b("link.busy", "rt_link_busy", "rt_link", &[], Bool),
    b("link.value", "rt_link_get_value", "rt_link", &[Int], Int),
    b(
        "link.set_value",
        "rt_link_set_value",
        "rt_link",
        &[Int, Int],
        Void,
    ),
    b("system.exit", "rt_system_exit", "rt_system", &[], Void),
    b("system.frame", "rt_system_frame", "rt_system", &[], Int),
    b("system.docked", "rt_system_docked", "rt_system", &[], Bool),
    b("system.speed", "rt_system_speed", "rt_system", &[Int], Void),
    b("system.fps", "rt_set_fps", "rt_set_fps", &[Int], Void),
    b("system.param", "rt_system_param", "rt_system", &[], Int),
    b("math.random", "rt_random", "rt_random", &[Int], Int),
    b("math.abs", "rt_math_abs", "rt_math", &[Int], Int),
    b("math.min", "rt_math_min", "rt_math", &[Int, Int], Int),
    b("math.max", "rt_math_max", "rt_math", &[Int, Int], Int),
    b(
        "math.clamp",
        "rt_math_clamp",
        "rt_math",
        &[Int, Int, Int],
        Int,
    ),
];

const CONSTANTS: &[(&str, i64)] = &[
    ("button.FIRE", 1),
    ("button.ACTION", 1),
    ("button.RIGHT", 2),
    ("button.LEFT", 4),
    ("button.DOWN", 8),
    ("button.UP", 16),
    ("button.ANY", 31),
    ("screen.WIDTH", 32),
    ("screen.HEIGHT", 32),
];

const NAMESPACES: &[&str] = &[
    "screen", "button", "sound", "clock", "save", "link", "system", "math",
];
const LIFECYCLE: &[&str] = &["init", "update", "draw"];

/// LuaLS definitions derived from the same library table used by the compiler.
/// Pocket Lua has its own workspace so its screen/button APIs never pollute
/// gameplay Lua's `epok` vocabulary.
pub fn language_definitions() -> String {
    use std::fmt::Write;
    fn annotation(ty: Ty) -> &'static str {
        match ty {
            Int => "integer",
            Bool => "boolean",
            Sprite => "PocketSprite",
            Tune => "PocketTune",
            Table => "table",
            Array => "integer[]",
            Str => "string",
            Void => "nil",
        }
    }
    let mut out = String::from(
        "---@meta\n-- Generated from the Pocket Lua compiler's builtin registry.\n---@class PocketSprite\n---@class PocketTune\n",
    );
    for name in NAMESPACES {
        let _ = writeln!(out, "{name} = {{}}\n");
    }
    for (name, value) in CONSTANTS {
        let _ = writeln!(out, "---@type integer\n{name} = {value}\n");
    }
    for builtin in BUILTINS {
        let mut parameters = Vec::new();
        for (index, ty) in builtin.params.iter().enumerate() {
            let name = format!("arg{}", index + 1);
            let _ = writeln!(out, "---@param {name} {}", annotation(*ty));
            parameters.push(name);
        }
        for (index, (ty, _)) in builtin.optional.iter().enumerate() {
            let name = format!("arg{}", builtin.params.len() + index + 1);
            let _ = writeln!(out, "---@param {name}? {}", annotation(*ty));
            parameters.push(name);
        }
        if builtin.ret != Void {
            let _ = writeln!(out, "---@return {}", annotation(builtin.ret));
        }
        let _ = writeln!(
            out,
            "function {}({}) end\n",
            builtin.name,
            parameters.join(", ")
        );
    }
    out.push_str("---@param ... any\nfunction print(...) end\n---@param size integer\n---@return integer[]\nfunction array(size) end\n");
    out.push_str("---@param name_or_pixels string\n---@return PocketSprite\nfunction sprite(name_or_pixels) end\n---@param notes string\n---@return PocketTune\nfunction tune(notes) end\n");
    for name in LIFECYCLE {
        let _ = writeln!(
            out,
            "-- Called by the PocketStation runtime.\nfunction {name}() end\n"
        );
    }
    out
}

fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut previous = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let current = row[j];
            row[j] = (row[j] + 1)
                .min(row[j - 1] + 1)
                .min(previous + (a[i - 1] != b[j - 1]) as usize);
            previous = current;
        }
    }
    row[b.len()]
}

fn suggest<'a>(name: &str, candidates: impl Iterator<Item = &'a str>) -> String {
    let best = candidates
        .map(|c| (distance(name, c), c))
        .filter(|(d, _)| *d <= 2.max(name.len() / 3))
        .min_by_key(|(d, _)| *d);
    match best {
        Some((_, c)) => format!("; did you mean `{c}`?"),
        None => String::new(),
    }
}

/// What the generated program needs from the runtime and the RAM map.
#[derive(Clone, Debug, Default)]
pub struct Output {
    pub source: String,
    pub uses_link: bool,
    pub uses_save: bool,
    pub uses_sound: bool,
    pub uses_print: bool,
    pub ram_end: u32,
    /// Program variables: (name, address, type name).
    pub variables: Vec<(String, u32, &'static str)>,
    pub warnings: Vec<Diagnostic>,
}

struct Local {
    slot: u32,
    ty: Ty,
}

/// Start-up work for one program variable.
enum Init<'p> {
    Value(&'p super::parser::Global, u32),
    Array(&'p super::parser::Global, u32),
    Label(u32, String, u32),
}

pub struct Generator<'a> {
    file: &'a str,
    sprites: &'a BTreeMap<String, Sprite>,
    code: Vec<String>,
    data: Vec<String>,
    errors: Vec<Diagnostic>,
    warnings: Vec<Diagnostic>,
    globals: HashMap<String, Global>,
    functions: HashMap<String, Signature>,
    routines: BTreeSet<&'static str>,
    labels: usize,
    strings: HashMap<String, String>,
    // Function state.
    scopes: Vec<HashMap<String, Local>>,
    slots: u32,
    max_slots: u32,
    pushed: u32,
    loops: Vec<String>,
    return_label: String,
    return_type: Ty,
    error_stubs: Vec<String>,
    since_pool: u32,
    line: u32,
    function_name: String,
    externs: Vec<String>,
}

type R = u32;

impl<'a> Generator<'a> {
    pub fn new(file: &'a str, sprites: &'a BTreeMap<String, Sprite>) -> Self {
        Self {
            file,
            sprites,
            code: Vec::new(),
            data: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            globals: HashMap::new(),
            functions: HashMap::new(),
            routines: BTreeSet::new(),
            labels: 0,
            strings: HashMap::new(),
            scopes: Vec::new(),
            slots: 0,
            max_slots: 0,
            pushed: 0,
            loops: Vec::new(),
            return_label: String::new(),
            return_type: Void,
            error_stubs: Vec::new(),
            since_pool: 0,
            line: 0,
            function_name: String::new(),
            externs: Vec::new(),
        }
    }

    /// Assembly functions callable from Pocket Lua (numbers in, number out).
    pub fn set_externs(&mut self, names: &[String]) {
        self.externs = names.to_vec();
    }

    fn error(&mut self, line: u32, column: u32, message: String) {
        if self.errors.len() < 50 {
            self.errors.push(Diagnostic {
                file: self.file.to_string(),
                line,
                column,
                message,
            });
        }
    }
    fn err(&mut self, e: &Expr, message: String) {
        self.error(e.line, e.column, message);
    }
    fn label(&mut self, hint: &str) -> String {
        self.labels += 1;
        format!(".L{hint}{}", self.labels)
    }
    fn emit(&mut self, line: impl Into<String>) {
        let line = line.into();
        if !line.ends_with(':') && !line.starts_with('.') {
            self.since_pool += 1;
        }
        self.code.push(format!("    {line}"));
    }
    fn place(&mut self, label: &str) {
        self.code.push(format!("{label}:"));
    }
    fn loc(&mut self, line: u32) {
        if line != self.line {
            self.line = line;
            self.code
                .push(format!("    .loc \"{}\", {line}", self.file));
        }
    }
    /// Places a literal pool when the code since the last one grows large.
    fn maybe_pool(&mut self) {
        if self.since_pool > 150 {
            let skip = self.label("pool");
            self.emit(format!("b {skip}"));
            self.code.push("    .pool".into());
            self.place(&skip);
            self.since_pool = 0;
        }
    }

    fn mov_imm(&mut self, r: R, value: i64) {
        let v = value as i32 as i64;
        if (0..=255).contains(&v) {
            self.emit(format!("mov r{r}, #{v}"));
        } else if (-255..0).contains(&v) {
            self.emit(format!("mov r{r}, #{}", -v));
            self.emit(format!("neg r{r}, r{r}"));
        } else {
            self.emit(format!("ldr r{r}, ={}", v as i32));
        }
    }

    fn push(&mut self, regs: &str, count: u32) {
        self.emit(format!("push {{{regs}}}"));
        self.pushed += count * 4;
    }
    fn pop(&mut self, regs: &str, count: u32) {
        self.emit(format!("pop {{{regs}}}"));
        self.pushed -= count * 4;
    }

    // ------------------------------------------------------------- program

    pub fn program(mut self, program: &Program) -> Result<Output, Vec<Diagnostic>> {
        let mut out = Output::default();
        let used = used_calls(program);
        out.uses_link = used.iter().any(|n| n.starts_with("link."));
        out.uses_save = used.iter().any(|n| n.starts_with("save."));
        out.uses_sound = used.iter().any(|n| n.starts_with("sound."));
        out.uses_print = used.contains("print");
        // RAM map: runtime, optional blocks, then variables.
        let mut ram = runtime::RAM_END;
        let mut ram_symbols: Vec<(String, u32)> = runtime::RAM_LAYOUT
            .iter()
            .map(|(n, a)| (n.to_string(), *a))
            .collect();
        if out.uses_link {
            for (name, size) in [
                ("rt_link_info", 8),
                ("rt_inbox_length", 4),
                ("rt_outbox_length", 4),
                ("rt_inbox", 64),
                ("rt_outbox", 64),
                ("rt_values", 64),
            ] {
                ram_symbols.push((name.into(), ram));
                ram += size;
            }
        }
        if out.uses_save {
            ram_symbols.push(("rt_save_buffer".into(), ram));
            ram += runtime::SAVE_RAM;
        }
        let assigned = assigned_names(program);
        // Functions first, so globals and calls can refer to them.
        for f in &program.functions {
            if self.functions.contains_key(&f.name) {
                self.error(f.line, 1, format!("function `{}` is defined twice", f.name));
                continue;
            }
            if NAMESPACES.contains(&f.name.as_str()) || f.name == "print" {
                self.error(
                    f.line,
                    1,
                    format!(
                        "`{}` is a library name; choose another function name",
                        f.name
                    ),
                );
            }
            let mut params = Vec::new();
            for p in &f.params {
                let ty = match &p.ty {
                    Some(t) => Ty::from_annotation(t).unwrap_or_else(|| {
                        self.error(p.line, 1, format!("unknown type `{t}` (use int, bool, sprite, tune, table, array or string)"));
                        Int
                    }),
                    None => Int,
                };
                params.push(ty);
            }
            if params.len() > 4 {
                self.error(
                    f.line,
                    1,
                    format!(
                        "`{}` has {} parameters; at most 4 are supported",
                        f.name,
                        params.len()
                    ),
                );
            }
            let ret = match &f.ret {
                Some(t) => Ty::from_annotation(t).unwrap_or_else(|| {
                    self.error(f.line, 1, format!("unknown return type `{t}`"));
                    Int
                }),
                None => infer_return(&f.body),
            };
            if LIFECYCLE.contains(&f.name.as_str()) && !f.params.is_empty() {
                self.error(
                    f.line,
                    1,
                    format!("`{}` is called by the runtime without arguments", f.name),
                );
            }
            self.functions.insert(
                f.name.clone(),
                Signature {
                    params,
                    ret,
                    label: format!("fn_{}", f.name),
                },
            );
        }
        // Globals.
        let mut init: Vec<Init> = Vec::new();
        for g in &program.globals {
            if self.globals.contains_key(&g.name) || self.functions.contains_key(&g.name) {
                self.error(g.line, 1, format!("`{}` is declared twice", g.name));
                continue;
            }
            if NAMESPACES.contains(&g.name.as_str()) {
                self.error(
                    g.line,
                    1,
                    format!(
                        "`{}` is a library name; choose another variable name",
                        g.name
                    ),
                );
                continue;
            }
            let reassigned = assigned.contains(&g.name);
            if g.constant && reassigned {
                self.error(
                    g.line,
                    1,
                    format!("`{}` is `<const>` but is assigned later", g.name),
                );
            }
            // Literal data bound to a name that never changes stays in flash.
            if let Some((ty, label)) = self.data_literal(&g.value, &g.name) {
                if ty == Array {
                    let (label, size) = label;
                    ram_symbols.push((label.clone(), ram));
                    out.variables.push((g.name.clone(), ram, "array"));
                    init.push(Init::Array(g, ram));
                    ram += size;
                    if reassigned {
                        let address = ram;
                        ram += 4;
                        self.globals.insert(
                            g.name.clone(),
                            Global {
                                ty,
                                kind: GlobalKind::Ram(address),
                            },
                        );
                        init.push(Init::Label(address, label, g.line));
                    } else {
                        self.globals.insert(
                            g.name.clone(),
                            Global {
                                ty,
                                kind: GlobalKind::Label(label),
                            },
                        );
                    }
                    continue;
                }
                let (label, _) = label;
                if reassigned {
                    let address = ram;
                    ram += 4;
                    self.globals.insert(
                        g.name.clone(),
                        Global {
                            ty,
                            kind: GlobalKind::Ram(address),
                        },
                    );
                    out.variables.push((g.name.clone(), address, ty.name()));
                    init.push(Init::Label(address, label, g.line));
                } else {
                    self.globals.insert(
                        g.name.clone(),
                        Global {
                            ty,
                            kind: GlobalKind::Label(label),
                        },
                    );
                }
                continue;
            }
            if g.constant {
                match self.const_eval(&g.value) {
                    Some((value, ty)) => {
                        self.globals.insert(
                            g.name.clone(),
                            Global {
                                ty,
                                kind: GlobalKind::Const(value),
                            },
                        );
                    }
                    None => self.err(
                        &g.value,
                        format!("`<const>` `{}` needs a value known when compiling", g.name),
                    ),
                }
                continue;
            }
            let ty = self.peek_type(&g.value);
            let address = ram;
            ram += 4;
            self.globals.insert(
                g.name.clone(),
                Global {
                    ty,
                    kind: GlobalKind::Ram(address),
                },
            );
            out.variables.push((g.name.clone(), address, ty.name()));
            init.push(Init::Value(g, address));
        }
        out.ram_end = ram;
        let stack = 0x800i64 - ram as i64;
        if stack < 128 {
            self.error(1, 1, format!(
                "the program's variables need {} bytes of RAM; with the 1536 bytes of the PocketStation that leaves {stack} bytes for the stack (at least 128 are needed)",
                ram - 0x200
            ));
        } else if stack < 320 {
            self.warnings.push(Diagnostic {
                file: self.file.to_string(),
                line: 1,
                column: 1,
                message: format!(
                    "only {stack} bytes of RAM remain for the stack; deep calls may run out"
                ),
            });
        }
        let stack_limit = ram + 48;

        // Global initialisation routine.
        self.code.push("    .thumb_func".into());
        self.place("rt_globals");
        self.emit("push {r4-r7, lr}");
        self.function_name = "(top level)".into();
        self.scopes = vec![HashMap::new()];
        self.pushed = 0;
        for item in &init {
            match item {
                Init::Array(g, address) => {
                    self.loc(g.line);
                    let ExprKind::Call(_, args) = &g.value.kind else {
                        continue;
                    };
                    // RAM array: length word, then zeros or the listed values.
                    let length = self.array_length(args);
                    self.mov_imm(0, length as i64);
                    self.emit(format!("ldr r1, ={address}"));
                    self.emit("str r0, [r1]");
                    if let Some(Expr {
                        kind: ExprKind::Table(items),
                        ..
                    }) = args.first()
                    {
                        for (i, item) in items.iter().enumerate() {
                            let ty = self.eval(item, 0);
                            if ty != Int && ty != Bool {
                                self.err(item, format!("arrays hold numbers, not a {}", ty.name()));
                            }
                            self.emit(format!("ldr r1, ={}", address + 4 + i as u32 * 4));
                            self.emit("str r0, [r1]");
                            self.maybe_pool();
                        }
                    }
                }
                Init::Label(address, label, line) => {
                    self.loc(*line);
                    self.emit(format!("ldr r0, ={label}"));
                    self.emit(format!("ldr r1, ={address}"));
                    self.emit("str r0, [r1]");
                }
                Init::Value(g, address) => {
                    self.loc(g.line);
                    let ty = self.eval(&g.value, 0);
                    let expected = self.globals.get(&g.name).map(|g| g.ty).unwrap_or(ty);
                    if ty == Void {
                        self.err(&g.value, "this call returns nothing".into());
                    } else if ty != expected {
                        self.err(&g.value, format!("`{}` changes type", g.name));
                    }
                    self.emit(format!("ldr r1, ={address}"));
                    self.emit("str r0, [r1]");
                }
            }
            self.maybe_pool();
        }
        self.emit("pop {r4-r7}");
        self.emit("pop {r3}");
        self.emit("bx r3");
        self.code.push("    .pool".into());
        self.since_pool = 0;

        for f in &program.functions {
            self.function(f, stack_limit);
        }
        // Lifecycle hooks.
        for (hook, label) in [
            ("init", "rt_user_init"),
            ("update", "rt_user_update"),
            ("draw", "rt_user_draw"),
        ] {
            if self.functions.contains_key(hook) {
                self.code.push(format!("    .equ {label}, fn_{hook}"));
            } else {
                self.code.push("    .thumb_func".into());
                self.place(label);
                self.code.push("    bx lr".into());
            }
        }
        if !self.functions.contains_key("update") && !self.functions.contains_key("draw") {
            self.warnings.push(Diagnostic {
                file: self.file.to_string(),
                line: 1,
                column: 1,
                message: "the program has no `update` or `draw` function, so nothing happens after `init`".into(),
            });
        }
        // Setup routine: save data and print support.
        self.code.push("    .thumb_func".into());
        self.place("rt_setup");
        self.code.push("    push {lr}".into());
        if out.uses_save {
            self.code.push("    bl rt_save_load".into());
        }
        if out.uses_print {
            self.routines.insert("rt_setup_tty");
            self.code.push("    bl rt_setup_tty".into());
        }
        self.code.push("    pop {r3}".into());
        self.code.push("    bx r3".into());

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        // Assemble the full source.
        let mut source = String::new();
        source.push_str("; Generated by the Pocket Lua compiler.\n    .thumb\n");
        for (name, address) in &ram_symbols {
            source.push_str(&format!("    .equ {name}, {address:#x}\n"));
        }
        source.push_str("    .thumb_func\nmain:\n    ldr r1, =rt_start\n    bx r1\n    .pool\n");
        // Runtime routines with their dependencies.
        self.routines.insert("rt_start");
        self.routines.insert(if out.uses_sound {
            "rt_sound_frame"
        } else {
            "rt_sound_frame_silent"
        });
        if out.uses_link {
            self.routines.insert("rt_link");
        }
        let mut pending: Vec<&str> = self.routines.iter().copied().collect();
        let mut included: BTreeSet<&str> = BTreeSet::new();
        while let Some(name) = pending.pop() {
            if !included.insert(name) {
                continue;
            }
            if let Some(r) = runtime::routine(name) {
                pending.extend(r.needs.iter().copied());
            }
        }
        for name in &included {
            let routine = runtime::routine(name).expect("known routine");
            if *name == "rt_font" {
                source.push_str(&runtime::font_source());
            } else {
                source.push_str(routine.code);
            }
            source.push('\n');
        }

        for line in &self.code {
            source.push_str(line);
            source.push('\n');
        }
        source.push_str("    .pool\n    .align 2\n");
        for line in &self.data {
            source.push_str(line);
            source.push('\n');
        }
        if out.uses_save {
            source.push_str("    .balign 128\nrt_save_sector_data:\n    .space 128\n");
        }
        out.source = source;
        out.warnings = self.warnings;
        Ok(out)
    }

    fn array_length(&mut self, args: &[Expr]) -> u32 {
        match args.first() {
            Some(Expr {
                kind: ExprKind::Table(items),
                ..
            }) if args.len() == 1 => items.len() as u32,
            Some(e) if args.len() == 1 => match self.const_eval(e) {
                Some((n, Int)) if (1..=256).contains(&n) => n as u32,
                _ => {
                    self.err(e, "array(size) needs a constant size from 1 to 256".into());
                    1
                }
            },
            _ => 1,
        }
    }

    /// Flash or RAM data created by a literal: returns (type, (label, RAM bytes)).
    fn data_literal(&mut self, value: &Expr, name: &str) -> Option<(Ty, (String, u32))> {
        match &value.kind {
            ExprKind::Call(f, args) if f == "array" => {
                if args.len() != 1 {
                    self.err(value, "use `array(size)` or `array{1, 2, 3}`".into());
                    return Some((Array, (format!("ga_{name}"), 8)));
                }
                let length = self.array_length(args);
                Some((Array, (format!("ga_{name}"), (length + 1) * 4)))
            }
            ExprKind::Call(f, _) if f == "sprite" || f == "tune" => {
                let ty = self.peek_type(value);
                let label = self.literal_label(value)?;
                Some((ty, (label, 0)))
            }
            ExprKind::Table(_) | ExprKind::Str(_) => {
                let ty = self.peek_type(value);
                let label = self.literal_label(value)?;
                Some((ty, (label, 0)))
            }
            _ => None,
        }
    }

    /// Emits flash data for a literal and returns its label.
    fn literal_label(&mut self, value: &Expr) -> Option<String> {
        match &value.kind {
            ExprKind::Str(s) => Some(self.string(s)),
            ExprKind::Table(items) => {
                let label = self.label("table").replace(".L", "dt_");
                let mut words = vec![items.len().to_string()];
                for item in items {
                    match self.const_eval(item) {
                        Some((v, Int)) | Some((v, Bool)) => words.push((v as i32).to_string()),
                        _ => {
                            self.err(item, "table entries must be numbers known when compiling; use `array{...}` for values computed at run time".into());
                            words.push("0".into());
                        }
                    }
                }
                self.data.push("    .align 2".into());
                self.data.push(format!("{label}:"));
                for chunk in words.chunks(8) {
                    self.data.push(format!("    .word {}", chunk.join(", ")));
                }
                Some(label)
            }
            ExprKind::Call(f, args) if f == "sprite" => {
                let label = self.label("sprite").replace(".L", "ds_");
                let sprite = match args.as_slice() {
                    [
                        Expr {
                            kind: ExprKind::Str(text),
                            ..
                        },
                    ] if text.contains('\n')
                        || text.contains('#')
                        || text.contains('.') && !self.sprites.contains_key(text.as_str()) =>
                    {
                        match Sprite::parse(text) {
                            Ok(s) => s,
                            Err(m) => {
                                self.err(value, m);
                                return Some(label);
                            }
                        }
                    }
                    [
                        Expr {
                            kind: ExprKind::Str(name),
                            ..
                        },
                    ] => match self.sprites.get(name.as_str()) {
                        Some(s) => s.clone(),
                        None => {
                            let hint = suggest(name, self.sprites.keys().map(String::as_str));
                            self.err(value, format!("no sprite named `{name}` in this app{hint}"));
                            return Some(label);
                        }
                    },
                    _ => {
                        self.err(value, "use sprite [[ ... ]] with rows of `#` and `.`, or sprite \"name\" for a sprite drawn in the editor".into());
                        return Some(label);
                    }
                };
                self.data.push("    .align 2".into());
                self.data.push(format!("{label}:"));
                self.data.push(format!(
                    "    .word {}",
                    sprite.width | (sprite.height << 8) | ((sprite.frames.len() as u32) << 16)
                ));
                for frame in &sprite.frames {
                    for row in frame.iter().take(sprite.height as usize) {
                        self.data.push(format!("    .word {row:#x}"));
                    }
                }
                Some(label)
            }
            ExprKind::Call(f, args) if f == "tune" => {
                let label = self.label("tune").replace(".L", "dn_");
                let notes = match args.as_slice() {
                    [
                        Expr {
                            kind: ExprKind::Str(text),
                            ..
                        },
                    ] => match parse_tune(text) {
                        Ok(n) => n,
                        Err(m) => {
                            self.err(value, m);
                            Vec::new()
                        }
                    },
                    _ => {
                        self.err(
                            value,
                            "use tune \"C4:8 E4:8 G4:16\" (note, octave and length in frames)"
                                .into(),
                        );
                        Vec::new()
                    }
                };
                self.data.push("    .align 2".into());
                self.data.push(format!("{label}:"));
                self.data.push(format!("    .word {}", notes.len()));
                for (reload, frames) in notes {
                    self.data.push(format!("    .word {reload}, {frames}"));
                }
                Some(label)
            }
            _ => None,
        }
    }

    fn string(&mut self, text: &str) -> String {
        if let Some(label) = self.strings.get(text) {
            return label.clone();
        }
        let label = format!("dstr_{}", self.strings.len());
        self.data.push("    .align 2".into());
        self.data.push(format!("{label}:"));
        let mut halves = vec![text.chars().count().to_string()];
        halves.extend(text.chars().map(|c| (c as u32 & 0x7F).to_string()));
        for chunk in halves.chunks(12) {
            self.data.push(format!("    .hword {}", chunk.join(", ")));
        }
        self.strings.insert(text.to_string(), label.clone());
        label
    }

    /// Type of an expression without generating code (best effort for declarations).
    fn peek_type(&self, e: &Expr) -> Ty {
        match &e.kind {
            ExprKind::Int(_) => Int,
            ExprKind::Bool(_) => Bool,
            ExprKind::Str(_) => Str,
            ExprKind::Table(_) => Table,
            ExprKind::Nil => Int,
            ExprKind::Name(n) => self
                .lookup_local(n)
                .map(|l| l.ty)
                .or_else(|| self.globals.get(n).map(|g| g.ty))
                .unwrap_or(Int),
            ExprKind::Member(..) => Int,
            ExprKind::Index(..) => Int,
            ExprKind::Call(name, _) => match name.as_str() {
                "sprite" => Sprite,
                "tune" => Tune,
                "array" => Array,
                _ => self
                    .functions
                    .get(name)
                    .map(|s| s.ret)
                    .or_else(|| BUILTINS.iter().find(|b| b.name == name).map(|b| b.ret))
                    .unwrap_or(Int),
            },
            ExprKind::Unary(op, _) => {
                if *op == "not" {
                    Bool
                } else {
                    Int
                }
            }
            ExprKind::Binary(op, _, _) => match *op {
                "==" | "~=" | "<" | "<=" | ">" | ">=" | "and" | "or" => Bool,
                _ => Int,
            },
        }
    }

    /// Folds constant integer expressions.
    fn const_eval(&self, e: &Expr) -> Option<(i64, Ty)> {
        match &e.kind {
            ExprKind::Int(n) => Some((*n, Int)),
            ExprKind::Bool(v) => Some((*v as i64, Bool)),
            ExprKind::Name(n) => {
                if self.lookup_local(n).is_some() {
                    return None;
                }
                match self.globals.get(n) {
                    Some(Global {
                        kind: GlobalKind::Const(v),
                        ty,
                    }) => Some((*v, *ty)),
                    _ => None,
                }
            }
            ExprKind::Member(ns, field) => {
                let key = format!("{ns}.{field}");
                CONSTANTS
                    .iter()
                    .find(|(n, _)| *n == key)
                    .map(|(_, v)| (*v, Int))
            }
            ExprKind::Unary(op, inner) => {
                let (v, ty) = self.const_eval(inner)?;
                match *op {
                    "-" if ty == Int => Some((-v, Int)),
                    "~" if ty == Int => Some((!v, Int)),
                    "not" if ty == Bool => Some(((v == 0) as i64, Bool)),
                    _ => None,
                }
            }
            ExprKind::Binary(op, a, b) => {
                let (x, tx) = self.const_eval(a)?;
                let (y, ty) = self.const_eval(b)?;
                if tx != Int || ty != Int {
                    return None;
                }
                let x32 = x as i32;
                let y32 = y as i32;
                let v: i64 = match *op {
                    "+" => x32.wrapping_add(y32) as i64,
                    "-" => x32.wrapping_sub(y32) as i64,
                    "*" => x32.wrapping_mul(y32) as i64,
                    "//" if y32 != 0 => {
                        let q = x32.wrapping_div(y32);
                        let r = x32.wrapping_rem(y32);
                        (if r != 0 && ((r < 0) != (y32 < 0)) {
                            q - 1
                        } else {
                            q
                        }) as i64
                    }
                    "%" if y32 != 0 => {
                        let r = x32 % y32;
                        (if r != 0 && (r ^ y32) < 0 { r + y32 } else { r }) as i64
                    }
                    "&" => (x32 & y32) as i64,
                    "|" => (x32 | y32) as i64,
                    "~" => (x32 ^ y32) as i64,
                    "<<" => ((x32 as u32).wrapping_shl(y32 as u32)) as i32 as i64,
                    ">>" => ((x32 as u32).wrapping_shr(y32 as u32)) as i32 as i64,
                    _ => return None,
                };
                Some((v, Int))
            }
            _ => None,
        }
    }

    // ------------------------------------------------------------ functions

    fn lookup_local(&self, name: &str) -> Option<&Local> {
        self.scopes.iter().rev().find_map(|s| s.get(name))
    }

    fn declare(&mut self, name: &str, ty: Ty) -> u32 {
        let slot = self.slots;
        self.slots += 1;
        self.max_slots = self.max_slots.max(self.slots);
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), Local { slot, ty });
        slot
    }

    fn slot_offset(&self, slot: u32) -> u32 {
        slot * 4 + self.pushed
    }

    fn function(&mut self, f: &Function, stack_limit: u32) {
        let Some(signature) = self.functions.get(&f.name).cloned() else {
            return;
        };
        self.function_name = f.name.clone();
        self.scopes = vec![HashMap::new()];
        self.slots = 0;
        self.max_slots = 0;
        self.pushed = 0;
        self.loops.clear();
        self.error_stubs.clear();
        self.return_label = self.label("return");
        self.return_type = signature.ret;
        let mut params = Vec::new();
        for (p, ty) in f.params.iter().zip(&signature.params) {
            if self.lookup_local(&p.name).is_some() {
                self.error(p.line, 1, format!("parameter `{}` appears twice", p.name));
            }
            params.push(self.declare(&p.name, *ty));
        }
        // Body first, then wrap it with the prologue once the frame size is known.
        let body_start = self.code.len();
        self.line = 0;
        self.block(&f.body);
        let body: Vec<String> = self.code.drain(body_start..).collect();
        let frame = self.max_slots * 4;
        self.code.push("    .thumb_func".into());
        self.code
            .push(format!("    .loc \"{}\", {}", self.file, f.line));
        self.place(&signature.label);
        self.emit("push {r4-r7, lr}");
        self.emit(format!("ldr r4, ={stack_limit}"));
        self.emit("cmp sp, r4");
        let overflow = self.label("stack");
        self.emit(format!("blo {overflow}"));
        let mut remaining = frame;
        while remaining > 0 {
            let step = remaining.min(508);
            self.emit(format!("sub sp, #{step}"));
            remaining -= step;
        }
        for (i, slot) in params.iter().enumerate() {
            self.emit(format!("str r{i}, [sp, #{}]", slot * 4));
        }
        self.code.extend(body);
        if self.return_type != Void && !ends_with_return(&f.body) {
            // Falling off the end returns zero/false.
            self.emit("mov r0, #0");
        }
        self.code
            .push(format!("    .loc \"{}\", {}", self.file, f.end_line));
        self.place(&self.return_label.clone());
        let mut remaining = frame;
        while remaining > 0 {
            let step = remaining.min(508);
            self.emit(format!("add sp, #{step}"));
            remaining -= step;
        }
        self.emit("pop {r4-r7, pc}");
        self.place(&overflow);
        self.mov_imm(0, f.line as i64);
        self.mov_imm(1, runtime::ERROR_STACK as i64);
        self.routines.insert("rt_error");
        self.emit("bl rt_error");
        let stubs = std::mem::take(&mut self.error_stubs);
        for stub in stubs {
            self.code.push(stub);
        }
        self.code.push("    .pool".into());
        self.since_pool = 0;
        self.line = 0;
    }

    fn block(&mut self, statements: &[Stmt]) {
        self.scopes.push(HashMap::new());
        let saved_slots = self.slots;
        for s in statements {
            self.statement(s);
            self.maybe_pool();
        }
        self.slots = saved_slots;
        self.scopes.pop();
    }

    fn statement(&mut self, s: &Stmt) {
        self.loc(s.line);
        match &s.kind {
            StmtKind::Local(names, values) => {
                if !values.is_empty() && values.len() != names.len() {
                    self.error(
                        s.line,
                        1,
                        format!("{} names but {} values", names.len(), values.len()),
                    );
                    return;
                }
                let mut types = Vec::new();
                for v in values {
                    if let ExprKind::Call(name, _) = &v.kind
                        && (name == "array" || name == "sprite" || name == "tune")
                    {
                        self.err(
                            v,
                            format!("`{name}` values can only be created at the top of the file"),
                        );
                        types.push(Int);
                        continue;
                    }
                    if let ExprKind::Table(_) = &v.kind {
                        self.err(
                            v,
                            "tables can only be created at the top of the file".into(),
                        );
                        types.push(Int);
                        continue;
                    }
                    let ty = self.eval(v, 0);
                    if ty == Void {
                        self.err(v, "this call returns nothing".into());
                    }
                    self.push("r0", 1);
                    types.push(ty);
                }
                let slots: Vec<u32> = names
                    .iter()
                    .enumerate()
                    .map(|(i, n)| self.declare(n, types.get(i).copied().unwrap_or(Int)))
                    .collect();
                if values.is_empty() {
                    self.emit("mov r0, #0");
                    for slot in &slots {
                        let off = self.slot_offset(*slot);
                        self.emit(format!("str r0, [sp, #{off}]"));
                    }
                } else {
                    for slot in slots.iter().rev() {
                        self.pop("r0", 1);
                        let off = self.slot_offset(*slot);
                        self.emit(format!("str r0, [sp, #{off}]"));
                    }
                }
            }
            StmtKind::Assign(target, value) => self.assign(target, value, s.line),
            StmtKind::Call(e) => {
                self.eval(e, 0);
            }
            StmtKind::If(branches, otherwise) => {
                let end = self.label("endif");
                for (cond, body) in branches {
                    let next = self.label("else");
                    self.loc(cond.line);
                    self.condition(cond, &next, false);
                    self.block(body);
                    self.emit(format!("b {end}"));
                    self.place(&next);
                }
                if let Some(body) = otherwise {
                    self.block(body);
                }
                self.place(&end);
            }
            StmtKind::While(cond, body) => {
                let top = self.label("while");
                let end = self.label("wend");
                self.place(&top);
                self.condition(cond, &end, false);
                self.loops.push(end.clone());
                self.block(body);
                self.loops.pop();
                self.emit(format!("b {top}"));
                self.place(&end);
            }
            StmtKind::Repeat(body, cond) => {
                let top = self.label("repeat");
                let end = self.label("rend");
                self.place(&top);
                self.loops.push(end.clone());
                self.scopes.push(HashMap::new());
                let saved = self.slots;
                for st in body {
                    self.statement(st);
                    self.maybe_pool();
                }
                self.loc(cond.line);
                self.condition(cond, &top, false);
                self.slots = saved;
                self.scopes.pop();
                self.loops.pop();
                self.place(&end);
            }
            StmtKind::For(var, start, limit, step, body) => {
                self.for_loop(var, start, limit, step.as_ref(), body)
            }
            StmtKind::Break => match self.loops.last().cloned() {
                Some(end) => self.emit(format!("b {end}")),
                None => self.error(s.line, 1, "`break` outside a loop".into()),
            },
            StmtKind::Return(value) => {
                match value {
                    Some(v) => {
                        let ty = self.eval(v, 0);
                        if self.function_name == "(top level)" {
                            return;
                        }
                        if self.return_type == Void {
                            self.err(v, format!("`{}` returns nothing elsewhere; annotate it, e.g. `function {}(...): int`", self.function_name, self.function_name));
                        } else if ty != self.return_type {
                            self.err(
                                v,
                                format!(
                                    "`{}` returns a {} here but a {} elsewhere",
                                    self.function_name,
                                    ty.name(),
                                    self.return_type.name()
                                ),
                            );
                        }
                    }
                    None => {
                        if self.return_type != Void {
                            self.emit("mov r0, #0");
                        }
                    }
                }
                let label = self.return_label.clone();
                self.emit(format!("b {label}"));
            }
            StmtKind::Do(body) => self.block(body),
        }
    }

    fn for_loop(
        &mut self,
        var: &str,
        start: &Expr,
        limit: &Expr,
        step: Option<&Expr>,
        body: &[Stmt],
    ) {
        self.scopes.push(HashMap::new());
        let saved = self.slots;
        let step_value = match step {
            Some(e) => match self.const_eval(e) {
                Some((0, _)) => {
                    self.err(e, "a `for` step of 0 never ends".into());
                    Some(1)
                }
                Some((v, Int)) => Some(v),
                _ => None,
            },
            None => Some(1),
        };
        let counter = self.declare(&format!("({var} counter)"), Int);
        let end_slot = self.declare(&format!("({var} limit)"), Int);
        let step_slot = if step_value.is_none() {
            Some(self.declare(&format!("({var} step)"), Int))
        } else {
            None
        };
        self.expect_int(start, "the loop start");
        let off = self.slot_offset(counter);
        self.emit(format!("str r0, [sp, #{off}]"));
        self.expect_int(limit, "the loop limit");
        let off = self.slot_offset(end_slot);
        self.emit(format!("str r0, [sp, #{off}]"));
        if let (Some(slot), Some(e)) = (step_slot, step) {
            self.expect_int(e, "the loop step");
            let off = self.slot_offset(slot);
            self.emit(format!("str r0, [sp, #{off}]"));
        }
        let top = self.label("for");
        let next = self.label("fnext");
        let end = self.label("fend");
        self.place(&top);
        let off_counter = self.slot_offset(counter);
        let off_end = self.slot_offset(end_slot);
        self.emit(format!("ldr r0, [sp, #{off_counter}]"));
        self.emit(format!("ldr r1, [sp, #{off_end}]"));
        self.emit("cmp r0, r1");
        match step_value {
            Some(v) if v > 0 => self.emit(format!("bgt {end}")),
            Some(_) => self.emit(format!("blt {end}")),
            None => {
                let down = self.label("fdown");
                let go = self.label("fgo");
                let off_step = self.slot_offset(step_slot.unwrap());
                self.emit(format!("ldr r2, [sp, #{off_step}]"));
                self.emit("cmp r2, #0");
                self.emit(format!("blt {down}"));
                self.emit("cmp r0, r1");
                self.emit(format!("bgt {end}"));
                self.emit(format!("b {go}"));
                self.place(&down);
                self.emit("cmp r0, r1");
                self.emit(format!("blt {end}"));
                self.place(&go);
            }
        }
        // The visible variable is a copy, as in Lua.
        let visible = self.declare(var, Int);
        let off = self.slot_offset(visible);
        self.emit(format!("str r0, [sp, #{off}]"));
        self.loops.push(end.clone());
        self.block(body);
        self.loops.pop();
        self.place(&next);
        let off_counter = self.slot_offset(counter);
        self.emit(format!("ldr r0, [sp, #{off_counter}]"));
        match step_value {
            Some(v) => {
                self.mov_imm(1, v);
                self.emit("add r0, r0, r1");
            }
            None => {
                let off_step = self.slot_offset(step_slot.unwrap());
                self.emit(format!("ldr r1, [sp, #{off_step}]"));
                self.emit("add r0, r0, r1");
            }
        }
        self.emit(format!("str r0, [sp, #{off_counter}]"));
        self.emit(format!("b {top}"));
        self.place(&end);
        self.slots = saved;
        self.scopes.pop();
    }

    fn expect_int(&mut self, e: &Expr, what: &str) {
        let ty = self.eval(e, 0);
        if ty != Int {
            self.err(e, format!("{what} must be a number, not a {}", ty.name()));
        }
    }

    fn assign(&mut self, target: &Target, value: &Expr, line: u32) {
        match target {
            Target::Name(name) => {
                if let Some(local) = self.lookup_local(name) {
                    let (slot, ty) = (local.slot, local.ty);
                    let vt = self.eval(value, 0);
                    if vt != ty {
                        self.err(
                            value,
                            format!(
                                "`{name}` holds a {} but this value is a {}",
                                ty.name(),
                                vt.name()
                            ),
                        );
                    }
                    let off = self.slot_offset(slot);
                    self.emit(format!("str r0, [sp, #{off}]"));
                    return;
                }
                match self.globals.get(name).cloned() {
                    Some(Global {
                        ty,
                        kind: GlobalKind::Ram(address),
                    }) => {
                        let vt = self.eval(value, 0);
                        if vt != ty {
                            self.err(
                                value,
                                format!(
                                    "`{name}` holds a {} but this value is a {}",
                                    ty.name(),
                                    vt.name()
                                ),
                            );
                        }
                        self.emit(format!("ldr r1, ={address}"));
                        self.emit("str r0, [r1]");
                    }
                    Some(_) => self.error(
                        line,
                        1,
                        format!("`{name}` is a constant and cannot be assigned"),
                    ),
                    None => {
                        let hint = suggest(
                            name,
                            self.globals.keys().map(String::as_str).chain(
                                self.scopes
                                    .iter()
                                    .flat_map(|s| s.keys().map(String::as_str)),
                            ),
                        );
                        self.error(
                            line,
                            1,
                            format!(
                                "unknown variable `{name}`{hint}{}",
                                if hint.is_empty() {
                                    format!("; declare it first with `local {name} = 0`")
                                } else {
                                    String::new()
                                }
                            ),
                        );
                    }
                }
            }
            Target::Index(base, index) => {
                let bt = self.eval(base, 0);
                if bt == Table {
                    self.err(base, "this table is stored in flash and cannot be changed; declare it with `array{...}` to change its values".into());
                } else if bt != Array {
                    self.err(
                        base,
                        format!("only arrays can be indexed, not a {}", bt.name()),
                    );
                }
                self.expect_type(index, 1, Int, "an index");
                let vt = self.eval(value, 2);
                if vt != Int && vt != Bool {
                    self.err(value, format!("arrays hold numbers, not a {}", vt.name()));
                }
                let error = self.bounds_stub(base.line);
                self.emit("sub r1, #1");
                self.emit("ldr r3, [r0]");
                self.emit("cmp r1, r3");
                self.emit(format!("bhs {error}"));
                self.emit("lsl r1, r1, #2");
                self.emit("add r0, r0, r1");
                self.emit("str r2, [r0, #4]");
            }
        }
    }

    fn bounds_stub(&mut self, line: u32) -> String {
        let label = self.label("bounds");
        self.routines.insert("rt_error");
        self.error_stubs.push(format!(
            "{label}:\n    ldr r0, ={line}\n    mov r1, #{}\n    bl rt_error",
            runtime::ERROR_INDEX
        ));
        label
    }

    fn expect_type(&mut self, e: &Expr, r: R, ty: Ty, what: &str) {
        let actual = self.eval(e, r);
        if actual != ty && actual != Void {
            self.err(
                e,
                format!("{what} must be a {}, not a {}", ty.name(), actual.name()),
            );
        }
    }

    // ---------------------------------------------------------- conditions

    /// Branches to `target` when `e` evaluates to `when`.
    fn condition(&mut self, e: &Expr, target: &str, when: bool) {
        match &e.kind {
            ExprKind::Unary("not", inner) => self.condition(inner, target, !when),
            ExprKind::Binary("and", a, b) => {
                if when {
                    let skip = self.label("and");
                    self.condition(a, &skip, false);
                    self.condition(b, target, true);
                    self.place(&skip);
                } else {
                    self.condition(a, target, false);
                    self.condition(b, target, false);
                }
            }
            ExprKind::Binary("or", a, b) => {
                if when {
                    self.condition(a, target, true);
                    self.condition(b, target, true);
                } else {
                    let skip = self.label("or");
                    self.condition(a, &skip, true);
                    self.condition(b, target, false);
                    self.place(&skip);
                }
            }
            ExprKind::Binary(op @ ("==" | "~=" | "<" | "<=" | ">" | ">="), a, b) => {
                let ta = self.eval(a, 0);
                let small = match self.const_eval(b) {
                    Some((v, t))
                        if (0..=255).contains(&v) && (t == ta || (t == Int && ta == Int)) =>
                    {
                        Some(v)
                    }
                    _ => None,
                };
                if let Some(v) = small {
                    self.emit(format!("cmp r0, #{v}"));
                } else {
                    let tb = self.eval(b, 1);
                    if ta != tb {
                        self.err(
                            e,
                            format!("cannot compare a {} with a {}", ta.name(), tb.name()),
                        );
                    } else if !matches!(*op, "==" | "~=") && ta != Int {
                        self.err(e, format!("`{op}` compares numbers, not {}s", ta.name()));
                    }
                    self.emit("cmp r0, r1");
                }
                let cond = match (*op, when) {
                    ("==", true) | ("~=", false) => "eq",
                    ("~=", true) | ("==", false) => "ne",
                    ("<", true) | (">=", false) => "lt",
                    (">=", true) | ("<", false) => "ge",
                    ("<=", true) | (">", false) => "le",
                    (">", true) | ("<=", false) => "gt",
                    _ => unreachable!(),
                };
                self.emit(format!("b{cond} {target}"));
            }
            _ => {
                if let Some((v, Bool)) = self.const_eval(e) {
                    if (v != 0) == when {
                        self.emit(format!("b {target}"));
                    }
                    return;
                }
                let ty = self.eval(e, 0);
                if ty != Bool {
                    self.err(e, format!(
                        "conditions must be true or false, not a {}; compare it, e.g. `if x ~= 0 then`",
                        ty.name()
                    ));
                }
                self.emit("cmp r0, #0");
                self.emit(format!("b{} {target}", if when { "ne" } else { "eq" }));
            }
        }
    }

    // --------------------------------------------------------- expressions

    /// Evaluates `e` into register `r` (r0..r(r-1) hold live values).
    fn eval(&mut self, e: &Expr, r: R) -> Ty {
        if let Some((v, ty)) = self.const_eval(e) {
            self.mov_imm(r, v);
            return ty;
        }
        match &e.kind {
            ExprKind::Int(v) => {
                self.mov_imm(r, *v);
                Int
            }
            ExprKind::Bool(v) => {
                self.mov_imm(r, *v as i64);
                Bool
            }
            ExprKind::Nil => {
                self.err(e, "`nil` is not supported; use 0 or false".into());
                Int
            }
            ExprKind::Str(s) => {
                let label = self.string(s);
                self.emit(format!("ldr r{r}, ={label}"));
                Str
            }
            ExprKind::Table(_) => {
                self.err(
                    e,
                    "tables can only be created at the top of the file".into(),
                );
                Table
            }
            ExprKind::Name(name) => {
                if let Some(local) = self.lookup_local(name) {
                    let (slot, ty) = (local.slot, local.ty);
                    let off = self.slot_offset(slot);
                    self.emit(format!("ldr r{r}, [sp, #{off}]"));
                    return ty;
                }
                match self.globals.get(name).cloned() {
                    Some(Global {
                        ty,
                        kind: GlobalKind::Ram(address),
                    }) => {
                        self.emit(format!("ldr r{r}, ={address}"));
                        self.emit(format!("ldr r{r}, [r{r}]"));
                        ty
                    }
                    Some(Global {
                        ty,
                        kind: GlobalKind::Label(label),
                    }) => {
                        self.emit(format!("ldr r{r}, ={label}"));
                        ty
                    }
                    Some(Global {
                        ty,
                        kind: GlobalKind::Const(v),
                    }) => {
                        self.mov_imm(r, v);
                        ty
                    }
                    None => {
                        if self.functions.contains_key(name) {
                            self.err(
                                e,
                                format!("`{name}` is a function; call it with `{name}()`"),
                            );
                        } else if NAMESPACES.contains(&name.as_str()) {
                            self.err(e, format!("`{name}` is a library; use `{name}.something`"));
                        } else {
                            let hint = suggest(
                                name,
                                self.globals.keys().map(String::as_str).chain(
                                    self.scopes
                                        .iter()
                                        .flat_map(|s| s.keys().map(String::as_str)),
                                ),
                            );
                            self.err(e, format!("unknown variable `{name}`{hint}"));
                        }
                        Int
                    }
                }
            }
            ExprKind::Member(ns, field) => {
                let key = format!("{ns}.{field}");
                if BUILTINS.iter().any(|b| b.name == key) {
                    self.err(e, format!("`{key}` is a function; call it with `{key}()`"));
                } else if NAMESPACES.contains(&ns.as_str()) {
                    let hint = suggest(
                        &key,
                        CONSTANTS
                            .iter()
                            .map(|(n, _)| *n)
                            .chain(BUILTINS.iter().map(|b| b.name)),
                    );
                    self.err(e, format!("unknown `{key}`{hint}"));
                } else {
                    self.err(
                        e,
                        format!(
                            "`{ns}` is not a library; libraries are {}",
                            NAMESPACES.join(", ")
                        ),
                    );
                }
                Int
            }
            ExprKind::Index(base, index) => self.index(e, base, index, r),
            ExprKind::Call(name, args) => self.call(e, name, args, r),
            ExprKind::Unary(op, inner) => match *op {
                "not" => self.materialize(e, r),
                "-" => {
                    let ty = self.eval(inner, r);
                    self.require_int(inner, ty, "`-`");
                    self.emit(format!("neg r{r}, r{r}"));
                    Int
                }
                "~" => {
                    let ty = self.eval(inner, r);
                    self.require_int(inner, ty, "`~`");
                    self.emit(format!("mvn r{r}, r{r}"));
                    Int
                }
                _ => {
                    let ty = self.eval(inner, r);
                    match ty {
                        Str => self.emit(format!("ldrh r{r}, [r{r}]")),
                        Array | Table | Tune => self.emit(format!("ldr r{r}, [r{r}]")),
                        Sprite => {
                            self.emit(format!("ldr r{r}, [r{r}]"));
                            self.emit(format!("lsr r{r}, r{r}, #16"));
                        }
                        _ => self.err(
                            e,
                            format!(
                                "`#` measures tables, arrays, strings and sprites, not a {}",
                                ty.name()
                            ),
                        ),
                    }
                    Int
                }
            },
            ExprKind::Binary(op, a, b) => match *op {
                "==" | "~=" | "<" | "<=" | ">" | ">=" | "and" | "or" => self.materialize(e, r),
                "/" => {
                    self.err(
                        e,
                        "`/` would produce a fraction; use `//` for whole-number division".into(),
                    );
                    Int
                }
                "//" | "%" => self.divide(e, op, a, b, r),
                _ => self.arithmetic(e, op, a, b, r),
            },
        }
    }

    fn require_int(&mut self, e: &Expr, ty: Ty, what: &str) {
        if ty != Int {
            self.err(e, format!("{what} needs a number, not a {}", ty.name()));
        }
    }

    /// Turns a condition into 0/1 in register `r`.
    fn materialize(&mut self, e: &Expr, r: R) -> Ty {
        self.fresh(r, |g| {
            let yes = g.label("true");
            let end = g.label("bool");
            g.condition(e, &yes, true);
            g.emit("mov r0, #0");
            g.emit(format!("b {end}"));
            g.place(&yes);
            g.emit("mov r0, #1");
            g.place(&end);
            Bool
        })
    }

    /// Runs `f` (which computes into r0 and may use every register) with the
    /// live registers r0..r(r-1) saved, then moves its result to `r`.
    fn fresh(&mut self, r: R, f: impl FnOnce(&mut Self) -> Ty) -> Ty {
        if r > 0 {
            self.push(&format!("r0-r{}", r - 1), r);
        }
        let ty = f(self);
        if r > 0 {
            self.emit(format!("mov r{r}, r0"));
            self.pop(&format!("r0-r{}", r - 1), r);
        }
        ty
    }

    fn arithmetic(&mut self, e: &Expr, op: &str, a: &Expr, b: &Expr, r: R) -> Ty {
        let small = self
            .const_eval(b)
            .filter(|(v, t)| *t == Int && (0..=255).contains(v))
            .map(|(v, _)| v);
        if r >= 6 {
            // Out of scratch registers: evaluate the right side first and keep it on the stack.
            let tb = self.eval(b, r);
            self.push(&format!("r{r}"), 1);
            let ta = self.eval(a, r);
            self.pop(&format!("r{}", r + 1), 1);
            self.check_arith(e, op, ta, tb);
            self.binary_op(op, r, r + 1);
            return Int;
        }
        let ta = self.eval(a, r);
        if let (Some(v), "+" | "-" | "<<" | ">>") = (small, op) {
            self.check_arith(e, op, ta, Int);
            match op {
                "+" => self.emit(format!("add r{r}, #{v}")),
                "-" => self.emit(format!("sub r{r}, #{v}")),
                "<<" => self.emit(format!("lsl r{r}, r{r}, #{}", v.min(31))),
                _ => {
                    if v >= 32 {
                        self.emit(format!("mov r{r}, #0"));
                    } else {
                        self.emit(format!("lsr r{r}, r{r}, #{v}"));
                    }
                }
            }
            return Int;
        }
        let tb = self.eval(b, r + 1);
        self.check_arith(e, op, ta, tb);
        self.binary_op(op, r, r + 1);
        Int
    }

    fn check_arith(&mut self, e: &Expr, op: &str, ta: Ty, tb: Ty) {
        if ta != Int || tb != Int {
            let other = if ta != Int { ta } else { tb };
            self.err(
                e,
                format!("`{op}` works on numbers, not a {}", other.name()),
            );
        }
    }

    fn binary_op(&mut self, op: &str, d: R, s: R) {
        match op {
            "+" => self.emit(format!("add r{d}, r{d}, r{s}")),
            "-" => self.emit(format!("sub r{d}, r{d}, r{s}")),
            "*" => self.emit(format!("mul r{d}, r{s}")),
            "&" => self.emit(format!("and r{d}, r{s}")),
            "|" => self.emit(format!("orr r{d}, r{s}")),
            "~" => self.emit(format!("eor r{d}, r{s}")),
            "<<" => self.emit(format!("lsl r{d}, r{s}")),
            ">>" => self.emit(format!("lsr r{d}, r{s}")),
            _ => {}
        }
    }

    fn divide(&mut self, e: &Expr, op: &str, a: &Expr, b: &Expr, r: R) -> Ty {
        // Powers of two need no division routine.
        if let Some((v, Int)) = self.const_eval(b)
            && v > 0
            && (v & (v - 1)) == 0
            && r < 7
        {
            let ta = self.eval(a, r);
            self.check_arith(e, op, ta, Int);
            let shift = v.trailing_zeros();
            if op == "//" {
                if shift > 0 {
                    self.emit(format!("asr r{r}, r{r}, #{shift}"));
                }
            } else {
                self.mov_imm(r + 1, v - 1);
                self.emit(format!("and r{r}, r{}", r + 1));
            }
            return Int;
        }
        self.routines.insert("rt_idiv");
        let result = if op == "//" { 0 } else { 1 };
        self.call_routine(
            e,
            "rt_idiv",
            &[a, b],
            &[e.line as i64],
            r,
            result,
            &[Int, Int],
            op,
        );
        Int
    }

    fn index(&mut self, e: &Expr, base: &Expr, index: &Expr, r: R) -> Ty {
        if r > 5 {
            // Registers are scarce: compute from r0 with everything live saved.
            return self.fresh(r, |g| g.index(e, base, index, 0));
        }
        let bt = self.eval(base, r);
        if bt != Array && bt != Table {
            self.err(
                base,
                format!("only tables and arrays can be indexed, not a {}", bt.name()),
            );
        }
        self.expect_type(index, r + 1, Int, "an index");
        let error = self.bounds_stub(e.line);
        let (b, i, t) = (r, r + 1, r + 2);
        self.emit(format!("sub r{i}, #1"));
        self.emit(format!("ldr r{t}, [r{b}]"));
        self.emit(format!("cmp r{i}, r{t}"));
        self.emit(format!("bhs {error}"));
        self.emit(format!("lsl r{i}, r{i}, #2"));
        self.emit(format!("add r{b}, r{b}, r{i}"));
        self.emit(format!("ldr r{b}, [r{b}, #4]"));
        Int
    }

    fn call(&mut self, e: &Expr, name: &str, args: &[Expr], r: R) -> Ty {
        if name == "print" {
            return self.print(e, args, r);
        }
        if name == "sprite" || name == "tune" || name == "array" {
            self.err(
                e,
                format!("`{name}` values can only be created at the top of the file"),
            );
            return Int;
        }
        if let Some(signature) = self.functions.get(name).cloned() {
            if args.len() != signature.params.len() {
                self.err(
                    e,
                    format!(
                        "`{name}` takes {} argument{} but {} {} given",
                        signature.params.len(),
                        if signature.params.len() == 1 { "" } else { "s" },
                        args.len(),
                        if args.len() == 1 { "was" } else { "were" }
                    ),
                );
            }
            let refs: Vec<&Expr> = args.iter().collect();
            self.call_routine(
                e,
                &signature.label,
                &refs,
                &[],
                r,
                0,
                &signature.params,
                name,
            );
            return signature.ret;
        }
        if self.externs.iter().any(|n| n == name) {
            if args.len() > 4 {
                self.err(e, format!("`{name}` can take at most 4 arguments"));
                return Int;
            }
            let refs: Vec<&Expr> = args.iter().collect();
            let types = vec![Int; args.len()];
            self.call_routine(e, name, &refs, &[], r, 0, &types, name);
            return Int;
        }
        if let Some(builtin) = BUILTINS.iter().find(|b| b.name == name) {
            let required = builtin.params.len();
            let allowed = required + builtin.optional.len();
            if args.len() < required || args.len() > allowed {
                let count = if required == allowed {
                    required.to_string()
                } else {
                    format!("{required} to {allowed}")
                };
                self.err(
                    e,
                    format!(
                        "`{name}` takes {count} arguments but {} were given",
                        args.len()
                    ),
                );
                return builtin.ret;
            }
            self.routines.insert(builtin.include);
            let mut types: Vec<Ty> = builtin.params.to_vec();
            types.extend(builtin.optional.iter().map(|(t, _)| *t));
            let refs: Vec<&Expr> = args.iter().collect();
            let mut extra: Vec<i64> = builtin.optional[args.len().saturating_sub(required)..]
                .iter()
                .map(|(_, v)| *v)
                .collect();
            if builtin.line {
                extra.push(e.line as i64);
            }
            self.call_routine(e, builtin.routine, &refs, &extra, r, 0, &types, name);
            return builtin.ret;
        }
        let hint = if let Some((ns, _)) = name.split_once('.') {
            if NAMESPACES.contains(&ns) {
                suggest(name, BUILTINS.iter().map(|b| b.name))
            } else {
                format!("; libraries are {}", NAMESPACES.join(", "))
            }
        } else {
            suggest(
                name,
                self.functions.keys().map(String::as_str).chain(["print"]),
            )
        };
        self.err(e, format!("unknown function `{name}`{hint}"));
        Int
    }

    fn print(&mut self, e: &Expr, args: &[Expr], r: R) -> Ty {
        if args.len() != 1 {
            self.err(e, "`print` takes one value".into());
            return Void;
        }
        self.routines.insert("rt_tty");
        let ty = self.peek_type(&args[0]);
        let routine = match ty {
            Str => "rt_print_string",
            Bool => {
                self.fresh(r, |g| {
                    g.eval(&args[0], 0);
                    let yes = g.string("true");
                    let no = g.string("false");
                    let done = g.label("pb");
                    g.emit("cmp r0, #0");
                    g.emit(format!("ldr r0, ={no}"));
                    g.emit(format!("beq {done}"));
                    g.emit(format!("ldr r0, ={yes}"));
                    g.place(&done);
                    g.emit("bl rt_print_string");
                    Void
                });
                return Void;
            }
            _ => "rt_print_int",
        };
        self.call_routine(e, routine, &[&args[0]], &[], r, 0, &[ty], "print");
        Void
    }

    /// Calls `label` with `args` (and constant `extra` arguments) in r0-r3,
    /// leaving result register `result` (0 or 1) in `r`.
    #[allow(clippy::too_many_arguments)]
    fn call_routine(
        &mut self,
        e: &Expr,
        label: &str,
        args: &[&Expr],
        extra: &[i64],
        r: R,
        result: R,
        types: &[Ty],
        name: &str,
    ) {
        let total = args.len() + extra.len();
        if total > 4 {
            self.err(e, "too many arguments".into());
            return;
        }
        let live = r.min(4);
        if live > 0 {
            self.push(&format!("r0-r{}", live - 1), live);
        }
        let leaf = args.iter().all(|a| is_leaf(a));
        let mut actual = Vec::new();
        if leaf {
            for (i, a) in args.iter().enumerate() {
                actual.push(self.eval(a, i as R));
            }
        } else {
            for a in args.iter().rev() {
                let base = r.max(live);
                let ty = self.eval(a, base.min(6));
                actual.push(ty);
                self.push(&format!("r{}", base.min(6)), 1);
            }
            actual.reverse();
            if !args.is_empty() {
                let count = args.len() as u32;
                self.pop(&format!("r0-r{}", count - 1), count);
            }
        }
        for (i, v) in extra.iter().enumerate() {
            self.mov_imm((args.len() + i) as R, *v);
        }
        for (i, (got, want)) in actual.iter().zip(types).enumerate() {
            if got != want
                && *got != Void
                && let Some(a) = args.get(i)
            {
                self.err(
                    a,
                    format!(
                        "argument {} of `{name}` must be a {}, not a {}",
                        i + 1,
                        want.name(),
                        got.name()
                    ),
                );
            }
            if *got == Void
                && let Some(a) = args.get(i)
            {
                self.err(a, "this call returns nothing".into());
            }
        }
        self.emit(format!("bl {label}"));
        if r != result {
            self.emit(format!("mov r{r}, r{result}"));
        }
        if live > 0 {
            self.pop(&format!("r0-r{}", live - 1), live);
        }
    }
}

fn is_leaf(e: &Expr) -> bool {
    matches!(
        e.kind,
        ExprKind::Int(_)
            | ExprKind::Bool(_)
            | ExprKind::Name(_)
            | ExprKind::Str(_)
            | ExprKind::Member(..)
    )
}

fn ends_with_return(body: &[Stmt]) -> bool {
    matches!(body.last().map(|s| &s.kind), Some(StmtKind::Return(_)))
}

/// Return type from the function's `return` statements.
fn infer_return(body: &[Stmt]) -> Ty {
    fn visit(statements: &[Stmt], out: &mut Option<Ty>) {
        for s in statements {
            match &s.kind {
                StmtKind::Return(Some(e)) => {
                    let ty = match &e.kind {
                        ExprKind::Bool(_) => Bool,
                        ExprKind::Unary("not", _) => Bool,
                        ExprKind::Binary(op, _, _)
                            if matches!(
                                *op,
                                "==" | "~=" | "<" | "<=" | ">" | ">=" | "and" | "or"
                            ) =>
                        {
                            Bool
                        }
                        ExprKind::Call(name, _)
                            if BUILTINS.iter().any(|b| b.name == name && b.ret == Bool) =>
                        {
                            Bool
                        }
                        _ => Int,
                    };
                    if out.is_none() || (*out == Some(Int) && ty == Bool) {
                        *out = Some(ty);
                    }
                }
                StmtKind::If(branches, otherwise) => {
                    for (_, b) in branches {
                        visit(b, out);
                    }
                    if let Some(b) = otherwise {
                        visit(b, out);
                    }
                }
                StmtKind::While(_, b)
                | StmtKind::Repeat(b, _)
                | StmtKind::Do(b)
                | StmtKind::For(_, _, _, _, b) => visit(b, out),
                _ => {}
            }
        }
    }
    let mut out = None;
    visit(body, &mut out);
    out.unwrap_or(Void)
}

/// Names assigned anywhere inside functions.
fn assigned_names(program: &Program) -> BTreeSet<String> {
    fn visit(statements: &[Stmt], out: &mut BTreeSet<String>) {
        for s in statements {
            match &s.kind {
                StmtKind::Assign(Target::Name(n), _) => {
                    out.insert(n.clone());
                }
                StmtKind::If(branches, otherwise) => {
                    for (_, b) in branches {
                        visit(b, out);
                    }
                    if let Some(b) = otherwise {
                        visit(b, out);
                    }
                }
                StmtKind::While(_, b)
                | StmtKind::Repeat(b, _)
                | StmtKind::Do(b)
                | StmtKind::For(_, _, _, _, b) => visit(b, out),
                _ => {}
            }
        }
    }
    let mut out = BTreeSet::new();
    for f in &program.functions {
        visit(&f.body, &mut out);
    }
    out
}

/// Every function name called anywhere in the program.
fn used_calls(program: &Program) -> BTreeSet<String> {
    fn expr(e: &Expr, out: &mut BTreeSet<String>) {
        match &e.kind {
            ExprKind::Call(name, args) => {
                out.insert(name.clone());
                for a in args {
                    expr(a, out);
                }
            }
            ExprKind::Index(a, b) | ExprKind::Binary(_, a, b) => {
                expr(a, out);
                expr(b, out);
            }
            ExprKind::Unary(_, a) => expr(a, out),
            ExprKind::Table(items) => items.iter().for_each(|i| expr(i, out)),
            _ => {}
        }
    }
    fn stmts(statements: &[Stmt], out: &mut BTreeSet<String>) {
        for s in statements {
            match &s.kind {
                StmtKind::Local(_, values) => values.iter().for_each(|v| expr(v, out)),
                StmtKind::Assign(t, v) => {
                    if let Target::Index(a, b) = t {
                        expr(a, out);
                        expr(b, out);
                    }
                    expr(v, out);
                }
                StmtKind::Call(e) => expr(e, out),
                StmtKind::If(branches, otherwise) => {
                    for (c, b) in branches {
                        expr(c, out);
                        stmts(b, out);
                    }
                    if let Some(b) = otherwise {
                        stmts(b, out);
                    }
                }
                StmtKind::While(c, b) | StmtKind::Repeat(b, c) => {
                    expr(c, out);
                    stmts(b, out);
                }
                StmtKind::For(_, a, b, c, body) => {
                    expr(a, out);
                    expr(b, out);
                    if let Some(c) = c {
                        expr(c, out);
                    }
                    stmts(body, out);
                }
                StmtKind::Return(Some(e)) => expr(e, out),
                StmtKind::Do(b) => stmts(b, out),
                _ => {}
            }
        }
    }
    let mut out = BTreeSet::new();
    for g in &program.globals {
        expr(&g.value, &mut out);
    }
    for f in &program.functions {
        stmts(&f.body, &mut out);
    }
    out
}

/// Parses tune notation: `C4:8 D#4:8 R:4` (note, optional sharp/flat, octave, frames).
pub fn parse_tune(text: &str) -> Result<Vec<(u32, u32)>, String> {
    let mut notes = Vec::new();
    for word in text.split_whitespace() {
        let (note, frames) = match word.split_once(':') {
            Some((n, f)) => (
                n,
                f.parse::<u32>()
                    .map_err(|_| format!("bad length in `{word}`"))?,
            ),
            None => (word, 8),
        };
        if frames == 0 || frames > 255 {
            return Err(format!("note `{word}` must last 1 to 255 frames"));
        }
        let upper = note.to_ascii_uppercase();
        if upper == "R" || upper == "-" {
            notes.push((0, frames));
            continue;
        }
        let mut chars = upper.chars();
        let letter = chars.next().ok_or("empty note")?;
        let base = match letter {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => {
                return Err(format!(
                    "unknown note `{word}` (use C D E F G A B or R for a rest)"
                ));
            }
        };
        let rest: String = chars.collect();
        let (shift, octave) = if let Some(o) = rest.strip_prefix('#') {
            (1, o)
        } else if let Some(o) = rest.strip_prefix('B') {
            (-1, o)
        } else {
            (0, rest.as_str())
        };
        let octave: i32 = octave
            .parse()
            .map_err(|_| format!("note `{word}` needs an octave, e.g. C4"))?;
        if !(1..=8).contains(&octave) {
            return Err(format!("octave of `{word}` must be 1 to 8"));
        }
        let semitone = (octave + 1) * 12 + base + shift;
        let frequency = 440.0 * 2f64.powf((semitone - 69) as f64 / 12.0);
        let reload = (999_424.0 / frequency).round() as u32 - 1;
        notes.push((reload.clamp(1, 0xFFFF), frames));
    }
    if notes.len() > 1024 {
        return Err("tunes hold at most 1024 notes".into());
    }
    Ok(notes)
}
