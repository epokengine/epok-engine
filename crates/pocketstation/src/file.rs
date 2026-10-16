//! PocketStation executable files: the memory card title sector with the
//! PocketStation fields, the PlayStation colour icon, the function table used
//! by the PlayStation link, the monochrome menu icons and the program body.

use crate::card::{BLOCK_SIZE, SECTOR_SIZE};

/// The address where the file's first byte appears while it runs.
pub const FLASH_BASE: u32 = 0x0200_0000;

/// A 32x32 monochrome image: one word per row, bit 0 is the left pixel, 1 = black.
pub type MonoImage = [u32; 32];

/// The 16x16 16-colour icon shown by the PlayStation memory card manager.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorIcon {
    /// BGR555 colours (bit 15 is the semi-transparency flag).
    pub palette: [u16; 16],
    /// 1-3 frames of 16x16 4-bit pixels, low nibble first.
    pub frames: Vec<[u8; 128]>,
}

impl Default for ColorIcon {
    fn default() -> Self {
        let mut palette = [0u16; 16];
        palette[1] = 0x7FFF;
        palette[2] = 0x8000;
        Self {
            palette,
            frames: vec![[0x11; 128]],
        }
    }
}

/// An animated monochrome icon for the PocketStation's application menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuIcon {
    pub frames: Vec<MonoImage>,
    /// Frames are shown for this many 1/30 s ticks.
    pub delay: u16,
}

/// One entry of the function table called by the PlayStation (FUNC 80h+).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkFunction {
    /// Number of parameter bytes the PlayStation sends first (0..=128).
    pub parameters: u32,
    /// Handler address (bit 0 set for Thumb).
    pub address: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub title: String,
    pub color_icon: ColorIcon,
    pub viewer_icons: Vec<MonoImage>,
    pub menu_icons: Vec<MenuIcon>,
    pub function_count: usize,
    pub snapshot: bool,
}

/// Byte offsets of each region in an executable with this header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub color_icons: usize,
    pub snapshot: usize,
    pub functions: usize,
    pub viewer_icons: usize,
    pub menu_list: usize,
    pub menu_frames: usize,
    /// Where the program body starts (word aligned).
    pub body: usize,
}

impl Header {
    pub fn layout(&self) -> Layout {
        let color_icons = SECTOR_SIZE;
        let snapshot = color_icons + self.color_icon.frames.len().max(1) * SECTOR_SIZE;
        let functions = snapshot + if self.snapshot { 0x800 } else { 0 };
        let viewer_icons =
            functions + (self.function_count * 8).div_ceil(SECTOR_SIZE) * SECTOR_SIZE;
        let menu_list = viewer_icons + self.viewer_icons.len() * SECTOR_SIZE;
        let menu_frames =
            menu_list + (self.menu_icons.len().max(1) * 8).div_ceil(SECTOR_SIZE) * SECTOR_SIZE;
        let frames: usize = self.menu_icons.iter().map(|i| i.frames.len()).sum();
        let body = menu_frames + frames * SECTOR_SIZE;
        Layout {
            color_icons,
            snapshot,
            functions,
            viewer_icons,
            menu_list,
            menu_frames,
            body,
        }
    }

    /// Virtual address of the program body while the file runs.
    pub fn body_address(&self) -> u32 {
        FLASH_BASE + self.layout().body as u32
    }
}

/// Converts ASCII to the full-width Shift-JIS used by memory card titles.
pub fn shift_jis_title(title: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for c in title.chars().take(32) {
        let code: u16 = match c {
            '0'..='9' => 0x824F + (c as u16 - '0' as u16),
            'A'..='Z' => 0x8260 + (c as u16 - 'A' as u16),
            'a'..='z' => 0x8281 + (c as u16 - 'a' as u16),
            ' ' => 0x8140,
            '!' => 0x8149,
            '"' => 0x8168,
            '#' => 0x8194,
            '$' => 0x8190,
            '%' => 0x8193,
            '&' => 0x8195,
            '\'' => 0x8166,
            '(' => 0x8169,
            ')' => 0x816A,
            '*' => 0x8196,
            '+' => 0x817B,
            ',' => 0x8143,
            '-' => 0x817C,
            '.' => 0x8144,
            '/' => 0x815E,
            ':' => 0x8146,
            ';' => 0x8147,
            '<' => 0x8183,
            '=' => 0x8181,
            '>' => 0x8184,
            '?' => 0x8148,
            '@' => 0x8197,
            '[' => 0x816D,
            ']' => 0x816E,
            '^' => 0x814F,
            '_' => 0x8151,
            '{' => 0x816F,
            '|' => 0x8162,
            '}' => 0x8170,
            '~' => 0x8160,
            _ => 0x8148,
        };
        out.extend_from_slice(&code.to_be_bytes());
    }
    out.truncate(64);
    out
}

/// Converts a full-width Shift-JIS title back to ASCII where possible.
pub fn title_text(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() && bytes[i] != 0 {
        let b = bytes[i];
        if b >= 0x81 && i + 1 < bytes.len() {
            let code = u16::from_be_bytes([b, bytes[i + 1]]);
            let c = match code {
                0x824F..=0x8258 => (b'0' + (code - 0x824F) as u8) as char,
                0x8260..=0x8279 => (b'A' + (code - 0x8260) as u8) as char,
                0x8281..=0x829A => (b'a' + (code - 0x8281) as u8) as char,
                0x8140 => ' ',
                0x8149 => '!',
                0x817C => '-',
                0x8144 => '.',
                0x8146 => ':',
                0x8143 => ',',
                0x8148 => '?',
                _ => '?',
            };
            out.push(c);
            i += 2;
        } else {
            out.push(b as char);
            i += 1;
        }
    }
    out
}

/// Builds a complete executable file, padded to whole memory card blocks.
///
/// `body` is placed at [`Header::layout`]`.body` and must have been assembled
/// for [`Header::body_address`]. `entry` is a virtual address (bit 0 = Thumb).
pub fn build(
    header: &Header,
    functions: &[LinkFunction],
    body: &[u8],
    entry: u32,
) -> Result<Vec<u8>, String> {
    if functions.len() != header.function_count {
        return Err("function table size does not match the header".into());
    }
    if functions.len() > 0x7F {
        return Err("at most 127 link functions are supported".into());
    }
    if header.color_icon.frames.is_empty() || header.color_icon.frames.len() > 3 {
        return Err("the PlayStation icon needs 1 to 3 frames".into());
    }
    if header.menu_icons.is_empty() || header.menu_icons.iter().any(|i| i.frames.is_empty()) {
        return Err("at least one menu icon with one frame is required".into());
    }
    let layout = header.layout();
    let total = layout.body + body.len();
    let blocks = total.div_ceil(BLOCK_SIZE);
    if blocks > 15 {
        return Err(format!(
            "the program needs {blocks} blocks; a card holds 15"
        ));
    }
    let mut out = vec![0u8; blocks * BLOCK_SIZE];
    // Title sector.
    out[0] = b'S';
    out[1] = b'C';
    out[2] = 0x10 + header.color_icon.frames.len() as u8;
    out[3] = blocks as u8;
    let title = shift_jis_title(&header.title);
    out[4..4 + title.len()].copy_from_slice(&title);
    out[0x50..0x52].copy_from_slice(&(header.viewer_icons.len() as u16).to_le_bytes());
    out[0x52..0x56].copy_from_slice(if header.snapshot { b"MCX1" } else { b"MCX0" });
    out[0x56] = header.menu_icons.len() as u8;
    out[0x57] = functions.len() as u8;
    out[0x5C..0x60].copy_from_slice(&entry.to_le_bytes());
    for (i, colour) in header.color_icon.palette.iter().enumerate() {
        out[0x60 + i * 2..0x62 + i * 2].copy_from_slice(&colour.to_le_bytes());
    }
    for (i, frame) in header.color_icon.frames.iter().enumerate() {
        let at = layout.color_icons + i * SECTOR_SIZE;
        out[at..at + SECTOR_SIZE].copy_from_slice(frame);
    }
    for (i, f) in functions.iter().enumerate() {
        let at = layout.functions + i * 8;
        out[at..at + 4].copy_from_slice(&f.parameters.min(128).to_le_bytes());
        out[at + 4..at + 8].copy_from_slice(&f.address.to_le_bytes());
    }
    for (i, image) in header.viewer_icons.iter().enumerate() {
        write_mono(&mut out, layout.viewer_icons + i * SECTOR_SIZE, image);
    }
    let mut frame_at = layout.menu_frames;
    for (i, icon) in header.menu_icons.iter().enumerate() {
        let at = layout.menu_list + i * 8;
        out[at..at + 2].copy_from_slice(&(icon.frames.len() as u16).to_le_bytes());
        out[at + 2..at + 4].copy_from_slice(&icon.delay.max(1).to_le_bytes());
        out[at + 4..at + 8].copy_from_slice(&(FLASH_BASE + frame_at as u32).to_le_bytes());
        for image in &icon.frames {
            write_mono(&mut out, frame_at, image);
            frame_at += SECTOR_SIZE;
        }
    }
    out[layout.body..layout.body + body.len()].copy_from_slice(body);
    Ok(out)
}

fn write_mono(out: &mut [u8], at: usize, image: &MonoImage) {
    for (row, word) in image.iter().enumerate() {
        out[at + row * 4..at + row * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
}

fn read_mono(data: &[u8], at: usize) -> Option<MonoImage> {
    let bytes = data.get(at..at + SECTOR_SIZE)?;
    let mut image = [0u32; 32];
    for (row, word) in image.iter_mut().enumerate() {
        *word = u32::from_le_bytes(bytes[row * 4..row * 4 + 4].try_into().unwrap());
    }
    Some(image)
}

/// Facts read back from an executable's header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub title: String,
    pub snapshot: bool,
    pub entry: u32,
    pub blocks: u8,
    pub functions: Vec<LinkFunction>,
    pub menu_icons: Vec<MenuIcon>,
    pub viewer_icons: Vec<MonoImage>,
    pub color_frames: usize,
}

/// Parses a file's header; `None` when the file is not a PocketStation executable.
pub fn parse(data: &[u8]) -> Option<Info> {
    if data.len() < SECTOR_SIZE || &data[0..2] != b"SC" {
        return None;
    }
    let id = &data[0x52..0x56];
    let snapshot = match id {
        b"MCX0" => false,
        b"MCX1" => true,
        _ => return None,
    };
    let color_frames = (data[2] & 0x0F).clamp(1, 3) as usize;
    let viewer_count = u16::from_le_bytes([data[0x50], data[0x51]]) as usize;
    let menu_count = data[0x56] as usize;
    let function_count = data[0x57] as usize;
    let entry = u32::from_le_bytes(data[0x5C..0x60].try_into().unwrap());
    let functions_at = SECTOR_SIZE * (1 + color_frames) + if snapshot { 0x800 } else { 0 };
    let mut functions = Vec::new();
    for i in 0..function_count {
        let at = functions_at + i * 8;
        let entry = data.get(at..at + 8)?;
        functions.push(LinkFunction {
            parameters: u32::from_le_bytes(entry[0..4].try_into().unwrap()),
            address: u32::from_le_bytes(entry[4..8].try_into().unwrap()),
        });
    }
    let viewer_at = functions_at + (function_count * 8).div_ceil(SECTOR_SIZE) * SECTOR_SIZE;
    let viewer_icons = (0..viewer_count)
        .filter_map(|i| read_mono(data, viewer_at + i * SECTOR_SIZE))
        .collect();
    let list_at = viewer_at + viewer_count * SECTOR_SIZE;
    let mut menu_icons = Vec::new();
    for i in 0..menu_count {
        let at = list_at + i * 8;
        let entry = data.get(at..at + 8)?;
        let count = u16::from_le_bytes([entry[0], entry[1]]) as usize;
        let delay = u16::from_le_bytes([entry[2], entry[3]]);
        let address = u32::from_le_bytes(entry[4..8].try_into().unwrap());
        let offset = address.wrapping_sub(FLASH_BASE) as usize;
        let frames = (0..count.min(64))
            .filter_map(|f| read_mono(data, offset + f * SECTOR_SIZE))
            .collect();
        menu_icons.push(MenuIcon { frames, delay });
    }
    Some(Info {
        title: title_text(&data[4..0x44]),
        snapshot,
        entry,
        blocks: data[3],
        functions,
        menu_icons,
        viewer_icons,
        color_frames,
    })
}

/// Parses a picture made of `#`/`X` (black) and `.`/space (white) rows.
pub fn mono_from_text(text: &str) -> MonoImage {
    let mut image = [0u32; 32];
    for (row, line) in text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(32)
        .enumerate()
    {
        for (column, c) in line.trim().chars().take(32).enumerate() {
            if c == '#' || c == 'X' || c == 'x' || c == '@' {
                image[row] |= 1 << column;
            }
        }
    }
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header() -> Header {
        Header {
            title: "Pocket Demo".into(),
            color_icon: ColorIcon::default(),
            viewer_icons: vec![],
            menu_icons: vec![MenuIcon {
                frames: vec![mono_from_text("##\n.#"), [1; 32]],
                delay: 6,
            }],
            function_count: 2,
            snapshot: false,
        }
    }

    #[test]
    fn layout_and_round_trip() {
        let h = header();
        let layout = h.layout();
        assert_eq!(layout.color_icons, 0x80);
        assert_eq!(layout.functions, 0x100);
        assert_eq!(layout.viewer_icons, 0x180);
        assert_eq!(layout.menu_list, 0x180);
        assert_eq!(layout.menu_frames, 0x200);
        assert_eq!(layout.body, 0x300);
        let functions = [
            LinkFunction {
                parameters: 1,
                address: 0x0200_0301,
            },
            LinkFunction {
                parameters: 0,
                address: 0x0200_0311,
            },
        ];
        let file = build(&h, &functions, &[0xAA; 16], h.body_address() | 1).unwrap();
        assert_eq!(file.len(), BLOCK_SIZE);
        let info = parse(&file).unwrap();
        assert_eq!(info.title, "Pocket Demo");
        assert_eq!(info.entry, 0x0200_0301);
        assert_eq!(info.functions, functions);
        assert_eq!(info.menu_icons, h.menu_icons);
        assert_eq!(file[0x300], 0xAA);
    }
}
