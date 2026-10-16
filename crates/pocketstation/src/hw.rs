//! PocketStation hardware: memory map, flash banking, interrupt controller,
//! timers, clock control, real-time clock, LCD, speaker DAC, LED and the
//! remaining I/O registers.

use crate::cpu::Bus;

pub const RAM_SIZE: usize = 0x800;
pub const ROM_SIZE: usize = 0x4000;
pub const FLASH_SIZE: usize = 0x20000;

pub const IRQ_FIRE: u32 = 1 << 0;
pub const IRQ_RIGHT: u32 = 1 << 1;
pub const IRQ_LEFT: u32 = 1 << 2;
pub const IRQ_DOWN: u32 = 1 << 3;
pub const IRQ_UP: u32 = 1 << 4;
pub const IRQ_COM: u32 = 1 << 6;
pub const IRQ_TIMER0: u32 = 1 << 7;
pub const IRQ_TIMER1: u32 = 1 << 8;
pub const IRQ_RTC: u32 = 1 << 9;
pub const IRQ_BATTERY: u32 = 1 << 10;
pub const IRQ_DOCK: u32 = 1 << 11;
pub const IRQ_IRDA: u32 = 1 << 12;
pub const IRQ_TIMER2: u32 = 1 << 13;
/// Sources delivered through the FIQ vector.
pub const FIQ_SOURCES: u32 = IRQ_COM | IRQ_TIMER2;
/// Level-triggered sources (the buttons): the latch mirrors the input.
const LEVEL_SOURCES: u32 = 0x3F;

/// Button bits as reported in `INT_INPUT`.
pub mod button {
    pub const FIRE: u8 = 1 << 0;
    pub const RIGHT: u8 = 1 << 1;
    pub const LEFT: u8 = 1 << 2;
    pub const DOWN: u8 = 1 << 3;
    pub const UP: u8 = 1 << 4;
}

/// CPU frequency in Hz for a `CLK_MODE` ratio.
pub fn clock_hz(mode: u32) -> u32 {
    match mode & 0xF {
        0 => 32_768,
        1 => 63_488,
        2 => 126_976,
        3 => 253_952,
        4 => 507_904,
        5 => 1_015_808,
        6 => 1_998_848,
        7 => 3_997_696,
        _ => 7_995_392,
    }
}

#[derive(Clone, Debug, Default)]
pub struct Timer {
    pub reload: u32,
    pub count: u32,
    pub mode: u32,
    prescale: u32,
}

impl Timer {
    fn divider(&self) -> u32 {
        match self.mode & 3 {
            1 => 32,
            2 => 512,
            _ => 2,
        }
    }
    /// Advances by CPU cycles, returning the number of underflows.
    fn advance(&mut self, cycles: u32) -> u32 {
        if self.mode & 4 == 0 {
            return 0;
        }
        self.prescale += cycles;
        let divider = self.divider();
        let mut ticks = self.prescale / divider;
        self.prescale %= divider;
        let mut underflows = 0;
        while ticks > 0 {
            if ticks <= self.count {
                self.count -= ticks;
                ticks = 0;
            } else {
                ticks -= self.count + 1;
                self.count = self.reload & 0xFFFF;
                underflows += 1;
                if self.reload == 0 && ticks > 0 {
                    // A zero reload underflows on every tick.
                    underflows += ticks;
                    ticks = 0;
                }
            }
        }
        underflows
    }
}

/// Battery-backed calendar clock in BCD, as the RTC registers present it.
#[derive(Clone, Debug)]
pub struct Rtc {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    /// 1 = Sunday .. 7 = Saturday.
    pub weekday: u8,
    pub day: u8,
    pub month: u8,
    /// Two-digit year (the century lives in kernel RAM).
    pub year: u8,
    pub mode: u32,
    /// Fraction of the current second elapsed, in seconds.
    phase: f64,
    /// Level of the 1 Hz (or 4096 Hz when paused) square wave.
    pub square: bool,
}

impl Default for Rtc {
    fn default() -> Self {
        Self {
            second: 0,
            minute: 0,
            hour: 0,
            weekday: 6,
            day: 1,
            month: 1,
            year: 99,
            mode: 0,
            phase: 0.0,
            square: false,
        }
    }
}

fn bcd(v: u8) -> u32 {
    (((v / 10) << 4) | (v % 10)) as u32
}
pub fn from_bcd(v: u32) -> u8 {
    (((v >> 4) & 0xF) * 10 + (v & 0xF)) as u8
}

fn days_in_month(month: u8, year: u8) -> u8 {
    match month {
        2 => {
            if year.is_multiple_of(4) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Rtc {
    pub fn time_register(&self) -> u32 {
        bcd(self.second)
            | (bcd(self.minute) << 8)
            | (bcd(self.hour) << 16)
            | ((self.weekday as u32) << 24)
    }
    pub fn date_register(&self) -> u32 {
        bcd(self.day) | (bcd(self.month) << 8) | (bcd(self.year) << 16)
    }
    /// Advances one second, carrying into the calendar.
    pub fn tick(&mut self) {
        self.second += 1;
        if self.second < 60 {
            return;
        }
        self.second = 0;
        self.minute += 1;
        if self.minute < 60 {
            return;
        }
        self.minute = 0;
        self.hour += 1;
        if self.hour < 24 {
            return;
        }
        self.hour = 0;
        self.weekday = self.weekday % 7 + 1;
        self.day += 1;
        if self.day <= days_in_month(self.month, self.year) {
            return;
        }
        self.day = 1;
        self.month += 1;
        if self.month <= 12 {
            return;
        }
        self.month = 1;
        self.year = (self.year + 1) % 100;
    }
    /// `RTC_ADJUST`: increments the field selected in `RTC_MODE`.
    fn adjust(&mut self) {
        match (self.mode >> 1) & 7 {
            0 => self.second = (self.second + 1) % 60,
            1 => self.minute = (self.minute + 1) % 60,
            2 => self.hour = (self.hour + 1) % 24,
            3 => self.weekday = self.weekday % 7 + 1,
            4 => self.day = self.day % 31 + 1,
            5 => self.month = self.month % 12 + 1,
            6 => self.year = (self.year + 1) % 100,
            _ => {}
        }
    }
    /// Advances by `seconds`, returning the number of rising edges of the square wave.
    fn advance(&mut self, seconds: f64) -> u32 {
        let paused = self.mode & 1 != 0;
        let period = if paused { 1.0 / 4096.0 } else { 1.0 };
        let mut edges = 0;
        self.phase += seconds;
        while self.phase >= period {
            self.phase -= period;
            if !paused {
                self.tick();
            }
            edges += 1;
        }
        self.square = self.phase < period / 2.0;
        edges
    }
}

/// A hardware condition that behaves differently on a real PocketStation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
    pub pc: u32,
    pub message: String,
}

pub struct Hardware {
    pub ram: Vec<u8>,
    pub rom: Vec<u8>,
    pub flash: Vec<u8>,
    /// Extra flash bytes at 06000300h (serial number, LCD calibration).
    pub flash_extra: Vec<u8>,
    pub remap: u32,
    pub bank_flags: u32,
    pub bank_values: [u32; 16],
    pub flash_wait1: u32,
    pub flash_wait2: u32,
    flash_unlock: u8,
    flash_status: u32,
    pub irq_raw: u32,
    pub irq_latch: u32,
    pub irq_mask: u32,
    pub timers: [Timer; 3],
    pub clock_mode: u32,
    clock_settle: u32,
    pub standby: bool,
    pub rtc: Rtc,
    pub lcd_mode: u32,
    pub lcd_cal: u32,
    pub vram: [u32; 32],
    pub iop_control: u32,
    /// PIO outputs: a set bit drives the line high (LED off, speaker stand-by).
    pub iop_output: u32,
    pub dac_control: u32,
    pub dac_data: u32,
    pub battery_control: u32,
    pub irda_mode: u32,
    pub irda_data: u32,
    pub com_mode: u32,
    pub com_control1: u32,
    pub com_control2: u32,
    pub com_data: u32,
    pub buttons: u8,
    pub docked: bool,
    pub battery_low: bool,
    /// CPU cycles executed since power on.
    pub cycles: u64,
    access_cycles: u32,
    abort: bool,
    /// Address and state of the executing instruction (for diagnostics).
    pub current_pc: u32,
    pub thumb: bool,
    last_ram_read: u32,
    pub warnings: Vec<Warning>,
    /// Speaker level changes: (cycle, signed level in -512..511).
    pub audio_events: Vec<(u64, i16)>,
    /// Data abort addresses for the debugger.
    pub last_abort: Option<u32>,
}

impl Default for Hardware {
    fn default() -> Self {
        Self::new()
    }
}

impl Hardware {
    pub fn new() -> Self {
        let mut flash_extra = vec![0xFF; 0x100];
        // Serial number, an unknown word and the LCD calibration value.
        flash_extra[0..2].copy_from_slice(&0x4550u16.to_le_bytes());
        flash_extra[2..4].copy_from_slice(&0x4B4Fu16.to_le_bytes());
        flash_extra[4..6].copy_from_slice(&0x0000u16.to_le_bytes());
        flash_extra[8..10].copy_from_slice(&0x001Au16.to_le_bytes());
        flash_extra[12..14].copy_from_slice(&0x0010u16.to_le_bytes());
        Self {
            ram: vec![0; RAM_SIZE],
            rom: vec![0; ROM_SIZE],
            flash: vec![0xFF; FLASH_SIZE],
            flash_extra,
            remap: 0,
            bank_flags: 0,
            bank_values: [0xF; 16],
            flash_wait1: 0,
            flash_wait2: 0x14,
            flash_unlock: 0,
            flash_status: 0,
            irq_raw: 0,
            irq_latch: 0,
            irq_mask: 0,
            timers: Default::default(),
            clock_mode: 7,
            clock_settle: 0,
            standby: false,
            rtc: Rtc::default(),
            lcd_mode: 0,
            lcd_cal: 0,
            vram: [0; 32],
            iop_control: 0,
            iop_output: 0x20,
            dac_control: 0,
            dac_data: 0,
            battery_control: 3,
            irda_mode: 2,
            irda_data: 0,
            com_mode: 0,
            com_control1: 0,
            com_control2: 0,
            com_data: 0,
            buttons: 0,
            docked: false,
            battery_low: false,
            cycles: 0,
            access_cycles: 0,
            abort: false,
            current_pc: 0,
            thumb: false,
            last_ram_read: 0,
            warnings: Vec::new(),
            audio_events: Vec::new(),
            last_abort: None,
        }
    }

    pub fn cpu_hz(&self) -> u32 {
        clock_hz(self.clock_mode)
    }

    pub fn serial(&self) -> u32 {
        u32::from_le_bytes(self.flash_extra[0..4].try_into().unwrap())
    }

    fn warn(&mut self, message: String) {
        let pc = self.current_pc;
        if self.warnings.len() < 64
            && !self
                .warnings
                .iter()
                .any(|w| w.pc == pc && w.message == message)
        {
            self.warnings.push(Warning { pc, message });
        }
    }

    // ------------------------------------------------------------ interrupts

    pub fn input_bits(&self) -> u32 {
        let mut raw = self.irq_raw & !(LEVEL_SOURCES | IRQ_RTC | IRQ_DOCK | IRQ_BATTERY);
        raw |= self.buttons as u32 & 0x1F;
        if self.rtc.square {
            raw |= IRQ_RTC;
        }
        if self.docked {
            raw |= IRQ_DOCK;
        }
        if self.battery_low {
            raw |= IRQ_BATTERY;
        }
        raw
    }

    /// Requests edge-triggered interrupts.
    pub fn raise(&mut self, bits: u32) {
        self.irq_latch |= bits & !LEVEL_SOURCES;
        if bits & (IRQ_FIRE | IRQ_DOCK) != 0 {
            self.standby = false;
        }
    }

    pub fn pending(&self) -> u32 {
        ((self.irq_latch & !LEVEL_SOURCES) | (self.input_bits() & LEVEL_SOURCES)) & self.irq_mask
    }
    pub fn irq_line(&self) -> bool {
        self.pending() & !FIQ_SOURCES != 0
    }
    pub fn fiq_line(&self) -> bool {
        self.pending() & FIQ_SOURCES != 0
    }

    pub fn set_buttons(&mut self, buttons: u8) {
        let pressed = buttons & !self.buttons;
        self.buttons = buttons & 0x1F;
        if pressed != 0 && self.irq_mask & pressed as u32 != 0 {
            self.standby = false;
        }
    }

    pub fn set_docked(&mut self, docked: bool) {
        if docked != self.docked {
            self.docked = docked;
            self.raise(IRQ_DOCK);
        }
    }

    // ----------------------------------------------------------------- time

    /// Advances the timers and the clock by `cycles` CPU cycles.
    pub fn advance(&mut self, cycles: u32) {
        self.cycles += cycles as u64;
        if self.clock_settle > 0 {
            self.clock_settle = self.clock_settle.saturating_sub(cycles);
        }
        let bits = [IRQ_TIMER0, IRQ_TIMER1, IRQ_TIMER2];
        for (timer, bit) in self.timers.iter_mut().zip(bits) {
            if timer.advance(cycles) > 0 {
                self.irq_latch |= bit;
            }
        }
        let seconds = cycles as f64 / self.cpu_hz() as f64;
        self.advance_clock(seconds);
    }

    /// Advances only the real-time clock (used while the CPU is stopped).
    pub fn advance_clock(&mut self, seconds: f64) {
        if self.rtc.advance(seconds) > 0 {
            self.raise(IRQ_RTC);
        }
    }

    // ----------------------------------------------------------------- flash

    /// Physical flash offset for a virtual flash address, honouring the bank map.
    pub fn virtual_to_physical(&self, address: u32) -> Option<u32> {
        if self.remap & 2 == 0 {
            return None;
        }
        let offset = address & 0x00FF_FFFF;
        if offset >= FLASH_SIZE as u32 {
            return None;
        }
        let virtual_bank = offset >> 13;
        (0..16u32)
            .find(|&p| {
                self.bank_flags & (1 << p) != 0
                    && self.bank_values[p as usize] & 0xF == virtual_bank
            })
            .map(|p| (p << 13) | (offset & 0x1FFF))
    }

    fn flash_write(&mut self, offset: u32, value: u16) {
        // Unlock sequence: [55AA]=FFAA, [2A54]=FF55, [55AA]=FFA0, then data.
        match (self.flash_unlock, offset & 0x1FFFE, value) {
            (0, 0x55AA, 0xFFAA) => self.flash_unlock = 1,
            (1, 0x2A54, 0xFF55) => self.flash_unlock = 2,
            (2, 0x55AA, 0xFFA0) => self.flash_unlock = 3,
            (3, _, _) if self.flash_wait2 & 1 != 0 => {
                let at = (offset & 0x1FFFE) as usize;
                self.flash[at..at + 2].copy_from_slice(&value.to_le_bytes());
            }
            _ => {
                self.flash_unlock = 0;
                self.warn(format!(
                    "write to flash at {:#010x} without the unlock sequence is ignored",
                    0x0800_0000 | offset
                ));
            }
        }
    }

    // ------------------------------------------------------------------ I/O

    // Specific registers come before the mirrored ranges they belong to.
    #[allow(clippy::match_overlapping_arm)]
    fn io_read(&mut self, address: u32) -> Option<u32> {
        Some(match address & 0x0FFF_FFFC {
            0x0600_0000 => self.remap,
            0x0600_0004 => std::mem::take(&mut self.flash_status),
            0x0600_0008 => self.bank_flags,
            0x0600_000C => self.flash_wait1,
            // Bit 2: write complete / ready.
            0x0600_0010 => self.flash_wait2 | 4,
            a @ 0x0600_0100..=0x0600_013C => self.bank_values[((a - 0x0600_0100) / 4) as usize],
            a @ 0x0600_0300..=0x0600_03FC => {
                let at = (a - 0x0600_0300) as usize;
                u32::from_le_bytes(self.flash_extra[at..at + 4].try_into().unwrap())
            }
            a @ 0x0600_0000..=0x06FF_FFFF => {
                let _ = a;
                0
            }
            0x0A00_0000 => self.pending(),
            0x0A00_0004 => self.input_bits(),
            0x0A00_0008 => self.irq_mask,
            0x0A00_000C | 0x0A00_0010 => self.irq_mask,
            0x0A00_0014..=0x0A7F_FFFC => self.irq_mask,
            0x0A80_0000 | 0x0A80_000C => self.timers[0].reload,
            0x0A80_0004 => self.timers[0].count,
            0x0A80_0008 => self.timers[0].mode,
            0x0A80_0010 => self.timers[1].reload,
            0x0A80_0014 => self.timers[1].count,
            0x0A80_0018 => self.timers[1].mode,
            0x0A80_0020 => self.timers[2].reload,
            0x0A80_0024 => self.timers[2].count,
            0x0A80_0028 => self.timers[2].mode,
            0x0A80_0000..=0x0AFF_FFFC => self.timers[0].reload,
            0x0B00_0000 => {
                let ready = if self.clock_settle == 0 { 0x10 } else { 0 };
                self.clock_mode | ready
            }
            0x0B00_0004 => 0,
            0x0B80_0000 => self.rtc.mode,
            0x0B80_0004 => 0,
            0x0B80_000C => self.rtc.date_register(),
            0x0B80_0008..=0x0BFF_FFFC => self.rtc.time_register(),
            0x0C00_0000 => self.com_mode,
            0x0C00_0004 => 0,
            0x0C00_0008 => self.com_data,
            0x0C00_0010 => self.com_control1,
            0x0C00_0014 => 1,
            0x0C00_0018 => self.com_control2,
            0x0C00_0000..=0x0C7F_FFFC => 0,
            0x0C80_0000 => self.irda_mode,
            0x0C80_0004 => self.irda_data,
            0x0C80_0000..=0x0CFF_FFFC => 0,
            0x0D00_0000 => self.lcd_mode,
            0x0D00_0004 => self.lcd_cal,
            a @ 0x0D00_0100..=0x0D00_017C => self.vram[((a - 0x0D00_0100) / 4) as usize],
            0x0D00_0000..=0x0D7F_FFFC => 0,
            0x0D80_0000 => self.iop_control,
            0x0D80_0004 => self.iop_output,
            0x0D80_0008 => 0,
            0x0D80_000C => (self.iop_output & 0x0F) | ((self.docked as u32) << 4),
            0x0D80_0010 => self.dac_control,
            0x0D80_0014 => self.dac_data,
            0x0D80_0020 => self.battery_control,
            0x0D80_0000..=0x0DFF_FFFC => 0,
            _ => return None,
        })
    }

    #[allow(clippy::match_overlapping_arm)]
    fn io_write(&mut self, address: u32, value: u32) -> bool {
        match address & 0x0FFF_FFFC {
            0x0600_0000 => {
                self.remap = value & 3;
                if self.remap & 2 != 0 {
                    // Report overlapping or missing virtual banks.
                    let mut seen = 0u32;
                    let mut status = 0;
                    for p in 0..16 {
                        if self.bank_flags & (1 << p) != 0 {
                            let v = self.bank_values[p] & 0xF;
                            if seen & (1 << v) != 0 {
                                status |= 1;
                            }
                            seen |= 1 << v;
                        }
                    }
                    if seen == 0 {
                        status |= 2;
                    }
                    self.flash_status = status;
                }
            }
            0x0600_0008 => self.bank_flags = value & 0xFFFF,
            0x0600_000C => self.flash_wait1 = value,
            0x0600_0010 => {
                self.flash_wait2 = value;
                if value & 1 == 0 {
                    self.flash_unlock = 0;
                }
            }
            a @ 0x0600_0100..=0x0600_013C => {
                self.bank_values[((a - 0x0600_0100) / 4) as usize] = value & 0xF
            }
            0x0600_0000..=0x06FF_FFFC => {}
            0x0A00_0008 => self.irq_mask |= value & 0x3FFF,
            0x0A00_000C => self.irq_mask &= !value,
            0x0A00_0010 => self.irq_latch &= !value,
            0x0A00_0000..=0x0A7F_FFFC => {}
            0x0A80_0000 => self.timers[0].reload = value & 0xFFFF,
            0x0A80_0004 => self.timers[0].count = value & 0xFFFF,
            0x0A80_0008 => self.timer_mode(0, value),
            0x0A80_0010 => self.timers[1].reload = value & 0xFFFF,
            0x0A80_0014 => self.timers[1].count = value & 0xFFFF,
            0x0A80_0018 => self.timer_mode(1, value),
            0x0A80_0020 => self.timers[2].reload = value & 0xFFFF,
            0x0A80_0024 => self.timers[2].count = value & 0xFFFF,
            0x0A80_0028 => self.timer_mode(2, value),
            0x0A80_0000..=0x0AFF_FFFC => {}
            0x0B00_0000 => {
                if value & 0xF == 0 {
                    self.warn("CLK_MODE 0 hangs the hardware".into());
                }
                self.clock_mode = value & 0xF;
                self.clock_settle = 64;
            }
            0x0B00_0004 => {
                if value & 1 != 0 {
                    self.standby = true;
                }
            }
            0x0B80_0000 => self.rtc.mode = value & 0xF,
            0x0B80_0004 => self.rtc.adjust(),
            0x0B80_0000..=0x0BFF_FFFC => {}
            0x0C00_0000 => self.com_mode = value,
            0x0C00_0008 => self.com_data = value & 0xFF,
            0x0C00_0010 => self.com_control1 = value,
            0x0C00_0018 => self.com_control2 = value,
            0x0C00_0000..=0x0C7F_FFFC => {}
            0x0C80_0000 => self.irda_mode = value & 0xF,
            0x0C80_0004 => self.irda_data = value & 1,
            0x0C80_0000..=0x0CFF_FFFC => {}
            0x0D00_0000 => self.lcd_mode = value & 0xFF,
            0x0D00_0004 => self.lcd_cal = value & 0x3F,
            a @ 0x0D00_0100..=0x0D00_017C => self.vram[((a - 0x0D00_0100) / 4) as usize] = value,
            0x0D00_0000..=0x0D7F_FFFC => {}
            0x0D80_0000 => self.iop_control = value & 0xF,
            0x0D80_0004 => {
                self.iop_output |= value;
                self.speaker_changed();
            }
            0x0D80_0008 => {
                self.iop_output &= !value;
                self.speaker_changed();
            }
            0x0D80_0010 => {
                self.dac_control = value & 1;
                self.speaker_changed();
            }
            0x0D80_0014 => {
                self.dac_data = value;
                self.speaker_changed();
            }
            0x0D80_0020 => self.battery_control = value & 7,
            0x0D80_0000..=0x0DFF_FFFC => {}
            _ => return false,
        }
        true
    }

    fn timer_mode(&mut self, index: usize, value: u32) {
        let timer = &mut self.timers[index];
        if value & 4 != 0 && timer.mode & 4 == 0 {
            timer.prescale = 0;
        }
        timer.mode = value & 7;
    }

    pub fn speaker_on(&self) -> bool {
        self.dac_control & 1 != 0 && self.iop_output & 0x20 == 0
    }

    /// The red LED is lit while PIO1 is driven low.
    pub fn led(&self) -> bool {
        self.iop_output & 2 == 0
    }

    pub fn speaker_level(&self) -> i16 {
        if !self.speaker_on() {
            return 0;
        }
        // Bits 6-15: signed 10-bit level.
        ((self.dac_data as u16 as i16) >> 6).clamp(-512, 511)
    }

    fn speaker_changed(&mut self) {
        let level = self.speaker_level();
        if self.audio_events.last().is_none_or(|(_, l)| *l != level) {
            if self.audio_events.len() > 1 << 16 {
                self.audio_events.drain(..1 << 15);
            }
            self.audio_events.push((self.cycles, level));
        }
    }

    // --------------------------------------------------------------- memory

    fn garbage_byte(&mut self, address: u32) -> u8 {
        self.warn(format!(
            "8-bit read from {address:#010x} (flash or BIOS) returns garbage on a real PocketStation; read 16 bits instead"
        ));
        let pc = self.current_pc;
        let prefetch = if self.thumb {
            let low = self.peek16(pc.wrapping_add(4)) as u32;
            (low << 16) | low
        } else {
            self.peek32(pc.wrapping_add(8))
        };
        let word = prefetch | (self.last_ram_read & 0xFFFF_FFD0);
        (word >> ((address & 3) * 8)) as u8
    }

    /// Reads memory without side effects (debugger and diagnostics).
    pub fn peek32(&self, address: u32) -> u32 {
        let a = address & !3;
        match a >> 24 {
            0x00 if self.remap & 1 == 0 => {
                let at = (a as usize) & (ROM_SIZE - 1);
                u32::from_le_bytes(self.rom[at..at + 4].try_into().unwrap())
            }
            0x00 => {
                let at = (a as usize) & (RAM_SIZE - 1);
                u32::from_le_bytes(self.ram[at..at + 4].try_into().unwrap())
            }
            0x02 => match self.virtual_to_physical(a) {
                Some(p) => {
                    u32::from_le_bytes(self.flash[p as usize..p as usize + 4].try_into().unwrap())
                }
                None => 0,
            },
            0x04 => {
                let at = (a as usize) & (ROM_SIZE - 1);
                u32::from_le_bytes(self.rom[at..at + 4].try_into().unwrap())
            }
            0x08 => {
                let at = (a as usize) & (FLASH_SIZE - 1);
                u32::from_le_bytes(self.flash[at..at + 4].try_into().unwrap())
            }
            0x0D if (0x0D00_0100..0x0D00_0180).contains(&a) => {
                self.vram[((a - 0x0D00_0100) / 4) as usize]
            }
            _ => 0,
        }
    }
    pub fn peek16(&self, address: u32) -> u16 {
        (self.peek32(address) >> ((address & 2) * 8)) as u16
    }
    pub fn peek8(&self, address: u32) -> u8 {
        (self.peek32(address) >> ((address & 3) * 8)) as u8
    }

    /// Writes RAM, VRAM or physical flash directly (kernel and debugger).
    pub fn poke8(&mut self, address: u32, value: u8) {
        match address >> 24 {
            0x00 => self.ram[(address as usize) & (RAM_SIZE - 1)] = value,
            0x08 => self.flash[(address as usize) & (FLASH_SIZE - 1)] = value,
            0x0D if (0x0D00_0100..0x0D00_0180).contains(&address) => {
                let index = ((address - 0x0D00_0100) / 4) as usize;
                let shift = (address & 3) * 8;
                self.vram[index] =
                    (self.vram[index] & !(0xFF << shift)) | ((value as u32) << shift);
            }
            _ => {}
        }
    }
    pub fn poke32(&mut self, address: u32, value: u32) {
        for (i, b) in value.to_le_bytes().into_iter().enumerate() {
            self.poke8(address + i as u32, b);
        }
    }
    pub fn ram_word(&self, address: u32) -> u32 {
        let at = (address as usize) & (RAM_SIZE - 4);
        u32::from_le_bytes(self.ram[at..at + 4].try_into().unwrap())
    }
    pub fn set_ram_word(&mut self, address: u32, value: u32) {
        let at = (address as usize) & (RAM_SIZE - 4);
        self.ram[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn fault(&mut self, address: u32) {
        self.abort = true;
        self.last_abort = Some(address);
    }

    fn read_word(&mut self, address: u32, fetch: bool) -> u32 {
        let a = address & !3;
        match a >> 24 {
            0x00 => {
                self.access_cycles += if fetch { 0 } else { 1 };
                if self.remap & 1 == 0 {
                    let at = (a as usize) & (ROM_SIZE - 1);
                    return u32::from_le_bytes(self.rom[at..at + 4].try_into().unwrap());
                }
                let at = (a as usize) & (RAM_SIZE - 1);
                let v = u32::from_le_bytes(self.ram[at..at + 4].try_into().unwrap());
                if !fetch {
                    self.last_ram_read = v;
                }
                v
            }
            0x02 => match self.virtual_to_physical(a) {
                Some(p) => {
                    self.access_cycles += if fetch { 0 } else { 2 };
                    u32::from_le_bytes(self.flash[p as usize..p as usize + 4].try_into().unwrap())
                }
                None => {
                    self.flash_status |= if fetch { 8 } else { 4 };
                    self.fault(address);
                    0
                }
            },
            0x04 => {
                self.access_cycles += if fetch { 0 } else { 1 };
                let at = (a as usize) & (ROM_SIZE - 1);
                u32::from_le_bytes(self.rom[at..at + 4].try_into().unwrap())
            }
            0x08 => {
                self.access_cycles += if fetch { 0 } else { 2 };
                let at = (a as usize) & (FLASH_SIZE - 1);
                u32::from_le_bytes(self.flash[at..at + 4].try_into().unwrap())
            }
            0x06 | 0x0A..=0x0D => {
                self.access_cycles += 2;
                match self.io_read(a) {
                    Some(v) => v,
                    None => {
                        self.fault(address);
                        0
                    }
                }
            }
            _ => {
                self.fault(address);
                0
            }
        }
    }

    fn write_word(&mut self, address: u32, value: u32, mask: u32) {
        let a = address & !3;
        match a >> 24 {
            0x00 => {
                self.access_cycles += 1;
                let at = (a as usize) & (RAM_SIZE - 1);
                let old = u32::from_le_bytes(self.ram[at..at + 4].try_into().unwrap());
                let new = (old & !mask) | (value & mask);
                self.ram[at..at + 4].copy_from_slice(&new.to_le_bytes());
            }
            0x02 => {
                self.access_cycles += 2;
                match self.virtual_to_physical(a) {
                    Some(p) => {
                        for half in 0..2 {
                            if mask & (0xFFFF << (half * 16)) != 0 {
                                self.flash_write(p + half * 2, (value >> (half * 16)) as u16);
                            }
                        }
                    }
                    None => self.fault(address),
                }
            }
            0x04 => self.access_cycles += 2,
            0x08 => {
                self.access_cycles += 2;
                let offset = a & (FLASH_SIZE as u32 - 1);
                for half in 0..2 {
                    if mask & (0xFFFF << (half * 16)) != 0 {
                        self.flash_write(offset + half * 2, (value >> (half * 16)) as u16);
                    }
                }
            }
            0x06 | 0x0A..=0x0D => {
                self.access_cycles += 2;
                let merged = if mask == 0xFFFF_FFFF {
                    value
                } else if (0x0D00_0100..0x0D00_0180).contains(&a) {
                    let old = self.vram[((a - 0x0D00_0100) / 4) as usize];
                    (old & !mask) | (value & mask)
                } else {
                    // Narrow writes to registers place the value in its byte lanes.
                    let old = self.io_read(a).unwrap_or(0);
                    (old & !mask) | (value & mask)
                };
                if !self.io_write(a, merged) {
                    self.fault(address);
                }
            }
            _ => self.fault(address),
        }
    }
}

impl Bus for Hardware {
    fn read32(&mut self, address: u32) -> u32 {
        self.read_word(address, false)
    }
    fn read16(&mut self, address: u32) -> u16 {
        (self.read_word(address, false) >> ((address & 2) * 8)) as u16
    }
    fn read8(&mut self, address: u32) -> u8 {
        let region = address >> 24;
        let narrow_rom = region == 0x02
            || region == 0x04
            || region == 0x08
            || (region == 0x00 && self.remap & 1 == 0)
            || (0x0600_0300..0x0600_0400).contains(&address);
        let word = self.read_word(address, false);
        if narrow_rom && !self.abort {
            return self.garbage_byte(address);
        }
        (word >> ((address & 3) * 8)) as u8
    }
    fn write32(&mut self, address: u32, value: u32) {
        self.write_word(address, value, 0xFFFF_FFFF);
    }
    fn write16(&mut self, address: u32, value: u16) {
        let shift = (address & 2) * 8;
        self.write_word(address, (value as u32) << shift, 0xFFFF << shift);
    }
    fn write8(&mut self, address: u32, value: u8) {
        let shift = (address & 3) * 8;
        self.write_word(address, (value as u32) << shift, 0xFF << shift);
    }
    fn fetch32(&mut self, address: u32) -> u32 {
        self.current_pc = address;
        self.thumb = false;
        self.access_cycles += match address >> 24 {
            0x02 | 0x08 => 1,
            _ => 0,
        };
        self.read_word(address, true)
    }
    fn fetch16(&mut self, address: u32) -> u16 {
        self.current_pc = address;
        self.thumb = true;
        (self.read_word(address, true) >> ((address & 2) * 8)) as u16
    }
    fn take_abort(&mut self) -> bool {
        std::mem::take(&mut self.abort)
    }
    fn take_cycles(&mut self) -> u32 {
        std::mem::take(&mut self.access_cycles)
    }
    fn is_trap_region(&self, address: u32) -> bool {
        address >> 24 == 0x04 || (address >> 24 == 0x00 && self.remap & 1 == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timers_reload_and_raise_interrupts() {
        let mut hw = Hardware::new();
        hw.irq_mask = IRQ_TIMER0;
        hw.timers[0].reload = 9;
        hw.timers[0].count = 9;
        hw.timers[0].mode = 4; // div 2
        hw.advance(19);
        assert!(!hw.irq_line());
        hw.advance(1);
        assert!(hw.irq_line());
        assert_eq!(hw.timers[0].count, 9);
        hw.write32(0x0A00_0010, IRQ_TIMER0);
        assert!(!hw.irq_line());
    }

    #[test]
    fn rtc_carries_into_the_calendar() {
        let mut rtc = Rtc {
            second: 59,
            minute: 59,
            hour: 23,
            day: 31,
            month: 12,
            year: 99,
            weekday: 7,
            ..Default::default()
        };
        rtc.tick();
        assert_eq!((rtc.day, rtc.month, rtc.year, rtc.weekday), (1, 1, 0, 1));
        assert_eq!(rtc.time_register(), 0x0100_0000);
    }

    #[test]
    fn virtual_flash_follows_the_bank_table() {
        let mut hw = Hardware::new();
        hw.flash[3 * 0x2000..3 * 0x2000 + 4].copy_from_slice(&0xCAFEBABEu32.to_le_bytes());
        hw.write32(0x0600_0008, 1 << 3);
        hw.write32(0x0600_010C, 0);
        hw.write32(0x0600_0000, 3);
        assert_eq!(hw.read32(0x0200_0000), 0xCAFE_BABE);
        assert!(!hw.take_abort());
        hw.read32(0x0200_2000);
        assert!(hw.take_abort());
        // Byte reads from flash are garbage and produce a warning.
        hw.read8(0x0200_0000);
        assert!(!hw.warnings.is_empty());
    }

    #[test]
    fn vram_accepts_narrow_writes() {
        let mut hw = Hardware::new();
        hw.write8(0x0D00_0101, 0xFF);
        assert_eq!(hw.vram[0], 0xFF00);
        hw.write16(0x0D00_0106, 0x1234);
        assert_eq!(hw.vram[1], 0x1234_0000);
    }

    #[test]
    fn flash_programming_needs_the_unlock_sequence() {
        let mut hw = Hardware::new();
        hw.write16(0x0800_4000, 0x1234);
        assert_eq!(hw.flash[0x4000], 0xFF);
        hw.write32(0x0600_0010, 0x21);
        hw.write16(0x0800_55AA, 0xFFAA);
        hw.write16(0x0800_2A54, 0xFF55);
        hw.write16(0x0800_55AA, 0xFFA0);
        hw.write16(0x0800_4000, 0x1234);
        assert_eq!(&hw.flash[0x4000..0x4002], &[0x34, 0x12]);
    }
}
