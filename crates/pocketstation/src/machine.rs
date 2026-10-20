//! A complete PocketStation: CPU, hardware, kernel and start-up application.

use crate::cpu::{
    Cpu, Event, FLAG_F, FLAG_I, MODE_ABT, MODE_FIQ, MODE_IRQ, MODE_SVC, MODE_UND, MODE_USR,
};
use crate::hw::{Hardware, IRQ_DOCK};
use crate::kernel::{self, ram};
use crate::launcher::Launcher;
use crate::link::Link;
use crate::rom;
use std::collections::BTreeSet;

/// What the machine is running.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The built-in start-up application (clock and application menu).
    Launcher,
    /// A program from the card, started from this directory index.
    Program(u8),
    /// Execution stopped because of an error the console cannot recover from.
    Crashed(String),
}

/// Why [`Machine::run`] returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The requested time elapsed.
    Elapsed,
    /// The CPU reached a breakpoint (address).
    Breakpoint(u32),
    /// The program crashed (see [`State::Crashed`]).
    Crashed(String),
}

/// The real-time clock setting: (year, month, day, hour, minute, second, weekday 1=Sunday).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub weekday: u8,
}

impl Default for DateTime {
    fn default() -> Self {
        Self {
            year: 1999,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            weekday: 6,
        }
    }
}

pub struct Machine {
    pub cpu: Cpu,
    pub hw: Hardware,
    pub state: State,
    pub launcher: Launcher,
    pub link: Link,
    pub breakpoints: BTreeSet<u32>,
    /// Characters written by programs through the TTY convention (undefined
    /// instruction E6000010h with the character in r0).
    pub tty: String,
    /// Cycles left to run before the next launcher frame.
    launcher_clock: f64,
    /// Set by a kernel trap that replaced the whole CPU context.
    pub(crate) context_switched: bool,
    pub(crate) nested_return: Option<u32>,
}

impl Machine {
    /// A PocketStation with this flash (memory card image), powered on and
    /// showing the start-up application.
    pub fn new(card: Vec<u8>) -> Self {
        let mut hw = Hardware::new();
        hw.rom.copy_from_slice(&rom::image().0);
        let mut flash = card;
        flash.resize(crate::hw::FLASH_SIZE, 0xFF);
        hw.flash = flash;
        let mut machine = Self {
            cpu: Cpu::new(),
            hw,
            state: State::Launcher,
            launcher: Launcher::default(),
            link: Link::default(),
            breakpoints: BTreeSet::new(),
            tty: String::new(),
            launcher_clock: 0.0,
            context_switched: false,
            nested_return: None,
        };
        machine.power_on();
        machine
    }

    /// Cold start: runs the ROM reset vector, which initialises the kernel.
    pub fn power_on(&mut self) {
        self.cpu = Cpu::new();
        self.hw.remap = 0;
        self.hw.irq_latch = 0;
        self.hw.irq_mask = 0;
        self.cpu.r[15] = 0;
        // The reset vector traps into the kernel immediately.
        for _ in 0..4 {
            let (_, event) = self.cpu.step(&mut self.hw);
            if let Event::Trap(n) = event {
                self.trap(n);
                break;
            }
        }
    }

    pub fn set_date_time(&mut self, t: DateTime) {
        self.hw.rtc.year = (t.year % 100) as u8;
        self.hw.rtc.month = t.month.clamp(1, 12);
        self.hw.rtc.day = t.day.clamp(1, 31);
        self.hw.rtc.hour = t.hour.min(23);
        self.hw.rtc.minute = t.minute.min(59);
        self.hw.rtc.second = t.second.min(59);
        self.hw.rtc.weekday = t.weekday.clamp(1, 7);
        let century = (t.year / 100) as u8;
        self.hw.ram[ram::CENTURY as usize] = ((century / 10) << 4) | (century % 10);
        self.hw.ram[ram::OLD_YEAR as usize] =
            ((self.hw.rtc.year / 10) << 4) | (self.hw.rtc.year % 10);
        self.alarm_flags_set(1 << 23);
    }

    pub fn date_time(&self) -> DateTime {
        let century = crate::hw::from_bcd(self.hw.ram[ram::CENTURY as usize] as u32) as u16;
        let rtc = &self.hw.rtc;
        DateTime {
            year: century * 100 + rtc.year as u16,
            month: rtc.month,
            day: rtc.day,
            hour: rtc.hour,
            minute: rtc.minute,
            second: rtc.second,
            weekday: rtc.weekday,
        }
    }

    fn alarm_flags_set(&mut self, bits: u32) {
        let v = self.hw.ram_word(ram::ALARM) | bits;
        self.hw.set_ram_word(ram::ALARM, v);
    }

    pub fn set_buttons(&mut self, buttons: u8) {
        self.hw.set_buttons(buttons);
    }

    /// Inserts the PocketStation into (or removes it from) a PlayStation.
    pub fn set_docked(&mut self, docked: bool) {
        self.hw.set_docked(docked);
        if self.state == State::Launcher {
            self.sync_launcher_docking();
        }
    }

    pub(crate) fn sync_launcher_docking(&mut self) {
        let mut flags = self.hw.ram_word(ram::COM_FLAGS);
        if self.hw.docked {
            flags |= 1 << 9;
        } else {
            flags &= !(1 << 9);
        }
        self.hw.set_ram_word(ram::COM_FLAGS, flags);
        self.hw.irq_latch &= !IRQ_DOCK;
        let rotate = self.hw.docked;
        self.hw.lcd_mode = (self.hw.lcd_mode & !0x80) | ((rotate as u32) << 7);
    }

    /// The 32x32 picture as a person holding the PocketStation sees it.
    /// Returns rows with bit 0 = left pixel, 1 = black.
    pub fn screen(&self) -> [u32; 32] {
        let mode = self.hw.lcd_mode;
        if mode & 0x40 == 0 {
            return [0; 32];
        }
        let visible: std::ops::Range<usize> = match mode & 7 {
            1 => 0..8,
            2 => 8..16,
            3 => 16..24,
            4 => 24..32,
            5 => 0..16,
            6 => 8..24,
            7 => 16..32,
            _ => 0..32,
        };
        let mut rows = [0u32; 32];
        for (i, row) in rows.iter_mut().enumerate() {
            if visible.contains(&i) {
                *row = self.hw.vram[i];
            }
        }
        // The rotate bit flips the scan; docking flips the device itself.
        let flipped = (mode & 0x80 != 0) != self.hw.docked;
        if flipped {
            let mut out = [0u32; 32];
            for i in 0..32 {
                out[i] = rows[31 - i].reverse_bits();
            }
            out
        } else {
            rows
        }
    }

    pub fn led(&self) -> bool {
        self.hw.led()
    }

    /// Takes the speaker output as signed 16-bit samples at `rate` Hz covering
    /// everything since the previous call.
    pub fn take_audio(&mut self, rate: u32, since: &mut u64) -> Vec<i16> {
        let now = self.hw.cycles;
        let hz = self.hw.cpu_hz() as f64;
        let start = *since;
        if now <= start {
            return Vec::new();
        }
        let samples = (((now - start) as f64 / hz) * rate as f64) as usize;
        let mut out = Vec::with_capacity(samples);
        let events = &self.hw.audio_events;
        let mut level = events
            .iter()
            .rev()
            .find(|(c, _)| *c <= start)
            .map(|(_, l)| *l)
            .unwrap_or(0);
        let mut index = events.partition_point(|(c, _)| *c <= start);
        for s in 0..samples {
            let at = start + ((s as f64 / rate as f64) * hz) as u64;
            while index < events.len() && events[index].0 <= at {
                level = events[index].1;
                index += 1;
            }
            out.push(level.saturating_mul(48));
        }
        *since = start + ((samples as f64 / rate as f64) * hz) as u64;
        let keep_from = events
            .partition_point(|(c, _)| *c < *since)
            .saturating_sub(1);
        if keep_from > 0 {
            self.hw.audio_events.drain(..keep_from);
        }
        out
    }

    /// Starts the program whose first block is `block` with parameter `param`.
    pub fn launch(&mut self, block: u8, param: u32) -> Result<(), String> {
        self.start(block, param)
    }

    /// Returns to the start-up application.
    pub fn exit_to_launcher(&mut self) {
        self.start_launcher();
    }

    /// Runs for `seconds` of emulated time, or until a breakpoint or crash.
    pub fn run(&mut self, seconds: f64) -> Stop {
        let mut remaining = seconds;
        let mut first = true;
        while remaining > 0.0 {
            match self.state.clone() {
                State::Crashed(message) => return Stop::Crashed(message),
                State::Launcher => {
                    let step = remaining.min(1.0 / 120.0);
                    self.hw.advance_clock(step);
                    self.launcher_clock += step;
                    while self.launcher_clock >= 1.0 / 30.0 {
                        self.launcher_clock -= 1.0 / 30.0;
                        self.launcher_frame();
                    }
                    remaining -= step;
                }
                State::Program(_) => {
                    if self.hw.standby {
                        let step = remaining.min(1.0 / 120.0);
                        self.hw.advance_clock(step);
                        remaining -= step;
                        if self.hw.pending() != 0 {
                            self.hw.standby = false;
                        }
                        continue;
                    }
                    let hz = self.hw.cpu_hz() as f64;
                    let mut budget = (remaining * hz).ceil() as i64;
                    let start_hz = self.hw.cpu_hz();
                    while budget > 0 {
                        if !first && self.breakpoints.contains(&self.cpu.pc()) {
                            return Stop::Breakpoint(self.cpu.pc());
                        }
                        first = false;
                        let cycles = self.step_cpu();
                        budget -= cycles as i64;
                        if !matches!(self.state, State::Program(_))
                            || self.hw.standby
                            || self.hw.cpu_hz() != start_hz
                        {
                            break;
                        }
                    }
                    let used = ((remaining * hz).ceil() as i64 - budget.max(0)) as f64 / hz;
                    remaining -= used.max(1e-9);
                    if let State::Crashed(message) = &self.state {
                        return Stop::Crashed(message.clone());
                    }
                }
            }
        }
        Stop::Elapsed
    }

    /// Executes one instruction (taking a pending interrupt first). Returns cycles.
    pub fn step_cpu(&mut self) -> u32 {
        if self.cpu.interrupt(self.hw.irq_line(), self.hw.fiq_line()) {
            self.hw.advance(3);
        }
        let (cycles, event) = self.cpu.step(&mut self.hw);
        self.hw.advance(cycles);
        match event {
            Event::Trap(n) => self.trap(n),
            Event::Undefined(0xE600_0010) => self.tty_output(self.cpu.r[0] as u8),
            _ => {}
        }
        cycles
    }

    /// Runs program code at `address` from the kernel (FIQ context, interrupts
    /// masked) with r0/r1 set, returning r0. The interrupted context is restored.
    pub(crate) fn call_user(&mut self, address: u32, r0: u32, r1: u32) -> Result<u32, String> {
        let saved = self.cpu.clone();
        let saved_state = self.state.clone();
        let fiq_stack = if self.cpu.mode() == MODE_FIQ {
            self.cpu.r[13].saturating_sub(0x20) & !7
        } else {
            ram::FIQ_STACK_TOP
        };
        self.cpu
            .exception(rom::symbol("call_user"), MODE_FIQ, 0, true);
        self.cpu.cpsr |= FLAG_I | FLAG_F;
        self.cpu.r[13] = fiq_stack;
        self.cpu.r[0] = r0;
        self.cpu.r[1] = r1;
        self.cpu.r[12] = address;
        self.nested_return = None;
        let mut cycles = 0u64;
        while self.nested_return.is_none() {
            let (c, event) = self.cpu.step(&mut self.hw);
            self.hw.advance(c);
            cycles += c as u64;
            if let Event::Trap(n) = event {
                if n == rom::TRAP_RETURN {
                    self.nested_return = Some(self.cpu.r[0]);
                    break;
                }
                self.trap(n);
            }
            if matches!(self.state, State::Crashed(_)) || cycles > 4_000_000 {
                let message = match &self.state {
                    State::Crashed(m) => m.clone(),
                    _ => format!("link function at {address:#010x} did not return"),
                };
                self.cpu = saved;
                self.state = saved_state;
                return Err(message);
            }
        }
        let result = self.nested_return.take().unwrap_or(0);
        self.cpu = saved;
        self.state = saved_state;
        Ok(result)
    }

    /// TTY convention: the undefined instruction E6000010h prints chr(r0).
    fn tty_output(&mut self, c: u8) {
        if c == 10 || (0x20..0x7F).contains(&c) {
            self.tty.push(c as char);
            if self.tty.len() > 16 * 1024 {
                self.tty.drain(..8 * 1024);
            }
        }
    }

    pub(crate) fn crash(&mut self, message: String) {
        self.state = State::Crashed(message);
    }

    /// Resets the banked stack pointers the kernel uses.
    pub(crate) fn reset_stacks(&mut self) {
        let user_mode = self.cpu.mode();
        for (mode, sp) in [
            (MODE_SVC, ram::IRQ_STACK_TOP),
            (MODE_IRQ, ram::IRQ_STACK_TOP),
            (MODE_FIQ, ram::FIQ_STACK_TOP),
            (MODE_UND, ram::IRQ_STACK_TOP),
            (MODE_ABT, ram::IRQ_STACK_TOP),
        ] {
            self.cpu.set_banked_register(mode, 13, sp);
        }
        let _ = user_mode;
        self.cpu
            .set_banked_register(MODE_USR, 13, ram::USER_STACK_TOP);
    }

    pub fn kernel_ram(&self) -> kernel::Status {
        kernel::Status::read(&self.hw)
    }
}
