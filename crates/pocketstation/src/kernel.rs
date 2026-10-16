//! The kernel's system calls and exception work, executed when the ROM traps.

use crate::card;
use crate::cpu::{FLAG_T, MODE_FIQ, MODE_IRQ, MODE_SVC, MODE_USR};
use crate::file;
use crate::hw::{Hardware, IRQ_DOCK, IRQ_FIRE, IRQ_RTC};
use crate::machine::{Machine, State};
use crate::rom;

/// Kernel RAM layout (the first 200h bytes of RAM).
pub mod ram {
    pub const VECTORS: u32 = 0x00;
    pub const VECTOR_TARGETS: u32 = 0x20;
    pub const SECTOR_BUFFER: u32 = 0x40;
    pub const COM_FLAGS: u32 = 0xC0;
    pub const FUNC3: u32 = 0xC8;
    pub const BU50_VALUE: u32 = 0xCA;
    pub const OLD_YEAR: u32 = 0xCD;
    pub const ALTERNATE_DIR: u32 = 0xCE;
    pub const CENTURY: u32 = 0xCF;
    pub const DIR_INDEX: u32 = 0xD0;
    pub const NEW_DIR_INDEX: u32 = 0xD2;
    pub const NEW_PARAM: u32 = 0xD4;
    pub const ALARM: u32 = 0xD8;
    pub const SWI_TABLE: u32 = 0xE0;
    pub const CARD_FLAG: u32 = 0xF0;
    pub const CALLBACKS: u32 = 0xF8;
    pub const SNAPSHOT_ID: u32 = 0x108;
    pub const IRQ_STACK_TOP: u32 = 0x180;
    pub const FIQ_STACK_TOP: u32 = 0x200;
    pub const USER_RAM: u32 = 0x200;
    pub const USER_STACK_TOP: u32 = 0x800;
}

/// ComFlags bits.
pub mod com {
    pub const DOCK_EVENT: u32 = 1 << 8;
    pub const ENABLED: u32 = 1 << 9;
    pub const PROTECT_BROKEN: u32 = 1 << 10;
    pub const START_REQUEST: u32 = 1 << 11;
    pub const AUTO_AUDIO: u32 = 1 << 16;
    pub const AUTO_IRDA: u32 = 1 << 17;
    pub const AUTO_ROTATE: u32 = 1 << 18;
    pub const STANDARD_COMMAND: u32 = 1 << 28;
    pub const SET_TIME: u32 = 1 << 29;
    pub const RESET_REQUEST: u32 = 1 << 30;
}

/// A readable summary of the kernel variables (for the debugger).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub com_flags: u32,
    pub dir_index: u16,
    pub func3: u16,
    pub callbacks: [u32; 4],
    pub card_flag: u8,
}

impl Status {
    pub fn read(hw: &Hardware) -> Self {
        Self {
            com_flags: hw.ram_word(ram::COM_FLAGS),
            dir_index: u16::from_le_bytes([hw.ram[0xD0], hw.ram[0xD1]]),
            func3: u16::from_le_bytes([hw.ram[0xC8], hw.ram[0xC9]]),
            callbacks: [0, 1, 2, 3].map(|i| hw.ram_word(ram::CALLBACKS + i * 4)),
            card_flag: hw.ram[ram::CARD_FLAG as usize],
        }
    }
}

fn to_bcd(v: u32) -> u32 {
    ((v / 10) << 4) | (v % 10)
}

impl Machine {
    pub(crate) fn ram16(&self, address: u32) -> u16 {
        u16::from_le_bytes([
            self.hw.ram[address as usize],
            self.hw.ram[address as usize + 1],
        ])
    }
    pub(crate) fn set_ram16(&mut self, address: u32, value: u16) {
        self.hw.ram[address as usize..address as usize + 2].copy_from_slice(&value.to_le_bytes());
    }
    pub(crate) fn com_flags(&self) -> u32 {
        self.hw.ram_word(ram::COM_FLAGS)
    }
    pub(crate) fn set_com_flags(&mut self, value: u32) {
        self.hw.set_ram_word(ram::COM_FLAGS, value);
    }

    /// Dispatches an emulator trap executed by the ROM.
    pub(crate) fn trap(&mut self, number: u32) {
        match number {
            rom::TRAP_RESET => self.boot(),
            rom::TRAP_UNDEFINED => self.undefined_instruction(),
            rom::TRAP_PREFETCH_ABORT => {
                let at = self.cpu.r[14].wrapping_sub(4);
                self.crash(format!(
                    "Prefetch abort: the program jumped to {at:#010x}, which is not mapped"
                ));
            }
            rom::TRAP_DATA_ABORT => {
                let at = self.cpu.r[14].wrapping_sub(8);
                let address = self.hw.last_abort.unwrap_or(0);
                self.crash(format!(
                    "Data abort at {at:#010x}: the program accessed {address:#010x}, which is not mapped"
                ));
            }
            rom::TRAP_RESERVED => self.crash("The reserved exception vector was taken".into()),
            rom::TRAP_IRQ => self.kernel_irq(),
            rom::TRAP_FIQ => {}
            rom::TRAP_IDLE => {}
            n if (rom::TRAP_SWI_BASE..rom::TRAP_SWI_BASE + rom::SWI_COUNT).contains(&n) => {
                self.context_switched = false;
                let result = self.system_call(n - rom::TRAP_SWI_BASE);
                if !self.context_switched
                    && let Some(r0) = result
                {
                    self.cpu.r[0] = r0;
                }
            }
            _ => self.crash(format!("Unknown kernel trap {number:#x}")),
        }
    }

    fn undefined_instruction(&mut self) {
        // lr points after the instruction; the saved state says which set it was.
        let thumb = self.cpu.spsr().unwrap_or(0) & FLAG_T != 0;
        let at = self.cpu.r[14].wrapping_sub(if thumb { 2 } else { 4 });
        let op = if thumb {
            self.hw.peek16(at) as u32
        } else {
            self.hw.peek32(at)
        };
        if !thumb && op == 0xE600_0010 {
            // TTY output was captured when the instruction executed.
            return;
        }
        self.crash(format!("Undefined instruction {op:#010x} at {at:#010x}"));
    }

    /// Power-on initialisation of the kernel RAM and the hardware.
    fn boot(&mut self) {
        self.hw.ram.fill(0);
        // RAM vectors: ldr pc, [pc, #18h] jumping through the table at 20h.
        let targets = [
            "reset_entry",
            "undefined_entry",
            "swi_entry",
            "prefetch_abort_entry",
            "data_abort_entry",
            "reserved_entry",
            "irq_entry",
            "fiq_entry",
        ];
        for (i, name) in targets.iter().enumerate() {
            self.hw
                .set_ram_word(ram::VECTORS + i as u32 * 4, 0xE59F_F018);
            self.hw
                .set_ram_word(ram::VECTOR_TARGETS + i as u32 * 4, rom::symbol(name));
        }
        self.hw
            .set_ram_word(ram::SWI_TABLE, rom::symbol("swi_table"));
        // Alarm/settings word: upper half holds the kernel font address.
        self.hw.set_ram_word(ram::ALARM, 0);
        self.hw
            .set_ram_word(ram::ALARM + 4, rom::symbol("kernel_font"));
        self.hw.ram[ram::CARD_FLAG as usize] = 0x08;
        self.hw.ram[ram::CENTURY as usize] = 0x19;
        self.hw.ram[ram::OLD_YEAR as usize] = 0x99;
        self.hw.ram[ram::SNAPSHOT_ID as usize..ram::SNAPSHOT_ID as usize + 4]
            .copy_from_slice(&[0, 0, b'S', b'E']);
        self.hw.remap = 1;
        self.hw.iop_control = 0xF;
        self.hw.iop_output = 0x22;
        self.hw.clock_mode = 7;
        self.hw.lcd_cal = self.hw.flash_extra[8] as u32 & 0x3F;
        self.hw.lcd_mode = 0x58;
        self.hw.battery_control = 0;
        self.cpu.set_mode(MODE_SVC);
        self.reset_stacks();
        self.start_launcher();
    }

    /// Unmaps any program and shows the start-up application.
    pub(crate) fn start_launcher(&mut self) {
        self.clear_program_context();
        self.hw.bank_flags = 0;
        self.hw.bank_values = [0xF; 16];
        self.hw.remap = 1;
        self.set_ram16(ram::DIR_INDEX, 0);
        self.set_ram16(ram::NEW_DIR_INDEX, 0);
        self.hw.set_ram_word(ram::NEW_PARAM, 0);
        self.hw.irq_mask = IRQ_DOCK | IRQ_RTC | IRQ_FIRE;
        self.hw.clock_mode = 7;
        self.hw.lcd_mode = 0x58;
        self.cpu.set_mode(MODE_SVC);
        self.reset_stacks();
        self.cpu.cpsr = MODE_SVC | crate::cpu::FLAG_I | crate::cpu::FLAG_F;
        self.cpu.r[15] = rom::symbol("idle");
        self.state = State::Launcher;
        self.launcher.reset();
        self.sync_launcher_docking();
        self.context_switched = true;
    }

    fn clear_program_context(&mut self) {
        for i in 0..4 {
            self.hw.set_ram_word(ram::CALLBACKS + i * 4, 0);
        }
        self.set_ram16(ram::FUNC3, 0);
        let flags = self.com_flags() & !(com::ENABLED | com::START_REQUEST);
        self.set_com_flags(flags);
        self.hw.dac_control = 0;
        self.hw.iop_output |= 0x20;
        self.hw.timers = Default::default();
        self.hw.irq_latch = 0;
    }

    /// Validates `block` as the first block of an executable; returns its header info.
    pub fn executable(&self, block: u8) -> Option<file::Info> {
        let files = card::files(&self.hw.flash);
        let f = files.iter().find(|f| f.block == block)?;
        let at = block as usize * card::BLOCK_SIZE;
        let info = file::parse(&self.hw.flash[at..at + card::BLOCK_SIZE])?;
        let _ = f;
        Some(info)
    }

    /// Maps a file into the virtual flash window and starts it in user mode.
    pub(crate) fn start(&mut self, block: u8, param: u32) -> Result<(), String> {
        if block == 0 {
            self.start_launcher();
            return Ok(());
        }
        let files = card::files(&self.hw.flash);
        let file_entry = files
            .iter()
            .find(|f| f.block == block)
            .ok_or_else(|| format!("no file starts at block {block}"))?;
        let at = block as usize * card::BLOCK_SIZE;
        let info = file::parse(&self.hw.flash[at..at + card::BLOCK_SIZE])
            .ok_or_else(|| format!("{} is not a PocketStation program", file_entry.name))?;
        self.clear_program_context();
        self.hw.bank_flags = 0;
        self.hw.bank_values = [0xF; 16];
        for (v, p) in file_entry.blocks.iter().enumerate() {
            self.hw.bank_flags |= 1 << p;
            self.hw.bank_values[*p as usize] = v as u32;
        }
        self.hw.remap = 3;
        self.set_ram16(ram::DIR_INDEX, block as u16);
        self.set_ram16(ram::NEW_DIR_INDEX, block as u16);
        self.hw.set_ram_word(ram::NEW_PARAM, param);
        self.hw.ram[ram::USER_RAM as usize..].fill(0);
        self.hw.irq_mask = IRQ_DOCK;
        self.hw.clock_mode = 7;
        self.hw.lcd_mode = 0x58 | ((self.hw.docked as u32) << 7);
        self.cpu.set_mode(MODE_SVC);
        self.reset_stacks();
        self.cpu.set_cpsr(MODE_USR);
        for r in 0..13 {
            self.cpu.r[r] = 0;
        }
        self.cpu.r[0] = param;
        self.cpu.r[14] = 0;
        self.cpu.r[13] = ram::USER_STACK_TOP;
        self.cpu.branch_exchange(info.entry);
        self.state = State::Program(block);
        self.context_switched = true;
        Ok(())
    }

    /// Kernel work after the program's IRQ callback: docking and the century.
    fn kernel_irq(&mut self) {
        if self.hw.irq_latch & IRQ_DOCK != 0 {
            self.hw.irq_latch &= !IRQ_DOCK;
            let mut flags = self.com_flags() | com::DOCK_EVENT;
            flags &= !com::PROTECT_BROKEN;
            if flags & com::AUTO_ROTATE != 0 {
                self.hw.lcd_mode = (self.hw.lcd_mode & !0x80) | ((self.hw.docked as u32) << 7);
            }
            if flags & com::AUTO_AUDIO != 0 && self.hw.docked {
                self.hw.dac_control = 0;
            }
            if !self.hw.docked {
                flags &= !com::ENABLED;
            }
            self.set_com_flags(flags);
        }
        let year = (self.hw.rtc.date_register() >> 16) & 0xFF;
        let old = self.hw.ram[ram::OLD_YEAR as usize] as u32;
        if year == 0 && old == 0x99 {
            let century = crate::hw::from_bcd(self.hw.ram[ram::CENTURY as usize] as u32) as u32 + 1;
            self.hw.ram[ram::CENTURY as usize] = to_bcd(century % 100) as u8;
        }
        self.hw.ram[ram::OLD_YEAR as usize] = year as u8;
    }

    fn set_com(&mut self, on: bool) {
        let mut flags = self.com_flags();
        if on && self.hw.docked {
            flags |= com::ENABLED;
            self.hw.clock_mode = 7;
        } else {
            flags &= !com::ENABLED;
        }
        self.set_com_flags(flags);
        if on {
            self.hw.irq_mask |= crate::hw::IRQ_COM;
        } else {
            self.hw.irq_mask &= !crate::hw::IRQ_COM;
        }
    }

    pub(crate) fn translate_virtual_sector(&self, sector: u32) -> Option<u32> {
        let physical = self.hw.virtual_to_physical(0x0200_0000 + sector * 0x80)?;
        Some(physical / 0x80)
    }

    /// Writes 128 bytes from `source` to a physical flash sector.
    pub(crate) fn flash_write_sector(&mut self, sector: u32, source: u32) -> u32 {
        if sector >= 0x400 {
            return 1;
        }
        let mut data = [0u8; 128];
        for (i, b) in data.iter_mut().enumerate() {
            *b = self.read_any8(source + i as u32);
        }
        let at = sector as usize * 0x80;
        self.hw.flash[at..at + 0x80].copy_from_slice(&data);
        // Writing takes about 4000 cycles on the console.
        self.hw.advance(4000);
        0
    }

    /// Reads a byte as the kernel does, without garbage-byte effects.
    pub(crate) fn read_any8(&self, address: u32) -> u8 {
        self.hw.peek8(address)
    }

    fn system_call(&mut self, n: u32) -> Option<u32> {
        let r0 = self.cpu.r[0];
        let r1 = self.cpu.r[1];
        let r2 = self.cpu.r[2];
        Some(match n {
            0x00 => {
                self.power_on();
                self.context_switched = true;
                return None;
            }
            0x01 => {
                if r0 > 3 {
                    return Some(0);
                }
                let slot = ram::CALLBACKS + r0 * 4;
                let old = self.hw.ram_word(slot);
                self.hw.set_ram_word(slot, r1);
                old
            }
            0x03 => match self.translate_virtual_sector(r0) {
                Some(physical) => self.flash_write_sector(physical, r1),
                None => 1,
            },
            0x04 => {
                let old = self.hw.clock_mode & 0xF;
                let wait = if r0 & 8 != 0 { 0x10 } else { 0 };
                self.hw.flash_wait1 = wait;
                self.hw.flash_wait2 = wait;
                self.hw.clock_mode = r0 & 0xF;
                old
            }
            0x05 => {
                let flags = self.com_flags();
                if flags & com::DOCK_EVENT != 0 {
                    self.set_com_flags(flags & !com::DOCK_EVENT);
                    if self.hw.docked {
                        self.set_ram16(ram::FUNC3, 0);
                        self.hw.ram[ram::BU50_VALUE as usize] = 0;
                    }
                    self.set_com(true);
                }
                0
            }
            0x06 => ram::COM_FLAGS,
            0x07 => {
                let mask = com::AUTO_AUDIO | com::AUTO_IRDA | com::AUTO_ROTATE;
                let flags = (self.com_flags() & !mask) | (r0 & mask);
                self.set_com_flags(flags);
                r0 & mask
            }
            0x08 => {
                let old = self.ram16(ram::NEW_DIR_INDEX) as u32;
                if r0 == 0 {
                    return Some(self.ram16(ram::DIR_INDEX) as u32);
                }
                let valid = r1 == 0 || (r1 < 16 && self.executable(r1 as u8).is_some());
                if valid {
                    self.set_ram16(ram::NEW_DIR_INDEX, r1 as u16);
                    self.hw.set_ram_word(ram::NEW_PARAM, r2);
                    r1
                } else {
                    old
                }
            }
            0x09 => {
                if self.com_flags() & com::ENABLED != 0 {
                    return Some(r0);
                }
                let target = self.ram16(ram::NEW_DIR_INDEX) as u8;
                let param = self.hw.ram_word(ram::NEW_PARAM);
                if let Err(message) = self.start(target, param) {
                    self.crash(message);
                }
                self.context_switched = true;
                return None;
            }
            0x0A => self.hw.serial(),
            0x0B => {
                let flags = self.com_flags() & !com::PROTECT_BROKEN;
                self.set_com_flags(flags);
                flags
            }
            0x0C => {
                self.set_bcd_date_time(r0, r1);
                self.hw.rtc.date_register() >> 16
            }
            0x0D => {
                ((self.hw.ram[ram::CENTURY as usize] as u32) << 24) | self.hw.rtc.date_register()
            }
            0x0E => self.hw.rtc.time_register(),
            0x0F => {
                self.hw.flash_extra[0..4].copy_from_slice(&r0.to_le_bytes());
                0
            }
            0x10 => self.flash_write_sector(r0, r1),
            0x11 => {
                self.set_com(r0 != 0);
                rom::symbol("swi_entry")
            }
            0x12 => {
                if r0 == 0 || r0 > 15 {
                    return Some(0);
                }
                match self.executable(r0 as u8) {
                    Some(info) if info.snapshot => {
                        let layout_at =
                            r0 as usize * card::BLOCK_SIZE + 0x80 * (1 + info.color_frames) + 0x40;
                        (self.hw.flash.get(layout_at..layout_at + 4)
                            == Some(&[1, 0, b'S', b'E'][..])) as u32
                    }
                    _ => 0,
                }
            }
            0x13 => ram::ALARM,
            0x14 => ram::SWI_TABLE,
            0x15 => {
                let current = self.ram16(ram::DIR_INDEX) as u32;
                let alternate = self.hw.ram[ram::ALTERNATE_DIR as usize] as u32;
                if r0 == 1 && (r1 == 0 || r1 == current) {
                    self.hw.ram[ram::ALTERNATE_DIR as usize] = r1 as u8;
                    r1
                } else {
                    alternate
                }
            }
            0x16 => {
                let current = self.ram16(ram::DIR_INDEX) as u32;
                if current == 0 {
                    self.hw.ram[ram::ALTERNATE_DIR as usize] as u32
                } else {
                    current
                }
            }
            0x17 => ram::FUNC3,
            0x18 => {
                let at = (r0 as usize & 0x3FF) * 0x80 + 0x7E;
                self.hw.flash[at] as u32
            }
            _ => 0,
        })
    }

    /// SWI 0Ch: date = (century, year, month, day) BCD; time = (weekday, hour, minute, second).
    pub(crate) fn set_bcd_date_time(&mut self, date: u32, time: u32) {
        use crate::hw::from_bcd;
        let rtc = &mut self.hw.rtc;
        rtc.day = from_bcd(date & 0xFF).clamp(1, 31);
        rtc.month = from_bcd((date >> 8) & 0xFF).clamp(1, 12);
        rtc.year = from_bcd((date >> 16) & 0xFF) % 100;
        rtc.second = from_bcd(time & 0xFF) % 60;
        rtc.minute = from_bcd((time >> 8) & 0xFF) % 60;
        rtc.hour = from_bcd((time >> 16) & 0xFF) % 24;
        rtc.weekday = ((time >> 24) & 0xFF).clamp(1, 7) as u8;
        let century = (date >> 24) & 0xFF;
        if century != 0 {
            self.hw.ram[ram::CENTURY as usize] = century as u8;
        }
        self.hw.ram[ram::OLD_YEAR as usize] = ((date >> 16) & 0xFF) as u8;
        let alarm = self.hw.ram_word(ram::ALARM) | (1 << 23);
        self.hw.set_ram_word(ram::ALARM, alarm);
    }

    /// True while the kernel answers PlayStation commands.
    pub fn link_enabled(&self) -> bool {
        self.hw.docked
            && match self.state {
                State::Launcher => true,
                State::Program(_) => self.com_flags() & com::ENABLED != 0,
                State::Crashed(_) => false,
            }
    }

    /// The handler mode used for interrupts (for the debugger's mode display).
    pub fn mode_name(&self) -> &'static str {
        match self.cpu.mode() {
            MODE_USR => "User",
            MODE_FIQ => "FIQ",
            MODE_IRQ => "IRQ",
            MODE_SVC => "Supervisor",
            0x17 => "Abort",
            0x1B => "Undefined",
            _ => "System",
        }
    }
}
