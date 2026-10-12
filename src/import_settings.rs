//! Versioned importer options. Legacy audio packages remain readable.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub enum Settings {
    Texture,
    Audio(crate::audio_import::Settings),
    MusicSequence(crate::sequence::Settings),
    SoundBank(crate::sound_bank::Settings),
    Authored,
    Fbx(crate::model_import::Settings),
    Font(FontSettings),
    Derived,
}
/// Named blocks an authored font can cover, beyond its explicit character list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharRange {
    Ascii,
    Latin1Supplement,
    SpanishExtras,
    Digits,
}
impl CharRange {
    pub const ALL: [Self; 4] = [
        Self::Ascii,
        Self::Latin1Supplement,
        Self::SpanishExtras,
        Self::Digits,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Ascii => "ASCII (32-126)",
            Self::Latin1Supplement => "Latin-1 supplement",
            Self::SpanishExtras => "Spanish accents",
            Self::Digits => "Digits only",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FontSettings {
    pub version: u32,
    pub pixel_height: u32,
    pub characters: String,
    pub ranges: Vec<CharRange>,
    pub antialias: bool,
    pub padding: u8,
    pub monospace: bool,
}
impl Default for FontSettings {
    fn default() -> Self {
        Self {
            version: 1,
            pixel_height: 16,
            characters: String::new(),
            ranges: vec![CharRange::Ascii, CharRange::SpanishExtras],
            antialias: false,
            padding: 1,
            monospace: false,
        }
    }
}
impl FontSettings {
    /// Every codepoint the atlas must carry, deduplicated and ordered.
    pub fn charset(&self) -> std::collections::BTreeSet<char> {
        let mut set = std::collections::BTreeSet::new();
        for range in &self.ranges {
            match range {
                CharRange::Ascii => set.extend((0x20u8..=0x7e).map(char::from)),
                CharRange::Latin1Supplement => set.extend((0xa0u8..=0xff).map(char::from)),
                CharRange::SpanishExtras => set.extend(crate::bitmap_font::EXTRA.chars()),
                CharRange::Digits => set.extend('0'..='9'),
            }
        }
        set.extend(self.characters.chars().filter(|c| !c.is_control()));
        set
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported font importer settings version".into());
        }
        if !(6..=64).contains(&self.pixel_height) {
            return Err("Font pixel height must be 6 to 64".into());
        }
        if self.padding > 4 {
            return Err("Font glyph padding must be 0 to 4".into());
        }
        if self.charset().is_empty() {
            return Err("Select a character range or list explicit characters".into());
        }
        Ok(())
    }
}
impl Serialize for Settings {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(tag = "type", content = "options")]
        enum Tagged<'a> {
            Texture,
            Authored,
            Fbx(&'a crate::model_import::Settings),
            Font(&'a FontSettings),
            Derived,
        }
        match self {
            Self::Audio(s) => {
                let mut value =
                    serde_json::to_value(&s.envelope_extra).map_err(serde::ser::Error::custom)?;
                let map = value.as_object_mut().unwrap();
                map.insert("type".into(), "Audio".into());
                map.insert(
                    "options".into(),
                    serde_json::to_value(s).map_err(serde::ser::Error::custom)?,
                );
                value.serialize(serializer)
            }
            Self::Texture => Tagged::Texture.serialize(serializer),
            Self::MusicSequence(s) => extended("MusicSequence", s, &s.envelope_extra, serializer),
            Self::SoundBank(s) => extended("SoundBank", s, &s.envelope_extra, serializer),
            Self::Authored => Tagged::Authored.serialize(serializer),
            Self::Fbx(s) => Tagged::Fbx(s).serialize(serializer),
            Self::Font(s) => Tagged::Font(s).serialize(serializer),
            Self::Derived => Tagged::Derived.serialize(serializer),
        }
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self::Audio(Default::default())
    }
}
impl<'de> Deserialize<'de> for Settings {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", content = "options")]
        enum Tagged {
            Texture,
            Audio(crate::audio_import::Settings),
            MusicSequence(crate::sequence::Settings),
            SoundBank(crate::sound_bank::Settings),
            Authored,
            Fbx(crate::model_import::Settings),
            Font(FontSettings),
            Derived,
        }
        use serde::de::Error;
        let value = serde_json::Value::deserialize(d)?;
        if value.get("type").is_none() {
            return serde_json::from_value(value)
                .map(Self::Audio)
                .map_err(D::Error::custom);
        }
        let mut extra = value
            .as_object()
            .cloned()
            .ok_or_else(|| D::Error::custom("Importer settings must be an object"))?;
        extra.remove("type");
        extra.remove("options");
        Ok(
            match serde_json::from_value(value).map_err(D::Error::custom)? {
                Tagged::Texture => Self::Texture,
                Tagged::Audio(mut s) => {
                    s.envelope_extra = extra.into_iter().collect();
                    Self::Audio(s)
                }
                Tagged::Authored => Self::Authored,
                Tagged::MusicSequence(mut s) => {
                    s.validate().map_err(D::Error::custom)?;
                    s.envelope_extra = extra.into_iter().collect();
                    Self::MusicSequence(s)
                }
                Tagged::SoundBank(mut s) => {
                    s.validate().map_err(D::Error::custom)?;
                    s.envelope_extra = extra.into_iter().collect();
                    Self::SoundBank(s)
                }
                Tagged::Fbx(s) => Self::Fbx(s),
                Tagged::Font(s) => {
                    s.validate().map_err(D::Error::custom)?;
                    Self::Font(s)
                }
                Tagged::Derived => Self::Derived,
            },
        )
    }
}
impl Settings {
    pub fn sequence(&self) -> Result<&crate::sequence::Settings, String> {
        match self {
            Self::MusicSequence(s) => Ok(s),
            _ => Err("Expected MusicSequence settings".into()),
        }
    }
    pub fn sound_bank(&self) -> Result<&crate::sound_bank::Settings, String> {
        match self {
            Self::SoundBank(s) => Ok(s),
            _ => Err("Expected SoundBank settings".into()),
        }
    }
    pub fn audio(&self) -> Result<&crate::audio_import::Settings, String> {
        match self {
            Self::Audio(s) => Ok(s),
            _ => Err("Asset does not contain audio import settings".into()),
        }
    }
    pub fn font(&self) -> Result<&FontSettings, String> {
        match self {
            Self::Font(s) => Ok(s),
            _ => Err("Expected Font settings".into()),
        }
    }
}

fn extended<S: serde::Serializer>(
    kind: &str,
    options: &impl Serialize,
    extra: &std::collections::BTreeMap<String, serde_json::Value>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut value = serde_json::to_value(extra).map_err(serde::ser::Error::custom)?;
    let map = value.as_object_mut().unwrap();
    map.insert("type".into(), kind.into());
    map.insert(
        "options".into(),
        serde_json::to_value(options).map_err(serde::ser::Error::custom)?,
    );
    value.serialize(serializer)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_settings_round_trip_through_the_tagged_envelope() {
        let settings = Settings::Font(FontSettings {
            pixel_height: 24,
            characters: "€".into(),
            ranges: vec![CharRange::Digits, CharRange::Latin1Supplement],
            antialias: true,
            padding: 2,
            monospace: true,
            ..Default::default()
        });
        let value = serde_json::to_value(&settings).unwrap();
        assert_eq!(value["type"], "Font");
        assert_eq!(value["options"]["pixel_height"], 24);
        assert_eq!(
            value["options"]["ranges"],
            serde_json::json!(["digits", "latin1_supplement"])
        );
        assert_eq!(
            serde_json::from_value::<Settings>(value).unwrap(),
            settings,
            "the envelope must survive a full write/read cycle"
        );
    }
    #[test]
    fn omitted_font_fields_fall_back_to_the_defaults() {
        let settings: Settings =
            serde_json::from_value(serde_json::json!({"type":"Font","options":{}})).unwrap();
        assert_eq!(settings.font().unwrap(), &FontSettings::default());
        assert_eq!(settings.font().unwrap().pixel_height, 16);
        assert!(!settings.font().unwrap().antialias);
        assert_eq!(settings.font().unwrap().padding, 1);
    }
    #[test]
    fn unknown_or_invalid_font_options_are_rejected() {
        let parse = |options: serde_json::Value| {
            serde_json::from_value::<Settings>(serde_json::json!({"type":"Font","options":options}))
        };
        assert!(parse(serde_json::json!({"bit_depth": 8})).is_err());
        assert!(parse(serde_json::json!({"pixel_height": 4})).is_err());
        assert!(parse(serde_json::json!({"padding": 9})).is_err());
        assert!(parse(serde_json::json!({"ranges": []})).is_err());
        assert!(parse(serde_json::json!({"ranges": ["Ascii"]})).is_err());
        assert!(parse(serde_json::json!({"ranges": ["ascii"]})).is_ok());
        assert!(Settings::Texture.font().is_err());
    }
}
