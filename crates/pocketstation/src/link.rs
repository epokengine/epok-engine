//! The PlayStation side of the memory card connector: the kernel's answers
//! to memory card commands (read, write, identify) and to the PocketStation
//! commands 50h and 58h-5Fh, one byte at a time.

use crate::card;
use crate::cpu::Bus;
use crate::file;
use crate::kernel::{com, ram};
use crate::machine::{Machine, State};
use std::collections::VecDeque;

/// One finished transaction, for the editor's link monitor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub command: u8,
    pub sent: Vec<u8>,
    pub received: Vec<u8>,
    pub summary: String,
}

#[derive(Default)]
pub struct Link {
    index: usize,
    command: u8,
    stopped: bool,
    sector: u16,
    data: Vec<u8>,
    status: u8,
    function: u8,
    parameters_len: usize,
    parameters: Vec<u8>,
    data_len: usize,
    buffer: u32,
    /// Bytes produced by built-in functions (FUNC 00h/02h reads).
    local: Vec<u8>,
    user_function: Option<u32>,
    sent: Vec<u8>,
    received: Vec<u8>,
    pub log: VecDeque<Transaction>,
    pub transactions: u64,
}

pub fn command_name(command: u8) -> &'static str {
    match command {
        0x50 => "Set value 50h",
        0x52 => "Read sector",
        0x53 => "Identify card",
        0x57 => "Write sector",
        0x58 => "Probe PocketStation",
        0x59 => "Start program",
        0x5A => "Read status",
        0x5B => "Call function (to PlayStation)",
        0x5C => "Call function (from PlayStation)",
        0x5D => "Transfer notice",
        0x5E => "Device power",
        0x5F => "Flash access",
        _ => "Unknown command",
    }
}

impl Machine {
    /// The PlayStation selects the card (start of a transaction).
    pub fn link_select(&mut self) {
        let log = std::mem::take(&mut self.link.log);
        let transactions = self.link.transactions;
        self.link = Link {
            log,
            transactions,
            ..Default::default()
        };
    }

    /// The PlayStation releases the card (end of a transaction).
    pub fn link_deselect(&mut self) {
        if self.link.sent.len() >= 2 {
            let command = self.link.sent[1];
            let summary = self.link_summary();
            self.link.transactions += 1;
            let entry = Transaction {
                command,
                sent: std::mem::take(&mut self.link.sent),
                received: std::mem::take(&mut self.link.received),
                summary,
            };
            if self.link.log.len() >= 64 {
                self.link.log.pop_front();
            }
            self.link.log.push_back(entry);
        }
        self.link.index = 0;
        self.link.stopped = true;
    }

    fn link_summary(&self) -> String {
        let l = &self.link;
        let name = command_name(l.command);
        match l.command {
            0x52 | 0x57 => {
                let status = match l.status {
                    0x47 => "ok".to_string(),
                    0 => "incomplete".to_string(),
                    s => format!("error {s:02X}h"),
                };
                format!("{name} {:#05x} ({status})", l.sector)
            }
            0x59 => {
                let new = l
                    .sent
                    .get(3..5)
                    .map(|b| u16::from_be_bytes([b[0], b[1]]))
                    .unwrap_or(0xFFFF);
                if new == 0xFFFF {
                    format!("{name}: query")
                } else {
                    format!("{name}: block {new}")
                }
            }
            0x5B | 0x5C => format!("{name} {:02X}h, {} bytes", l.function, l.data_len),
            _ => name.to_string(),
        }
    }

    /// Exchanges one byte. Returns the card's reply and whether it acknowledged
    /// (a byte without acknowledge ends the transaction).
    pub fn link_exchange(&mut self, byte: u8) -> (u8, bool) {
        if self.link.stopped {
            return (0xFF, false);
        }
        let i = self.link.index;
        self.link.index += 1;
        let (reply, ack) = self.link_byte(i, byte);
        self.link.sent.push(byte);
        self.link.received.push(reply);
        if !ack {
            self.link.stopped = true;
        }
        (reply, ack)
    }

    /// Runs a whole transaction: select, exchange every byte while the card
    /// acknowledges, deselect. Returns the replies received.
    pub fn link_transfer(&mut self, bytes: &[u8]) -> Vec<u8> {
        self.link_select();
        let mut replies = Vec::with_capacity(bytes.len());
        for &b in bytes {
            let (reply, ack) = self.link_exchange(b);
            replies.push(reply);
            if !ack {
                break;
            }
        }
        self.link_deselect();
        replies
    }

    fn link_byte(&mut self, i: usize, byte: u8) -> (u8, bool) {
        if i == 0 {
            if byte != 0x81 || !self.link_enabled() {
                return (0xFF, false);
            }
            return (0xFF, true);
        }
        let flag = self.hw.ram[ram::CARD_FLAG as usize];
        if i == 1 {
            self.link.command = byte;
            return match byte {
                0x52 | 0x53 | 0x57 => {
                    let flags = self.com_flags() | com::STANDARD_COMMAND;
                    self.set_com_flags(flags);
                    (flag, true)
                }
                0x50 | 0x58..=0x5F => (flag, true),
                _ => (flag, false),
            };
        }
        match self.link.command {
            0x52 => self.link_read(i, byte),
            0x57 => self.link_write(i, byte),
            0x53 => match i {
                2 => (0x5A, true),
                3 => (0x5D, true),
                4 => (0x5C, true),
                5 => (0x5D, true),
                6 => (0x04, true),
                7 => (0x00, true),
                8 => (0x00, true),
                _ => (0x80, false),
            },
            0x50 => {
                self.hw.ram[ram::BU50_VALUE as usize] = byte;
                (0x00, false)
            }
            0x58 => match i {
                2 => (0x02, true),
                3 => (0x01, true),
                _ => (0x01, false),
            },
            0x59 => self.link_start_program(i, byte),
            0x5A => self.link_status(i),
            0x5B | 0x5C => self.link_function(i, byte),
            0x5D => self.link_notice(i, byte),
            0x5E => {
                let bit = match i {
                    2 => return (0x03, true),
                    3 => 1,
                    4 => 3,
                    _ => 2,
                };
                let flags = self.com_flags();
                let old = (flags >> bit) & 1;
                self.set_com_flags((flags & !(1 << bit)) | (((byte & 1) as u32) << bit));
                (old as u8, i < 5)
            }
            0x5F => {
                if i == 2 {
                    return (0x01, true);
                }
                let flags = self.com_flags();
                self.set_com_flags((flags & !1) | (byte & 1) as u32);
                ((flags & 1) as u8, false)
            }
            _ => (0xFF, false),
        }
    }

    fn link_read(&mut self, i: usize, byte: u8) -> (u8, bool) {
        let l = &mut self.link;
        match i {
            2 => (0x5A, true),
            3 => (0x5D, true),
            4 => {
                l.sector = (byte as u16) << 8;
                (0x00, true)
            }
            5 => {
                l.sector |= byte as u16;
                (0x00, true)
            }
            6 => (0x5C, true),
            7 => (0x5D, true),
            8 | 9 if l.sector >= 0x400 => (0xFF, i == 8),
            8 => {
                let at = l.sector as usize * 0x80;
                l.data = self.hw.flash[at..at + 0x80].to_vec();
                (l.sector.to_be_bytes()[0], true)
            }
            9 => (l.sector.to_be_bytes()[1], true),
            10..=137 => (l.data[i - 10], true),
            138 => {
                let [msb, lsb] = l.sector.to_be_bytes();
                (l.data.iter().fold(msb ^ lsb, |a, b| a ^ b), true)
            }
            _ => {
                l.status = 0x47;
                (0x47, false)
            }
        }
    }

    fn link_write(&mut self, i: usize, byte: u8) -> (u8, bool) {
        match i {
            2 => (0x5A, true),
            3 => (0x5D, true),
            4 => {
                self.link.sector = (byte as u16) << 8;
                (0x00, true)
            }
            5 => {
                self.link.sector |= byte as u16;
                self.link.data.clear();
                (0x00, true)
            }
            6..=133 => {
                self.link.data.push(byte);
                (0x00, true)
            }
            134 => {
                let [msb, lsb] = self.link.sector.to_be_bytes();
                let expected = self.link.data.iter().fold(msb ^ lsb, |a, b| a ^ b);
                self.link.status = self.commit_write(expected == byte);
                (0x00, true)
            }
            135 => (0x5C, true),
            136 => (0x5D, true),
            _ => (self.link.status, false),
        }
    }

    fn commit_write(&mut self, checksum_ok: bool) -> u8 {
        let sector = self.link.sector as usize;
        if !checksum_ok {
            return 0x4E;
        }
        if sector >= 0x400 {
            return 0xFF;
        }
        if let State::Program(block) = self.state {
            let running = card::files(&self.hw.flash)
                .into_iter()
                .find(|f| f.block == block)
                .map(|f| f.blocks)
                .unwrap_or_default();
            if running.iter().any(|b| *b as usize == sector) {
                self.hw.ram[ram::CARD_FLAG as usize] |= 0x08;
                return 0xFD;
            }
        }
        if self.com_flags() & com::PROTECT_BROKEN != 0 && (16..=55).contains(&sector) {
            return 0xFE;
        }
        let at = sector * 0x80;
        let data = std::mem::take(&mut self.link.data);
        self.hw.flash[at..at + 0x80].copy_from_slice(&data);
        self.hw.ram[ram::CARD_FLAG as usize] &= !0x08;
        0x47
    }

    fn link_start_program(&mut self, i: usize, byte: u8) -> (u8, bool) {
        let current = self.ram16(ram::DIR_INDEX).to_be_bytes();
        match i {
            2 => (0x06, true),
            3 => {
                self.link.parameters = vec![byte];
                (current[0], true)
            }
            4 => {
                self.link.parameters.push(byte);
                (current[1], true)
            }
            5..=7 => {
                self.link.parameters.push(byte);
                (0x00, true)
            }
            _ => {
                self.link.parameters.push(byte);
                let p = std::mem::take(&mut self.link.parameters);
                let block = u16::from_be_bytes([p[0], p[1]]);
                let param = u32::from_le_bytes([p[2], p[3], p[4], p[5]]);
                match block {
                    0..=15 => {
                        let valid = block == 0 || self.executable(block as u8).is_some();
                        if valid {
                            self.set_ram16(ram::NEW_DIR_INDEX, block);
                            self.hw.set_ram_word(ram::NEW_PARAM, param);
                            let flags = self.com_flags() | com::START_REQUEST;
                            self.set_com_flags(flags);
                        }
                    }
                    0xFFFE => {
                        let flags = self.com_flags() | com::RESET_REQUEST;
                        self.set_com_flags(flags);
                    }
                    _ => {}
                }
                (0x00, false)
            }
        }
    }

    fn link_status(&mut self, i: usize) -> (u8, bool) {
        if i == 2 {
            return (0x12, true);
        }
        let flags = self.com_flags();
        let dir = self.ram16(ram::DIR_INDEX).to_be_bytes();
        let serial = self.hw.serial().to_le_bytes();
        let date = self.hw.rtc.date_register();
        let time = self.hw.rtc.time_register();
        let century = self.hw.ram[ram::CENTURY as usize];
        let bytes = [
            dir[0],
            dir[1],
            (flags & 1) as u8,
            ((flags >> 1) & 1) as u8,
            ((flags >> 3) & 1) as u8,
            ((flags >> 2) & 1) as u8,
            serial[0],
            serial[1],
            serial[2],
            serial[3],
            date as u8,
            (date >> 8) as u8,
            (date >> 16) as u8,
            century,
            time as u8,
            (time >> 8) as u8,
            (time >> 16) as u8,
            (time >> 24) as u8,
        ];
        let index = i - 3;
        (
            bytes.get(index).copied().unwrap_or(0),
            index + 1 < bytes.len(),
        )
    }

    fn link_notice(&mut self, i: usize, byte: u8) -> (u8, bool) {
        match i {
            2 => (0x03, true),
            3 | 4 => {
                self.link.parameters.push(byte);
                (0x00, true)
            }
            _ => {
                self.link.parameters.push(byte);
                let p = std::mem::take(&mut self.link.parameters);
                let value = ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | p[2] as u32;
                let mut flags = self.com_flags();
                if (value >> 8) & 0xFF == 0 {
                    flags |= com::PROTECT_BROKEN;
                } else {
                    flags &= !com::PROTECT_BROKEN;
                }
                self.set_com_flags(flags);
                let callback = self.hw.ram_word(ram::CALLBACKS + 12);
                if callback != 0
                    && matches!(self.state, State::Program(_))
                    && let Err(message) = self.call_user(callback, value, 0)
                {
                    self.crash(message);
                }
                if self.state == State::Launcher {
                    self.launcher.transfer_notice(value);
                }
                (0x00, false)
            }
        }
    }

    /// Parameter count of a link function, or `None` when it does not exist.
    fn function_parameters(&self, function: u8) -> Option<(usize, Option<u32>)> {
        match function {
            0x00 => Some((0, None)),
            0x01 => Some((5, None)),
            0x02 => Some((0, None)),
            0x03 => {
                if !matches!(self.state, State::Program(_)) {
                    return None;
                }
                let offset = self.ram16(ram::FUNC3) as u32;
                if offset == 0 {
                    return None;
                }
                Some((4, Some(file::FLASH_BASE + offset)))
            }
            0x80..=0xFF => {
                let State::Program(block) = self.state else {
                    return None;
                };
                let info = self.executable(block)?;
                let entry = info.functions.get((function - 0x80) as usize)?;
                Some((entry.parameters.min(128) as usize, Some(entry.address)))
            }
            _ => None,
        }
    }

    fn link_function(&mut self, i: usize, byte: u8) -> (u8, bool) {
        let to_playstation = self.link.command == 0x5B;
        if i == 2 {
            self.link.function = byte;
            match self.function_parameters(byte) {
                Some((len, user)) => {
                    self.link.parameters_len = len;
                    self.link.user_function = user;
                    self.link.parameters.clear();
                    return (0xFF, true);
                }
                None => return (0xFF, false),
            }
        }
        if i == 3 {
            return (self.link.parameters_len as u8, true);
        }
        let params_end = 4 + self.link.parameters_len;
        if i < params_end {
            self.link.parameters.push(byte);
            return (0x00, true);
        }
        if i == params_end {
            return match self.function_first_call(to_playstation) {
                Ok(len) => (len as u8, true),
                Err(message) => {
                    self.crash(message);
                    (0x00, false)
                }
            };
        }
        let data_end = params_end + 1 + self.link.data_len;
        if i < data_end {
            let offset = i - params_end - 1;
            if to_playstation {
                let reply = if self.link.local.is_empty() {
                    self.hw.peek8(self.link.buffer + offset as u32)
                } else {
                    self.link.local[offset]
                };
                return (reply, true);
            }
            if self.link.function == 0x00 || self.link.function == 0x02 {
                self.link.local.push(byte);
            } else {
                let address = self.link.buffer + offset as u32;
                if address < 0x800 || address >> 24 != 0 {
                    self.hw.write8(address, byte);
                    self.hw.take_abort();
                    self.hw.take_cycles();
                } else {
                    self.hw.ram[address as usize] = byte;
                }
            }
            return (0x00, true);
        }
        // Final byte: second call.
        if let Err(message) = self.function_second_call(to_playstation) {
            self.crash(message);
        }
        (0xFF, false)
    }

    fn function_first_call(&mut self, to_playstation: bool) -> Result<usize, String> {
        let p = self.link.parameters.clone();
        self.link.local.clear();
        let len = match self.link.function {
            0x00 => {
                if to_playstation {
                    let date = self.hw.rtc.date_register();
                    let time = self.hw.rtc.time_register();
                    self.link.local = vec![
                        date as u8,
                        (date >> 8) as u8,
                        (date >> 16) as u8,
                        self.hw.ram[ram::CENTURY as usize],
                        time as u8,
                        (time >> 8) as u8,
                        (time >> 16) as u8,
                        (time >> 24) as u8,
                    ];
                }
                8
            }
            0x01 => {
                self.link.buffer = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
                (p[4] as usize).min(0x80)
            }
            0x02 => {
                if to_playstation {
                    self.link.local =
                        self.hw.ram[ram::ALARM as usize..ram::ALARM as usize + 8].to_vec();
                }
                8
            }
            _ => {
                let address = self.link.user_function.unwrap_or(0);
                // Parameters go to the kernel's sector buffer.
                let at = ram::SECTOR_BUFFER as usize;
                self.hw.ram[at..at + 0x80].fill(0);
                self.hw.ram[at..at + p.len()].copy_from_slice(&p);
                let flags = if to_playstation { 0x09 } else { 0x0A };
                let result = self.call_user(address, flags, ram::SECTOR_BUFFER)?;
                if result == 0 {
                    0
                } else {
                    self.link.buffer = self.hw.peek32(result);
                    (self.hw.peek32(result + 4) as usize).min(0x80)
                }
            }
        };
        self.link.data_len = len;
        Ok(len)
    }

    fn function_second_call(&mut self, to_playstation: bool) -> Result<(), String> {
        match self.link.function {
            0x00 if !to_playstation && self.link.local.len() == 8 => {
                let d = &self.link.local;
                let date =
                    (d[3] as u32) << 24 | (d[2] as u32) << 16 | (d[1] as u32) << 8 | d[0] as u32;
                let time =
                    (d[7] as u32) << 24 | (d[6] as u32) << 16 | (d[5] as u32) << 8 | d[4] as u32;
                self.set_bcd_date_time(date, time);
            }
            0x02 if !to_playstation && self.link.local.len() == 8 => {
                let at = ram::ALARM as usize;
                let data = self.link.local.clone();
                self.hw.ram[at..at + 4].copy_from_slice(&data[..4]);
            }
            0x03 | 0x80..=0xFF => {
                let address = self.link.user_function.unwrap_or(0);
                let flags = if to_playstation { 0x11 } else { 0x12 };
                let buffer = self.link.buffer;
                self.call_user(address, flags, buffer)?;
            }
            _ => {}
        }
        Ok(())
    }
}
