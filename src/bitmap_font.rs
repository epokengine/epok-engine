//! Shared editor/console 8x16 bitmap glyphs. The added Spanish glyphs are derived
//! from the bundled mig68000 font; its attribution travels with every export.
pub const EXTRA: &str = "áéíóúüñÁÉÍÓÚÜÑ¿¡";
pub const MAX_BYTES: usize = 511;
const DEBUG: &str = " 0123456789.FPSCUGTE";
const DEBUG_U: usize = 120;
const DEBUG_Y: usize = 48;
pub fn index(c: char) -> Option<usize> {
    if (' '..='~').contains(&c) {
        Some(c as usize - 32)
    } else {
        EXTRA.chars().position(|v| v == c).map(|v| 95 + v)
    }
}
pub fn glyph(c: char) -> Option<[u8; 16]> {
    let i = index(c)?;
    let font = include_bytes!("../resources/editor/psx-font.bin");
    if i < 95 {
        return Some(font[i * 16..i * 16 + 16].try_into().unwrap());
    }
    let (base, accent) = match c {
        'á' => ('a', 0),
        'é' => ('e', 0),
        'í' => ('i', 0),
        'ó' => ('o', 0),
        'ú' => ('u', 0),
        'Á' => ('A', 0),
        'É' => ('E', 0),
        'Í' => ('I', 0),
        'Ó' => ('O', 0),
        'Ú' => ('U', 0),
        'ü' => ('u', 1),
        'Ü' => ('U', 1),
        'ñ' => ('n', 2),
        'Ñ' => ('N', 2),
        '¿' => ('?', 3),
        '¡' => ('!', 3),
        _ => return None,
    };
    let mut rows = glyph(base)?;
    if accent == 3 {
        rows.reverse();
        return Some(rows);
    }
    // Reserve the top three rows without changing the baseline.
    rows[0] = 0;
    rows[1] = match accent {
        0 => 0x20,
        1 => 0x24,
        _ => 0x14,
    };
    rows[2] = match accent {
        0 => 0x10,
        1 => 0x24,
        _ => 0x28,
    };
    Some(rows)
}
pub fn validate(text: &str) -> Result<(), String> {
    if text.len() > MAX_BYTES {
        return Err(format!("Text exceeds {MAX_BYTES} UTF-8 bytes"));
    }
    if let Some(c) = text.chars().find(|c| *c != '\n' && index(*c).is_none()) {
        return Err(format!("Bitmap font has no glyph for {c:?}"));
    }
    Ok(())
}

// Five visible columns in a six-pixel cell. Bit zero is the leftmost pixel.
// The profiler needs only this small alphabet, so it fits in the unused tail of
// the regular 8x16 atlas without consuming another texture page or CLUT.
fn debug_glyph(c: char) -> Option<[u8; 7]> {
    Some(match c {
        ' ' => [0, 0, 0, 0, 0, 0, 0],
        '0' => [14, 17, 17, 17, 17, 17, 14],
        '1' => [4, 6, 4, 4, 4, 4, 14],
        '2' => [14, 17, 16, 8, 4, 2, 31],
        '3' => [14, 16, 16, 14, 16, 16, 14],
        '4' => [8, 12, 10, 9, 31, 8, 8],
        '5' => [31, 1, 1, 15, 16, 16, 15],
        '6' => [14, 1, 1, 15, 17, 17, 14],
        '7' => [31, 16, 8, 4, 2, 2, 2],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 30, 16, 16, 14],
        '.' => [0, 0, 0, 0, 0, 6, 6],
        'F' => [31, 1, 1, 15, 1, 1, 1],
        'P' => [15, 17, 17, 15, 1, 1, 1],
        'S' => [14, 17, 1, 14, 16, 17, 14],
        'C' => [14, 17, 1, 1, 1, 17, 14],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'G' => [14, 17, 1, 29, 17, 17, 14],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'E' => [31, 1, 1, 15, 1, 1, 31],
        _ => return None,
    })
}
/// Positions in glyph cells, with character wrapping and explicit newlines.
#[cfg(test)]
pub fn layout(text: &str, columns: usize, rows: usize, wrap: bool) -> Vec<(usize, usize, char)> {
    let mut out = Vec::new();
    if columns == 0 || rows == 0 {
        return out;
    }
    let (mut x, mut y) = (0, 0);
    for c in text.chars() {
        if c == '\n' {
            x = 0;
            y += 1;
            continue;
        }
        if x >= columns && wrap {
            x = 0;
            y += 1;
        }
        if y >= rows {
            break;
        }
        if x < columns {
            out.push((x, y, c));
        }
        x += 1;
    }
    out
}
pub fn header() -> String {
    let mut words = [0u16; 64 * 64];
    for c in (' '..='~').chain(EXTRA.chars()) {
        let i = index(c).unwrap();
        let rows = glyph(c).unwrap();
        for (y, bits) in rows.iter().enumerate() {
            for x in 0..8 {
                if bits & (1 << x) != 0 {
                    let px = (i % 32) * 8 + x;
                    let py = (i / 32) * 16 + y;
                    words[py * 64 + px / 4] |= 1 << ((px % 4) * 4);
                }
            }
        }
    }
    for (i, c) in DEBUG.chars().enumerate() {
        let rows = debug_glyph(c).unwrap();
        for (y, bits) in rows.iter().enumerate() {
            for x in 0..5 {
                if bits & (1 << x) != 0 {
                    let px = DEBUG_U + i * 6 + x;
                    let py = DEBUG_Y + y;
                    words[py * 64 + px / 4] |= 1 << ((px % 4) * 4);
                }
            }
        }
    }
    // Palette in the first transparent space glyph, outside all drawn ink.
    words[1] = 0x7fff;
    format!(
        "#pragma once\n#include <stdint.h>\nnamespace epok {{\ninline constexpr char debug_font_characters[]=\"{DEBUG}\";\ninline constexpr unsigned debug_font_u={DEBUG_U},debug_font_v={},debug_font_width=5,debug_font_height=7,debug_font_advance=6;\nalignas(4) inline constexpr uint16_t hud_font_pixels[4096]={{{}}};\n}}\n",
        192 + DEBUG_Y,
        words
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extended_glyphs_and_wrapping() {
        validate(&format!("Extended glyphs:\n{EXTRA}")).unwrap();
        assert!(validate("漢").is_err());
        assert!(validate(&"a".repeat(512)).is_err());
        assert_ne!(glyph('ñ'), glyph('n'));
        assert_eq!(
            layout("áb\ncde", 2, 3, true),
            vec![
                (0, 0, 'á'),
                (1, 0, 'b'),
                (0, 1, 'c'),
                (1, 1, 'd'),
                (0, 2, 'e')
            ]
        );
        assert_eq!(
            layout("abc\nd", 2, 2, false),
            vec![(0, 0, 'a'), (1, 0, 'b'), (0, 1, 'd')]
        );
    }
}
