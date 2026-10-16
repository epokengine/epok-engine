//! PocketStation apps: the authored asset, its build into a PocketStation
//! executable and the emulated PocketStation the editor runs it on.
//!
//! A PocketStation app asset holds everything one program needs: its Pocket
//! Lua or assembly sources, sprites drawn in the editor, the monochrome menu
//! icon, the PlayStation memory card icon, the title and the memory card file
//! name. Building compiles the sources with Epok's own compiler and
//! assembler and lays out the executable file the PocketStation runs.
use crate::assets;
use epok_pocketstation::{
    self as ps, card,
    file::{self, ColorIcon, Header, MenuIcon},
    plua,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub const VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    PocketLua,
    Assembly,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Language::PocketLua => "Pocket Lua",
            Language::Assembly => "Assembly (ARM/Thumb)",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub name: String,
    pub text: String,
}

/// A monochrome image with frames; rows are strings of `#` (black) and `.`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteDoc {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuIconDoc {
    /// 32 rows of 32 `#`/`.` per frame.
    pub frames: Vec<Vec<String>>,
    /// 1/30 s ticks per frame.
    pub delay: u16,
}

/// The 16x16 icon the PlayStation memory card screen shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorIconDoc {
    /// 16 RGB colours; index 0 is transparent.
    pub palette: Vec<[u8; 3]>,
    /// 16 rows of 16 hexadecimal palette indices per frame (1-3 frames).
    pub frames: Vec<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PocketApp {
    pub version: u32,
    /// Shown by the PlayStation memory card screen (up to 32 characters).
    pub title: String,
    /// Memory card file name, up to 20 characters; the 7th is `P`.
    pub file_name: String,
    pub language: Language,
    pub files: Vec<SourceFile>,
    #[serde(default)]
    pub sprites: Vec<SpriteDoc>,
    pub menu_icon: MenuIconDoc,
    pub color_icon: ColorIconDoc,
}

fn blank_rows(width: usize, height: usize) -> Vec<String> {
    vec![".".repeat(width); height]
}

impl Default for MenuIconDoc {
    fn default() -> Self {
        Self {
            frames: vec![rows_from_text(DEFAULT_MENU_ICON, 32, 32)],
            delay: 8,
        }
    }
}

impl Default for ColorIconDoc {
    fn default() -> Self {
        let mut palette = vec![[0, 0, 0]; 16];
        palette[1] = [16, 20, 24];
        palette[2] = [150, 170, 140];
        palette[3] = [235, 200, 60];
        palette[4] = [250, 250, 250];
        Self {
            palette,
            frames: vec![DEFAULT_COLOR_ICON.lines().map(str::to_owned).collect()],
        }
    }
}

const DEFAULT_MENU_ICON: &str = "
................................
................................
....########################....
...#........................#...
...#.######################.#...
...#.#....................#.#...
...#.#....................#.#...
...#.#.......#....#.......#.#...
...#.#......###..###......#.#...
...#.#.......#....#.......#.#...
...#.#....................#.#...
...#.#.....#........#.....#.#...
...#.#......########......#.#...
...#.#....................#.#...
...#.######################.#...
...#........................#...
...#........................#...
...#....##...........##.....#...
...#...####.........#..#....#...
...#....##...........##.....#...
...#........##..............#...
...#.......####.............#...
...#........##..............#...
...#........................#...
....########################....
................................
................................
................................
................................
................................
................................
................................
";

const DEFAULT_COLOR_ICON: &str = "0000000000000000
0011111111111100
0122222222222210
0122222222222210
0122422224222210
0122222222222210
0122422224222210
0122244442222210
0122222222222210
0111111111111110
0111133111111110
0111333311111110
0111133111331110
0111111111331110
0011111111111100
0000000000000000";

/// Normalises a text picture into `height` rows of `width` `#`/`.` characters.
pub fn rows_from_text(text: &str, width: usize, height: usize) -> Vec<String> {
    let mut rows: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| {
            let mut row: String = l
                .chars()
                .take(width)
                .map(|c| if c == '#' { '#' } else { '.' })
                .collect();
            while row.len() < width {
                row.push('.');
            }
            row
        })
        .take(height)
        .collect();
    while rows.len() < height {
        rows.push(".".repeat(width));
    }
    rows
}

pub fn mono_image(rows: &[String]) -> file::MonoImage {
    let mut image = [0u32; 32];
    for (y, row) in rows.iter().take(32).enumerate() {
        for (x, c) in row.chars().take(32).enumerate() {
            if c == '#' {
                image[y] |= 1 << x;
            }
        }
    }
    image
}

impl SpriteDoc {
    pub fn new(name: &str, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            frames: vec![blank_rows(width as usize, height as usize)],
        }
    }
    /// Pads or crops every frame to the sprite's size.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.clamp(1, 32);
        self.height = height.clamp(1, 32);
        for frame in &mut self.frames {
            let text = frame.join("\n");
            *frame = rows_from_text(&text, self.width as usize, self.height as usize);
        }
    }
    pub fn compiled(&self) -> plua::Sprite {
        plua::Sprite {
            width: self.width,
            height: self.height,
            frames: self.frames.iter().map(|f| mono_image(f)).collect(),
        }
    }
}

impl PocketApp {
    pub fn template(kind: Template, title: &str) -> Self {
        let (language, files) = match kind {
            Template::StarCatcher => (
                Language::PocketLua,
                vec![SourceFile {
                    name: "main.plua".into(),
                    text: STAR_CATCHER.into(),
                }],
            ),
            Template::BlankLua => (
                Language::PocketLua,
                vec![SourceFile {
                    name: "main.plua".into(),
                    text: BLANK_LUA.into(),
                }],
            ),
            Template::Assembly => (
                Language::Assembly,
                vec![SourceFile {
                    name: "main.s".into(),
                    text: ASSEMBLY.into(),
                }],
            ),
        };
        let mut app = Self {
            version: VERSION,
            title: title.into(),
            file_name: file_name_for(title),
            language,
            files,
            sprites: Vec::new(),
            menu_icon: MenuIconDoc::default(),
            color_icon: ColorIconDoc::default(),
        };
        if kind == Template::BlankLua {
            let mut hero = SpriteDoc::new("hero", 8, 8);
            for (y, row) in [
                "..####..", ".######.", "##.##.##", "########", "########", ".##..##.", ".######.",
                "..#..#..",
            ]
            .iter()
            .enumerate()
            {
                hero.frames[0][y] = row.to_string();
            }
            app.sprites.push(hero);
        }
        app
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        crate::document::to_vec(self)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let app: PocketApp =
            crate::document::from_slice(bytes).map_err(|e| format!("PocketStation app: {e}"))?;
        app.validate()?;
        Ok(app)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION {
            return Err(format!(
                "Unsupported PocketStation app version {}",
                self.version
            ));
        }
        validate_file_name(&self.file_name)?;
        if self.files.is_empty() {
            return Err("A PocketStation app needs at least one source file".into());
        }
        if self.color_icon.palette.len() != 16
            || self.color_icon.frames.is_empty()
            || self.color_icon.frames.len() > 3
        {
            return Err("The PlayStation icon needs 16 colours and 1-3 frames".into());
        }
        if self.menu_icon.frames.is_empty() {
            return Err("The menu icon needs at least one frame".into());
        }
        for s in &self.sprites {
            if s.name.is_empty()
                || !s
                    .name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return Err(format!(
                    "Sprite name `{}` must use letters, digits and _",
                    s.name
                ));
            }
            if !(1..=32).contains(&s.width) || !(1..=32).contains(&s.height) || s.frames.is_empty()
            {
                return Err(format!(
                    "Sprite `{}` must be 1-32 pixels per side with a frame",
                    s.name
                ));
            }
        }
        Ok(())
    }

    pub fn header(&self) -> Header {
        Header {
            title: self.title.clone(),
            color_icon: self.color_icon(),
            viewer_icons: Vec::new(),
            menu_icons: vec![MenuIcon {
                frames: self
                    .menu_icon
                    .frames
                    .iter()
                    .map(|f| mono_image(f))
                    .collect(),
                delay: self.menu_icon.delay.max(1),
            }],
            function_count: 0,
            snapshot: false,
        }
    }

    pub fn color_icon(&self) -> ColorIcon {
        let mut palette = [0u16; 16];
        for (i, [r, g, b]) in self.color_icon.palette.iter().enumerate().take(16) {
            let c = (*r as u16 >> 3) | ((*g as u16 >> 3) << 5) | ((*b as u16 >> 3) << 10);
            // Index 0 stays transparent; black elsewhere needs the mask bit.
            palette[i] = if i == 0 {
                0
            } else if c == 0 {
                0x8000
            } else {
                c
            };
        }
        let frames = self
            .color_icon
            .frames
            .iter()
            .take(3)
            .map(|rows| {
                let mut pixels = [0u8; 128];
                for (y, row) in rows.iter().take(16).enumerate() {
                    for (x, c) in row.chars().take(16).enumerate() {
                        let v = c.to_digit(16).unwrap_or(0) as u8;
                        let at = y * 8 + x / 2;
                        if x % 2 == 0 {
                            pixels[at] |= v;
                        } else {
                            pixels[at] |= v << 4;
                        }
                    }
                }
                pixels
            })
            .collect();
        ColorIcon { palette, frames }
    }

    pub fn sprite_library(&self) -> BTreeMap<String, plua::Sprite> {
        self.sprites
            .iter()
            .map(|s| (s.name.clone(), s.compiled()))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    StarCatcher,
    BlankLua,
    Assembly,
}

impl Template {
    pub const ALL: [Template; 3] = [
        Template::StarCatcher,
        Template::BlankLua,
        Template::Assembly,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Template::StarCatcher => "Star Catcher (Pocket Lua game)",
            Template::BlankLua => "Empty Pocket Lua app",
            Template::Assembly => "Assembly (ARM/Thumb) app",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Template::StarCatcher => {
                "A complete mini-game: sprites, tunes, save data and the PlayStation link."
            }
            Template::BlankLua => "init, update and draw with a sprite drawn in the editor.",
            Template::Assembly => {
                "A Thumb program that draws on the screen, for full control of the ARM CPU."
            }
        }
    }
}

pub const STAR_CATCHER: &str =
    include_str!("../crates/pocketstation/src/plua/samples/star_catcher.plua");

const BLANK_LUA: &str = r#"-- A PocketStation app in Pocket Lua.
-- init() runs once; update() and draw() run 30 times per second.
-- Sprites drawn in the Sprites tab are available as sprite "name".

local hero <const> = sprite "hero"
local x = 12
local y = 12

function init()
end

function update()
    if button.down(button.LEFT) then x = x - 1 end
    if button.down(button.RIGHT) then x = x + 1 end
    if button.down(button.UP) then y = y - 1 end
    if button.down(button.DOWN) then y = y + 1 end
    if button.pressed(button.FIRE) then
        sound.tone(880, 3)
    end
end

function draw()
    screen.clear()
    screen.sprite(hero, x, y)
    screen.text("HELLO", 6, 0)
end
"#;

const ASSEMBLY: &str = r#"; A PocketStation program in Thumb assembly.
; The kernel starts `main` in user mode with r0 = start parameter.
; Screen: 32 words at 0D000100h, one row each, bit 0 = left pixel.

    .thumb
    .thumb_func
main:
    mov r0, #1
    swi 0x11                ; let the PlayStation talk to us
    ldr r4, =0x0D000100     ; LCD rows
    mov r5, #0              ; frame counter
loop:
    ldr r0, =0x0A000004     ; buttons (bit 0 = Fire)
    ldr r0, [r0]
    mov r1, #1
    tst r0, r1
    beq draw
    add r5, #1              ; Fire scrolls the pattern
draw:
    mov r2, #0
row:
    add r3, r2, r5
    mov r1, #0x55
    lsl r1, r3              ; a moving checker pattern
    lsl r0, r2, #2
    str r1, [r4, r0]
    add r2, #1
    cmp r2, #32
    blt row
    ldr r0, =20000
wait:
    sub r0, #1
    bne wait
    b loop
    .pool
"#;

pub fn file_name_for(title: &str) -> String {
    let mut stem: String = title
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    stem.truncate(8);
    if stem.is_empty() {
        stem = "APP".into();
    }
    format!("BEEPOKP{:05}{stem}", fastrand_like(title) % 100_000)
}

fn fastrand_like(seed: &str) -> u32 {
    seed.bytes()
        .fold(2166136261u32, |h, b| (h ^ b as u32).wrapping_mul(16777619))
}

pub fn validate_file_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 20 {
        return Err("The memory card file name needs 1-20 characters".into());
    }
    if !name
        .bytes()
        .all(|b| (0x21..0x7F).contains(&b) && b != b'*' && b != b'?')
    {
        return Err(
            "The memory card file name uses letters, digits and punctuation without spaces".into(),
        );
    }
    Ok(())
}

// ---- asset storage ---------------------------------------------------------------

pub fn fresh_path(name: &str) -> String {
    let stem: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let stem = if stem.is_empty() {
        "PocketApp".into()
    } else {
        stem
    };
    format!(
        "assets/PocketStation/{stem}-{}.epokasset",
        &Uuid::new_v4().simple().to_string()[..8]
    )
}

pub fn create(root: &Path, path: &str, app: &PocketApp) -> Result<Uuid, String> {
    app.validate()?;
    let id = Uuid::new_v4();
    let source = app.encode()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::PocketStationApp,
            importer_version: 1,
            source: path.into(),
            source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::Authored,
            extra: Default::default(),
        },
        source,
    };
    assets::atomic_write(&assets::inside(root, path)?, &package.bytes()?, None)?;
    Ok(id)
}

pub fn document(record: &assets::Record) -> Result<PocketApp, String> {
    if record.meta.kind != assets::Kind::PocketStationApp {
        return Err("Select a PocketStation app asset".into());
    }
    PocketApp::parse(&assets::Package::load(&record.path)?.source)
}

/// Saves an edited app over its asset. Returns the new revision.
pub fn save(record: &assets::Record, app: &PocketApp) -> Result<String, String> {
    app.validate()?;
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.kind != assets::Kind::PocketStationApp || package.meta.id != record.meta.id {
        return Err("The PocketStation app asset was replaced; reload before editing".into());
    }
    package.source = app.encode()?;
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, Some(&record.revision))?;
    Ok(assets::hash(&bytes))
}

// ---- build -----------------------------------------------------------------------

/// A successful build: the executable file and what the debugger needs.
#[derive(Clone, Debug)]
pub struct Built {
    pub bytes: Vec<u8>,
    pub program: plua::Program,
    pub blocks: usize,
    pub warnings: Vec<plua::Diagnostic>,
}

pub fn build(app: &PocketApp) -> Result<Built, Vec<plua::Diagnostic>> {
    let mut header = app.header();
    let program = match app.language {
        Language::PocketLua => {
            let main = &app.files[0];
            let assembly: Vec<(String, String)> = app
                .files
                .iter()
                .skip(1)
                .filter(|f| f.name.ends_with(".s") || f.name.ends_with(".asm"))
                .map(|f| (f.name.clone(), f.text.clone()))
                .collect();
            let options = plua::Options {
                file: main.name.clone(),
                sprites: app.sprite_library(),
                assembly,
            };
            plua::compile(&main.text, &mut header, &options)?
        }
        Language::Assembly => {
            let sources: Vec<(String, String)> = app
                .files
                .iter()
                .map(|f| (f.name.clone(), f.text.clone()))
                .collect();
            plua::assemble_program(&sources, &mut header)?
        }
    };
    let bytes = plua::executable(&header, &program).map_err(|message| {
        vec![plua::Diagnostic {
            file: app.files[0].name.clone(),
            line: 1,
            column: 1,
            message,
        }]
    })?;
    Ok(Built {
        blocks: bytes.len() / card::BLOCK_SIZE,
        warnings: program.warnings.clone(),
        bytes,
        program,
    })
}

/// A single-save file (`.mcs`): the directory frame followed by the file.
pub fn single_save(file_name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut image = card::formatted();
    card::write(&mut image, file_name, bytes).map_err(|e| e.to_string())?;
    let entry = card::find(&image, file_name).ok_or("file not written")?;
    Ok(card::export_single(&image, &entry))
}

// ---- the emulated PocketStation ------------------------------------------------------

/// Where the editor keeps the emulated PocketStation's flash (its memory card).
pub fn card_path(root: &Path) -> PathBuf {
    root.join("UserSettings/PocketStation.mcd")
}

pub fn load_card(root: &Path) -> Vec<u8> {
    match std::fs::read(card_path(root)) {
        Ok(bytes) if bytes.len() == card::CARD_SIZE && card::is_formatted(&bytes) => bytes,
        _ => card::formatted(),
    }
}

pub fn save_card(root: &Path, flash: &[u8]) -> Result<(), String> {
    let path = card_path(root);
    let parent = path.parent().ok_or("no folder for the card")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    // Write a sibling file, then replace the card in one step.
    let temporary = parent.join(".PocketStation.mcd.tmp");
    std::fs::write(&temporary, flash).map_err(|e| e.to_string())?;
    std::fs::rename(&temporary, &path).map_err(|e| e.to_string())
}

pub fn now() -> ps::DateTime {
    use chrono::{Datelike, Timelike};
    let t = chrono::Local::now();
    ps::DateTime {
        year: t.year().clamp(1900, 2099) as u16,
        month: t.month() as u8,
        day: t.day() as u8,
        hour: t.hour() as u8,
        minute: t.minute() as u8,
        second: t.second() as u8,
        weekday: t.weekday().num_days_from_sunday() as u8 + 1,
    }
}

/// Classic monochrome LCD colours (paper, ink).
pub const LCD_PAPER: [u8; 3] = [166, 178, 146];
pub const LCD_INK: [u8; 3] = [28, 34, 26];

/// RGBA pixels of a 32x32 screen picture.
pub fn screen_rgba(rows: &[u32; 32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 * 32 * 4);
    for row in rows {
        for x in 0..32 {
            let c = if row & (1 << x) != 0 {
                LCD_INK
            } else {
                LCD_PAPER
            };
            out.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_build_and_round_trip() {
        for template in Template::ALL {
            let app = PocketApp::template(template, "Test App");
            let encoded = app.encode().unwrap();
            assert_eq!(PocketApp::parse(&encoded).unwrap(), app);
            let built = build(&app).unwrap_or_else(|e| panic!("{template:?}: {e:?}"));
            assert!(built.blocks >= 1);
            assert!(file::parse(&built.bytes).is_some());
        }
    }

    #[test]
    fn sources_stay_readable_in_the_document() {
        let app = PocketApp::template(Template::BlankLua, "Hello");
        let text = String::from_utf8(app.encode().unwrap()).unwrap();
        assert!(text.contains("function update()"), "{text}");
        assert!(text.contains("..####.."));
    }

    #[test]
    fn apps_are_created_saved_and_reloaded_as_assets() {
        let root = std::env::temp_dir().join(format!("epok-pocketstation-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = fresh_path("Saved App");
        let id = create(
            &root,
            &path,
            &PocketApp::template(Template::BlankLua, "Saved App"),
        )
        .unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap().clone();
        let mut app = document(&record).unwrap();
        app.files[0].text.push_str("\n-- edited\n");
        app.sprites[0].resize(12, 4);
        save(&record, &app).unwrap();
        // A second save needs the new revision, as editors get after a rescan.
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap().clone();
        let reloaded = document(&record).unwrap();
        assert!(reloaded.files[0].text.ends_with("-- edited\n"));
        assert_eq!(
            (reloaded.sprites[0].width, reloaded.sprites[0].height),
            (12, 4)
        );
        save(&record, &reloaded).unwrap();
        save_card(&root, &card::formatted()).unwrap();
        save_card(&root, &card::formatted()).unwrap();
        assert_eq!(load_card(&root).len(), card::CARD_SIZE);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn built_apps_run_on_the_emulated_pocketstation() {
        let app = PocketApp::template(Template::BlankLua, "Hello");
        let built = build(&app).unwrap();
        let mut flash = card::formatted();
        let block = card::write(&mut flash, &app.file_name, &built.bytes).unwrap();
        let mut machine = ps::Machine::new(flash);
        machine.launch(block, 0).unwrap();
        machine.run(0.5);
        assert!(matches!(machine.state, ps::State::Program(_)));
        assert!(machine.screen().iter().any(|r| *r != 0));
    }
}
