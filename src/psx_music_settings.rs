//! Versioned PSX conversion recipes, stored under a namespaced target override.
//! Settings describe actual conversion; source intent/residency stay independent.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PROFILE: &str = "psx-library-v2";
pub const MAX_RATE: u32 = 44100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    Compact,
    #[default]
    Balanced,
    High,
    Custom,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleLoops {
    /// Require integral 28-frame boundaries after resampling; otherwise fail.
    ExactBlocks,
    /// Explicit approximation. Every changed endpoint appears in the report.
    #[default]
    AlignOutward,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterPolicy {
    /// Reject a required dynamic filter rather than replace its envelope.
    RequireStatic,
    /// Bake a constant cutoff at the modulation envelope sustain level and a
    /// reported reference velocity. Sample pitch also transposes this baked filter.
    #[default]
    BakeSustain,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effects {
    #[default]
    Dry,
    Room,
}
pub const ROOM_REVERB_BYTES: u32 = 0x26c0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selection {
    #[default]
    Reachable,
    FullLibrary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Recipe {
    pub version: u32,
    pub preset: Preset,
    /// A maximum, never an instruction to upsample a lower-rate source.
    pub max_sample_rate: u32,
    pub encoder_effort: crate::spu_encoder::Effort,
    pub sample_loops: SampleLoops,
    /// Zero preserves the aligned seam. Otherwise blend at most this many
    /// frames before the loop end with the source immediately before its start.
    pub loop_crossfade_frames: u16,
    pub selection: Selection,
    pub filter_policy: FilterPolicy,
    pub effects: Effects,
    /// Global PSX reverb output gain; the hardware send is binary per voice.
    pub reverb_depth_permille: u16,
    /// Explicit target adaptation, in milliseconds for a full-scale release.
    /// None preserves the authored envelope. Source Preview always preserves it.
    pub maximum_release_ms: Option<u16>,
    /// 0.1 dB attenuation; one shared scale preserves relative instrument levels.
    pub headroom_centibels: u16,
    pub bank_budget_bytes: u32,
    /// Additional project demands, not part of this bank's sample bytes.
    pub other_resident_bytes: u32,
    pub optimization: Optimization,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Optimization {
    pub allow_lower_rate: bool,
    pub minimum_sample_rate: u32,
    pub max_candidates: u16,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Default for Optimization {
    fn default() -> Self {
        Self {
            allow_lower_rate: true,
            minimum_sample_rate: 8000,
            max_candidates: 32,
            extra: BTreeMap::new(),
        }
    }
}
impl Default for Recipe {
    fn default() -> Self {
        Self {
            version: 1,
            preset: Preset::Balanced,
            max_sample_rate: 22050,
            encoder_effort: crate::spu_encoder::Effort::Fast,
            sample_loops: SampleLoops::AlignOutward,
            loop_crossfade_frames: 0,
            selection: Selection::Reachable,
            filter_policy: FilterPolicy::BakeSustain,
            effects: Effects::Dry,
            reverb_depth_permille: 250,
            maximum_release_ms: None,
            headroom_centibels: 60,
            bank_budget_bytes: crate::audio_import::SPU_BUDGET as u32,
            other_resident_bytes: 0,
            optimization: Optimization::default(),
            extra: BTreeMap::new(),
        }
    }
}
impl Recipe {
    pub fn select_preset(&mut self, preset: Preset) {
        if preset == Preset::Custom {
            self.preset = preset;
            return;
        }
        self.preset = preset;
        self.max_sample_rate = match preset {
            Preset::Compact => 11025,
            Preset::Balanced => 22050,
            Preset::High => 44100,
            Preset::Custom => unreachable!(),
        };
        self.encoder_effort = crate::spu_encoder::Effort::Fast;
        self.sample_loops = SampleLoops::AlignOutward;
        self.loop_crossfade_frames = 0;
        self.filter_policy = FilterPolicy::BakeSustain;
        self.effects = Effects::Dry;
        self.reverb_depth_permille = 250;
        self.maximum_release_ms = None;
        self.headroom_centibels = 60;
        // Project budgets and user optimization limits are never reset by quality.
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported PSX music conversion recipe version".into());
        }
        if !(400..=MAX_RATE).contains(&self.max_sample_rate) {
            return Err("PSX sample-rate maximum must be 400–44100 Hz".into());
        }
        if self.headroom_centibels > 960 {
            return Err("Music headroom must be 0–96 dB of attenuation".into());
        }
        if self.reverb_depth_permille > 1000 {
            return Err("PSX reverb output depth must be 0–1000 permille".into());
        }
        if self
            .maximum_release_ms
            .is_some_and(|value| !(10..=30000).contains(&value))
        {
            return Err("Maximum release must be 10–30000 milliseconds, or Preserve".into());
        }
        if self.loop_crossfade_frames > 256 {
            return Err("Loop crossfade must be 0–256 frames; zero preserves the seam".into());
        }
        if self.bank_budget_bytes == 0
            || self.bank_budget_bytes > crate::audio_import::SPU_BUDGET as u32
            || self.other_resident_bytes >= crate::audio_import::SPU_BUDGET as u32
        {
            return Err("PSX bank budget must fit SPU memory after the capture and other resident reservations".into());
        }
        if !(400..=MAX_RATE).contains(&self.optimization.minimum_sample_rate)
            || !(1..=32).contains(&self.optimization.max_candidates)
        {
            return Err(
                "Optimization requires a minimum of 400–44100 Hz and 1–32 candidates".into(),
            );
        }
        if self.preset != Preset::Custom {
            let mut expected = self.clone();
            expected.select_preset(self.preset);
            if expected != *self {
                return Err("Changed conversion values must use the Custom preset".into());
            }
        }
        Ok(())
    }
    pub fn available_bytes(&self) -> u32 {
        self.bank_budget_bytes.min(
            (crate::audio_import::SPU_BUDGET as u32).saturating_sub(
                self.other_resident_bytes
                    .saturating_add(self.reverb_bytes()),
            ),
        )
    }
    pub fn reverb_bytes(&self) -> u32 {
        if self.effects == Effects::Room {
            ROOM_REVERB_BYTES
        } else {
            0
        }
    }
    pub fn reverb_depth_q15(&self) -> u16 {
        if self.effects == Effects::Room {
            (u32::from(self.reverb_depth_permille) * 32767 / 1000) as u16
        } else {
            0
        }
    }
    pub fn gain(&self) -> f64 {
        10_f64.powf(-(self.headroom_centibels as f64) / 200.)
    }
    pub fn from_settings(settings: &crate::sequence::Settings) -> Result<Self, String> {
        if settings
            .target_overrides
            .get("psx")
            .is_some_and(|value| !value.is_object())
        {
            return Err("PSX overrides must be an object".into());
        }
        let recipe: Self = settings
            .target_overrides
            .get("psx")
            .and_then(|value| value.get("music_conversion"))
            .map(|value| serde_json::from_value(value.clone()).map_err(|error| error.to_string()))
            .transpose()?
            .unwrap_or_default();
        recipe.validate()?;
        Ok(recipe)
    }
    pub fn store(&self, settings: &mut crate::sequence::Settings) -> Result<(), String> {
        self.validate()?;
        let target = settings
            .target_overrides
            .entry("psx".into())
            .or_insert_with(|| serde_json::json!({}));
        target
            .as_object_mut()
            .ok_or("PSX overrides must be an object")?
            .insert(
                "music_conversion".into(),
                serde_json::to_value(self).map_err(|error| error.to_string())?,
            );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn psx_music_presets_roundtrip_without_changing_intent_or_other_targets() {
        let mut settings = crate::sequence::Settings::default();
        settings.role = crate::audio_import::AudioRole::Ambience;
        settings
            .target_overrides
            .insert("future_console".into(), serde_json::json!({"keep":17}));
        settings
            .target_overrides
            .insert("psx".into(), serde_json::json!({"future_option":42}));
        let mut recipe = Recipe::default();
        recipe.other_resident_bytes = 65536;
        recipe
            .optimization
            .extra
            .insert("future_search_limit".into(), serde_json::json!({"keep":11}));
        recipe.select_preset(Preset::Compact);
        recipe.store(&mut settings).unwrap();
        assert_eq!(Recipe::from_settings(&settings).unwrap(), recipe);
        assert_eq!(settings.role, crate::audio_import::AudioRole::Ambience);
        assert_eq!(settings.target_overrides["future_console"]["keep"], 17);
        assert_eq!(settings.target_overrides["psx"]["future_option"], 42);
        assert_eq!(
            recipe.available_bytes(),
            crate::audio_import::SPU_BUDGET as u32 - 65536
        );
        recipe.max_sample_rate = 10000;
        assert!(recipe.validate().unwrap_err().contains("Custom"));
        recipe.preset = Preset::Custom;
        recipe.validate().unwrap();
        assert!((recipe.gain() - 0.5011872336272722).abs() < 1e-12);
        recipe.select_preset(Preset::High);
        assert_eq!(recipe.max_sample_rate, 44100);
        assert_eq!(recipe.other_resident_bytes, 65536);
        settings
            .target_overrides
            .insert("psx".into(), serde_json::json!("invalid"));
        assert!(Recipe::from_settings(&settings).is_err());
    }
}
