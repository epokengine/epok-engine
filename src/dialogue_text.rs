//! Page dialogue text with the renderer's character wrapping and font advances.
//! Each emitted UTF-8 page fits Text's payload, including carried color markup.
use std::collections::BTreeMap;
pub const MAX_PAGE_BYTES: usize = 511;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metrics {
    pub line_height: u16,
    pub advances: BTreeMap<char, u16>,
}
impl Metrics {
    pub fn builtin() -> Self {
        let advances = (' '..='~')
            .chain(crate::bitmap_font::EXTRA.chars())
            .map(|c| (c, 8))
            .collect();
        Self {
            line_height: 16,
            advances,
        }
    }
    pub fn imported(font: &crate::font_asset::FontData) -> Self {
        Self {
            line_height: u16::from(font.line_height).max(1),
            advances: font
                .metrics
                .iter()
                .filter_map(|g| char::from_u32(g.codepoint).map(|c| (c, u16::from(g.advance))))
                .collect(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    pub text: String,
    pub glyphs: u16,
}
#[derive(Clone, Debug)]
enum Token<'a> {
    Color(&'a str, bool),
    Glyph(&'a str, char),
}
fn tokenize(mut source: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    while !source.is_empty() {
        if source.starts_with("[/color]") {
            tokens.push(Token::Color(&source[..8], false));
            source = &source[8..];
        } else if source.len() >= 15
            && source.starts_with("[color=#")
            && source.as_bytes()[14] == b']'
            && source.as_bytes()[8..14].iter().all(u8::is_ascii_hexdigit)
        {
            tokens.push(Token::Color(&source[..15], true));
            source = &source[15..];
        } else {
            let c = source.chars().next().unwrap();
            let bytes = c.len_utf8();
            tokens.push(Token::Glyph(&source[..bytes], c));
            source = &source[bytes..];
        }
    }
    tokens
}
pub fn unpaged(text: &str) -> Result<Page, String> {
    if text.len() > MAX_PAGE_BYTES {
        return Err(
            "Text exceeds the 511-byte payload; bind an authored dialogue widget for pagination"
                .into(),
        );
    }
    if text.contains('\0') {
        return Err("Dialogue text contains a NUL byte".into());
    }
    Ok(Page {
        text: text.into(),
        glyphs: tokenize(text)
            .iter()
            .filter(|token| matches!(token,Token::Glyph(_,c) if *c!='\n'))
            .count() as u16,
    })
}
#[cfg(test)]
pub fn paginate(
    text: &str,
    metrics: &Metrics,
    width: u16,
    height: u16,
) -> Result<Vec<Page>, String> {
    paginate_control(text, metrics, width, height, true)
}
pub fn paginate_control(
    text: &str,
    metrics: &Metrics,
    width: u16,
    height: u16,
    wrap: bool,
) -> Result<Vec<Page>, String> {
    if metrics.line_height == 0 || width == 0 || height < metrics.line_height {
        return Err("Dialogue page has no complete font line".into());
    }
    let rows = height / metrics.line_height;
    let mut pages = Vec::new();
    let mut output = String::new();
    let mut glyphs = 0u16;
    let mut row = 0u16;
    let mut pen = 0u32;
    let mut color: Option<&str> = None;
    let flush = |pages: &mut Vec<Page>,
                 output: &mut String,
                 glyphs: &mut u16,
                 color: Option<&str>|
     -> Result<(), String> {
        if pages.len() >= usize::from(u16::MAX) {
            return Err("Dialogue text exceeds 65535 displayed pages".into());
        }
        pages.push(Page {
            text: std::mem::take(output),
            glyphs: *glyphs,
        });
        *glyphs = 0;
        if let Some(color) = color {
            output.push_str(color);
        }
        Ok(())
    };
    let tokens = tokenize(text);
    for token in tokens {
        let raw = match token {
            Token::Color(raw, _) | Token::Glyph(raw, _) => raw,
        };
        if output.len() + raw.len() > MAX_PAGE_BYTES {
            if glyphs == 0 {
                return Err(
                    "Color markup exceeds a dialogue page's 511-byte payload before any glyph"
                        .into(),
                );
            }
            flush(&mut pages, &mut output, &mut glyphs, color)?;
            row = 0;
            pen = 0;
        }
        match token {
            Token::Color(raw, open) => {
                output.push_str(raw);
                color = open.then_some(raw);
            }
            Token::Glyph(raw, '\n') => {
                if row + 1 >= rows {
                    flush(&mut pages, &mut output, &mut glyphs, color)?;
                    row = 0;
                    pen = 0;
                } else {
                    output.push_str(raw);
                    row += 1;
                    pen = 0;
                }
            }
            Token::Glyph(raw, c) => {
                let advance = u32::from(
                    *metrics
                        .advances
                        .get(&c)
                        .ok_or_else(|| format!("Dialogue font has no glyph for {c:?}"))?,
                );
                if advance > u32::from(width) {
                    return Err(format!(
                        "Dialogue glyph {c:?} is {advance} px wide; page width is {width} px"
                    ));
                }
                if pen + advance > u32::from(width) {
                    pen = 0;
                    row += 1;
                    if !wrap || row >= rows {
                        flush(&mut pages, &mut output, &mut glyphs, color)?;
                        row = 0;
                    }
                }
                output.push_str(raw);
                glyphs += 1;
                pen += advance;
            }
        }
    }
    // A carried color tag alone after a terminal page break is not a new page.
    if pages.is_empty() || glyphs > 0 || crate::hud::visible_text(&output).contains('\n') {
        if pages.len() >= usize::from(u16::MAX) {
            return Err("Dialogue text exceeds 65535 displayed pages".into());
        }
        pages.push(Page {
            text: output,
            glyphs,
        });
    }
    debug_assert!(pages.iter().all(|p| p.text.len() <= MAX_PAGE_BYTES));
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialogue_pages_fit_the_actual_hud_compiler() {
        let pages = paginate(
            &format!("[color=#ff8000]{}[/color]", "ÁW i\n".repeat(120)),
            &Metrics::builtin(),
            160,
            48,
        )
        .unwrap();
        for page in pages {
            let mut scene = crate::scene::Scene::default();
            scene.actors.clear();
            let mut canvas = crate::scene::Actor::cube("Canvas".into());
            canvas.kind = "Empty".into();
            canvas.canvas = Some(Default::default());
            scene.actors.push(canvas);
            let mut label = crate::scene::Actor::cube("Page".into());
            label.kind = "Empty".into();
            label.parent = Some(0);
            label.rect = Some(crate::hud::RectTransform {
                size: [160., 48.],
                ..Default::default()
            });
            label.text = Some(crate::hud::Text {
                text: page.text.clone(),
                ..Default::default()
            });
            scene.actors.push(label);
            let (commands, stats) = crate::hud_native::compile(&scene);
            let drawn = crate::hud::visible_text(&page.text)
                .chars()
                .filter(|c| !c.is_whitespace())
                .count();
            assert_eq!(commands.len(), drawn, "{}", page.text);
            assert_eq!(stats[1] as usize, drawn);
            assert_eq!(stats[4], 0, "no dropped primitives");
        }
    }
    #[test]
    fn dialogue_pages_preserve_utf8_color_and_all_long_text() {
        let source = format!("[color=#ff0000]{}[/color]", "áBC".repeat(300));
        let pages = paginate(&source, &Metrics::builtin(), 160, 48).unwrap();
        assert!(pages.len() > 1 && pages.iter().all(|p| p.text.len() <= 511));
        assert!(
            pages
                .iter()
                .skip(1)
                .all(|p| p.text.starts_with("[color=#ff0000]"))
        );
        let joined = pages
            .iter()
            .map(|p| crate::hud::visible_text(&p.text))
            .collect::<String>();
        assert_eq!(joined, crate::hud::visible_text(&source));
        assert_eq!(
            pages.iter().map(|p| usize::from(p.glyphs)).sum::<usize>(),
            900
        );
    }
    #[test]
    fn dialogue_pages_handle_overlong_tokens_and_variable_font_advances() {
        let mut metrics = Metrics::builtin();
        metrics.line_height = 10;
        metrics.advances.insert('W', 9);
        metrics.advances.insert('i', 2);
        let pages = paginate("WiiWiiWii", &metrics, 13, 10).unwrap();
        assert_eq!(
            pages.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            ["Wii", "Wii", "Wii"]
        );
        let pages = paginate(&"A".repeat(600), &Metrics::builtin(), u16::MAX, u16::MAX).unwrap();
        assert_eq!(
            pages.iter().map(|p| p.text.len()).collect::<Vec<_>>(),
            [511, 89]
        );
        assert!(paginate("W", &metrics, 8, 10).unwrap_err().contains("wide"));
        assert!(
            paginate("漢", &metrics, 40, 40)
                .unwrap_err()
                .contains("no glyph")
        );
        assert!(paginate("x", &metrics, 40, 9).is_err());
    }
    #[test]
    fn dialogue_pages_keep_explicit_lines_and_reject_unbounded_markup() {
        let pages = paginate("A\nB\nC", &Metrics::builtin(), 64, 32).unwrap();
        assert_eq!(
            pages.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            ["A\nB", "C"]
        );
        assert_eq!(
            paginate("", &Metrics::builtin(), 64, 32).unwrap(),
            [Page {
                text: String::new(),
                glyphs: 0
            }]
        );
        assert!(
            paginate(&"[color=#123456]".repeat(40), &Metrics::builtin(), 64, 32)
                .unwrap_err()
                .contains("markup")
        );
    }

    #[test]
    fn dialogue_pages_respect_nonwrapping_controls_and_explicit_rows() {
        let metrics = Metrics::builtin();
        let wrapped = paginate_control("ABCD", &metrics, 16, 32, true).unwrap();
        assert_eq!(wrapped.len(), 1);
        let flat = paginate_control("ABCD", &metrics, 16, 32, false).unwrap();
        assert_eq!(
            flat.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            ["AB", "CD"]
        );
        let explicit = paginate_control("AB\nCD", &metrics, 16, 32, false).unwrap();
        assert_eq!(explicit[0].text, "AB\nCD");
        // The unchanged renderer draws every character of each emitted page.
        for page in flat.into_iter().chain(explicit) {
            let mut scene = crate::scene::Scene::default();
            scene.actors.clear();
            let mut canvas = crate::scene::Actor::cube("Canvas".into());
            canvas.canvas = Some(Default::default());
            scene.actors.push(canvas);
            let mut control = crate::scene::Actor::cube("Text".into());
            control.parent = Some(0);
            control.rect = Some(crate::hud::RectTransform {
                size: [16., 32.],
                ..Default::default()
            });
            control.text = Some(crate::hud::Text {
                text: page.text.clone(),
                wrap: false,
                ..Default::default()
            });
            scene.actors.push(control);
            let (_, stats) = crate::hud_native::compile(&scene);
            assert_eq!(stats[1], u32::from(page.glyphs));
            assert_eq!(stats[4], 0);
        }
    }
}
