//! Epok's PocketStation kernel ROM.
//!
//! The ROM is original code assembled at start-up by this crate's assembler.
//! It provides the exception handlers, the system call dispatcher and the
//! kernel RAM layout that PocketStation programs expect. Each system call
//! entry executes an emulator trap (`udf #0x100 + n`), so the work itself is
//! done by [`crate::kernel`] in Rust. Programs may still replace the system
//! call table, the RAM vectors and the callbacks exactly as on the console.

use crate::asm::{self, Assembly, Options};
use std::sync::OnceLock;

pub const BASE: u32 = 0x0400_0000;

pub const TRAP_RESET: u32 = 0x10;
pub const TRAP_UNDEFINED: u32 = 0x11;
pub const TRAP_PREFETCH_ABORT: u32 = 0x13;
pub const TRAP_DATA_ABORT: u32 = 0x14;
pub const TRAP_IRQ: u32 = 0x15;
pub const TRAP_FIQ: u32 = 0x16;
pub const TRAP_RETURN: u32 = 0x17;
pub const TRAP_IDLE: u32 = 0x18;
pub const TRAP_RESERVED: u32 = 0x19;
/// `udf #(TRAP_SWI_BASE + n)` performs system call `n`.
pub const TRAP_SWI_BASE: u32 = 0x100;
pub const SWI_COUNT: u32 = 0x20;

/// 8x8 glyphs for the kernel character set: digits, space, colon and status symbols.
pub const FONT: [&str; 20] = [
    // 0
    "..###...|.#...#..|.#..##..|.#.#.#..|.##..#..|.#...#..|..###...|........",
    // 1
    "...#....|..##....|...#....|...#....|...#....|...#....|..###...|........",
    // 2
    "..###...|.#...#..|.....#..|....#...|...#....|..#.....|.#####..|........",
    // 3
    "..###...|.#...#..|.....#..|...##...|.....#..|.#...#..|..###...|........",
    // 4
    "....#...|...##...|..#.#...|.#..#...|.#####..|....#...|....#...|........",
    // 5
    ".#####..|.#......|.####...|.....#..|.....#..|.#...#..|..###...|........",
    // 6
    "...##...|..#.....|.#......|.####...|.#...#..|.#...#..|..###...|........",
    // 7
    ".#####..|.....#..|....#...|...#....|...#....|...#....|...#....|........",
    // 8
    "..###...|.#...#..|.#...#..|..###...|.#...#..|.#...#..|..###...|........",
    // 9
    "..###...|.#...#..|.#...#..|..####..|.....#..|....#...|..##....|........",
    // space
    "........|........|........|........|........|........|........|........",
    // colon
    "........|...##...|...##...|........|...##...|...##...|........|........",
    // key lock
    "...##...|..#..#..|..#..#..|.######.|.##.###.|.######.|.######.|........",
    // speaker (medium)
    "....#...|...##...|.###.#..|.###.#..|.###.#..|...##...|....#...|........",
    // speaker waves (loud, follows the speaker)
    "..#.....|...#....|#...#...|.#..#...|#...#...|...#....|..#.....|........",
    // speaker off
    "....#...|...##..#|.###.##.|.###.#..|.###.##.|...##..#|....#...|........",
    // battery low
    "........|.######.|##....#.|##....#.|##....#.|.######.|........|........",
    // alarm off
    "...##...|..####..|..####..|..####..|.######.|........|...##...|........",
    // alarm on
    "#..##..#|..####..|#.####.#|..####..|.######.|........|...##...|........",
    // memory card
    ".#####..|.#.#.##.|.#.#.##.|.######.|.######.|.######.|.######.|........",
];

fn font_source() -> String {
    let mut out = String::from("    .align 2\nkernel_font:\n");
    for glyph in FONT {
        let rows: Vec<String> = glyph
            .split('|')
            .map(|row| {
                let mut v = 0u8;
                for (i, c) in row.chars().enumerate() {
                    if c == '#' {
                        v |= 1 << i;
                    }
                }
                format!("{v:#04x}")
            })
            .collect();
        out.push_str(&format!("    .byte {}\n", rows.join(", ")));
    }
    out
}

fn source() -> String {
    let mut table = String::new();
    for n in 0..SWI_COUNT {
        table.push_str(&format!("    .word swi_{n:02x}\n"));
    }
    let mut stubs = String::new();
    for n in 0..SWI_COUNT {
        if n == 2 {
            continue;
        }
        stubs.push_str(&format!(
            "swi_{n:02x}:\n    udf #{}\n    bx lr\n",
            TRAP_SWI_BASE + n
        ));
    }
    format!(
        r#"
    .arm
; Exception vectors (also visible at address 0 until RAM is mapped there).
    b reset_entry
    b undefined_entry
    b swi_entry
    b prefetch_abort_entry
    b data_abort_entry
    b reserved_entry
    b irq_entry
    b fiq_entry

reset_entry:
    udf #{TRAP_RESET}
    b reset_entry

reserved_entry:
    udf #{TRAP_RESERVED}
    b reserved_entry

undefined_entry:
    udf #{TRAP_UNDEFINED}
    movs pc, lr

prefetch_abort_entry:
    udf #{TRAP_PREFETCH_ABORT}
    subs pc, lr, #4

data_abort_entry:
    udf #{TRAP_DATA_ABORT}
    subs pc, lr, #8

; System calls: the number is the low byte of the SWI instruction. The
; dispatcher reads it with a halfword load and calls the entry in the table
; whose address is stored at [0E0h].
swi_entry:
    stmfd sp!, {{r1-r12, lr}}
    mrs r12, spsr
    tst r12, #0x20
    ldrneh r12, [lr, #-2]
    ldreqh r12, [lr, #-4]
    and r12, r12, #0xFF
    cmp r12, #{SWI_COUNT}
    movhs r12, #{SWI_COUNT_LAST}
    mov r11, #0xE0
    ldr r11, [r11]
    ldr r11, [r11, r12, lsl #2]
    mov lr, pc
    bx r11
    ldmfd sp!, {{r1-r12, pc}}^

; SWI 02h calls the program's own privileged callback (slot [0F8h]).
swi_02:
    mov r12, #0xF8
    ldr r12, [r12]
    cmp r12, #0
    bxne r12
    b swi_00

; IRQ: program callback first ([0FCh]), then the kernel's docking and clock work.
irq_entry:
    stmfd sp!, {{r0, r1, r12, lr}}
    mov r0, #0xFC
    ldr r0, [r0]
    cmp r0, #0
    movne lr, pc
    bxne r0
    udf #{TRAP_IRQ}
    ldmfd sp!, {{r0, r1, r12, lr}}
    subs pc, lr, #4

; FIQ: kernel work first, then the program callback ([100h]).
fiq_entry:
    stmfd sp!, {{r0, r1, r12, lr}}
    udf #{TRAP_FIQ}
    mov r0, #0x100
    ldr r0, [r0]
    cmp r0, #0
    movne lr, pc
    bxne r0
    ldmfd sp!, {{r0, r1, r12, lr}}
    subs pc, lr, #4

; Kernel calls into program code (link functions, notifications): r12 = target.
call_user:
    mov lr, pc
    bx r12
    udf #{TRAP_RETURN}
    b call_user

; The start-up application runs in the emulator; the CPU waits here.
idle:
    udf #{TRAP_IDLE}
    b idle

    .align 2
swi_table:
{table}
{stubs}
{font}
    .pool
"#,
        SWI_COUNT_LAST = SWI_COUNT - 1,
        font = font_source(),
    )
}

/// The assembled kernel ROM image (16 KiB) and its symbols.
pub fn image() -> &'static (Vec<u8>, Assembly) {
    static IMAGE: OnceLock<(Vec<u8>, Assembly)> = OnceLock::new();
    IMAGE.get_or_init(|| {
        let assembly = asm::assemble(
            &source(),
            &Options {
                base: BASE,
                file: "kernel.s".into(),
                ..Default::default()
            },
        )
        .unwrap_or_else(|errors| {
            panic!(
                "kernel ROM: {}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        });
        let mut rom = vec![0u8; crate::hw::ROM_SIZE];
        rom[..assembly.bytes.len()].copy_from_slice(&assembly.bytes);
        // Kernel identification at the end of the ROM.
        rom[0x3FF8..0x4000].copy_from_slice(b"EPOKPS01");
        (rom, assembly)
    })
}

pub fn symbol(name: &str) -> u32 {
    image().1.labels[name]
}
