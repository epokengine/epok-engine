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
    Derived,
}
impl Serialize for Settings {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(tag = "type", content = "options")]
        enum Tagged<'a> {
            Texture,
            Authored,
            Fbx(&'a crate::model_import::Settings),
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
