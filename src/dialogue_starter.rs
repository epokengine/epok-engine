//! One-time, project-owned widget defaults. The runtime never dispatches a
//! visual preset: all rectangles, textures and fonts live on ordinary controls.
use crate::dialogue_asset::Presentation;
use std::path::Path;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Appearance {
    pub text_color: [f32; 3],
    pub panel_color: [f32; 3],
    pub panel_borders: [u16; 4],
}
#[derive(Clone, Debug)]
pub struct Layout {
    pub source: Appearance,
    pub panel: [u16; 4],
    pub choice_panel: [u16; 4],
    pub body: [u16; 4],
    pub choices: [u16; 4],
    pub name: [u16; 4],
    pub portrait: [u16; 4],
}
#[derive(Clone, Debug)]
pub struct WidgetDefaults {
    pub layout: Layout,
    pub prompt: [u16; 4],
    pub continuation: [u16; 4],
}

pub fn widget_preset(
    presentation: Presentation,
    display: [u16; 2],
) -> Result<WidgetDefaults, String> {
    let [width, height] = display;
    if width < 256 || height < 224 {
        return Err("Dialogue starters need a viewport of at least 256 x 224".into());
    }
    let bubble = presentation == Presentation::SpeechBubble;
    let subtitle = presentation == Presentation::Subtitles;
    let portrait = presentation == Presentation::PortraitConversation;
    let panel = if bubble {
        [(width - 224) / 2, 16, 224, 104]
    } else if subtitle {
        [0, 0, width, 56]
    } else {
        [8, 8, width - 16, 96]
    };
    let popup_width = if bubble { 224 } else { 240 };
    let popup_x = if bubble || subtitle {
        (width - popup_width) / 2
    } else {
        width - 8 - popup_width
    };
    let popup_y = panel[1] + panel[3] + 8;
    let choice_panel = [popup_x, popup_y, popup_width, 80];
    let inset = if portrait { 88 } else { 12 };
    let body = if subtitle {
        [16, 12, width - 32, 32]
    } else {
        [panel[0] + inset, panel[1] + 20, panel[2] - inset - 12, 40]
    };
    let name = [
        panel[0] + inset,
        panel[1] + panel[3] - 28,
        panel[2] - inset - 12,
        16,
    ];
    let portrait_rect = [panel[0] + 12, panel[1] + 16, 64, 64];
    let choices = [
        choice_panel[0] + 12,
        choice_panel[1] + 8,
        choice_panel[2] - 24,
        64,
    ];
    let continuation = [panel[0] + panel[2] - 20, panel[1] + 8, 8, 16];
    Ok(WidgetDefaults {
        layout: Layout {
            source: Appearance {
                text_color: if bubble { [0.08, 0.08, 0.1] } else { [1.; 3] },
                panel_color: if subtitle { [0.; 3] } else { [1.; 3] },
                panel_borders: if bubble || subtitle { [0; 4] } else { [4; 4] },
            },
            panel,
            choice_panel,
            body,
            choices,
            name,
            portrait: portrait_rect,
        },
        prompt: body,
        continuation,
    })
}

pub struct Skin {
    pub panel: Option<Uuid>,
    pub choices: Uuid,
}
#[cfg(test)]
pub fn skin(root: &Path, presentation: Presentation) -> Result<Skin, String> {
    let mut transaction = crate::helper_install::Transaction::new(root);
    let skin = skin_in(root, presentation, &mut transaction)?;
    transaction.commit();
    Ok(skin)
}
pub fn skin_in(
    root: &Path,
    presentation: Presentation,
    transaction: &mut crate::helper_install::Transaction,
) -> Result<Skin, String> {
    let mut framed = |name: &str, gold: bool| {
        install(root, transaction, name, 16, 16, |x, y| {
            if !(2..14).contains(&x) && !(2..14).contains(&y) {
                return [0; 4];
            }
            let edge = x < 2 || y < 2 || x >= 14 || y >= 14;
            if edge {
                if gold {
                    [224, 192, 112, 255]
                } else {
                    [192, 208, 232, 255]
                }
            } else if x == 2 || y == 2 || x == 13 || y == 13 {
                [48, 56, 80, 255]
            } else {
                [12, 16, 32, 255]
            }
        })
    };
    let choices = framed("dialogue-frame", false)?;
    let panel = match presentation {
        Presentation::Subtitles => None,
        Presentation::ClassicBox => Some(choices),
        Presentation::PortraitConversation => Some(framed("portrait-frame", true)?),
        Presentation::SpeechBubble => Some(install(
            root,
            transaction,
            "speech-bubble",
            224,
            104,
            |x, y| {
                fn inside(x: i32, y: i32) -> bool {
                    if x < 0 || y < 0 || x >= 224 || y >= 104 {
                        return false;
                    }
                    let body = if y < 96 {
                        let cx = x.clamp(8, 215);
                        let cy = y.clamp(8, 87);
                        (x - cx) * (x - cx) + (y - cy) * (y - cy) <= 64
                    } else {
                        false
                    };
                    let tail = (94..=103).contains(&y) && (x - 112).abs() <= ((103 - y) * 8 / 9);
                    body || tail
                }
                let x = i32::from(x);
                let y = i32::from(y);
                if !inside(x, y) {
                    [0; 4]
                } else if [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                    .into_iter()
                    .any(|(x, y)| !inside(x, y))
                {
                    [48, 52, 64, 255]
                } else {
                    [240, 240, 228, 255]
                }
            },
        )?),
    };
    Ok(Skin { panel, choices })
}
fn install(
    root: &Path,
    transaction: &mut crate::helper_install::Transaction,
    name: &str,
    width: u16,
    height: u16,
    pixel: impl Fn(u16, u16) -> [u8; 4],
) -> Result<Uuid, String> {
    let asset = format!("assets/GameHelpers/UI/{name}.epokasset");
    let destination = crate::assets::inside(root, &asset)?;
    if destination.exists() {
        let package = crate::assets::Package::load(&destination)?;
        if package.meta.kind != crate::assets::Kind::Texture {
            return Err(format!(
                "{asset} must be a Texture; preserve it and choose another helper asset path"
            ));
        }
        crate::texture::decode(&package.source)?;
        return Ok(package.meta.id);
    }
    let source = format!("assets/GameHelpers/UI/{name}.png");
    let mut bytes = vec![];
    let mut encoder = png::Encoder::new(&mut bytes, u32::from(width), u32::from(height));
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut pixels = Vec::with_capacity(usize::from(width) * usize::from(height) * 4);
    for y in 0..height {
        for x in 0..width {
            pixels.extend(pixel(x, y));
        }
    }
    encoder
        .write_header()
        .map_err(|e| e.to_string())?
        .write_image_data(&pixels)
        .map_err(|e| e.to_string())?;
    transaction.create(&source, &bytes)?;
    transaction.asset(
        crate::texture::prepare(root, &source, &asset, None, false)?,
        "epok.game_helpers.ui",
        1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn helper_skin_failed_install_removes_owned_files_and_retains_conflict() {
        let root = crate::workspace::tests::temp("helper-skin-transaction");
        let ui = root.join("assets/GameHelpers/UI");
        std::fs::create_dir_all(&ui).unwrap();
        let conflicting = ui.join("portrait-frame.epokasset");
        std::fs::write(&conflicting, b"existing user file").unwrap();
        assert!(skin(&root, Presentation::PortraitConversation).is_err());
        assert_eq!(std::fs::read(&conflicting).unwrap(), b"existing user file");
        assert!(!ui.join("dialogue-frame.png").exists());
        assert!(!ui.join("dialogue-frame.epokasset").exists());
        let first = skin(&root, Presentation::ClassicBox).unwrap();
        let package = crate::assets::Package::load(&ui.join("dialogue-frame.epokasset")).unwrap();
        assert_eq!(package.meta.id, first.choices);
        assert_eq!(
            package.meta.extra["game_helper_package"],
            serde_json::json!({"id":"epok.game_helpers.ui","version":1})
        );
        let bytes = package.bytes().unwrap();
        assert!(skin(&root, Presentation::PortraitConversation).is_err());
        assert_eq!(
            std::fs::read(ui.join("dialogue-frame.epokasset")).unwrap(),
            bytes
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn dialogue_starter_textures_are_project_assets_and_preserve_user_edits() {
        let root = crate::workspace::tests::temp("dialogue-starter-skin");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let classic = skin(&root, Presentation::ClassicBox).unwrap();
        let path = root.join("assets/GameHelpers/UI/dialogue-frame.epokasset");
        let mut package = crate::assets::Package::load(&path).unwrap();
        assert_eq!(package.meta.id, classic.choices);
        package
            .meta
            .extra
            .insert("project_note".into(), serde_json::json!("User-owned theme"));
        let authored = package.bytes().unwrap();
        std::fs::write(&path, &authored).unwrap();
        for preset in Presentation::ALL {
            assert_eq!(skin(&root, preset).unwrap().choices, classic.choices);
            assert_eq!(std::fs::read(&path).unwrap(), authored);
        }
        package.meta.kind = crate::assets::Kind::DialogueStyle;
        package.meta.settings = crate::import_settings::Settings::Authored;
        package.source =
            crate::document::to_vec(&crate::dialogue_asset::DialogueStyle::default()).unwrap();
        package.meta.source_hash = crate::assets::hash(&package.source);
        let conflicting = package.bytes().unwrap();
        std::fs::write(&path, &conflicting).unwrap();
        assert!(
            skin(&root, Presentation::ClassicBox)
                .err()
                .unwrap()
                .contains("must be a Texture")
        );
        assert_eq!(std::fs::read(&path).unwrap(), conflicting);
        std::fs::remove_dir_all(root).unwrap();
    }
}
