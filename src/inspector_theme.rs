//! The property editor's visual language.
//!
//! The Inspector reads as one continuous sheet: sections introduced by an
//! edge-to-edge band and compact rows that pair a clipped label with its control.
//! Only the layout is the panel's own; every tone comes from the editor theme in
//! [`crate::gui::theme`], so the property editor never looks like a second
//! product bolted onto the window next to it.
//!
//! [`scope`] pushes the whole language for one window. While it is alive,
//! [`active`] is true, which is how shared widgets (`gui::heading`, `gui::field`)
//! know to draw their panel variant instead of their generic one.

use imgui::{StyleColor as C, StyleVar as V};
use std::cell::Cell;

/// The band a section header sits on, the tone `gui::theme` already gives a
/// collapsing header, so a section reads the same here as anywhere else.
pub const BAND: [f32; 4] = crate::gui::gray(43);
pub const BAND_HOVERED: [f32; 4] = crate::gui::gray(57);
pub const BAND_ACTIVE: [f32; 4] = crate::gui::gray(52);
/// The hairline that closes a band against the panel below it, the editor's own
/// title-bar tone.
pub const BAND_EDGE: [f32; 4] = crate::gui::gray(18);
const CLEAR: [f32; 4] = [0., 0., 0., 0.];

/// Panel margin. Controls end this far from the panel's right edge.
pub const PADDING: [f32; 2] = [5., 4.];
/// Properties sit inside their section; only bands reach the panel edges.
pub const INDENT: f32 = 14.;
/// Row metrics: a 17 px control with a 4 px gutter between rows.
const FRAME_PADDING: [f32; 2] = [4., 2.];
const ITEM_SPACING: [f32; 2] = [4., 4.];
/// Bands are taller than a row so a section reads before its properties do.
const BAND_PADDING: [f32; 2] = [4., 6.];

const VARS: [V; 15] = [
    V::WindowPadding(PADDING),
    V::FramePadding(FRAME_PADDING),
    V::ItemSpacing(ITEM_SPACING),
    V::ItemInnerSpacing([4., 2.]),
    V::CellPadding([4., 2.]),
    V::IndentSpacing(15.),
    V::FrameRounding(1.),
    V::FrameBorderSize(1.),
    V::ChildRounding(0.),
    V::ChildBorderSize(1.),
    V::PopupRounding(0.),
    V::GrabMinSize(10.),
    V::GrabRounding(1.),
    V::ScrollbarSize(11.),
    V::ScrollbarRounding(2.),
];

thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
}

/// True while a [`scope`] is on the style stack.
pub fn active() -> bool {
    ACTIVE.with(Cell::get)
}

/// The pushed style, released when the window's contents end.
pub struct Scope<'ui> {
    previous: bool,
    _font: Option<imgui::FontStackToken<'ui>>,
    _vars: [imgui::StyleStackToken<'ui>; VARS.len()],
}
impl Drop for Scope<'_> {
    fn drop(&mut self) {
        ACTIVE.with(|active| active.set(self.previous));
    }
}

/// Dress one window as the property editor. The colors stay the editor's own, so
/// only the panel's density and its proportional face are pushed here.
pub fn scope(ui: &imgui::Ui, font: Option<imgui::FontId>) -> Scope<'_> {
    let previous = ACTIVE.replace(true);
    Scope {
        previous,
        _font: font.map(|font| ui.push_font(font)),
        _vars: VARS.map(|var| ui.push_style_var(var)),
    }
}

/// Where a row's value column starts, measured from the row's left edge.
pub fn label_column(width: f32) -> f32 {
    (width * 0.40).clamp(88., 170.)
}

/// The identity strip at the top of the panel: one row of controls on the band
/// color, closed by the same hairline a section band uses.
pub fn strip<R>(ui: &imgui::Ui, contents: impl FnOnce() -> R) -> R {
    let left = ui.window_pos()[0];
    let right = left + ui.window_content_region_max()[0] + PADDING[0];
    let draw = ui.get_window_draw_list();
    let mut result = None;
    draw.channels_split(2, |channels| {
        channels.set_current(1);
        let top = ui.cursor_screen_pos()[1] - PADDING[1];
        result = Some(contents());
        let bottom = ui.item_rect_max()[1] + PADDING[1];
        channels.set_current(0);
        draw.add_rect([left, top], [right, bottom], BAND)
            .filled(true)
            .build();
        draw.add_line([left, bottom - 1.], [right, bottom - 1.], BAND_EDGE)
            .build();
    });
    result.expect("the strip's contents always run")
}

/// A section band: an edge-to-edge strip that opens and closes one group of
/// properties. The strip is painted behind the disclosure widget so it can reach
/// past the panel's padding without the label or the arrow moving with it.
pub fn band(ui: &imgui::Ui, text: &str) -> bool {
    let left = ui.window_pos()[0];
    let right = left + ui.window_content_region_max()[0] + PADDING[0];
    let draw = ui.get_window_draw_list();
    let mut open = false;
    draw.channels_split(2, |channels| {
        channels.set_current(1);
        let _flat = [
            ui.push_style_color(C::Header, CLEAR),
            ui.push_style_color(C::HeaderHovered, CLEAR),
            ui.push_style_color(C::HeaderActive, CLEAR),
        ];
        let _padding = ui.push_style_var(V::FramePadding(BAND_PADDING));
        ui.unindent_by(INDENT);
        open = ui.collapsing_header(
            text,
            imgui::TreeNodeFlags::DEFAULT_OPEN | imgui::TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        );
        let fill = if ui.is_item_active() {
            BAND_ACTIVE
        } else if ui.is_item_hovered() {
            BAND_HOVERED
        } else {
            BAND
        };
        let top = ui.item_rect_min()[1];
        let bottom = ui.item_rect_max()[1];
        ui.indent_by(INDENT);
        channels.set_current(0);
        draw.add_rect([left, top], [right, bottom], fill)
            .filled(true)
            .build();
        draw.add_line([left, bottom - 1.], [right, bottom - 1.], BAND_EDGE)
            .build();
    });
    open
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bands_are_drawn_in_the_editor_theme_grays() {
        assert_eq!(BAND, crate::gui::gray(43));
        assert_eq!(BAND_EDGE, crate::gui::gray(18));
        const { assert!(BAND_HOVERED[0] > BAND[0] && BAND_EDGE[0] < BAND[0]) };
    }

    #[test]
    fn the_value_column_keeps_room_for_both_sides() {
        assert_eq!(label_column(360.), 360. * 0.40);
        assert_eq!(label_column(120.), 88.);
        assert_eq!(label_column(900.), 170.);
    }

    #[test]
    fn the_panel_is_not_styled_outside_a_scope() {
        assert!(!active());
    }
}
