//! Pocket Lua programs compiled and run on the emulated PocketStation.

use super::*;
use crate::card;
use crate::file::{ColorIcon, MenuIcon};
use crate::hw::button;
use crate::machine::{Machine, State};

fn header() -> Header {
    Header {
        title: "Pocket Lua test".into(),
        color_icon: ColorIcon::default(),
        viewer_icons: vec![],
        menu_icons: vec![MenuIcon {
            frames: vec![[0x0F; 32]],
            delay: 1,
        }],
        function_count: 0,
        snapshot: false,
    }
}

fn build(source: &str) -> (Program, Vec<u8>) {
    let mut h = header();
    let program = match compile(source, &mut h, &Options::default()) {
        Ok(p) => p,
        Err(errors) => panic!(
            "{}",
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ),
    };
    let bytes = executable(&h, &program).unwrap();
    (program, bytes)
}

fn run(source: &str, seconds: f64) -> (Program, Machine) {
    let (program, bytes) = build(source);
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000PLUA", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.launch(block, 0).unwrap();
    let stop = machine.run(seconds);
    assert!(
        !matches!(machine.state, State::Crashed(_)),
        "crashed: {:?} / {stop:?}\n{}",
        machine.state,
        program.source
    );
    (program, machine)
}

fn variable(program: &Program, machine: &Machine, name: &str) -> i32 {
    let (_, address, _) = program
        .variables
        .iter()
        .find(|(n, _, _)| n == name)
        .unwrap_or_else(|| panic!("no variable {name}"));
    machine.hw.ram_word(*address) as i32
}

fn errors(source: &str) -> Vec<String> {
    let mut h = header();
    match compile(source, &mut h, &Options::default()) {
        Ok(_) => panic!("expected errors"),
        Err(e) => e
            .into_iter()
            .map(|d| format!("{}: {}", d.line, d.message))
            .collect(),
    }
}

#[test]
fn draws_and_counts_frames() {
    let (program, machine) = run(
        r#"
local frames = 0
function update()
    frames = frames + 1
end
function draw()
    screen.clear()
    screen.box(0, 0, 4, 2)
    screen.pixel(31, 31)
end
"#,
        1.0,
    );
    let frames = variable(&program, &machine, "frames");
    assert!((28..=31).contains(&frames), "frames = {frames}");
    assert_eq!(machine.hw.vram[0], 0xF);
    assert_eq!(machine.hw.vram[1], 0xF);
    assert_eq!(machine.hw.vram[2], 0);
    assert_eq!(machine.hw.vram[31], 1 << 31);
}

#[test]
fn integer_semantics_match_lua() {
    let (program, machine) = run(
        r#"
local a = 0
local b = 0
local c = 0
local d = 0
local e = 0
local f = 0
local g = 0
local h = 0
local yes = false
function init()
    local seven = 7
    local minus = -7
    a = minus // 2
    b = minus % 3
    c = seven // -2
    d = seven % -3
    e = (seven << 4) | 3
    f = 0x80000000 >> 31
    g = seven * seven - 1
    h = minus // 4
    yes = seven > 3 and not (seven == 8) or false
end
"#,
        0.1,
    );
    assert_eq!(variable(&program, &machine, "a"), -4);
    assert_eq!(variable(&program, &machine, "b"), 2);
    assert_eq!(variable(&program, &machine, "c"), -4);
    assert_eq!(variable(&program, &machine, "d"), -2);
    assert_eq!(variable(&program, &machine, "e"), 115);
    assert_eq!(variable(&program, &machine, "f"), 1);
    assert_eq!(variable(&program, &machine, "g"), 48);
    assert_eq!(variable(&program, &machine, "h"), -2);
    assert_eq!(variable(&program, &machine, "yes"), 1);
}

#[test]
fn loops_functions_and_recursion() {
    let (program, machine) = run(
        r#"
local sum = 0
local down = 0
local fib10 = 0
local loops = 0
local data <const> = {5, 6, 7}
local cells = array(4)
local seeded = array{10, 20, 30}

function fib(n)
    if n < 2 then
        return n
    end
    return fib(n - 1) + fib(n - 2)
end

function init()
    for i = 1, 10 do
        sum = sum + i
    end
    for i = 10, 1, -3 do
        down = down + i
    end
    fib10 = fib(10)
    local n = 0
    while true do
        n = n + 1
        if n >= 5 then
            break
        end
    end
    repeat
        n = n + 1
    until n == 8
    loops = n
    for i = 1, #data do
        cells[i] = data[i] * 2
    end
    cells[4] = seeded[3] + #seeded
end
"#,
        0.1,
    );
    assert_eq!(variable(&program, &machine, "sum"), 55);
    assert_eq!(variable(&program, &machine, "down"), 10 + 7 + 4 + 1);
    assert_eq!(variable(&program, &machine, "fib10"), 55);
    assert_eq!(variable(&program, &machine, "loops"), 8);
    let cells = program.variables.iter().find(|v| v.0 == "cells").unwrap().1;
    let words: Vec<u32> = (0..5).map(|i| machine.hw.ram_word(cells + i * 4)).collect();
    assert_eq!(words, vec![4, 10, 12, 14, 33]);
}

#[test]
fn sprites_text_and_clipping() {
    let (_, machine) = run(
        r#"
local ship = sprite [[
    .##.
    ####
]]
function draw()
    screen.clear()
    screen.sprite(ship, -1, 0)
    screen.sprite(ship, 30, 30)
    screen.text("HI", 0, 10)
    screen.number(42, 20, 20)
end
"#,
        0.2,
    );
    let v = machine.hw.vram;
    // .##. shifted left by one: ##. ; #### shifted: ###
    assert_eq!(v[0], 0b011);
    assert_eq!(v[1], 0b111);
    assert_eq!(v[30], 0b10 << 30);
    assert_eq!(v[31], 0b11 << 30);
    // "H" rows: #.#, so bits 0 and 2 on row 10.
    assert_eq!(v[10] & 0x7, 0b101);
    assert_ne!(v[20] & (0xFF << 20), 0);
}

#[test]
fn buttons_report_presses_once() {
    let (program, bytes) = build(
        r#"
local presses = 0
local held = 0
function update()
    if button.pressed(button.FIRE) then
        presses = presses + 1
    end
    if button.down(button.LEFT) then
        held = held + 1
    end
end
"#,
    );
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000PLUA", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.launch(block, 0).unwrap();
    machine.run(0.1);
    machine.set_buttons(button::FIRE | button::LEFT);
    machine.run(0.5);
    machine.set_buttons(0);
    machine.run(0.2);
    machine.set_buttons(button::FIRE);
    machine.run(0.2);
    assert_eq!(variable(&program, &machine, "presses"), 2);
    let held = variable(&program, &machine, "held");
    assert!((13..=17).contains(&held), "held = {held}");
}

#[test]
fn runtime_errors_stop_with_the_line() {
    let (program, bytes) = build(
        r#"
local cells = array(3)
local i = 0
function update()
    i = i + 1
    cells[i] = 1
end
"#,
    );
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000PLUA", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.launch(block, 0).unwrap();
    machine.run(0.5);
    let halt = program.assembly.labels["rt_halt"];
    assert_eq!(machine.cpu.pc(), halt);
    assert_eq!(machine.hw.ram_word(0x230), 6, "line of the failing index");
    assert_eq!(machine.hw.ram_word(0x234), runtime::ERROR_INDEX);
    // The screen shows E and the line number.
    assert_eq!(machine.hw.vram[13] & 7, 7);
}

#[test]
fn division_by_zero_is_reported() {
    let (program, bytes) = build(
        r#"
local zero = 0
local x = 0
function update()
    x = 10 // zero
end
"#,
    );
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000PLUA", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.launch(block, 0).unwrap();
    machine.run(0.2);
    assert_eq!(machine.cpu.pc(), program.assembly.labels["rt_halt"]);
    assert_eq!(machine.hw.ram_word(0x234), runtime::ERROR_DIVISION);
    assert_eq!(machine.hw.ram_word(0x230), 5);
}

#[test]
fn save_data_survives_a_restart() {
    let source = r#"
local loaded = 0
function init()
    loaded = save.get(1)
    save.set(1, loaded + 5)
    save.write()
end
"#;
    let (program, bytes) = build(source);
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000PLUA", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.launch(block, 0).unwrap();
    machine.run(0.1);
    assert_eq!(variable(&program, &machine, "loaded"), 0);
    machine.launch(block, 0).unwrap();
    machine.run(0.1);
    assert_eq!(variable(&program, &machine, "loaded"), 5);
}

#[test]
fn the_playstation_exchanges_messages_and_values() {
    let (program, bytes) = build(
        r#"
local got = 0
function update()
    local n = link.received()
    if n > 0 then
        got = link.byte(1) + link.byte(n)
        link.clear()
        link.put(1, got)
        link.put(2, 99)
        link.send(2)
    end
    link.set_value(3, got * 2)
end
"#,
    );
    assert_eq!(program.functions.len(), 4);
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000PLUA", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.set_docked(true);
    machine.launch(block, 0).unwrap();
    machine.run(0.1);
    // FUNC 80h: send three bytes.
    let mut frame = vec![0x81, 0x5C, 0x80, 0, 3, 0, 10, 20, 32, 0];
    let reply = machine.link_transfer(&frame);
    assert_eq!(reply[3], 1, "one parameter: the length");
    assert_eq!(reply[5], 3);
    assert_eq!(*reply.last().unwrap(), 0xFF);
    machine.run(0.1);
    assert_eq!(variable(&program, &machine, "got"), 42);
    // FUNC 81h: read the answer.
    frame = vec![0x81, 0x5B, 0x81, 0, 0, 0, 0, 0];
    let reply = machine.link_transfer(&frame);
    assert_eq!(reply[4], 2);
    assert_eq!(&reply[5..7], &[42, 99]);
    // FUNC 82h: values; value 3 = 84.
    let mut frame = vec![0x81, 0x5B, 0x82, 0, 0];
    frame.extend([0u8; 65]);
    let reply = machine.link_transfer(&frame);
    assert_eq!(reply[4], 64);
    assert_eq!(&reply[5 + 8..5 + 12], &84u32.to_le_bytes());
}

#[test]
fn tunes_and_tones_drive_the_speaker() {
    let (_, mut machine) = run(
        r#"
local jingle = tune "C5:4 E5:4 R:2 G5:8"
function init()
    sound.play(jingle)
end
"#,
        0.3,
    );
    let mut since = 0;
    let samples = machine.take_audio(32_000, &mut since);
    let toggles = samples.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(toggles > 50, "toggles = {toggles}");
}

#[test]
fn print_goes_to_the_console() {
    let (_, machine) = run(
        r#"
function init()
    print("ready")
    print(-12)
    print(3 > 2)
end
"#,
        0.1,
    );
    assert_eq!(machine.tty, "ready\n-12\ntrue\n");
}

#[test]
fn helpful_errors() {
    let e = errors("local score = 0\nfunction update()\n    scroe = 1\nend\n");
    assert!(e[0].contains("did you mean `score`"), "{e:?}");
    let e = errors("function update()\n    local x = 7 / 2\nend\n");
    assert!(e[0].contains("`//`"), "{e:?}");
    let e = errors("local x = 1\nfunction update()\n    if x then end\nend\n");
    assert!(e[0].contains("true or false"), "{e:?}");
    let e = errors("local x = 1.5\n");
    assert!(e[0].contains("whole numbers"), "{e:?}");
    let e = errors("function update()\n    screen.pixle(1, 2)\nend\n");
    assert!(e[0].contains("did you mean `screen.pixel`"), "{e:?}");
    let e = errors("x = 1\n");
    assert!(e[0].contains("init()"), "{e:?}");
    let e = errors("local t = {1, 2}\nfunction update()\n    t[1] = 3\nend\n");
    assert!(e[0].contains("flash"), "{e:?}");
    let e = errors("function update()\n    screen.text(5, 1, 1)\nend\n");
    assert!(e[0].contains("must be a string"), "{e:?}");
}

#[test]
fn assembly_functions_are_callable() {
    let mut h = header();
    let options = Options {
        file: "main.plua".into(),
        assembly: vec![(
            "fast.s".into(),
            "    .global triple\n    .thumb_func\ntriple:\n    mov r1, r0\n    add r0, r0, r1\n    add r0, r0, r1\n    bx lr\n".into(),
        )],
        ..Default::default()
    };
    let program = compile(
        "local x = 0\nfunction init()\n    x = triple(14)\nend\n",
        &mut h,
        &options,
    )
    .unwrap();
    let bytes = executable(&h, &program).unwrap();
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000PLUA", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.launch(block, 0).unwrap();
    machine.run(0.1);
    assert_eq!(variable(&program, &machine, "x"), 42);
    // The debugger maps addresses back to Pocket Lua lines.
    let init = program.assembly.labels["fn_init"];
    assert_eq!(program.line_at(init, "main.plua"), Some(2));
    assert_eq!(program.line_at(init + 8, "main.plua"), Some(3));
}

#[test]
fn pure_assembly_programs_build() {
    let mut h = header();
    let source = "    .thumb\n    .thumb_func\nmain:\n    ldr r0, =0x0D000100\n    ldr r1, =0x12345678\n    str r1, [r0]\n1:  b 1b\n    .pool\n";
    let program = assemble_program(&[("main.s".into(), source.into())], &mut h).unwrap();
    let bytes = executable(&h, &program).unwrap();
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000ASM0", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.launch(block, 0).unwrap();
    machine.run(0.01);
    assert_eq!(machine.hw.vram[0], 0x1234_5678);
}

#[test]
fn clock_reads_the_calendar() {
    let (program, bytes) = build(
        r#"
local stamp = 0
function init()
    stamp = clock.year() * 10000 + clock.month() * 100 + clock.day()
end
"#,
    );
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000PLUA", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.set_date_time(crate::DateTime {
        year: 2026,
        month: 10,
        day: 16,
        hour: 9,
        minute: 30,
        second: 0,
        weekday: 6,
    });
    machine.launch(block, 0).unwrap();
    machine.run(0.1);
    assert_eq!(variable(&program, &machine, "stamp"), 20261016);
}

pub const STAR_CATCHER: &str = include_str!("samples/star_catcher.plua");

#[test]
fn the_sample_game_plays() {
    let (program, bytes) = build(STAR_CATCHER);
    assert!(
        program.stack_bytes() >= 320,
        "stack {}",
        program.stack_bytes()
    );
    assert!(bytes.len() <= 2 * card::BLOCK_SIZE, "{} bytes", bytes.len());
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000STAR", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.set_docked(true);
    machine.launch(block, 0).unwrap();
    machine.run(0.5);
    // Title screen, then Fire starts a round.
    machine.set_buttons(button::FIRE);
    machine.run(0.1);
    machine.set_buttons(0);
    for second in 0..20 {
        let buttons = if second % 2 == 0 {
            button::LEFT
        } else {
            button::RIGHT
        };
        machine.set_buttons(buttons);
        machine.run(1.0);
        assert!(
            !matches!(machine.state, State::Crashed(_)),
            "{:?}",
            machine.state
        );
        assert_ne!(
            machine.cpu.pc(),
            program.assembly.labels["rt_halt"],
            "runtime error"
        );
    }
    // The PlayStation reads the shared score and best values (FUNC 82h).
    let mut frame = vec![0x81, 0x5B, 0x82, 0, 0];
    frame.extend([0u8; 65]);
    let reply = machine.link_transfer(&frame);
    let score = u32::from_le_bytes(reply[5..9].try_into().unwrap());
    assert_eq!(score as i32, variable(&program, &machine, "score"));
    let drawn = machine.hw.vram.iter().filter(|r| **r != 0).count();
    assert!(drawn > 2);
}

#[test]
#[ignore]
fn show_sample_screens() {
    let (program, bytes) = build(STAR_CATCHER);
    let mut flash = card::formatted();
    let block = card::write(&mut flash, "BESLESP00000STAR", &bytes).unwrap();
    let mut machine = Machine::new(flash);
    machine.launch(block, 0).unwrap();
    let show = |m: &Machine| {
        for row in m.screen() {
            let line: String = (0..32)
                .map(|x| if row & (1 << x) != 0 { '#' } else { '.' })
                .collect();
            println!("{line}");
        }
        println!();
    };
    machine.run(0.5);
    show(&machine);
    machine.set_buttons(button::FIRE);
    machine.run(0.1);
    machine.set_buttons(0);
    machine.run(4.0);
    show(&machine);
    println!(
        "score {} lives {}",
        variable(&program, &machine, "score"),
        variable(&program, &machine, "lives")
    );
}
