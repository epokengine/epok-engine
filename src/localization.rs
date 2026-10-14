//! String tables (World2D initiative, phase 3): the project's translated texts. A
//! StringTable asset holds languages (columns) and keys (rows); a HUD Text can show a key
//! instead of its own text. Builds cook every language into the game, start in the first
//! one and let gameplay switch (`LocalizationLibrary::set_language`); the editor previews
//! any language. Tables import from and export to CSV, so translators can work in a
//! spreadsheet.
use crate::assets;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path, sync::Arc};
use uuid::Uuid;

pub const VERSION: u32 = 1;
pub const MAX_KEYS: usize = 4096;
pub const MAX_LANGUAGES: usize = 16;
/// A HUD text holds 511 UTF-8 bytes.
pub const MAX_BYTES: usize = 511;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Entry {
    pub key: String,
    /// One string per language, in `StringTable::languages` order.
    pub values: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct StringTable {
    pub version: u32,
    pub languages: Vec<String>,
    pub entries: Vec<Entry>,
}
impl Default for StringTable {
    fn default() -> Self {
        Self {
            version: VERSION,
            languages: vec!["English".into()],
            entries: Vec::new(),
        }
    }
}
impl StringTable {
    pub fn key_index(&self, key: &str) -> Option<usize> {
        self.entries.iter().position(|e| e.key == key)
    }
    /// The text of `key` in `language`, falling back to the first language when that
    /// entry is empty, as the console does.
    pub fn lookup(&self, key: &str, language: usize) -> Option<&str> {
        let entry = self.entries.iter().find(|e| e.key == key)?;
        let value = entry.values.get(language).map(String::as_str).unwrap_or("");
        Some(if value.is_empty() {
            entry.values.first().map(String::as_str).unwrap_or("")
        } else {
            value
        })
    }
    /// Keeps every entry's values aligned with the languages.
    pub fn normalize(&mut self) {
        let count = self.languages.len();
        for entry in &mut self.entries {
            entry.values.resize(count, String::new());
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version > VERSION {
            return Err("This string table is newer than the editor".into());
        }
        if self.languages.is_empty() || self.languages.len() > MAX_LANGUAGES {
            return Err(format!(
                "A string table needs between 1 and {MAX_LANGUAGES} languages"
            ));
        }
        if self.entries.len() > MAX_KEYS {
            return Err(format!("A string table holds at most {MAX_KEYS} keys"));
        }
        let mut keys = BTreeSet::new();
        for entry in &self.entries {
            if entry.key.trim().is_empty() {
                return Err("Every string needs a key".into());
            }
            if !keys.insert(entry.key.as_str()) {
                return Err(format!("The key {} is used twice", entry.key));
            }
            if let Some(value) = entry.values.iter().find(|v| v.len() > MAX_BYTES) {
                return Err(format!(
                    "{}: a text holds at most {MAX_BYTES} bytes ({} given)",
                    entry.key,
                    value.len()
                ));
            }
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(self).map_err(|e| e.to_string())
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let mut table: StringTable =
            serde_json::from_slice(bytes).map_err(|e| format!("String table: {e}"))?;
        table.normalize();
        table.validate()?;
        Ok(table)
    }
    /// CSV with a header row `key,<language>,...` and one row per key.
    pub fn to_csv(&self) -> String {
        let quote = |value: &str| {
            if value.contains([',', '"', '\n', '\r']) {
                format!("\"{}\"", value.replace('"', "\"\""))
            } else {
                value.to_owned()
            }
        };
        let mut out = std::iter::once("key".to_string())
            .chain(self.languages.iter().map(|l| quote(l)))
            .collect::<Vec<_>>()
            .join(",");
        out.push('\n');
        for entry in &self.entries {
            let row = std::iter::once(quote(&entry.key))
                .chain(entry.values.iter().map(|v| quote(v)))
                .collect::<Vec<_>>()
                .join(",");
            out.push_str(&row);
            out.push('\n');
        }
        out
    }
    pub fn from_csv(text: &str) -> Result<Self, String> {
        let rows = parse_csv(text);
        let mut rows = rows.into_iter().filter(|r| r.iter().any(|c| !c.is_empty()));
        let header = rows.next().ok_or("The CSV is empty")?;
        if header.len() < 2 {
            return Err("The first row must be: key, then one column per language".into());
        }
        let mut table = StringTable {
            languages: header[1..].iter().map(|l| l.trim().to_owned()).collect(),
            entries: Vec::new(),
            ..Default::default()
        };
        for row in rows {
            let key = row.first().map(|k| k.trim().to_owned()).unwrap_or_default();
            if key.is_empty() {
                continue;
            }
            table.entries.push(Entry {
                key,
                values: row.into_iter().skip(1).collect(),
            });
        }
        table.normalize();
        table.validate()?;
        Ok(table)
    }
}
/// RFC 4180 style: quoted fields may hold commas, quotes ("") and line breaks.
fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.trim_start_matches('\u{feff}').chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' if field.is_empty() => quoted = true,
            ',' => row.push(std::mem::take(&mut field)),
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            _ => field.push(c),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

// ---- asset storage -------------------------------------------------------------------

pub fn create(root: &Path, path: &str, table: &StringTable) -> Result<Uuid, String> {
    table.validate()?;
    let id = Uuid::new_v4();
    let source = table.encode()?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::StringTable,
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
pub fn document(record: &assets::Record) -> Result<StringTable, String> {
    if record.meta.kind != assets::Kind::StringTable {
        return Err("Select a String Table asset".into());
    }
    StringTable::parse(&assets::Package::load(&record.path)?.source)
}
pub fn save(record: &assets::Record, table: &StringTable) -> Result<(), String> {
    table.validate()?;
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.kind != assets::Kind::StringTable || package.meta.id != record.meta.id {
        return Err("The string table asset was replaced; reload before editing".into());
    }
    package.source = table.encode()?;
    package.meta.source_hash = assets::hash(&package.source);
    assets::atomic_write(&record.path, &package.bytes()?, None)
}
/// The project's string table: the first StringTable asset by path.
pub fn project_table(index: &assets::Index) -> Option<(assets::Record, StringTable)> {
    let mut records: Vec<_> = index
        .usable()
        .filter(|r| r.meta.kind == assets::Kind::StringTable)
        .cloned()
        .collect();
    records.sort_by(|a, b| a.meta.source.cmp(&b.meta.source));
    let record = records.into_iter().next()?;
    let table = document(&record).ok()?;
    Some((record, table))
}
/// Loads the project's string table into the scene when any of its texts shows a key.
pub fn resolve(scene: &mut crate::scene::Scene, index: &assets::Index) {
    let keyed = scene
        .actors
        .iter()
        .any(|a| a.text.as_ref().is_some_and(|t| !t.key.is_empty()));
    scene.strings = if keyed {
        project_table(index).map(|(_, table)| Arc::new(table))
    } else {
        None
    };
}

// ---- cooking ---------------------------------------------------------------------------

fn c_string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .bytes()
            .take(MAX_BYTES)
            .map(|b| format!("\\{b:03o}"))
            .collect::<String>()
    )
}
/// The bank's copy of the table: all languages, language-major. It is declared in the
/// bank's namespace, so banks never clash and a bank's strings leave with the bank.
pub fn header(table: &StringTable) -> String {
    let strings = table
        .languages
        .iter()
        .enumerate()
        .flat_map(|(language, _)| {
            table
                .entries
                .iter()
                .map(move |e| c_string(e.values.get(language).map_or("", String::as_str)))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "inline constexpr const char* const localization_strings[]={{{strings}}};\ninline constexpr uint16_t localization_keys={},localization_languages={};\n",
        table.entries.len(),
        table.languages.len()
    )
}
/// Statements at the start of `initialize_components`. A bank with keyed texts installs
/// its table and the function that rewrites them; any other bank clears both, so nothing
/// points into a bank that is gone. The chosen language survives either way.
pub fn install(table: Option<&StringTable>) -> &'static str {
    if table.is_some() {
        "localization.strings=localization_strings;localization.keys=localization_keys;localization.languages=localization_languages;\nlocalization.apply=[]{for(size_t i=0;i<object_count;++i)if(objects[i].text.key>=0)objects[i].text.set_text(localization.get(objects[i].text.key));};\n"
    } else {
        "localization.strings=nullptr;localization.keys=localization.languages=0;localization.apply=nullptr;\n"
    }
}
/// Statements that rewrite the scene's keyed texts at the end of `initialize_components`
/// when the game is not in its first language.
pub fn apply(table: Option<&StringTable>) -> &'static str {
    if table.is_some() {
        "if(localization.language&&localization.apply)localization.apply();\n"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> StringTable {
        StringTable {
            languages: vec!["English".into(), "Español".into()],
            entries: vec![
                Entry {
                    key: "menu.play".into(),
                    values: vec!["Play".into(), "Jugar".into()],
                },
                Entry {
                    key: "menu.quote".into(),
                    values: vec!["Say \"hi\", then go".into(), String::new()],
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn lookups_fall_back_to_the_first_language() {
        let table = sample();
        assert_eq!(table.lookup("menu.play", 1), Some("Jugar"));
        assert_eq!(table.lookup("menu.quote", 1), Some("Say \"hi\", then go"));
        assert_eq!(table.lookup("missing", 0), None);
        assert_eq!(table.key_index("menu.quote"), Some(1));
    }

    #[test]
    fn tables_round_trip_through_csv_and_json() {
        let table = sample();
        let csv = table.to_csv();
        assert!(csv.starts_with("key,English,Español\n"));
        assert!(csv.contains("\"Say \"\"hi\"\", then go\""));
        assert_eq!(StringTable::from_csv(&csv).unwrap(), table);
        assert_eq!(StringTable::parse(&table.encode().unwrap()).unwrap(), table);
        // A spreadsheet export with a BOM, CRLF lines and a short row still loads.
        let loose = "\u{feff}key,English,Español\r\nhello,Hello\r\n";
        let parsed = StringTable::from_csv(loose).unwrap();
        assert_eq!(
            parsed.entries[0].values,
            vec!["Hello".to_string(), String::new()]
        );
    }

    #[test]
    fn duplicate_keys_and_long_texts_are_rejected() {
        let mut table = sample();
        table.entries[1].key = "menu.play".into();
        assert!(table.validate().is_err());
        let mut table = sample();
        table.entries[0].values[0] = "x".repeat(MAX_BYTES + 1);
        assert!(table.validate().is_err());
        assert!(StringTable::from_csv("key\n").is_err());
    }

    #[test]
    fn the_cooked_table_is_language_major() {
        let text = header(&sample());
        assert!(text.contains("localization_keys=2,localization_languages=2;"));
        // English play, English quote, then Spanish play.
        let play = c_string("Play");
        let jugar = c_string("Jugar");
        assert!(text.find(&play).unwrap() < text.find(&jugar).unwrap());
        // Declared where the bank puts it: no namespace of its own.
        assert!(!text.contains("namespace"));
        assert!(install(Some(&sample())).contains("localization.strings=localization_strings;"));
        assert!(install(None).contains("localization.strings=nullptr;"));
        assert!(apply(None).is_empty());
    }
}
