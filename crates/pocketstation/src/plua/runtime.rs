//! The Pocket Lua runtime library: Thumb routines linked into every program
//! that uses them.
//!
//! Conventions: arguments in r0-r3, result in r0; routines may change r0-r3
//! and r12 and preserve r4-r7. Data in flash is only read with 16- or 32-bit
//! loads, because 8-bit reads from flash return garbage on the console.

/// A runtime routine with the routines it calls.
pub struct Routine {
    pub name: &'static str,
    pub needs: &'static [&'static str],
    pub code: &'static str,
}

/// Fixed RAM used by the runtime (the program's variables follow it).
pub const RAM_LAYOUT: &[(&str, u32)] = &[
    ("rt_frame_flag", 0x200),
    ("rt_frames", 0x204),
    ("rt_buttons", 0x208),
    ("rt_previous", 0x20C),
    ("rt_sound_frames", 0x210),
    ("rt_sound_note", 0x214),
    ("rt_sound_left", 0x218),
    ("rt_random_state", 0x21C),
    ("rt_comflags", 0x220),
    ("rt_dac_level", 0x224),
    ("rt_param", 0x228),
    ("rt_undefined_vector", 0x22C),
    ("rt_error_line", 0x230),
    ("rt_error_code", 0x234),
    ("rt_frame_reload", 0x238),
    ("rt_digits", 0x240),
    ("rt_screen", 0x270),
];
/// First byte after the fixed runtime RAM.
pub const RAM_END: u32 = 0x2F0;

/// Optional RAM blocks, allocated after the fixed area when used.
pub const LINK_RAM: u32 = 0x10 + 64 + 64 + 64;
pub const SAVE_RAM: u32 = 128;

pub const ERROR_INDEX: u32 = 1;
pub const ERROR_DIVISION: u32 = 2;
pub const ERROR_SAVE_SLOT: u32 = 3;
pub const ERROR_STACK: u32 = 4;

pub fn error_message(code: u32) -> &'static str {
    match code {
        ERROR_INDEX => "index out of range",
        ERROR_DIVISION => "division by zero",
        ERROR_SAVE_SLOT => "save slot out of range (1-32)",
        ERROR_STACK => "stack overflow (too many nested calls)",
        _ => "runtime error",
    }
}

pub const ROUTINES: &[Routine] = &[
    Routine {
        name: "rt_start",
        needs: &[
            "rt_irq",
            "rt_wait_frame",
            "rt_poll",
            "rt_present",
            "rt_set_fps",
        ],
        code: r#"
; Program entry (user mode, Thumb). r0 = start parameter.
    .thumb_func
rt_start:
    ldr r1, =rt_param
    str r0, [r1]
    swi 0x06
    ldr r1, =rt_comflags
    str r0, [r1]
    mov r0, #1
    ldr r1, =rt_irq
    swi 0x01
    mov r0, #30
    bl rt_set_fps
    ldr r1, =0x0A000008
    mov r2, #0x80
    str r2, [r1]
    mov r0, #1
    swi 0x11
    swi 0x0E
    ldr r1, =0x9E3779B9
    eor r0, r1
    ldr r1, =rt_random_state
    str r0, [r1]
    bl rt_setup
    bl rt_globals
    bl rt_user_init
rt_loop:
    bl rt_wait_frame
    bl rt_poll
    bl rt_user_update
    bl rt_user_draw
    bl rt_present
    bl rt_sound_frame
    ldr r0, =rt_frames
    ldr r1, [r0]
    add r1, #1
    str r1, [r0]
    b rt_loop
    .pool
"#,
    },
    Routine {
        name: "rt_set_fps",
        needs: &["rt_udiv"],
        code: r#"
; Frame timer: timer 0 at the CPU clock / 32 (124928 Hz at 4 MHz).
    .thumb_func
rt_set_fps:
    push {r4, lr}
    cmp r0, #1
    bge 1f
    mov r0, #1
1:  cmp r0, #60
    ble 2f
    mov r0, #60
2:  mov r1, r0
    ldr r0, =124928
    bl rt_udiv
    sub r0, #1
    ldr r4, =0x0A800000
    mov r1, #0
    str r1, [r4, #8]
    str r0, [r4]
    str r0, [r4, #4]
    mov r1, #5
    str r1, [r4, #8]
    pop {r4}
    pop {r3}
    bx r3
    .pool
"#,
    },
    Routine {
        name: "rt_irq",
        needs: &[],
        code: r#"
; IRQ callback (called by the kernel in IRQ mode; r0, r1, r12 are free).
    .thumb_func
rt_irq:
    push {r2, r3}
    ldr r3, =0x0A000000
    ldr r2, [r3]
    mov r1, #0x80
    tst r2, r1
    beq 1f
    str r1, [r3, #0x10]
    ldr r0, =rt_frame_flag
    str r1, [r0]
1:  mov r1, #1
    lsl r1, #8
    tst r2, r1
    beq 2f
    str r1, [r3, #0x10]
    ldr r0, =rt_dac_level
    ldr r2, [r0]
    mov r1, #1
    lsl r1, #15
    eor r2, r1
    str r2, [r0]
    ldr r0, =0x0D800014
    str r2, [r0]
2:  pop {r2, r3}
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_wait_frame",
        needs: &[],
        code: r#"
    .thumb_func
rt_wait_frame:
    ldr r0, =rt_frame_flag
1:  ldr r1, [r0]
    cmp r1, #0
    beq 1b
    mov r1, #0
    str r1, [r0]
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_poll",
        needs: &[],
        code: r#"
; Buttons, start requests from the PlayStation and the docked screen rotation.
    .thumb_func
rt_poll:
    ldr r0, =0x0A000004
    ldr r0, [r0]
    mov r1, #0x1F
    and r0, r1
    ldr r2, =rt_buttons
    ldr r1, [r2]
    str r1, [r2, #4]
    str r0, [r2]
    ldr r2, =rt_comflags
    ldr r2, [r2]
    ldr r0, [r2]
    mov r1, #1
    lsl r1, #11
    tst r0, r1
    beq 1f
    bic r0, r1
    str r0, [r2]
    mov r0, #0
    swi 0x11
    swi 0x09
    mov r0, #1
    swi 0x11
1:  ldr r0, =0x0A000004
    ldr r0, [r0]
    lsr r0, #11
    mov r1, #1
    and r0, r1
    lsl r0, #7
    ldr r2, =0x0D000000
    ldr r1, [r2]
    mov r3, #0x80
    bic r1, r3
    orr r1, r0
    str r1, [r2]
    swi 0x05
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_present",
        needs: &[],
        code: r#"
    .thumb_func
rt_present:
    push {r4-r6}
    ldr r0, =rt_screen
    ldr r1, =0x0D000100
    mov r2, #8
1:  ldmia r0!, {r3-r6}
    stmia r1!, {r3-r6}
    sub r2, #1
    bne 1b
    pop {r4-r6}
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_udiv",
        needs: &[],
        code: r#"
; Unsigned division: r0 / r1 -> r0 quotient, r1 remainder (r1 != 0).
    .thumb_func
rt_udiv:
    mov r2, r0
    mov r0, #0
    mov r3, #1
    cmp r1, #0
    beq 4f
1:  cmp r1, r2
    bhs 2f
    cmp r1, #0
    blt 2f
    lsl r1, #1
    lsl r3, #1
    b 1b
2:  cmp r2, r1
    blo 3f
    sub r2, r1
    orr r0, r3
3:  lsr r1, #1
    lsr r3, #1
    bne 2b
4:  mov r1, r2
    bx lr
"#,
    },
    Routine {
        name: "rt_idiv",
        needs: &["rt_udiv", "rt_error"],
        code: r#"
; Floor division as in Lua: r0 // r1 -> r0, r0 % r1 -> r1. r2 = line for errors.
    .thumb_func
rt_idiv:
    cmp r1, #0
    bne 1f
    mov r0, r2
    mov r1, #2
    bl rt_error
1:  push {r4, r5, lr}
    mov r4, r0
    mov r5, r1
    cmp r0, #0
    bge 2f
    neg r0, r0
2:  cmp r1, #0
    bge 3f
    neg r1, r1
3:  bl rt_udiv
    ; Signs: quotient negative when the operands differ in sign.
    mov r2, r4
    eor r2, r5
    bpl 4f
    neg r0, r0
4:  cmp r4, #0
    bge 5f
    neg r1, r1
5:  ; Floor adjustment: remainder nonzero with a sign different from the divisor.
    cmp r1, #0
    beq 6f
    mov r2, r1
    eor r2, r5
    bpl 6f
    sub r0, #1
    add r1, r5
6:  pop {r4, r5}
    pop {r3}
    bx r3
"#,
    },
    Routine {
        name: "rt_error",
        needs: &["rt_present", "rt_draw_digits"],
        code: r#"
; Runtime error: r0 = source line, r1 = error code. Shows "E" and the line, then stops.
    .thumb_func
rt_error:
    ldr r2, =rt_error_line
    str r0, [r2]
    str r1, [r2, #4]
    ldr r1, =rt_screen
    mov r2, #32
    mov r3, #0
1:  str r3, [r1]
    add r1, #4
    sub r2, #1
    bne 1b
    ldr r0, =rt_error_line
    ldr r0, [r0]
    mov r1, #6
    mov r2, #13
    bl rt_draw_digits
    ldr r1, =rt_screen + 13 * 4
    ldr r0, =0x7
    str r0, [r1]
    mov r0, #1
    str r0, [r1, #4]
    mov r0, #3
    str r0, [r1, #8]
    mov r0, #1
    str r0, [r1, #12]
    mov r0, #7
    str r0, [r1, #16]
    bl rt_present
    .thumb_func
rt_halt:
    b rt_halt
    .pool
"#,
    },
    Routine {
        name: "rt_draw_digits",
        needs: &["rt_udiv", "rt_draw_glyph"],
        code: r#"
; Draws the decimal number r0 at (r1, r2) with the 3x5 font.
    .thumb_func
rt_draw_digits:
    push {r4-r7, lr}
    mov r6, r1
    mov r7, r2
    cmp r0, #0
    bge 1f
    neg r0, r0
    push {r0}
    mov r0, #13
    mov r1, r6
    mov r2, r7
    bl rt_draw_glyph
    add r6, #4
    pop {r0}
1:  ldr r4, =rt_digits
    mov r5, #0
2:  mov r1, #10
    bl rt_udiv
    str r1, [r4]
    add r4, #4
    add r5, #1
    cmp r0, #0
    bne 2b
3:  sub r4, #4
    ldr r0, [r4]
    add r0, #16
    mov r1, r6
    mov r2, r7
    bl rt_draw_glyph
    add r6, #4
    sub r5, #1
    bne 3b
    mov r0, r6
    pop {r4-r7}
    pop {r3}
    bx r3
    .pool
"#,
    },
    Routine {
        name: "rt_draw_glyph",
        needs: &["rt_font"],
        code: r#"
; Draws glyph r0 (0 = space .. 63 = '_') at (r1, r2).
    .thumb_func
rt_draw_glyph:
    push {r4-r7}
    ldr r3, =rt_font
    lsl r0, #1
    ldrh r4, [r3, r0]
    ldr r5, =rt_screen
    mov r6, #5
1:  cmp r2, #31
    bhi 3f
    mov r7, #7
    and r7, r4
    cmp r1, #0
    blt 2f
    lsl r7, r1
    b 4f
2:  neg r3, r1
    lsr r7, r3
4:  lsl r3, r2, #2
    ldr r0, [r5, r3]
    orr r0, r7
    str r0, [r5, r3]
3:  lsr r4, #3
    add r2, #1
    sub r6, #1
    bne 1b
    pop {r4-r7}
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_font",
        needs: &[],
        code: "",
    },
    Routine {
        name: "rt_screen_clear",
        needs: &[],
        code: r#"
    .thumb_func
rt_screen_clear:
    mov r1, #0
    b rt_screen_fill_with
    .thumb_func
rt_screen_fill:
    mov r1, #0
    mvn r1, r1
    .thumb_func
rt_screen_fill_with:
    ldr r0, =rt_screen
    mov r2, #32
1:  str r1, [r0]
    add r0, #4
    sub r2, #1
    bne 1b
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_screen_invert",
        needs: &[],
        code: r#"
    .thumb_func
rt_screen_invert:
    ldr r0, =rt_screen
    mov r2, #32
1:  ldr r1, [r0]
    mvn r1, r1
    str r1, [r0]
    add r0, #4
    sub r2, #1
    bne 1b
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_screen_pixel",
        needs: &[],
        code: r#"
; pixel(x, y, on)
    .thumb_func
rt_screen_pixel:
    cmp r0, #31
    bhi 2f
    cmp r1, #31
    bhi 2f
    push {r4}
    ldr r3, =rt_screen
    lsl r1, #2
    add r3, r1
    ldr r1, [r3]
    mov r4, #1
    lsl r4, r0
    cmp r2, #0
    beq 1f
    orr r1, r4
    b 3f
1:  bic r1, r4
3:  str r1, [r3]
    pop {r4}
2:  bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_screen_get",
        needs: &[],
        code: r#"
; get(x, y) -> bool
    .thumb_func
rt_screen_get:
    cmp r0, #31
    bhi 1f
    cmp r1, #31
    bhi 1f
    ldr r3, =rt_screen
    lsl r1, #2
    ldr r1, [r3, r1]
    lsr r1, r0
    mov r0, #1
    and r0, r1
    bx lr
1:  mov r0, #0
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_screen_box",
        needs: &["rt_row_mask"],
        code: r#"
; box(x, y, w, h): filled rectangle.
    .thumb_func
rt_screen_box:
    push {r4-r7, lr}
    mov r4, r1
    mov r5, r3
    mov r1, r2
    bl rt_row_mask
    mov r6, r0
    ldr r7, =rt_screen
1:  cmp r5, #0
    ble 3f
    cmp r4, #31
    bhi 2f
    lsl r0, r4, #2
    ldr r1, [r7, r0]
    orr r1, r6
    str r1, [r7, r0]
2:  add r4, #1
    sub r5, #1
    b 1b
3:  pop {r4-r7}
    pop {r3}
    bx r3
    .pool
"#,
    },
    Routine {
        name: "rt_row_mask",
        needs: &[],
        code: r#"
; Bits x .. x+w-1 clipped to 0..31: r0 = x, r1 = w -> r0 = mask.
    .thumb_func
rt_row_mask:
    add r1, r0
    cmp r0, #0
    bge 1f
    mov r0, #0
1:  cmp r1, #32
    ble 2f
    mov r1, #32
2:  cmp r1, r0
    ble 4f
    sub r1, r0
    cmp r1, #32
    bne 3f
    mov r1, #0
    mvn r1, r1
    lsl r1, r0
    mov r0, r1
    bx lr
3:  mov r2, #1
    lsl r2, r1
    sub r2, #1
    lsl r2, r0
    mov r0, r2
    bx lr
4:  mov r0, #0
    bx lr
"#,
    },
    Routine {
        name: "rt_screen_rect",
        needs: &["rt_screen_box"],
        code: r#"
; rect(x, y, w, h): outline.
    .thumb_func
rt_screen_rect:
    push {r4-r7, lr}
    mov r4, r0
    mov r5, r1
    mov r6, r2
    mov r7, r3
    mov r3, #1
    bl rt_screen_box
    mov r0, r4
    add r1, r5, r7
    sub r1, #1
    mov r2, r6
    mov r3, #1
    bl rt_screen_box
    mov r0, r4
    mov r1, r5
    mov r2, #1
    mov r3, r7
    bl rt_screen_box
    add r0, r4, r6
    sub r0, #1
    mov r1, r5
    mov r2, #1
    mov r3, r7
    bl rt_screen_box
    pop {r4-r7}
    pop {r3}
    bx r3
"#,
    },
    Routine {
        name: "rt_screen_line",
        needs: &["rt_screen_pixel"],
        code: r#"
; line(x0, y0, x1, y1): Bresenham.
    .thumb_func
rt_screen_line:
    push {r4-r7, lr}
    sub sp, #16
    str r2, [sp, #0]
    str r3, [sp, #4]
    mov r4, r0
    mov r5, r1
    ; dx = |x1 - x0|, sx
    sub r6, r2, r0
    mov r2, #1
    cmp r6, #0
    bge 1f
    neg r6, r6
    neg r2, r2
1:  str r2, [sp, #8]
    ; dy = -|y1 - y0|, sy
    sub r7, r3, r1
    mov r2, #1
    cmp r7, #0
    bge 2f
    neg r7, r7
    neg r2, r2
2:  str r2, [sp, #12]
    neg r7, r7
    ; err = dx + dy (kept in r12)
    add r2, r6, r7
    mov r12, r2
3:  mov r0, r4
    mov r1, r5
    mov r2, #1
    bl rt_screen_pixel
    ldr r0, [sp, #0]
    cmp r4, r0
    bne 4f
    ldr r0, [sp, #4]
    cmp r5, r0
    beq 7f
4:  mov r2, r12
    lsl r3, r2, #1
    cmp r3, r7
    blt 5f
    add r2, r7
    ldr r0, [sp, #8]
    add r4, r0
5:  cmp r3, r6
    bgt 6f
    add r2, r6
    ldr r0, [sp, #12]
    add r5, r0
6:  mov r12, r2
    b 3b
7:  add sp, #16
    pop {r4-r7}
    pop {r3}
    bx r3
"#,
    },
    Routine {
        name: "rt_screen_sprite",
        needs: &["rt_blit"],
        code: r#"
; sprite(s, x, y, frame): draws black pixels of the frame (1 = first).
    .thumb_func
rt_screen_sprite:
    push {r4, lr}
    mov r4, #0
    bl rt_blit
    pop {r4}
    pop {r3}
    bx r3
; erase(s, x, y, frame): clears the sprite's pixels.
    .thumb_func
rt_screen_erase:
    push {r4, lr}
    mov r4, #1
    bl rt_blit
    pop {r4}
    pop {r3}
    bx r3
"#,
    },
    Routine {
        name: "rt_blit",
        needs: &["rt_udiv"],
        code: r#"
; r0 = sprite, r1 = x, r2 = y, r3 = frame, r4 = 0 draw / 1 erase.
    .thumb_func
rt_blit:
    push {r4-r7, lr}
    sub sp, #12
    str r1, [sp, #0]
    str r2, [sp, #4]
    str r4, [sp, #8]
    ldr r4, [r0]
    add r0, #4
    mov r7, r0
    lsr r5, r4, #8
    mov r6, #0xFF
    and r5, r6
    lsr r6, r4, #16
    ; frame index = (frame - 1) mod frames
    sub r0, r3, #1
    cmp r0, #0
    bge 1f
    mov r0, #0
1:  mov r1, r6
    push {r5}
    bl rt_udiv
    pop {r5}
    mul r1, r5
    lsl r1, #2
    add r7, r1
    ldr r6, =rt_screen
    ldr r2, [sp, #4]
2:  cmp r5, #0
    beq 6f
    cmp r2, #31
    bhi 5f
    ldr r3, [r7]
    ldr r1, [sp, #0]
    cmp r1, #0
    blt 3f
    lsl r3, r1
    b 4f
3:  neg r1, r1
    lsr r3, r1
4:  lsl r1, r2, #2
    ldr r0, [r6, r1]
    ldr r4, [sp, #8]
    cmp r4, #0
    bne 7f
    orr r0, r3
    b 8f
7:  bic r0, r3
8:  str r0, [r6, r1]
5:  add r7, #4
    add r2, #1
    sub r5, #1
    b 2b
6:  add sp, #12
    pop {r4-r7}
    pop {r3}
    bx r3
    .pool
"#,
    },
    Routine {
        name: "rt_screen_text",
        needs: &["rt_draw_glyph"],
        code: r#"
; text(string, x, y): strings are halfword counts followed by halfword characters.
    .thumb_func
rt_screen_text:
    push {r4-r7, lr}
    mov r4, r0
    mov r6, r1
    mov r7, r2
    ldrh r5, [r4]
    add r4, #2
1:  cmp r5, #0
    beq 3f
    ldrh r0, [r4]
    add r4, #2
    cmp r0, #96
    blo 2f
    sub r0, #32
2:  sub r0, #32
    cmp r0, #63
    bls 4f
    mov r0, #31
4:  mov r1, r6
    mov r2, r7
    bl rt_draw_glyph
    add r6, #4
    sub r5, #1
    b 1b
3:  mov r0, r6
    pop {r4-r7}
    pop {r3}
    bx r3
"#,
    },
    Routine {
        name: "rt_screen_number",
        needs: &["rt_draw_digits"],
        code: r#"
    .thumb_func
rt_screen_number:
    push {lr}
    bl rt_draw_digits
    pop {r3}
    bx r3
"#,
    },
    Routine {
        name: "rt_button_down",
        needs: &[],
        code: r#"
    .thumb_func
rt_button_down:
    ldr r1, =rt_buttons
    ldr r1, [r1]
    and r1, r0
    mov r0, #0
    cmp r1, #0
    beq 1f
    mov r0, #1
1:  bx lr
    .thumb_func
rt_button_pressed:
    ldr r2, =rt_buttons
    ldr r1, [r2]
    ldr r2, [r2, #4]
    bic r1, r2
    and r1, r0
    mov r0, #0
    cmp r1, #0
    beq 1f
    mov r0, #1
1:  bx lr
    .thumb_func
rt_button_released:
    ldr r2, =rt_buttons
    ldr r1, [r2]
    ldr r2, [r2, #4]
    bic r2, r1
    and r2, r0
    mov r0, #0
    cmp r2, #0
    beq 1f
    mov r0, #1
1:  bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_sound",
        needs: &["rt_udiv"],
        code: r#"
; tone(frequency, frames)
    .thumb_func
rt_sound_tone:
    push {r4, lr}
    mov r4, r1
    mov r1, r0
    cmp r1, #20
    bge 1f
    mov r1, #20
1:  ldr r0, =999424
    bl rt_udiv
    sub r0, #1
    mov r1, r4
    bl rt_sound_start
    ldr r0, =rt_sound_left
    mov r1, #0
    str r1, [r0]
    pop {r4}
    pop {r3}
    bx r3
; Starts a square wave: r0 = timer 1 reload (0 = silence), r1 = frames.
    .thumb_func
rt_sound_start:
    ldr r2, =rt_sound_frames
    str r1, [r2]
    ldr r2, =0x0A800010
    mov r3, #0
    str r3, [r2, #8]
    cmp r0, #0
    beq 1f
    str r0, [r2]
    str r0, [r2, #4]
    mov r3, #4
    str r3, [r2, #8]
    ldr r2, =0x0A000008
    mov r3, #1
    lsl r3, #8
    str r3, [r2]
    ldr r2, =0x0D800010
    mov r3, #1
    str r3, [r2]
    ldr r2, =0x0D800008
    mov r3, #0x20
    str r3, [r2]
    ldr r2, =rt_dac_level
    mov r3, #1
    lsl r3, #14
    str r3, [r2]
    bx lr
1:  ldr r2, =0x0D800014
    str r3, [r2]
    bx lr
; play(tune): tunes are a count followed by (reload, frames) pairs.
    .thumb_func
rt_sound_play:
    ldr r1, [r0]
    add r0, #4
    ldr r2, =rt_sound_note
    str r0, [r2]
    ldr r2, =rt_sound_left
    str r1, [r2]
    ldr r2, =rt_sound_frames
    mov r1, #0
    str r1, [r2]
    bx lr
    .thumb_func
rt_sound_stop:
    ldr r0, =rt_sound_left
    mov r1, #0
    str r1, [r0]
    ldr r0, =rt_sound_frames
    str r1, [r0]
    .thumb_func
rt_sound_silence:
    ldr r2, =0x0A800010
    mov r3, #0
    str r3, [r2, #8]
    ldr r2, =0x0A00000C
    mov r3, #1
    lsl r3, #8
    str r3, [r2]
    ldr r2, =0x0D800010
    mov r3, #0
    str r3, [r2]
    ldr r2, =0x0D800004
    mov r3, #0x20
    str r3, [r2]
    bx lr
    .thumb_func
rt_sound_playing:
    ldr r0, =rt_sound_frames
    ldr r1, [r0]
    ldr r0, =rt_sound_left
    ldr r0, [r0]
    orr r0, r1
    beq 1f
    mov r0, #1
1:  bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_sound_frame",
        needs: &["rt_sound"],
        code: r#"
; Once per frame: count down the current note and start the next tune note.
    .thumb_func
rt_sound_frame:
    ldr r0, =rt_sound_frames
    ldr r1, [r0]
    cmp r1, #0
    beq 1f
    sub r1, #1
    str r1, [r0]
    bne 3f
1:  ldr r0, =rt_sound_left
    ldr r1, [r0]
    cmp r1, #0
    beq 2f
    sub r1, #1
    str r1, [r0]
    ldr r2, =rt_sound_note
    ldr r3, [r2]
    ldr r0, [r3]
    ldr r1, [r3, #4]
    add r3, #8
    str r3, [r2]
    b rt_sound_start_far
2:  ldr r0, =rt_sound_frames
    ldr r1, [r0]
    cmp r1, #0
    bne 3f
    b rt_sound_silence_far
3:  bx lr
    .pool
rt_sound_start_far:
    ldr r2, =rt_sound_start
    bx r2
rt_sound_silence_far:
    ldr r2, =rt_sound_silence
    bx r2
    .pool
"#,
    },
    Routine {
        name: "rt_sound_frame_silent",
        needs: &[],
        code: r#"
    .thumb_func
rt_sound_frame:
    bx lr
"#,
    },
    Routine {
        name: "rt_clock",
        needs: &[],
        code: r#"
; Converts the BCD byte at bit r1 of r0 to binary.
    .thumb_func
rt_bcd_field:
    lsr r0, r1
    mov r1, #0xFF
    and r0, r1
    lsr r1, r0, #4
    mov r2, #0xF
    and r0, r2
    lsl r2, r1, #3
    add r0, r2
    lsl r1, #1
    add r0, r1
    bx lr
    .thumb_func
rt_clock_second:
    mov r1, #0
    b rt_clock_time
    .thumb_func
rt_clock_minute:
    mov r1, #8
    b rt_clock_time
    .thumb_func
rt_clock_hour:
    mov r1, #16
    .thumb_func
rt_clock_time:
    swi 0x0E
    b rt_bcd_field
    .thumb_func
rt_clock_weekday:
    swi 0x0E
    lsr r0, #24
    bx lr
    .thumb_func
rt_clock_day:
    swi 0x0D
    mov r1, #0
    b rt_bcd_field
    .thumb_func
rt_clock_month:
    swi 0x0D
    mov r1, #8
    b rt_bcd_field
    .thumb_func
rt_clock_year:
    push {r4, lr}
    swi 0x0D
    mov r4, r0
    mov r1, #24
    bl rt_bcd_field
    mov r1, #100
    mul r0, r1
    mov r2, r0
    mov r0, r4
    push {r2}
    mov r1, #16
    bl rt_bcd_field
    pop {r2}
    add r0, r2
    pop {r4}
    pop {r3}
    bx r3
"#,
    },
    Routine {
        name: "rt_random",
        needs: &["rt_udiv"],
        code: r#"
; random(n) -> 1..n (xorshift32).
    .thumb_func
rt_random:
    push {lr}
    mov r3, r0
    ldr r2, =rt_random_state
    ldr r0, [r2]
    lsl r1, r0, #13
    eor r0, r1
    lsr r1, r0, #17
    eor r0, r1
    lsl r1, r0, #5
    eor r0, r1
    str r0, [r2]
    cmp r3, #1
    ble 1f
    mov r1, r3
    bl rt_udiv
    add r0, r1, #1
    pop {r3}
    bx r3
1:  mov r0, #1
    pop {r3}
    bx r3
    .pool
"#,
    },
    Routine {
        name: "rt_math",
        needs: &[],
        code: r#"
    .thumb_func
rt_math_abs:
    cmp r0, #0
    bge 1f
    neg r0, r0
1:  bx lr
    .thumb_func
rt_math_min:
    cmp r0, r1
    ble 1f
    mov r0, r1
1:  bx lr
    .thumb_func
rt_math_max:
    cmp r0, r1
    bge 1f
    mov r0, r1
1:  bx lr
    .thumb_func
rt_math_clamp:
    cmp r0, r1
    bge 1f
    mov r0, r1
1:  cmp r0, r2
    ble 2f
    mov r0, r2
2:  bx lr
"#,
    },
    Routine {
        name: "rt_system",
        needs: &[],
        code: r#"
    .thumb_func
rt_system_exit:
    ldr r0, =rt_undefined_vector
    ldr r1, [r0]
    cmp r1, #0
    beq 1f
    mov r0, #4
    str r1, [r0]
1:  swi 0x16
    mov r2, r0
    add r2, #0x30
    mov r0, #1
    mov r1, #0
    swi 0x08
    mov r0, #0
    swi 0x11
    swi 0x09
    bx lr
    .thumb_func
rt_system_frame:
    ldr r0, =rt_frames
    ldr r0, [r0]
    bx lr
    .thumb_func
rt_system_docked:
    ldr r0, =0x0A000004
    ldr r0, [r0]
    lsr r0, #11
    mov r1, #1
    and r0, r1
    bx lr
    .thumb_func
rt_system_speed:
    cmp r0, #1
    bge 1f
    mov r0, #1
1:  cmp r0, #8
    ble 2f
    mov r0, #8
2:  swi 0x04
    bx lr
    .thumb_func
rt_system_param:
    ldr r0, =rt_param
    ldr r0, [r0]
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_tty",
        needs: &["rt_udiv"],
        code: r#"
; print support: characters go out through the TTY convention (E6000010h).
    .thumb_func
rt_putc:
    ldr r3, =rt_tty_arm
    bx r3
    .thumb_func
rt_print_string:
    push {r4, r5, lr}
    mov r4, r0
    ldrh r5, [r4]
    add r4, #2
1:  cmp r5, #0
    beq 2f
    ldrh r0, [r4]
    add r4, #2
    bl rt_putc
    sub r5, #1
    b 1b
2:  mov r0, #10
    bl rt_putc
    pop {r4, r5}
    pop {r3}
    bx r3
    .thumb_func
rt_print_int:
    push {r4, r5, lr}
    cmp r0, #0
    bge 1f
    neg r0, r0
    push {r0}
    mov r0, #'-'
    bl rt_putc
    pop {r0}
1:  ldr r4, =rt_digits
    mov r5, #0
2:  mov r1, #10
    bl rt_udiv
    str r1, [r4]
    add r4, #4
    add r5, #1
    cmp r0, #0
    bne 2b
3:  sub r4, #4
    ldr r0, [r4]
    add r0, #'0'
    bl rt_putc
    sub r5, #1
    bne 3b
    mov r0, #10
    bl rt_putc
    pop {r4, r5}
    pop {r3}
    bx r3
    .pool
    .arm
    .align 2
rt_tty_arm:
    .word 0xE6000010
    bx lr
    .thumb
"#,
    },
    Routine {
        name: "rt_setup_tty",
        needs: &[],
        code: r#"
; Undefined-instruction handler that ignores the TTY instruction on the console.
    .thumb_func
rt_setup_tty:
    mov r0, #4
    ldr r1, [r0]
    ldr r2, =rt_undefined_vector
    str r1, [r2]
    ldr r1, =0xE1B0F00E
    str r1, [r0]
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_save",
        needs: &["rt_error"],
        code: r#"
; Save data: 32 words in RAM, mirrored to the program's last flash sector.
    .thumb_func
rt_save_load:
    ldr r0, =rt_save_sector_data
    ldr r1, =rt_save_buffer
    mov r2, #32
1:  ldr r3, [r0]
    str r3, [r1]
    add r0, #4
    add r1, #4
    sub r2, #1
    bne 1b
    bx lr
; get(slot) / set(slot, value): r2 = line for errors.
    .thumb_func
rt_save_get:
    sub r0, #1
    cmp r0, #31
    bhi 1f
    lsl r0, #2
    ldr r1, =rt_save_buffer
    ldr r0, [r1, r0]
    bx lr
    .thumb_func
rt_save_set:
    sub r0, #1
    cmp r0, #31
    bhi 1f
    lsl r0, #2
    ldr r3, =rt_save_buffer
    str r1, [r3, r0]
    bx lr
1:  mov r0, r2
    mov r1, #3
    bl rt_error
    .thumb_func
rt_save_write:
    ldr r0, =(rt_save_sector_data - 0x02000000) / 128
    ldr r1, =rt_save_buffer
    swi 0x03
    cmp r0, #0
    beq 2f
    mov r0, #0
    bx lr
2:  mov r0, #1
    bx lr
    .pool
"#,
    },
    Routine {
        name: "rt_link",
        needs: &[],
        code: r#"
; Link functions called by the kernel for the PlayStation (FIQ context).
; FUNC 80h: message from the PlayStation (1 parameter byte: length).
    .thumb_func
rt_link_receive:
    mov r2, #0x10
    tst r0, r2
    bne 2f
    ldrb r2, [r1]
    cmp r2, #64
    bls 1f
    mov r2, #64
1:  ldr r0, =rt_link_info
    ldr r3, =rt_inbox
    str r3, [r0]
    str r2, [r0, #4]
    bx lr
2:  mov r2, #4
    tst r0, r2
    bne 3f
    ldr r0, =rt_link_info
    ldr r2, [r0, #4]
    ldr r3, =rt_inbox_length
    str r2, [r3]
3:  bx lr
; FUNC 81h: message to the PlayStation.
    .thumb_func
rt_link_send:
    mov r2, #0x10
    tst r0, r2
    bne 1f
    ldr r0, =rt_link_info
    ldr r3, =rt_outbox
    str r3, [r0]
    ldr r3, =rt_outbox_length
    ldr r3, [r3]
    str r3, [r0, #4]
    bx lr
1:  mov r2, #4
    tst r0, r2
    bne 2f
    ldr r3, =rt_outbox_length
    mov r2, #0
    str r2, [r3]
2:  bx lr
; FUNC 82h: read the 16 shared values.
    .thumb_func
rt_link_values:
    ldr r0, =rt_link_info
    ldr r3, =rt_values
    str r3, [r0]
    mov r3, #64
    str r3, [r0, #4]
    bx lr
; FUNC 83h: write one shared value (parameter: index 1-16).
    .thumb_func
rt_link_value:
    mov r2, #0x10
    tst r0, r2
    bne 1f
    ldrb r2, [r1]
    sub r2, #1
    cmp r2, #15
    bhi 2f
    lsl r2, #2
    ldr r3, =rt_values
    add r3, r2
    ldr r0, =rt_link_info
    str r3, [r0]
    mov r3, #4
    str r3, [r0, #4]
    bx lr
2:  mov r0, #0
1:  bx lr
; Pocket Lua side.
    .thumb_func
rt_link_connected:
    ldr r0, =rt_comflags
    ldr r0, [r0]
    ldr r0, [r0]
    lsr r0, #9
    mov r1, #1
    and r0, r1
    bx lr
    .thumb_func
rt_link_received:
    ldr r0, =rt_inbox_length
    ldr r0, [r0]
    bx lr
    .thumb_func
rt_link_byte:
    sub r0, #1
    cmp r0, #63
    bhi 1f
    ldr r1, =rt_inbox
    ldrb r0, [r1, r0]
    bx lr
1:  mov r0, #0
    bx lr
    .thumb_func
rt_link_clear:
    ldr r0, =rt_inbox_length
    mov r1, #0
    str r1, [r0]
    bx lr
    .thumb_func
rt_link_put:
    sub r0, #1
    cmp r0, #63
    bhi 1f
    ldr r2, =rt_outbox
    strb r1, [r2, r0]
1:  bx lr
    .thumb_func
rt_link_publish:
    cmp r0, #64
    bls 1f
    mov r0, #64
1:  ldr r1, =rt_outbox_length
    str r0, [r1]
    bx lr
    .thumb_func
rt_link_busy:
    ldr r0, =rt_outbox_length
    ldr r0, [r0]
    cmp r0, #0
    beq 1f
    mov r0, #1
1:  bx lr
    .thumb_func
rt_link_get_value:
    sub r0, #1
    cmp r0, #15
    bhi 1f
    lsl r0, #2
    ldr r1, =rt_values
    ldr r0, [r1, r0]
    bx lr
1:  mov r0, #0
    bx lr
    .thumb_func
rt_link_set_value:
    sub r0, #1
    cmp r0, #15
    bhi 1f
    lsl r0, #2
    ldr r2, =rt_values
    str r1, [r2, r0]
1:  bx lr
    .pool
"#,
    },
];

pub fn routine(name: &str) -> Option<&'static Routine> {
    ROUTINES.iter().find(|r| r.name == name)
}

/// The 3x5 font as 64 halfwords (row r in bits 3r..3r+2), for `rt_font`.
pub fn font_source() -> String {
    let mut out = String::from("    .align 2\nrt_font:\n");
    for glyph in crate::font::SMALL {
        let mut v = 0u16;
        for (row, bits) in glyph.iter().enumerate() {
            v |= ((*bits as u16) & 7) << (row * 3);
        }
        out.push_str(&format!("    .hword {v:#06x}\n"));
    }
    out
}
