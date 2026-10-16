use super::*;
use crate::cpu::{Bus, Cpu, Event};

/// Flat 64 KiB test memory with a trap region at 0xF000.
pub struct Flat {
    pub memory: Vec<u8>,
}

impl Flat {
    pub fn new() -> Self {
        Self {
            memory: vec![0; 0x10000],
        }
    }
}

impl Bus for Flat {
    fn read32(&mut self, a: u32) -> u32 {
        let a = (a & 0xFFFC) as usize;
        u32::from_le_bytes(self.memory[a..a + 4].try_into().unwrap())
    }
    fn read16(&mut self, a: u32) -> u16 {
        let a = (a & 0xFFFE) as usize;
        u16::from_le_bytes(self.memory[a..a + 2].try_into().unwrap())
    }
    fn read8(&mut self, a: u32) -> u8 {
        self.memory[(a & 0xFFFF) as usize]
    }
    fn write32(&mut self, a: u32, v: u32) {
        let a = (a & 0xFFFC) as usize;
        self.memory[a..a + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn write16(&mut self, a: u32, v: u16) {
        let a = (a & 0xFFFE) as usize;
        self.memory[a..a + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn write8(&mut self, a: u32, v: u8) {
        self.memory[(a & 0xFFFF) as usize] = v;
    }
    fn is_trap_region(&self, addr: u32) -> bool {
        addr >= 0xF000
    }
}

fn build(source: &str) -> Assembly {
    match assemble(source, &Options::default()) {
        Ok(a) => a,
        Err(errors) => panic!(
            "{}",
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    }
}

/// Runs `source` (ARM state at 0) until it executes `udf #1` placed at the end.
pub fn run(source: &str) -> (Cpu, Flat) {
    let assembly = build(&format!("{source}\n.arm\n.org 0xF000\nhalt: udf #1\n"));
    let mut bus = Flat::new();
    bus.memory[..assembly.bytes.len()].copy_from_slice(&assembly.bytes);
    let mut cpu = Cpu::new();
    cpu.set_mode(crate::cpu::MODE_SYS);
    cpu.r[13] = 0x8000;
    cpu.r[14] = 0xF000;
    for _ in 0..100_000 {
        let (_, event) = cpu.step(&mut bus);
        if event == Event::Trap(1) {
            return (cpu, bus);
        }
        if let Event::Undefined(op) = event {
            panic!("undefined instruction {op:#x} at {:#x}", cpu.current);
        }
    }
    panic!("program did not finish");
}

#[test]
fn arm_data_processing_and_flags() {
    let (cpu, _) = run("
        mov r0, #5
        add r1, r0, #3
        subs r2, r0, #5
        mov r3, #0
        moveq r3, #1
        mvn r4, #0
        mov r5, r0, lsl #4
        rsb r6, r0, #100
        orr r7, r0, #0xF0
        bic r8, r7, #0x80
        cmp r0, #10
        movlt r9, #7
        bx lr
    ");
    assert_eq!(cpu.r[1], 8);
    assert_eq!(cpu.r[2], 0);
    assert_eq!(cpu.r[3], 1);
    assert_eq!(cpu.r[4], 0xFFFF_FFFF);
    assert_eq!(cpu.r[5], 80);
    assert_eq!(cpu.r[6], 95);
    assert_eq!(cpu.r[7], 0xF5);
    assert_eq!(cpu.r[8], 0x75);
    assert_eq!(cpu.r[9], 7);
}

#[test]
fn arm_memory_literals_and_loops() {
    let (cpu, bus) = run("
        ldr r0, =0x12345678
        ldr r1, =buffer
        str r0, [r1]
        ldrb r2, [r1, #1]
        ldrh r3, [r1, #2]
        mov r4, #0
        mov r5, #10
    1:  add r4, r4, r5
        subs r5, r5, #1
        bne 1b
        stmfd sp!, {r0-r3}
        ldmfd sp!, {r6-r9}
        ldr r10, [r1], #4
        ldr r11, =-2
        bx lr
        .pool
    buffer: .word 0
    ");
    assert_eq!(cpu.r[0], 0x1234_5678);
    assert_eq!(cpu.r[2], 0x56);
    assert_eq!(cpu.r[3], 0x1234);
    assert_eq!(cpu.r[4], 55);
    assert_eq!(cpu.r[6], 0x1234_5678);
    assert_eq!(cpu.r[9], 0x1234);
    assert_eq!(cpu.r[10], 0x1234_5678);
    assert_eq!(cpu.r[1] % 4, 0);
    assert_eq!(cpu.r[11], -2i32 as u32);
    let at = (cpu.r[1] - 4) as usize;
    assert_eq!(&bus.memory[at..at + 4], &0x1234_5678u32.to_le_bytes());
}

#[test]
fn arm_multiply_and_long_multiply() {
    let (cpu, _) = run("
        ldr r0, =100000
        ldr r1, =300000
        mul r2, r0, r1
        umull r3, r4, r0, r1
        mvn r5, #0
        smull r6, r7, r5, r0
        mla r8, r0, r0, r1
        bx lr
        .pool
    ");
    let product = 100_000u64 * 300_000;
    assert_eq!(cpu.r[2], product as u32);
    assert_eq!(cpu.r[3], product as u32);
    assert_eq!(cpu.r[4], (product >> 32) as u32);
    assert_eq!(cpu.r[6], -100_000i32 as u32);
    assert_eq!(cpu.r[7], 0xFFFF_FFFF);
    assert_eq!(
        cpu.r[8],
        (100_000u32.wrapping_mul(100_000)).wrapping_add(300_000)
    );
}

#[test]
fn thumb_interworking_and_arithmetic() {
    let (cpu, _) = run("
        .arm
        adr r0, thumb_code + 1
        bx r0
        .thumb
        .thumb_func
    thumb_code:
        mov r1, #200
        add r1, #55
        lsl r2, r1, #4
        mov r3, #3
        mul r3, r2
        sub r4, r3, #7
        neg r5, r4
        cmp r5, #0
        blt 1f
        mov r6, #1
        b 2f
    1:  mov r6, #2
    2:  push {r1, r2}
        pop {r6, r7}
        mov r0, #0
        ldr r0, =0xF000
        bx r0
        .pool
    ");
    assert_eq!(cpu.r[1], 255);
    assert_eq!(cpu.r[2], 255 * 16);
    assert_eq!(cpu.r[3], 255 * 16 * 3);
    assert_eq!(cpu.r[4], 255 * 16 * 3 - 7);
    assert_eq!(cpu.r[5], (-(255 * 16 * 3 - 7)) as u32);
    assert_eq!(cpu.r[6], 255);
    assert_eq!(cpu.r[7], 255 * 16);
}

#[test]
fn thumb_calls_and_memory() {
    let (cpu, _) = run("
        .arm
        ldr r0, =start
        bx r0
        .pool
        .thumb
        .thumb_func
    start:
        mov r0, #6
        bl square
        mov r4, r0
        ldr r1, =table
        ldrb r2, [r1, #2]
        ldrh r3, [r1, #2]
        mov r5, #1
        ldsb r6, [r1, r5]
        ldr r0, =0xF000
        bx r0
    square:
        mul r0, r0
        bx lr
        .align 2
    table: .byte 1, 0xFE, 3, 4
        .pool
    ");
    assert_eq!(cpu.r[4], 36);
    assert_eq!(cpu.r[2], 3);
    assert_eq!(cpu.r[3], 0x0403);
    assert_eq!(cpu.r[6], -2i32 as u32);
}

#[test]
fn far_thumb_conditional_branches_are_relaxed() {
    let (cpu, _) = run("
        .arm
        ldr r0, =start
        bx r0
        .pool
        .thumb
        .thumb_func
    start:
        mov r0, #0
        cmp r0, #0
        beq far
        mov r1, #9
    far_back:
        ldr r0, =0xF000
        bx r0
        .pool
        .space 600
    far:
        mov r1, #4
        b far_back
    ");
    assert_eq!(cpu.r[1], 4);
}

#[test]
fn errors_name_the_line() {
    let errors = assemble("mov r0, #1\nfoo r1\n", &Options::default()).unwrap_err();
    assert_eq!(errors[0].line, 2);
    assert!(errors[0].message.contains("foo"));
    let errors = assemble(".thumb\nmov r0, #300\n", &Options::default()).unwrap_err();
    assert!(errors[0].message.contains("out of range"));
}

#[test]
fn undefined_symbols_are_reported() {
    let errors = assemble("b nowhere\n", &Options::default()).unwrap_err();
    assert!(errors[0].message.contains("undefined"), "{errors:?}");
}

#[test]
fn swi_and_banked_registers() {
    let source = "
        .arm
        b reset
        .org 8
        b swi_handler
    reset:
        mov r0, #1
        swi 0x42
        add r0, r0, #10
        ldr r1, =0xF000
        bx r1
    swi_handler:
        mov r2, lr
        mov r0, #5
        movs pc, lr
        .pool
    ";
    let assembly = build(&format!("{source}\n.org 0xF000\nudf #1\n"));
    let mut bus = Flat::new();
    bus.memory[..assembly.bytes.len()].copy_from_slice(&assembly.bytes);
    let mut cpu = Cpu::new();
    cpu.set_mode(crate::cpu::MODE_USR);
    cpu.r[15] = assembly.labels["reset"];
    let mut swi = None;
    for _ in 0..100 {
        let (_, event) = cpu.step(&mut bus);
        if let Event::Swi(n) = event {
            swi = Some(n);
        }
        if event == Event::Trap(1) {
            break;
        }
    }
    assert_eq!(swi, Some(0x42));
    assert_eq!(cpu.r[0], 15);
    assert_eq!(cpu.mode(), crate::cpu::MODE_USR);
}
