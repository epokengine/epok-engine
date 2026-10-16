//! Pocket Lua: a statically typed subset of Lua compiled to Thumb code for
//! the PocketStation by Epok itself (no external toolchain).
//!
//! A program declares variables and functions at the top level and lets the
//! runtime call `init()` once, then `update()` and `draw()` every frame
//! (30 per second by default). Numbers are 32-bit integers; sprites, tunes and
//! tables are created at the top of the file and live in flash; arrays live in
//! RAM. Libraries: `screen`, `button`, `sound`, `clock`, `save`, `link`,
//! `system`, `math` and `print`.

pub mod codegen;
mod lexer;
mod parser;
pub mod runtime;
#[cfg(test)]
mod tests;

use crate::asm::{self, Assembly};
use crate::file::{self, Header, LinkFunction};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.file, self.line, self.column, self.message
        )
    }
}

/// A monochrome sprite: up to 32x32 pixels per frame, bit 0 = left, 1 = black.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<[u32; 32]>,
}

impl Sprite {
    /// Parses rows of `#` (black) and `.` (white); a blank line starts a new
    /// frame. Indentation around each row is ignored.
    pub fn parse(text: &str) -> Result<Sprite, String> {
        let mut frames: Vec<Vec<u32>> = vec![Vec::new()];
        let mut width = 0u32;
        for raw in text.lines() {
            let row = raw.trim();
            if row.is_empty() {
                if !frames.last().unwrap().is_empty() {
                    frames.push(Vec::new());
                }
                continue;
            }
            let mut bits = 0u32;
            let mut count = 0u32;
            for c in row.chars() {
                match c {
                    '#' | 'X' | 'x' | '@' | '*' | '1' => bits |= 1 << count.min(31),
                    '.' | '0' | '_' | '-' => {}
                    ' ' => {
                        return Err("use `.` for white pixels inside a sprite row (spaces are only allowed for indentation)".into());
                    }
                    other => {
                        return Err(format!("unexpected `{other}` in sprite (use `#` and `.`)"));
                    }
                }
                count += 1;
            }
            if count > 32 {
                return Err(format!(
                    "sprite rows can be 32 pixels wide; this one has {count}"
                ));
            }
            width = width.max(count);
            frames.last_mut().unwrap().push(bits);
        }
        if frames.last().is_some_and(|f| f.is_empty()) {
            frames.pop();
        }
        if frames.is_empty() {
            return Err("the sprite is empty".into());
        }
        let height = frames.iter().map(|f| f.len()).max().unwrap_or(0) as u32;
        if height > 32 {
            return Err(format!(
                "sprites can be 32 pixels tall; this one has {height}"
            ));
        }
        if frames.len() > 255 {
            return Err("sprites can have at most 255 frames".into());
        }
        let frames = frames
            .into_iter()
            .map(|rows| {
                let mut image = [0u32; 32];
                for (i, r) in rows.into_iter().enumerate() {
                    image[i] = r;
                }
                image
            })
            .collect();
        Ok(Sprite {
            width,
            height,
            frames,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Name used in diagnostics and the debugger's line map.
    pub file: String,
    /// Sprites drawn in the editor, available as `sprite "name"`.
    pub sprites: BTreeMap<String, Sprite>,
    /// Assembly sources added to the program: (file name, text). Their
    /// `.global` Thumb functions can be called from Pocket Lua.
    pub assembly: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub assembly: Assembly,
    /// The generated assembly text (for inspection).
    pub source: String,
    pub entry: u32,
    pub functions: Vec<LinkFunction>,
    /// First RAM byte not used by variables; the stack grows down from 800h.
    pub ram_end: u32,
    pub variables: Vec<(String, u32, &'static str)>,
    pub warnings: Vec<Diagnostic>,
    pub uses_link: bool,
    pub uses_save: bool,
    pub uses_sound: bool,
    pub uses_print: bool,
}

impl Program {
    pub fn stack_bytes(&self) -> u32 {
        0x800u32.saturating_sub(self.ram_end)
    }
    /// The Pocket Lua line for a program address, if it is in compiled code.
    pub fn line_at(&self, address: u32, file: &str) -> Option<u32> {
        let index = self
            .assembly
            .lines
            .partition_point(|l| l.address <= address);
        let info = self.assembly.lines.get(index.checked_sub(1)?)?;
        (info.file == file).then_some(info.line)
    }
}

/// Link functions provided by the runtime (FUNC 80h-83h).
pub const LINK_FUNCTIONS: [(&str, u32); 4] = [
    ("rt_link_receive", 1),
    ("rt_link_send", 0),
    ("rt_link_values", 0),
    ("rt_link_value", 1),
];

/// Global function names declared in assembly sources.
fn assembly_globals(sources: &[(String, String)]) -> Vec<String> {
    let mut out = Vec::new();
    for (_, text) in sources {
        for line in text.lines() {
            let line = line.trim();
            let lower = line.to_ascii_lowercase();
            if let Some(rest) = lower
                .strip_prefix(".global")
                .or_else(|| lower.strip_prefix(".globl"))
            {
                for name in rest.split(',') {
                    let name = name.trim();
                    if !name.is_empty() {
                        // Keep the original spelling.
                        let start = line.to_ascii_lowercase().find(name).unwrap_or(0);
                        out.push(line[start..start + name.len()].to_string());
                    }
                }
            }
        }
    }
    out
}

/// Compiles a Pocket Lua program for the executable described by `header`
/// (whose function count is set here).
pub fn compile(
    source: &str,
    header: &mut Header,
    options: &Options,
) -> Result<Program, Vec<Diagnostic>> {
    let file = if options.file.is_empty() {
        "main.plua"
    } else {
        options.file.as_str()
    };
    let tokens = lexer::tokenize(source, file).map_err(|d| vec![d])?;
    let ast = parser::parse(&tokens, file).map_err(|d| vec![d])?;
    let externs = assembly_globals(&options.assembly);
    let mut generator = codegen::Generator::new(file, &options.sprites);
    generator.set_externs(&externs);
    let output = generator.program(&ast)?;
    let mut text = output.source.clone();
    for (name, source) in &options.assembly {
        text.push_str(&format!(
            "\n    .loc \"{name}\", 1\n    .thumb\n    .align 2\n"
        ));
        // Keep each file's own line numbers in diagnostics.
        for (i, line) in source.lines().enumerate() {
            text.push_str(&format!("    .loc \"{name}\", {}\n{line}\n", i + 1));
        }
        text.push_str("\n    .pool\n");
    }
    header.function_count = if output.uses_link {
        LINK_FUNCTIONS.len()
    } else {
        0
    };
    let assembly = asm::assemble(
        &text,
        &asm::Options {
            base: header.body_address(),
            thumb: true,
            file: "pocket-runtime.s".into(),
            ..Default::default()
        },
    )
    .map_err(|errors| {
        errors
            .into_iter()
            .map(|e| Diagnostic {
                file: e.file,
                line: e.line,
                column: 1,
                message: e.message,
            })
            .collect::<Vec<_>>()
    })?;
    let functions = if output.uses_link {
        LINK_FUNCTIONS
            .iter()
            .map(|(name, parameters)| LinkFunction {
                parameters: *parameters,
                address: assembly.symbol(name).unwrap_or(0),
            })
            .collect()
    } else {
        Vec::new()
    };
    let entry = assembly.symbol("main").unwrap_or(header.body_address() | 1);
    Ok(Program {
        entry,
        functions,
        ram_end: output.ram_end,
        variables: output.variables,
        warnings: output.warnings,
        uses_link: output.uses_link,
        uses_save: output.uses_save,
        uses_sound: output.uses_sound,
        uses_print: output.uses_print,
        source: text,
        assembly,
    })
}

/// Assembles a program written only in assembly. The entry point is `main`
/// (a `.thumb_func` label for Thumb code). Link functions are the labels
/// `link_80`, `link_81`, ... with parameter counts given by `.equ link_80_parameters, n`.
pub fn assemble_program(
    sources: &[(String, String)],
    header: &mut Header,
) -> Result<Program, Vec<Diagnostic>> {
    let mut text = String::new();
    for (name, source) in sources {
        for (i, line) in source.lines().enumerate() {
            text.push_str(&format!("    .loc \"{name}\", {}\n{line}\n", i + 1));
        }
        text.push_str("\n    .pool\n");
    }
    // Count link functions first (they size the header).
    let count = (0x80..0x100)
        .take_while(|n| {
            text.contains(&format!("link_{n:02x}:")) || text.contains(&format!("link_{n:02X}:"))
        })
        .count();
    header.function_count = count;
    let to_diagnostics = |errors: Vec<asm::Diagnostic>| -> Vec<Diagnostic> {
        errors
            .into_iter()
            .map(|e| Diagnostic {
                file: e.file,
                line: e.line,
                column: 1,
                message: e.message,
            })
            .collect()
    };
    let assembly = asm::assemble(
        &text,
        &asm::Options {
            base: header.body_address(),
            thumb: false,
            file: "program.s".into(),
            ..Default::default()
        },
    )
    .map_err(to_diagnostics)?;
    let entry = assembly.symbol("main").ok_or_else(|| {
        vec![Diagnostic {
            file: sources.first().map(|s| s.0.clone()).unwrap_or_default(),
            line: 1,
            column: 1,
            message: "the program needs a `main:` label where it starts (mark it `.thumb_func` for Thumb code)".into(),
        }]
    })?;
    let mut functions = Vec::new();
    for n in 0x80..0x80 + count as u32 {
        let lower = format!("link_{n:02x}");
        let upper = format!("link_{n:02X}");
        let address = assembly
            .symbol(&lower)
            .or_else(|| assembly.symbol(&upper))
            .unwrap_or(0);
        let parameters = assembly
            .symbol(&format!("{lower}_parameters"))
            .or_else(|| assembly.symbol(&format!("{upper}_parameters")))
            .unwrap_or(0);
        functions.push(LinkFunction {
            parameters,
            address,
        });
    }
    Ok(Program {
        entry,
        functions,
        ram_end: 0x200,
        variables: Vec::new(),
        warnings: Vec::new(),
        uses_link: count > 0,
        uses_save: false,
        uses_sound: false,
        uses_print: false,
        source: text,
        assembly,
    })
}

/// The executable file for a compiled program.
pub fn executable(header: &Header, program: &Program) -> Result<Vec<u8>, String> {
    file::build(
        header,
        &program.functions,
        &program.assembly.bytes,
        program.entry,
    )
}
