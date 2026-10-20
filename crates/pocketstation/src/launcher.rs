//! Epok's start-up application: a clock face and the program menu.
//!
//! It runs natively in the emulator (the CPU idles in the kernel ROM), reads
//! the buttons, answers start requests from the PlayStation and launches the
//! program selected with Fire.

use crate::card;
use crate::file;
use crate::font::Canvas;
use crate::hw::button;
use crate::kernel::{com, ram};
use crate::machine::{Machine, State};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Clock,
    Menu,
}

#[derive(Clone, Debug, Default)]
pub struct Launcher {
    pub screen: Screen,
    pub selected: usize,
    frame: u32,
    previous_buttons: u8,
    transfer_frames: u32,
    /// Executable programs on the card: (first block, header).
    pub programs: Vec<(u8, file::Info)>,
    card_revision: u64,
}

impl Launcher {
    pub fn reset(&mut self) {
        self.screen = Screen::Clock;
        self.frame = 0;
        self.transfer_frames = 0;
        self.card_revision = u64::MAX;
    }

    pub fn transfer_notice(&mut self, value: u32) {
        self.transfer_frames = if value & 0x100 != 0 {
            0
        } else {
            (value & 0xFF) * 30
        };
    }
}

fn card_revision(flash: &[u8]) -> u64 {
    // Cheap change detector over the directory block.
    flash[..0x800].iter().enumerate().fold(0u64, |h, (i, b)| {
        h.wrapping_mul(31).wrapping_add((*b as u64) ^ i as u64)
    })
}

impl Machine {
    pub(crate) fn launcher_frame(&mut self) {
        let revision = card_revision(&self.hw.flash);
        if revision != self.launcher.card_revision {
            self.launcher.card_revision = revision;
            self.launcher.programs = card::files(&self.hw.flash)
                .into_iter()
                .filter_map(|f| {
                    let at = f.block as usize * card::BLOCK_SIZE;
                    file::parse(&self.hw.flash[at..at + card::BLOCK_SIZE])
                        .map(|info| (f.block, info))
                })
                .collect();
            if self.launcher.selected >= self.launcher.programs.len() {
                self.launcher.selected = 0;
            }
        }
        self.sync_launcher_docking();
        let flags = self.com_flags();
        if flags & com::START_REQUEST != 0 {
            self.set_com_flags(flags & !com::START_REQUEST);
            let block = self.ram16(ram::NEW_DIR_INDEX) as u8;
            let param = self.hw.ram_word(ram::NEW_PARAM);
            if block != 0 && self.start(block, param).is_ok() {
                return;
            }
        }
        if flags & com::RESET_REQUEST != 0 {
            self.set_com_flags(flags & !com::RESET_REQUEST);
        }
        let buttons = self.hw.buttons;
        let pressed = buttons & !self.launcher.previous_buttons;
        self.launcher.previous_buttons = buttons;
        let l = &mut self.launcher;
        l.frame = l.frame.wrapping_add(1);
        l.transfer_frames = l.transfer_frames.saturating_sub(1);
        let count = l.programs.len();
        match l.screen {
            Screen::Clock => {
                if pressed & (button::LEFT | button::RIGHT | button::FIRE) != 0 {
                    l.screen = Screen::Menu;
                }
            }
            Screen::Menu => {
                if count > 0 && pressed & button::RIGHT != 0 {
                    l.selected = (l.selected + 1) % count;
                }
                if count > 0 && pressed & button::LEFT != 0 {
                    l.selected = (l.selected + count - 1) % count;
                }
                if pressed & (button::UP | button::DOWN) != 0 {
                    l.screen = Screen::Clock;
                }
                if pressed & button::FIRE != 0 && count > 0 {
                    let block = l.programs[l.selected].0;
                    if self.start(block, 0).is_ok() {
                        return;
                    }
                }
            }
        }
        let canvas = self.launcher_draw();
        // The screen is drawn as seen in the hand; undo the docked rotation.
        let flipped = (self.hw.lcd_mode & 0x80 != 0) != self.hw.docked;
        for (i, row) in canvas.0.iter().enumerate() {
            if flipped {
                self.hw.vram[31 - i] = row.reverse_bits();
            } else {
                self.hw.vram[i] = *row;
            }
        }
        if self.state == State::Launcher {
            self.hw.lcd_mode |= 0x40;
        }
    }

    fn launcher_draw(&self) -> Canvas {
        let l = &self.launcher;
        let mut c = Canvas::default();
        match l.screen {
            Screen::Clock => {
                let rtc = &self.hw.rtc;
                let blink = (l.frame / 15).is_multiple_of(2);
                c.digit((rtc.hour / 10) as u32, 2, 4);
                c.digit((rtc.hour % 10) as u32, 8, 4);
                if blink {
                    c.set(15, 6, true);
                    c.set(15, 9, true);
                }
                c.digit((rtc.minute / 10) as u32, 18, 4);
                c.digit((rtc.minute % 10) as u32, 24, 4);
                // Seconds bar.
                let seconds = rtc.second as i32;
                c.rect(1, 14, 1 + seconds * 30 / 59, 1);
                let date = format!("{:02}/{:02}", rtc.month, rtc.day);
                c.text(&date, 6, 18);
                if self.hw.docked {
                    // Memory card symbol.
                    c.rect(2, 25, 6, 6);
                    c.set(3, 26, false);
                    c.set(5, 26, false);
                }
                if l.transfer_frames > 0 && blink {
                    c.text("<>", 12, 26);
                }
                if !l.programs.is_empty() {
                    c.text(&format!("{}", l.programs.len()), 26, 26);
                }
            }
            Screen::Menu => {
                if let Some((_, info)) = l.programs.get(l.selected) {
                    if let Some(icon) = info.menu_icons.first()
                        && !icon.frames.is_empty()
                    {
                        let delay = icon.delay.max(1) as u32;
                        let frame = (l.frame / delay) as usize % icon.frames.len();
                        c.image(&icon.frames[frame]);
                    }
                    if l.programs.len() > 1 && (l.frame / 20).is_multiple_of(3) {
                        c.set(0, 15, true);
                        c.set(31, 15, true);
                    }
                } else {
                    c.text("NO", 12, 8);
                    c.text("APPS", 8, 16);
                }
            }
        }
        c
    }
}
