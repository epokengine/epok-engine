//! Whole-machine tests: kernel, programs, interrupts and the PlayStation link.

use crate::asm::{self, Options};
use crate::card;
use crate::file::{self, ColorIcon, Header, LinkFunction, MenuIcon};
use crate::hw::button;
use crate::machine::{Machine, State, Stop};

/// Assembles `source` (Thumb, entry label `main`, link functions named
/// `link_0`, `link_1`, ...) into an executable and installs it on a fresh card.
pub fn install(source: &str, functions: &[u32]) -> (Machine, u8) {
    let header = Header {
        title: "Test".into(),
        color_icon: ColorIcon::default(),
        viewer_icons: vec![],
        menu_icons: vec![MenuIcon {
            frames: vec![file::mono_from_text("#")],
            delay: 1,
        }],
        function_count: functions.len(),
        snapshot: false,
    };
    let assembly = asm::assemble(
        source,
        &Options {
            base: header.body_address(),
            thumb: true,
            file: "test.s".into(),
            ..Default::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "{}",
            e.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    });
    let table: Vec<LinkFunction> = functions
        .iter()
        .enumerate()
        .map(|(i, parameters)| LinkFunction {
            parameters: *parameters,
            address: assembly.symbol(&format!("link_{i}")).unwrap(),
        })
        .collect();
    let entry = assembly.symbol("main").unwrap();
    let bytes = file::build(&header, &table, &assembly.bytes, entry).unwrap();
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000TEST", &bytes).unwrap();
    (Machine::new(flash), block)
}

const DRAW_AND_COUNT: &str = r#"
    .thumb
    .thumb_func
main:
    mov r0, #1
    swi 0x11                ; enable PlayStation communication
    ldr r1, =0x0D000100
    ldr r2, =0xF0F0F0F0
    str r2, [r1]
    mov r0, #1
    ldr r1, =irq_handler
    swi 0x01                ; IRQ callback
    ldr r1, =0x0A800000
    ldr r2, =999
    str r2, [r1]
    str r2, [r1, #4]
    mov r2, #4
    str r2, [r1, #8]        ; timer 0: div 2, running
    ldr r1, =0x0A000008
    mov r2, #0x80
    str r2, [r1]            ; enable the timer 0 interrupt
    ; link function data: info at 210h = {buffer 220h, length 4}
    ldr r1, =0x210
    ldr r2, =0x220
    str r2, [r1]
    mov r2, #4
    str r2, [r1, #4]
    ldr r2, =0x44332211
    ldr r1, =0x220
    str r2, [r1]
loop:
    ldr r1, =0x200
    ldr r2, [r1]
    ldr r3, =0x0D000104
    str r2, [r3]
    b loop

    .thumb_func
irq_handler:
    ldr r0, =0x0A000010
    mov r1, #0x80
    str r1, [r0]
    ldr r0, =0x200
    ldr r1, [r0]
    add r1, #1
    str r1, [r0]
    bx lr

    .thumb_func
link_0:                     ; r0 = flags, r1 = parameter/data pointer
    mov r2, #0x10
    tst r0, r2
    bne 1f
    ldrb r2, [r1]           ; first parameter byte
    ldr r3, =0x230
    str r2, [r3]
    ldr r0, =0x210
    bx lr
1:  ldr r3, =0x234
    str r0, [r3]
    bx lr
    .pool
"#;

#[test]
fn kernel_rom_assembles_and_boots_into_the_launcher() {
    let machine = Machine::new(card::formatted());
    assert_eq!(machine.state, State::Launcher);
    assert_eq!(machine.hw.remap & 1, 1);
    // RAM vectors jump through the table into the ROM.
    assert_eq!(machine.hw.ram_word(0x08), 0xE59F_F018);
    assert_eq!(machine.hw.ram_word(0x28), crate::rom::symbol("swi_entry"));
}

#[test]
fn programs_run_draw_and_take_timer_interrupts() {
    let (mut machine, block) = install(DRAW_AND_COUNT, &[1]);
    machine.launch(block, 0).unwrap();
    assert_eq!(machine.state, State::Program(block));
    assert_eq!(machine.run(0.05), Stop::Elapsed);
    assert_eq!(machine.hw.vram[0], 0xF0F0_F0F0);
    let ticks = machine.hw.ram_word(0x200);
    // 4 MHz / 2 / 1000 = 2000 interrupts per second.
    assert!((90..=110).contains(&ticks), "ticks = {ticks}");
    assert!(machine.hw.vram[1] > 0);
    assert_eq!(machine.screen()[0], 0xF0F0_F0F0);
}

#[test]
fn the_playstation_link_answers_standard_and_pocketstation_commands() {
    let (mut machine, block) = install(DRAW_AND_COUNT, &[1]);
    // Undocked: the card does not answer at all.
    assert_eq!(machine.link_transfer(&[0x81, 0x58, 0, 0, 0]), vec![0xFF]);
    machine.set_docked(true);
    // Probe: length 2, then 01h 01h.
    assert_eq!(
        machine.link_transfer(&[0x81, 0x58, 0, 0, 0])[2..],
        [0x02, 0x01, 0x01]
    );
    // Identify.
    let id = machine.link_transfer(&[0x81, 0x53, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(&id[2..], &[0x5A, 0x5D, 0x5C, 0x5D, 0x04, 0x00, 0x00, 0x80]);
    // Read the directory frame of the installed program.
    let mut read = vec![0x81, 0x52, 0, 0, 0x00, block];
    read.resize(140, 0);
    let reply = machine.link_transfer(&read);
    assert_eq!(reply.len(), 140);
    assert_eq!(reply[10], card::STATE_FIRST);
    assert_eq!(reply[139], 0x47);
    // Write a sector with a correct checksum.
    let mut write = vec![0x81, 0x57, 0, 0, 0x03, 0xC0];
    let data: Vec<u8> = (0..128).map(|i| i as u8).collect();
    write.extend(&data);
    write.push(data.iter().fold(0x03 ^ 0xC0, |a, b| a ^ b));
    write.extend([0, 0, 0]);
    let reply = machine.link_transfer(&write);
    assert_eq!(*reply.last().unwrap(), 0x47);
    assert_eq!(
        &machine.hw.flash[0x3C0 * 0x80..0x3C0 * 0x80 + 128],
        &data[..]
    );
    // Status: dir index 0 (launcher) and the time.
    let status = machine.link_transfer(&[
        0x81, 0x5A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);
    assert_eq!(status[2], 0x12);
    assert_eq!(&status[3..5], &[0, 0]);
    // Ask the launcher to start the program (59h), as libraries do.
    machine.link_transfer(&[0x81, 0x59, 0, 0x00, block, 0x34, 0x12, 0, 0]);
    machine.run(0.1);
    assert_eq!(machine.state, State::Program(block));
    machine.run(0.01);
    // The program enabled communication; call its link function 80h with one parameter.
    let mut call = vec![0x81, 0x5B, 0x80, 0, 0x77, 0];
    call.extend([0u8; 5]);
    let reply = machine.link_transfer(&call);
    assert_eq!(reply[2], 0xFF);
    assert_eq!(reply[3], 1, "LEN1 comes from the function table");
    assert_eq!(reply[5], 4, "LEN2 comes from the program's answer");
    assert_eq!(&reply[6..10], &[0x11, 0x22, 0x33, 0x44]);
    assert_eq!(reply[10], 0xFF);
    assert_eq!(machine.hw.ram_word(0x230), 0x77);
    assert_eq!(
        machine.hw.ram_word(0x234),
        0x11,
        "post-data call to the PlayStation"
    );
    assert!(matches!(machine.state, State::Program(_)));
    // Memory read through FUNC 01h.
    let mut mem = vec![0x81, 0x5B, 0x01, 0, 0x20, 0x02, 0, 0, 4, 0];
    mem.extend([0u8; 5]);
    let reply = machine.link_transfer(&mem);
    assert_eq!(reply[9], 4);
    assert_eq!(&reply[10..14], &[0x11, 0x22, 0x33, 0x44]);
    // Writing the running program's directory frame is refused.
    let mut write = vec![0x81, 0x57, 0, 0, 0x00, block];
    let frame = vec![0u8; 128];
    write.extend(&frame);
    write.push(block);
    write.extend([0, 0, 0]);
    assert_eq!(*machine.link_transfer(&write).last().unwrap(), 0xFD);
}

#[test]
fn launcher_starts_the_selected_program_with_fire() {
    let (mut machine, block) = install(DRAW_AND_COUNT, &[1]);
    machine.run(0.1);
    assert_eq!(machine.launcher.programs.len(), 1);
    machine.set_buttons(button::RIGHT);
    machine.run(0.1);
    machine.set_buttons(0);
    machine.run(0.1);
    machine.set_buttons(button::FIRE);
    machine.run(0.1);
    assert_eq!(machine.state, State::Program(block));
}

#[test]
fn returning_to_the_launcher_and_crashes() {
    let source = r#"
    .thumb
    .thumb_func
main:
    mov r0, #1
    mov r1, #0
    mov r2, #0x30
    swi 0x08            ; PrepareExecute(1, 0, 30h)
    swi 0x09            ; DoExecute
    b main
    "#;
    let (mut machine, block) = install(source, &[]);
    machine.launch(block, 0).unwrap();
    machine.run(0.01);
    assert_eq!(machine.state, State::Launcher);

    let crash = r#"
    .thumb
    .thumb_func
main:
    ldr r0, =0x01000000
    ldr r1, [r0]
    b main
    .pool
    "#;
    let (mut machine, block) = install(crash, &[]);
    machine.launch(block, 0).unwrap();
    match machine.run(0.01) {
        Stop::Crashed(message) => assert!(message.contains("0x01000000"), "{message}"),
        other => panic!("expected a crash, got {other:?}"),
    }
}

#[test]
fn tty_output_and_date_time_calls() {
    let source = r#"
    .arm
main:
    mov r0, #0
    ldr r1, =0xE1B0F00E     ; movs pc, lr as the undefined-instruction handler
    str r1, [r0, #4]
    mov r0, #'H'
    .word 0xE6000010
    mov r0, #'i'
    .word 0xE6000010
    swi 0x0D
    ldr r1, =0x300
    str r0, [r1]
    swi 0x0E
    str r0, [r1, #4]
1:  b 1b
    .pool
    "#;
    let (mut machine, block) = install(
        &format!(".arm\n{}", source.replace(".arm\nmain:", "main:")),
        &[],
    );
    machine.set_date_time(crate::DateTime {
        year: 2026,
        month: 10,
        day: 16,
        hour: 12,
        minute: 34,
        second: 56,
        weekday: 6,
    });
    // ARM entry point (no Thumb bit): install() reads `main` as assembled in ARM.
    machine.launch(block, 0).unwrap();
    machine.run(0.01);
    assert_eq!(machine.tty, "Hi");
    assert_eq!(machine.hw.ram_word(0x300), 0x2026_1016);
    assert_eq!(machine.hw.ram_word(0x304) & 0x00FF_FF00, 0x0012_3400);
}
