//! Property sections and independent scalar channels in the v3 authoring schema.
use crate::{
    reflection_schema::Type,
    timeline::{self, Key, Track},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub id: Uuid,
    /// Half-open sequence range; the final root sample may include its end.
    pub start_tick: i32,
    pub end_tick: i32,
    #[serde(default)]
    pub source_offset_tick: i32,
    #[serde(default = "one")]
    pub rate_numerator: i32,
    #[serde(default = "one")]
    pub rate_denominator: i32,
    pub channels: Vec<Channel>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
fn one() -> i32 {
    1
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub lane: u8,
    pub interpolation: timeline::Interpolation,
    pub keys: Vec<Key>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
impl Section {
    pub fn source_tick(&self, tick: i32) -> i32 {
        if self.rate_denominator <= 0 {
            return self.source_offset_tick;
        }
        (i64::from(self.source_offset_tick)
            + (i64::from(tick) - i64::from(self.start_tick)) * i64::from(self.rate_numerator)
                / i64::from(self.rate_denominator))
        .clamp(0, i64::from(i32::MAX)) as i32
    }
    pub fn sequence_tick(&self, tick: i32) -> i32 {
        if self.rate_numerator <= 0 {
            return self.start_tick;
        }
        (i64::from(self.start_tick)
            + (i64::from(tick) - i64::from(self.source_offset_tick))
                * i64::from(self.rate_denominator)
                / i64::from(self.rate_numerator))
        .clamp(0, i64::from(i32::MAX)) as i32
    }
}
pub fn scalar_type(ty: &Type) -> &Type {
    if matches!(ty, Type::Vector { .. }) {
        &Type::Fixed
    } else {
        ty
    }
}
impl Track {
    /// Explicit conversion is a document transaction, so old files keep their
    /// stable keys and serialized shape until the author needs section editing.
    pub fn make_section(&mut self, duration: i32) {
        if !self.sections.is_empty() || self.keys.is_empty() {
            return;
        }
        let channels = (0..timeline::channels(&self.value_type))
            .map(|lane| Channel {
                id: Uuid::new_v4(),
                lane: lane as u8,
                interpolation: self.interpolation,
                extra: Default::default(),
                keys: self
                    .keys
                    .iter()
                    .map(|key| Key {
                        id: if lane == 0 { key.id } else { Uuid::new_v4() },
                        tick: key.tick,
                        value: if matches!(self.value_type, Type::Vector { .. }) {
                            key.value.get(lane).cloned().unwrap_or_else(|| {
                                crate::script_values::default_value(&Type::Fixed)
                            })
                        } else {
                            key.value.clone()
                        },
                        extra: key.extra.clone(),
                    })
                    .collect(),
            })
            .collect();
        self.sections.push(Section {
            id: Uuid::new_v4(),
            start_tick: 0,
            end_tick: duration,
            source_offset_tick: 0,
            rate_numerator: 1,
            rate_denominator: 1,
            channels,
            extra: Default::default(),
        });
        self.keys.clear();
    }
    pub fn ranges(&self, duration: i32) -> Vec<(i32, i32)> {
        if self.sections.is_empty() {
            vec![(0, duration)]
        } else {
            self.sections
                .iter()
                .map(|s| (s.start_tick, s.end_tick))
                .collect()
        }
    }
}

pub fn validate(track: &Track, duration: i32) -> Vec<(Uuid, String)> {
    let mut errors = Vec::new();
    if track.sections.is_empty() {
        return errors;
    }
    if !track.keys.is_empty() {
        errors.push((
            track.id,
            "A section track cannot also contain legacy keys".into(),
        ));
    }
    for (i, section) in track.sections.iter().enumerate() {
        if section.start_tick < 0
            || section.end_tick <= section.start_tick
            || section.end_tick > duration
            || section.source_offset_tick < 0
            || !(1..=1024).contains(&section.rate_numerator)
            || !(1..=1024).contains(&section.rate_denominator)
        {
            errors.push((
                section.id,
                "Invalid section range or positive rational playback rate (1..1024)".into(),
            ));
        } else if i64::from(section.source_offset_tick)
            + (i64::from(section.end_tick) - i64::from(section.start_tick))
                * i64::from(section.rate_numerator)
                / i64::from(section.rate_denominator)
            > i64::from(i32::MAX)
        {
            errors.push((
                section.id,
                "Section source time exceeds the Q12 clock".into(),
            ));
        }
        if track.sections[..i]
            .iter()
            .any(|s| s.start_tick < section.end_tick && section.start_tick < s.end_tick)
        {
            errors.push((
                section.id,
                "Sections on one property track cannot overlap; use a separate priority track"
                    .into(),
            ));
        }
        let expected = timeline::channels(&track.value_type);
        let lanes = section
            .channels
            .iter()
            .map(|c| usize::from(c.lane))
            .collect::<std::collections::BTreeSet<_>>();
        if lanes != (0..expected).collect() || section.channels.len() != expected {
            errors.push((
                section.id,
                "A section must contain each typed property lane exactly once".into(),
            ));
        }
        for channel in &section.channels {
            if channel.keys.is_empty() || channel.keys.len() > timeline::KEY_LIMIT {
                errors.push((channel.id, "A channel requires 1–256 keys".into()));
            }
            if crate::reflection_schema::TimelineProperty::for_type(scalar_type(&track.value_type))
                .is_none_or(|profile| !profile.allows(channel.interpolation, track.blend))
            {
                errors.push((
                    channel.id,
                    "Interpolation is incompatible with the channel value type".into(),
                ));
            }
            let mut times = std::collections::BTreeSet::new();
            for key in &channel.keys {
                if key.tick < 0 || !times.insert(key.tick) {
                    errors.push((
                        key.id,
                        "Channel keys need unique nonnegative source times".into(),
                    ));
                }
                if let Err(error) = timeline::pack(&key.value, scalar_type(&track.value_type)) {
                    errors.push((key.id, error));
                }
            }
        }
    }
    errors
}
