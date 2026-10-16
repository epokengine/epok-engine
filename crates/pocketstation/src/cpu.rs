//! ARM7TDMI interpreter (ARMv4T: the ARM and Thumb instruction sets).
//!
//! The core executes one instruction per [`Cpu::step`] and leaves every memory
//! access to a [`Bus`]. It reproduces the ARM7TDMI behaviours that PocketStation
//! programs rely on: rotated unaligned word loads, banked registers per mode,
//! `LDM`/`STM` with the S bit, the empty register list quirk and the exception
//! entry rules for both states. Coprocessor instructions take the undefined
//! instruction exception because the PocketStation has no coprocessor.

pub const MODE_USR: u32 = 0x10;
pub const MODE_FIQ: u32 = 0x11;
pub const MODE_IRQ: u32 = 0x12;
pub const MODE_SVC: u32 = 0x13;
pub const MODE_ABT: u32 = 0x17;
pub const MODE_UND: u32 = 0x1B;
pub const MODE_SYS: u32 = 0x1F;

pub const FLAG_N: u32 = 1 << 31;
pub const FLAG_Z: u32 = 1 << 30;
pub const FLAG_C: u32 = 1 << 29;
pub const FLAG_V: u32 = 1 << 28;
pub const FLAG_I: u32 = 1 << 7;
pub const FLAG_F: u32 = 1 << 6;
pub const FLAG_T: u32 = 1 << 5;

/// Memory as seen by the CPU. Accesses report their cost through
/// [`Bus::take_cycles`]; a data abort is reported by [`Bus::take_abort`].
pub trait Bus {
    fn read32(&mut self, addr: u32) -> u32;
    fn read16(&mut self, addr: u32) -> u16;
    fn read8(&mut self, addr: u32) -> u8;
    fn write32(&mut self, addr: u32, value: u32);
    fn write16(&mut self, addr: u32, value: u16);
    fn write8(&mut self, addr: u32, value: u8);
    fn fetch32(&mut self, addr: u32) -> u32 {
        self.read32(addr)
    }
    fn fetch16(&mut self, addr: u32) -> u16 {
        self.read16(addr)
    }
    /// Returns and clears the data abort raised by the accesses since the last call.
    fn take_abort(&mut self) -> bool {
        false
    }
    /// Returns and clears the cycles spent by the accesses since the last call.
    fn take_cycles(&mut self) -> u32 {
        0
    }
    /// True when an undefined instruction at `addr` is an emulator trap.
    fn is_trap_region(&self, _addr: u32) -> bool {
        false
    }
}

/// What a step did besides executing its instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    None,
    /// An emulator trap (`UDF #n` executed from the trap region).
    Trap(u32),
    /// An undefined instruction exception was taken; the value is the opcode.
    Undefined(u32),
    /// A data or prefetch abort exception was taken at this address.
    Abort(u32),
    /// A software interrupt exception was taken with this comment number.
    Swi(u32),
}

#[derive(Clone, Debug)]
pub struct Cpu {
    /// Visible registers of the current mode. `r[15]` is the address of the
    /// next instruction between steps.
    pub r: [u32; 16],
    pub cpsr: u32,
    usr_high: [u32; 5],
    fiq_high: [u32; 5],
    /// r13/r14 of usr/sys, fiq, irq, svc, abt, und.
    banked: [[u32; 2]; 6],
    /// SPSR of fiq, irq, svc, abt, und (index - 1 of the bank).
    spsr: [u32; 6],
    /// Address of the instruction being executed.
    pub current: u32,
    branched: bool,
    extra_cycles: u32,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

fn bank(mode: u32) -> usize {
    match mode {
        MODE_FIQ => 1,
        MODE_IRQ => 2,
        MODE_SVC => 3,
        MODE_ABT => 4,
        MODE_UND => 5,
        _ => 0,
    }
}

#[inline]
fn add_flags(a: u32, b: u32, carry_in: u32) -> (u32, bool, bool) {
    let wide = a as u64 + b as u64 + carry_in as u64;
    let result = wide as u32;
    let carry = wide > 0xFFFF_FFFF;
    let overflow = ((a ^ result) & (b ^ result)) >> 31 != 0;
    (result, carry, overflow)
}

#[inline]
fn sub_flags(a: u32, b: u32, carry_in: u32) -> (u32, bool, bool) {
    // a - b - !carry == a + !b + carry
    add_flags(a, !b, carry_in)
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self {
            r: [0; 16],
            cpsr: MODE_SVC | FLAG_I | FLAG_F,
            usr_high: [0; 5],
            fiq_high: [0; 5],
            banked: [[0; 2]; 6],
            spsr: [0; 6],
            current: 0,
            branched: false,
            extra_cycles: 0,
        };
        cpu.r[15] = 0;
        cpu
    }

    pub fn mode(&self) -> u32 {
        self.cpsr & 0x1F
    }
    pub fn thumb(&self) -> bool {
        self.cpsr & FLAG_T != 0
    }
    pub fn pc(&self) -> u32 {
        self.r[15]
    }
    pub fn privileged(&self) -> bool {
        self.mode() != MODE_USR
    }

    /// Switches the processor mode, swapping banked registers.
    pub fn set_mode(&mut self, mode: u32) {
        let old = self.mode();
        let (from, to) = (bank(old), bank(mode));
        if from != to {
            if from == 1 {
                self.fiq_high.copy_from_slice(&self.r[8..13]);
            } else {
                self.usr_high.copy_from_slice(&self.r[8..13]);
            }
            self.banked[from] = [self.r[13], self.r[14]];
            if to == 1 {
                self.r[8..13].copy_from_slice(&self.fiq_high);
            } else {
                self.r[8..13].copy_from_slice(&self.usr_high);
            }
            [self.r[13], self.r[14]] = self.banked[to];
        }
        self.cpsr = (self.cpsr & !0x1F) | (mode & 0x1F);
    }

    /// Writes the whole CPSR, switching banks when the mode changes.
    pub fn set_cpsr(&mut self, value: u32) {
        self.set_mode(value & 0x1F);
        self.cpsr = value;
    }

    pub fn spsr(&self) -> Option<u32> {
        let index = bank(self.mode());
        (index != 0).then(|| self.spsr[index])
    }
    pub fn set_spsr(&mut self, value: u32) {
        let index = bank(self.mode());
        if index != 0 {
            self.spsr[index] = value;
        }
    }

    /// Reads a register of another mode (for debuggers and the kernel).
    pub fn banked_register(&self, mode: u32, reg: usize) -> u32 {
        let index = bank(mode);
        if index == bank(self.mode()) {
            return self.r[reg];
        }
        match reg {
            8..=12 if index == 1 => self.fiq_high[reg - 8],
            8..=12 if bank(self.mode()) == 1 => self.usr_high[reg - 8],
            13 | 14 => self.banked[index][reg - 13],
            _ => self.r[reg],
        }
    }
    pub fn set_banked_register(&mut self, mode: u32, reg: usize, value: u32) {
        let index = bank(mode);
        if index == bank(self.mode()) {
            self.r[reg] = value;
            return;
        }
        match reg {
            8..=12 if index == 1 => self.fiq_high[reg - 8] = value,
            8..=12 if bank(self.mode()) == 1 => self.usr_high[reg - 8] = value,
            13 | 14 => self.banked[index][reg - 13] = value,
            _ => self.r[reg] = value,
        }
    }

    fn flag(&self, flag: u32) -> bool {
        self.cpsr & flag != 0
    }
    fn set_flag(&mut self, flag: u32, on: bool) {
        if on {
            self.cpsr |= flag;
        } else {
            self.cpsr &= !flag;
        }
    }
    fn set_nz(&mut self, value: u32) {
        self.set_flag(FLAG_N, value >> 31 != 0);
        self.set_flag(FLAG_Z, value == 0);
    }

    fn condition(&self, cond: u32) -> bool {
        let n = self.flag(FLAG_N);
        let z = self.flag(FLAG_Z);
        let c = self.flag(FLAG_C);
        let v = self.flag(FLAG_V);
        match cond {
            0x0 => z,
            0x1 => !z,
            0x2 => c,
            0x3 => !c,
            0x4 => n,
            0x5 => !n,
            0x6 => v,
            0x7 => !v,
            0x8 => c && !z,
            0x9 => !c || z,
            0xA => n == v,
            0xB => n != v,
            0xC => !z && n == v,
            0xD => z || n != v,
            0xE => true,
            _ => false,
        }
    }

    /// Enters an exception, saving the CPSR into the new mode's SPSR.
    pub fn exception(&mut self, vector: u32, mode: u32, return_address: u32, mask_fiq: bool) {
        let old = self.cpsr;
        self.set_mode(mode);
        self.set_spsr(old);
        self.r[14] = return_address;
        self.cpsr &= !FLAG_T;
        self.cpsr |= FLAG_I;
        if mask_fiq {
            self.cpsr |= FLAG_F;
        }
        self.r[15] = vector;
        self.branched = true;
    }

    /// Takes a pending interrupt if it is enabled. Returns true when taken.
    pub fn interrupt(&mut self, irq: bool, fiq: bool) -> bool {
        if fiq && !self.flag(FLAG_F) {
            let next = self.r[15];
            self.exception(0x1C, MODE_FIQ, next + 4, true);
            return true;
        }
        if irq && !self.flag(FLAG_I) {
            let next = self.r[15];
            self.exception(0x18, MODE_IRQ, next + 4, false);
            return true;
        }
        false
    }

    fn set_pc(&mut self, value: u32) {
        self.r[15] = value;
        self.branched = true;
    }

    /// Jumps to `address`, selecting Thumb state from bit 0.
    pub fn branch_exchange(&mut self, address: u32) {
        if address & 1 != 0 {
            self.cpsr |= FLAG_T;
            self.r[15] = address & !1;
        } else {
            self.cpsr &= !FLAG_T;
            self.r[15] = address & !3;
        }
        self.branched = true;
    }

    /// Executes one instruction and returns the cycles it took with any event.
    pub fn step<B: Bus>(&mut self, bus: &mut B) -> (u32, Event) {
        self.branched = false;
        self.extra_cycles = 0;
        let address = self.r[15];
        self.current = address;
        let event = if self.thumb() {
            let op = bus.fetch16(address);
            if bus.take_abort() {
                self.exception(0x0C, MODE_ABT, address + 4, false);
                Event::Abort(address)
            } else {
                self.r[15] = address.wrapping_add(4);
                let event = self.thumb_execute(bus, op);
                if !self.branched {
                    self.r[15] = address.wrapping_add(2);
                }
                event
            }
        } else {
            let op = bus.fetch32(address);
            if bus.take_abort() {
                self.exception(0x0C, MODE_ABT, address + 4, false);
                Event::Abort(address)
            } else {
                self.r[15] = address.wrapping_add(8);
                let event = if self.condition(op >> 28) {
                    self.arm_execute(bus, op)
                } else {
                    Event::None
                };
                if !self.branched {
                    self.r[15] = address.wrapping_add(4);
                }
                event
            }
        };
        let mut cycles = 1 + self.extra_cycles + bus.take_cycles();
        if self.branched {
            cycles += 2;
        }
        (cycles, event)
    }

    fn data_abort(&mut self) -> Event {
        let at = self.current;
        self.exception(0x10, MODE_ABT, at + 8, false);
        Event::Abort(at)
    }

    fn undefined(&mut self, bus: &mut impl Bus, op: u32) -> Event {
        if bus.is_trap_region(self.current) && op & 0x0FF000F0 == 0x07F000F0 {
            let number = ((op >> 4) & 0xFFF0) | (op & 0xF);
            return Event::Trap(number);
        }
        let next = self.current + if self.thumb() { 2 } else { 4 };
        self.exception(0x04, MODE_UND, next, false);
        Event::Undefined(op)
    }

    // ---------------------------------------------------------------- shifts

    fn shift_immediate(&self, kind: u32, value: u32, amount: u32) -> (u32, bool) {
        let carry = self.flag(FLAG_C);
        match kind {
            0 => {
                if amount == 0 {
                    (value, carry)
                } else {
                    (value << amount, (value >> (32 - amount)) & 1 != 0)
                }
            }
            1 => {
                if amount == 0 {
                    (0, value >> 31 != 0)
                } else {
                    (value >> amount, (value >> (amount - 1)) & 1 != 0)
                }
            }
            2 => {
                if amount == 0 {
                    let fill = ((value as i32) >> 31) as u32;
                    (fill, value >> 31 != 0)
                } else {
                    (
                        ((value as i32) >> amount) as u32,
                        (value >> (amount - 1)) & 1 != 0,
                    )
                }
            }
            _ => {
                if amount == 0 {
                    (((carry as u32) << 31) | (value >> 1), value & 1 != 0)
                } else {
                    (value.rotate_right(amount), (value >> (amount - 1)) & 1 != 0)
                }
            }
        }
    }

    fn shift_register(&self, kind: u32, value: u32, amount: u32) -> (u32, bool) {
        let carry = self.flag(FLAG_C);
        if amount == 0 {
            return (value, carry);
        }
        match kind {
            0 => match amount {
                1..=31 => (value << amount, (value >> (32 - amount)) & 1 != 0),
                32 => (0, value & 1 != 0),
                _ => (0, false),
            },
            1 => match amount {
                1..=31 => (value >> amount, (value >> (amount - 1)) & 1 != 0),
                32 => (0, value >> 31 != 0),
                _ => (0, false),
            },
            2 => {
                if amount >= 32 {
                    (((value as i32) >> 31) as u32, value >> 31 != 0)
                } else {
                    (
                        ((value as i32) >> amount) as u32,
                        (value >> (amount - 1)) & 1 != 0,
                    )
                }
            }
            _ => {
                let rotate = amount & 31;
                if rotate == 0 {
                    (value, value >> 31 != 0)
                } else {
                    (value.rotate_right(rotate), (value >> (rotate - 1)) & 1 != 0)
                }
            }
        }
    }

    // ------------------------------------------------------------- ARM state

    fn arm_execute(&mut self, bus: &mut impl Bus, op: u32) -> Event {
        if op & 0x0FFF_FFF0 == 0x012F_FF10 {
            let target = self.r[(op & 0xF) as usize];
            self.branch_exchange(target);
            return Event::None;
        }
        if op & 0x0FB0_0FF0 == 0x0100_0090 {
            return self.arm_swap(bus, op);
        }
        if op & 0x0FC0_00F0 == 0x0000_0090 {
            self.arm_multiply(op);
            return Event::None;
        }
        if op & 0x0F80_00F0 == 0x0080_0090 {
            self.arm_multiply_long(op);
            return Event::None;
        }
        if op & 0x0E00_0090 == 0x0000_0090 && op & 0x60 != 0 {
            return self.arm_halfword(bus, op);
        }
        if op & 0x0FBF_0FFF == 0x010F_0000 {
            let rd = ((op >> 12) & 0xF) as usize;
            self.r[rd] = if op & (1 << 22) != 0 {
                self.spsr().unwrap_or(self.cpsr)
            } else {
                self.cpsr
            };
            return Event::None;
        }
        if op & 0x0FB0_FFF0 == 0x0120_F000 || op & 0x0FB0_F000 == 0x0320_F000 {
            self.arm_msr(op);
            return Event::None;
        }
        match (op >> 25) & 7 {
            0 | 1 => {
                // TST/TEQ/CMP/CMN without S that are not PSR transfers.
                let opcode = (op >> 21) & 0xF;
                if (8..=11).contains(&opcode) && op & (1 << 20) == 0 {
                    return self.undefined(bus, op);
                }
                self.arm_data_processing(op);
                Event::None
            }
            3 if op & 0x10 != 0 => self.undefined(bus, op),
            2 | 3 => self.arm_single_transfer(bus, op),
            4 => self.arm_block_transfer(bus, op),
            5 => {
                let offset = ((op << 8) as i32 >> 6) as u32;
                if op & (1 << 24) != 0 {
                    self.r[14] = self.current + 4;
                }
                let target = self.r[15].wrapping_add(offset);
                self.set_pc(target);
                Event::None
            }
            7 if op & (1 << 24) != 0 => {
                let next = self.current + 4;
                self.exception(0x08, MODE_SVC, next, false);
                Event::Swi(op & 0xFF)
            }
            _ => self.undefined(bus, op),
        }
    }

    fn arm_operand2(&mut self, op: u32) -> (u32, bool) {
        if op & (1 << 25) != 0 {
            let imm = op & 0xFF;
            let rotate = ((op >> 8) & 0xF) * 2;
            let value = imm.rotate_right(rotate);
            let carry = if rotate == 0 {
                self.flag(FLAG_C)
            } else {
                value >> 31 != 0
            };
            return (value, carry);
        }
        let rm = (op & 0xF) as usize;
        let kind = (op >> 5) & 3;
        if op & 0x10 != 0 {
            self.extra_cycles += 1;
            let rs = ((op >> 8) & 0xF) as usize;
            let amount = self.r[rs] & 0xFF;
            let value = if rm == 15 {
                self.r[15].wrapping_add(4)
            } else {
                self.r[rm]
            };
            self.shift_register(kind, value, amount)
        } else {
            let amount = (op >> 7) & 0x1F;
            self.shift_immediate(kind, self.r[rm], amount)
        }
    }

    fn arm_data_processing(&mut self, op: u32) {
        let opcode = (op >> 21) & 0xF;
        let set = op & (1 << 20) != 0;
        let rn = ((op >> 16) & 0xF) as usize;
        let rd = ((op >> 12) & 0xF) as usize;
        let (operand, shifter_carry) = self.arm_operand2(op);
        let mut first = self.r[rn];
        if rn == 15 && op & (1 << 25) == 0 && op & 0x10 != 0 {
            first = first.wrapping_add(4);
        }
        let carry_in = self.flag(FLAG_C) as u32;
        let (result, carry, overflow, write) = match opcode {
            0x0 => (first & operand, shifter_carry, None, true),
            0x1 => (first ^ operand, shifter_carry, None, true),
            0x2 => {
                let (r, c, v) = sub_flags(first, operand, 1);
                (r, c, Some(v), true)
            }
            0x3 => {
                let (r, c, v) = sub_flags(operand, first, 1);
                (r, c, Some(v), true)
            }
            0x4 => {
                let (r, c, v) = add_flags(first, operand, 0);
                (r, c, Some(v), true)
            }
            0x5 => {
                let (r, c, v) = add_flags(first, operand, carry_in);
                (r, c, Some(v), true)
            }
            0x6 => {
                let (r, c, v) = sub_flags(first, operand, carry_in);
                (r, c, Some(v), true)
            }
            0x7 => {
                let (r, c, v) = sub_flags(operand, first, carry_in);
                (r, c, Some(v), true)
            }
            0x8 => (first & operand, shifter_carry, None, false),
            0x9 => (first ^ operand, shifter_carry, None, false),
            0xA => {
                let (r, c, v) = sub_flags(first, operand, 1);
                (r, c, Some(v), false)
            }
            0xB => {
                let (r, c, v) = add_flags(first, operand, 0);
                (r, c, Some(v), false)
            }
            0xC => (first | operand, shifter_carry, None, true),
            0xD => (operand, shifter_carry, None, true),
            0xE => (first & !operand, shifter_carry, None, true),
            _ => (!operand, shifter_carry, None, true),
        };
        if set {
            if rd == 15 && write {
                if let Some(spsr) = self.spsr() {
                    self.set_cpsr(spsr);
                }
            } else {
                self.set_nz(result);
                self.set_flag(FLAG_C, carry);
                if let Some(v) = overflow {
                    self.set_flag(FLAG_V, v);
                }
            }
        }
        if write {
            if rd == 15 {
                let target = if self.thumb() {
                    result & !1
                } else {
                    result & !3
                };
                self.set_pc(target);
            } else {
                self.r[rd] = result;
            }
        }
    }

    fn arm_msr(&mut self, op: u32) {
        let value = if op & (1 << 25) != 0 {
            (op & 0xFF).rotate_right(((op >> 8) & 0xF) * 2)
        } else {
            self.r[(op & 0xF) as usize]
        };
        let mut mask = 0u32;
        if op & (1 << 19) != 0 {
            mask |= 0xFF00_0000;
        }
        if op & (1 << 16) != 0 && self.privileged() {
            mask |= 0x0000_00FF;
        }
        if op & (1 << 22) != 0 {
            if let Some(old) = self.spsr() {
                self.set_spsr((old & !mask) | (value & mask));
            }
        } else {
            let new = (self.cpsr & !mask) | (value & mask);
            self.set_cpsr(new);
        }
    }

    fn arm_multiply(&mut self, op: u32) {
        let rd = ((op >> 16) & 0xF) as usize;
        let rn = ((op >> 12) & 0xF) as usize;
        let rs = ((op >> 8) & 0xF) as usize;
        let rm = (op & 0xF) as usize;
        let mut result = self.r[rm].wrapping_mul(self.r[rs]);
        if op & (1 << 21) != 0 {
            result = result.wrapping_add(self.r[rn]);
            self.extra_cycles += 1;
        }
        self.extra_cycles += multiply_cycles(self.r[rs]);
        self.r[rd] = result;
        if op & (1 << 20) != 0 {
            self.set_nz(result);
        }
    }

    fn arm_multiply_long(&mut self, op: u32) {
        let hi = ((op >> 16) & 0xF) as usize;
        let lo = ((op >> 12) & 0xF) as usize;
        let rs = ((op >> 8) & 0xF) as usize;
        let rm = (op & 0xF) as usize;
        let signed = op & (1 << 22) != 0;
        let mut result = if signed {
            (self.r[rm] as i32 as i64).wrapping_mul(self.r[rs] as i32 as i64) as u64
        } else {
            (self.r[rm] as u64).wrapping_mul(self.r[rs] as u64)
        };
        if op & (1 << 21) != 0 {
            result = result.wrapping_add(((self.r[hi] as u64) << 32) | self.r[lo] as u64);
            self.extra_cycles += 1;
        }
        self.extra_cycles += 1 + multiply_cycles(self.r[rs]);
        self.r[lo] = result as u32;
        self.r[hi] = (result >> 32) as u32;
        if op & (1 << 20) != 0 {
            self.set_flag(FLAG_N, result >> 63 != 0);
            self.set_flag(FLAG_Z, result == 0);
        }
    }

    fn arm_swap(&mut self, bus: &mut impl Bus, op: u32) -> Event {
        let rn = ((op >> 16) & 0xF) as usize;
        let rd = ((op >> 12) & 0xF) as usize;
        let rm = (op & 0xF) as usize;
        let address = self.r[rn];
        let source = self.r[rm];
        self.extra_cycles += 1;
        if op & (1 << 22) != 0 {
            let old = bus.read8(address);
            bus.write8(address, source as u8);
            if bus.take_abort() {
                return self.data_abort();
            }
            self.r[rd] = old as u32;
        } else {
            let old = bus.read32(address & !3).rotate_right((address & 3) * 8);
            bus.write32(address & !3, source);
            if bus.take_abort() {
                return self.data_abort();
            }
            self.r[rd] = old;
        }
        Event::None
    }

    fn load_word(&mut self, bus: &mut impl Bus, address: u32) -> u32 {
        bus.read32(address & !3).rotate_right((address & 3) * 8)
    }

    fn arm_single_transfer(&mut self, bus: &mut impl Bus, op: u32) -> Event {
        let pre = op & (1 << 24) != 0;
        let up = op & (1 << 23) != 0;
        let byte = op & (1 << 22) != 0;
        let writeback = op & (1 << 21) != 0;
        let load = op & (1 << 20) != 0;
        let rn = ((op >> 16) & 0xF) as usize;
        let rd = ((op >> 12) & 0xF) as usize;
        let offset = if op & (1 << 25) != 0 {
            let rm = (op & 0xF) as usize;
            let kind = (op >> 5) & 3;
            let amount = (op >> 7) & 0x1F;
            self.shift_immediate(kind, self.r[rm], amount).0
        } else {
            op & 0xFFF
        };
        let base = self.r[rn];
        let moved = if up {
            base.wrapping_add(offset)
        } else {
            base.wrapping_sub(offset)
        };
        let address = if pre { moved } else { base };
        if load {
            self.extra_cycles += 1;
            let value = if byte {
                bus.read8(address) as u32
            } else {
                self.load_word(bus, address)
            };
            if bus.take_abort() {
                return self.data_abort();
            }
            if (!pre || writeback) && rn != 15 {
                self.r[rn] = moved;
            }
            if rd == 15 {
                self.set_pc(value & !3);
            } else {
                self.r[rd] = value;
            }
        } else {
            let value = if rd == 15 {
                self.r[15].wrapping_add(4)
            } else {
                self.r[rd]
            };
            if byte {
                bus.write8(address, value as u8);
            } else {
                bus.write32(address & !3, value);
            }
            if bus.take_abort() {
                return self.data_abort();
            }
            if (!pre || writeback) && rn != 15 {
                self.r[rn] = moved;
            }
        }
        Event::None
    }

    fn arm_halfword(&mut self, bus: &mut impl Bus, op: u32) -> Event {
        let pre = op & (1 << 24) != 0;
        let up = op & (1 << 23) != 0;
        let immediate = op & (1 << 22) != 0;
        let writeback = op & (1 << 21) != 0;
        let load = op & (1 << 20) != 0;
        let rn = ((op >> 16) & 0xF) as usize;
        let rd = ((op >> 12) & 0xF) as usize;
        let kind = (op >> 5) & 3;
        let offset = if immediate {
            ((op >> 4) & 0xF0) | (op & 0xF)
        } else {
            self.r[(op & 0xF) as usize]
        };
        let base = self.r[rn];
        let moved = if up {
            base.wrapping_add(offset)
        } else {
            base.wrapping_sub(offset)
        };
        let address = if pre { moved } else { base };
        if load {
            self.extra_cycles += 1;
            let value = match kind {
                1 => (bus.read16(address & !1) as u32).rotate_right((address & 1) * 8),
                2 => bus.read8(address) as i8 as i32 as u32,
                _ => {
                    if address & 1 != 0 {
                        bus.read8(address) as i8 as i32 as u32
                    } else {
                        bus.read16(address) as i16 as i32 as u32
                    }
                }
            };
            if bus.take_abort() {
                return self.data_abort();
            }
            if (!pre || writeback) && rn != 15 {
                self.r[rn] = moved;
            }
            if rd == 15 {
                self.set_pc(value & !3);
            } else {
                self.r[rd] = value;
            }
        } else {
            if kind != 1 {
                return self.undefined(bus, op);
            }
            let value = if rd == 15 {
                self.r[15].wrapping_add(4)
            } else {
                self.r[rd]
            };
            bus.write16(address & !1, value as u16);
            if bus.take_abort() {
                return self.data_abort();
            }
            if (!pre || writeback) && rn != 15 {
                self.r[rn] = moved;
            }
        }
        Event::None
    }

    fn arm_block_transfer(&mut self, bus: &mut impl Bus, op: u32) -> Event {
        let pre = op & (1 << 24) != 0;
        let up = op & (1 << 23) != 0;
        let user = op & (1 << 22) != 0;
        let writeback = op & (1 << 21) != 0;
        let load = op & (1 << 20) != 0;
        let rn = ((op >> 16) & 0xF) as usize;
        let list = op & 0xFFFF;
        self.block_transfer(bus, rn, list, pre, up, user, writeback, load)
    }

    #[allow(clippy::too_many_arguments)]
    fn block_transfer(
        &mut self,
        bus: &mut impl Bus,
        rn: usize,
        list: u32,
        pre: bool,
        up: bool,
        user: bool,
        writeback: bool,
        load: bool,
    ) -> Event {
        let (list, size) = if list == 0 {
            (1u32 << 15, 0x40u32)
        } else {
            (list, list.count_ones() * 4)
        };
        let base = self.r[rn];
        let (start, end) = if up {
            (base, base.wrapping_add(size))
        } else {
            (base.wrapping_sub(size), base.wrapping_sub(size))
        };
        let mut address = match (up, pre) {
            (true, false) => start,
            (true, true) => start.wrapping_add(4),
            (false, false) => start.wrapping_add(4),
            (false, true) => start,
        };
        let final_base = if up { end } else { start };
        // A user-bank transfer: STM^ always, LDM^ when r15 is not loaded.
        let user_bank = user && !(load && list & (1 << 15) != 0);
        let saved_mode = self.mode();
        if user_bank {
            self.set_mode(MODE_USR);
        }
        let first = list.trailing_zeros() as usize;
        let mut aborted = false;
        let mut loaded_pc = None;
        for reg in 0..16usize {
            if list & (1 << reg) == 0 {
                continue;
            }
            if load {
                let value = bus.read32(address & !3);
                if bus.take_abort() {
                    aborted = true;
                }
                if !aborted {
                    if reg == 15 {
                        loaded_pc = Some(value);
                    } else {
                        self.r[reg] = value;
                    }
                }
            } else {
                let value = if reg == 15 {
                    self.r[15].wrapping_add(4)
                } else if reg == rn && writeback && reg != first {
                    final_base
                } else {
                    self.r[reg]
                };
                bus.write32(address & !3, value);
                if bus.take_abort() {
                    aborted = true;
                }
            }
            address = address.wrapping_add(4);
            self.extra_cycles += 1;
        }
        if user_bank {
            self.set_mode(saved_mode);
        }
        if aborted {
            return self.data_abort();
        }
        if writeback && !(load && list & (1 << rn) != 0) {
            self.r[rn] = final_base;
        }
        if let Some(value) = loaded_pc {
            if user && let Some(spsr) = self.spsr() {
                self.set_cpsr(spsr);
            }
            let target = if self.thumb() { value & !1 } else { value & !3 };
            self.set_pc(target);
        }
        Event::None
    }

    // ----------------------------------------------------------- Thumb state

    fn thumb_execute(&mut self, bus: &mut impl Bus, op: u16) -> Event {
        let op = op as u32;
        match op >> 13 {
            0 => {
                let rd = (op & 7) as usize;
                let rs = ((op >> 3) & 7) as usize;
                if (op >> 11) & 3 == 3 {
                    let operand = if op & (1 << 10) != 0 {
                        (op >> 6) & 7
                    } else {
                        self.r[((op >> 6) & 7) as usize]
                    };
                    let (result, carry, overflow) = if op & (1 << 9) != 0 {
                        sub_flags(self.r[rs], operand, 1)
                    } else {
                        add_flags(self.r[rs], operand, 0)
                    };
                    self.r[rd] = result;
                    self.set_nz(result);
                    self.set_flag(FLAG_C, carry);
                    self.set_flag(FLAG_V, overflow);
                } else {
                    let amount = (op >> 6) & 0x1F;
                    let (result, carry) = self.shift_immediate((op >> 11) & 3, self.r[rs], amount);
                    self.r[rd] = result;
                    self.set_nz(result);
                    self.set_flag(FLAG_C, carry);
                }
                Event::None
            }
            1 => {
                let rd = ((op >> 8) & 7) as usize;
                let imm = op & 0xFF;
                match (op >> 11) & 3 {
                    0 => {
                        self.r[rd] = imm;
                        self.set_nz(imm);
                    }
                    1 => {
                        let (r, c, v) = sub_flags(self.r[rd], imm, 1);
                        self.set_nz(r);
                        self.set_flag(FLAG_C, c);
                        self.set_flag(FLAG_V, v);
                    }
                    2 => {
                        let (r, c, v) = add_flags(self.r[rd], imm, 0);
                        self.r[rd] = r;
                        self.set_nz(r);
                        self.set_flag(FLAG_C, c);
                        self.set_flag(FLAG_V, v);
                    }
                    _ => {
                        let (r, c, v) = sub_flags(self.r[rd], imm, 1);
                        self.r[rd] = r;
                        self.set_nz(r);
                        self.set_flag(FLAG_C, c);
                        self.set_flag(FLAG_V, v);
                    }
                }
                Event::None
            }
            2 => {
                if op >> 10 == 0b010000 {
                    self.thumb_alu(op);
                    Event::None
                } else if op >> 10 == 0b010001 {
                    self.thumb_hi_register(op);
                    Event::None
                } else if op >> 11 == 0b01001 {
                    let rd = ((op >> 8) & 7) as usize;
                    let address = (self.r[15] & !3).wrapping_add((op & 0xFF) * 4);
                    self.extra_cycles += 1;
                    let value = bus.read32(address);
                    if bus.take_abort() {
                        return self.data_abort();
                    }
                    self.r[rd] = value;
                    Event::None
                } else {
                    let ro = ((op >> 6) & 7) as usize;
                    let rb = ((op >> 3) & 7) as usize;
                    let rd = (op & 7) as usize;
                    let address = self.r[rb].wrapping_add(self.r[ro]);
                    let code = (op >> 9) & 7;
                    self.thumb_transfer(bus, code, rd, address)
                }
            }
            3 => {
                let rb = ((op >> 3) & 7) as usize;
                let rd = (op & 7) as usize;
                let imm = (op >> 6) & 0x1F;
                let byte = op & (1 << 12) != 0;
                let load = op & (1 << 11) != 0;
                let address = self.r[rb].wrapping_add(if byte { imm } else { imm * 4 });
                // Codes as in the register-offset form: STR, STRB, LDR, LDRB.
                let code = match (load, byte) {
                    (false, false) => 0,
                    (false, true) => 2,
                    (true, false) => 4,
                    (true, true) => 6,
                };
                self.thumb_transfer(bus, code, rd, address)
            }
            4 => {
                if op & (1 << 12) == 0 {
                    let rb = ((op >> 3) & 7) as usize;
                    let rd = (op & 7) as usize;
                    let address = self.r[rb].wrapping_add(((op >> 6) & 0x1F) * 2);
                    let code = if op & (1 << 11) != 0 { 5 } else { 1 };
                    self.thumb_transfer(bus, code, rd, address)
                } else {
                    let rd = ((op >> 8) & 7) as usize;
                    let address = self.r[13].wrapping_add((op & 0xFF) * 4);
                    let code = if op & (1 << 11) != 0 { 4 } else { 0 };
                    self.thumb_transfer(bus, code, rd, address)
                }
            }
            5 => {
                if op & (1 << 12) == 0 {
                    let rd = ((op >> 8) & 7) as usize;
                    let base = if op & (1 << 11) != 0 {
                        self.r[13]
                    } else {
                        self.r[15] & !3
                    };
                    self.r[rd] = base.wrapping_add((op & 0xFF) * 4);
                    Event::None
                } else if (op >> 8) & 0xF == 0 {
                    let offset = (op & 0x7F) * 4;
                    self.r[13] = if op & 0x80 != 0 {
                        self.r[13].wrapping_sub(offset)
                    } else {
                        self.r[13].wrapping_add(offset)
                    };
                    Event::None
                } else if op & 0x0600 == 0x0400 {
                    let load = op & (1 << 11) != 0;
                    let extra = op & (1 << 8) != 0;
                    let mut list = op & 0xFF;
                    if extra {
                        list |= if load { 1 << 15 } else { 1 << 14 };
                    }
                    if load {
                        self.block_transfer(bus, 13, list, false, true, false, true, true)
                    } else {
                        self.block_transfer(bus, 13, list, true, false, false, true, false)
                    }
                } else {
                    self.undefined(bus, op)
                }
            }
            6 => {
                if op & (1 << 12) == 0 {
                    let rb = ((op >> 8) & 7) as usize;
                    let load = op & (1 << 11) != 0;
                    self.block_transfer(bus, rb, op & 0xFF, false, true, false, true, load)
                } else {
                    let cond = (op >> 8) & 0xF;
                    match cond {
                        0xF => {
                            let next = self.current + 2;
                            self.exception(0x08, MODE_SVC, next, false);
                            Event::Swi(op & 0xFF)
                        }
                        0xE => self.undefined(bus, op),
                        _ => {
                            if self.condition(cond) {
                                let offset = ((op & 0xFF) as i8 as i32 * 2) as u32;
                                let target = self.r[15].wrapping_add(offset);
                                self.set_pc(target);
                            }
                            Event::None
                        }
                    }
                }
            }
            _ => match (op >> 11) & 3 {
                0 => {
                    let offset = ((((op & 0x7FF) << 21) as i32) >> 20) as u32;
                    let target = self.r[15].wrapping_add(offset);
                    self.set_pc(target);
                    Event::None
                }
                2 => {
                    let offset = ((((op & 0x7FF) << 21) as i32) >> 9) as u32;
                    self.r[14] = self.r[15].wrapping_add(offset);
                    Event::None
                }
                3 => {
                    let next = self.current + 2;
                    let target = self.r[14].wrapping_add((op & 0x7FF) << 1);
                    self.r[14] = next | 1;
                    self.set_pc(target & !1);
                    Event::None
                }
                _ => self.undefined(bus, op),
            },
        }
    }

    /// Thumb loads/stores. Codes: 0 STR, 1 STRH, 2 STRB, 3 LDSB, 4 LDR, 5 LDRH, 6 LDRB, 7 LDSH.
    fn thumb_transfer(&mut self, bus: &mut impl Bus, code: u32, rd: usize, address: u32) -> Event {
        match code {
            0 => bus.write32(address & !3, self.r[rd]),
            1 => bus.write16(address & !1, self.r[rd] as u16),
            2 => bus.write8(address, self.r[rd] as u8),
            _ => {
                self.extra_cycles += 1;
                let value = match code {
                    3 => bus.read8(address) as i8 as i32 as u32,
                    4 => self.load_word(bus, address),
                    5 => (bus.read16(address & !1) as u32).rotate_right((address & 1) * 8),
                    6 => bus.read8(address) as u32,
                    _ => {
                        if address & 1 != 0 {
                            bus.read8(address) as i8 as i32 as u32
                        } else {
                            bus.read16(address) as i16 as i32 as u32
                        }
                    }
                };
                if bus.take_abort() {
                    return self.data_abort();
                }
                self.r[rd] = value;
                return Event::None;
            }
        }
        if bus.take_abort() {
            return self.data_abort();
        }
        Event::None
    }

    fn thumb_alu(&mut self, op: u32) {
        let rd = (op & 7) as usize;
        let rs = ((op >> 3) & 7) as usize;
        let a = self.r[rd];
        let b = self.r[rs];
        let carry_in = self.flag(FLAG_C) as u32;
        match (op >> 6) & 0xF {
            0x0 => {
                self.r[rd] = a & b;
                self.set_nz(a & b);
            }
            0x1 => {
                self.r[rd] = a ^ b;
                self.set_nz(a ^ b);
            }
            0x2 | 0x3 | 0x4 | 0x7 => {
                let kind = match (op >> 6) & 0xF {
                    0x2 => 0,
                    0x3 => 1,
                    0x4 => 2,
                    _ => 3,
                };
                self.extra_cycles += 1;
                let (r, c) = self.shift_register(kind, a, b & 0xFF);
                self.r[rd] = r;
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
            }
            0x5 => {
                let (r, c, v) = add_flags(a, b, carry_in);
                self.r[rd] = r;
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
                self.set_flag(FLAG_V, v);
            }
            0x6 => {
                let (r, c, v) = sub_flags(a, b, carry_in);
                self.r[rd] = r;
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
                self.set_flag(FLAG_V, v);
            }
            0x8 => self.set_nz(a & b),
            0x9 => {
                let (r, c, v) = sub_flags(0, b, 1);
                self.r[rd] = r;
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
                self.set_flag(FLAG_V, v);
            }
            0xA => {
                let (r, c, v) = sub_flags(a, b, 1);
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
                self.set_flag(FLAG_V, v);
            }
            0xB => {
                let (r, c, v) = add_flags(a, b, 0);
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
                self.set_flag(FLAG_V, v);
            }
            0xC => {
                self.r[rd] = a | b;
                self.set_nz(a | b);
            }
            0xD => {
                let r = a.wrapping_mul(b);
                self.extra_cycles += multiply_cycles(a);
                self.r[rd] = r;
                self.set_nz(r);
            }
            0xE => {
                self.r[rd] = a & !b;
                self.set_nz(a & !b);
            }
            _ => {
                self.r[rd] = !b;
                self.set_nz(!b);
            }
        }
    }

    fn thumb_hi_register(&mut self, op: u32) {
        let rd = ((op & 7) | ((op >> 4) & 8)) as usize;
        let rs = ((op >> 3) & 0xF) as usize;
        let value = self.r[rs];
        match (op >> 8) & 3 {
            0 => {
                let result = self.r[rd].wrapping_add(value);
                if rd == 15 {
                    self.set_pc(result & !1);
                } else {
                    self.r[rd] = result;
                }
            }
            1 => {
                let (r, c, v) = sub_flags(self.r[rd], value, 1);
                self.set_nz(r);
                self.set_flag(FLAG_C, c);
                self.set_flag(FLAG_V, v);
            }
            2 => {
                if rd == 15 {
                    self.set_pc(value & !1);
                } else {
                    self.r[rd] = value;
                }
            }
            _ => self.branch_exchange(value),
        }
    }
}

fn multiply_cycles(multiplier: u32) -> u32 {
    let m = if multiplier >> 31 != 0 {
        !multiplier
    } else {
        multiplier
    };
    if m >> 8 == 0 {
        1
    } else if m >> 16 == 0 {
        2
    } else if m >> 24 == 0 {
        3
    } else {
        4
    }
}
