//! Typed, host-side evaluation IR. Authoring identities survive here for diagnostics;
//! the C++ cooker strips them from immutable runtime tables.
use crate::timeline;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledTrack {
    pub id: Uuid,
    pub slot: Uuid,
    pub property: String,
    pub class: String,
    pub field: String,
    pub priority: i16,
    pub restore: timeline::Restore,
    pub channels: Vec<Vec<(i32, i32)>>,
    pub value_type: crate::reflection_schema::Type,
    pub interpolation: timeline::Interpolation,
    pub blend: timeline::Blend,
    pub key_ids: Vec<Uuid>,
    #[serde(default)]
    pub range: Option<SectionTiming>,
    #[serde(default)]
    pub interpolation_modes: Vec<timeline::Interpolation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Compiled {
    pub asset: Uuid,
    pub duration_ticks: i32,
    pub loop_mode: timeline::LoopMode,
    pub slots: Vec<timeline::Slot>,
    pub tracks: Vec<CompiledTrack>,
    pub markers: Vec<(i32, Uuid)>,
    pub events: Vec<CompiledEvent>,
    pub dependencies: BTreeMap<String, String>,
    pub signature: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledEvent {
    pub track: Uuid,
    pub key: Uuid,
    pub slot: Uuid,
    pub tick: i32,
    pub class: String,
    pub function: String,
    pub method: String,
    pub call: crate::reflection_schema::TimelineCall,
    pub arguments: Vec<CookedArgument>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CookedArgument {
    pub value_type: crate::reflection_schema::Type,
    pub lanes: [i32; 4],
    pub resource: Option<Uuid>,
    pub slot: Option<Uuid>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SectionTiming {
    pub start: i32,
    pub end: i32,
    pub offset: i32,
    pub numerator: i32,
    pub denominator: i32,
}
impl CompiledTrack {
    pub fn active(&self, tick: i32, duration: i32) -> bool {
        self.range.is_none_or(|r| {
            tick >= r.start && (tick < r.end || (tick == duration && r.end == duration))
        })
    }
    pub fn source_tick(&self, tick: i32) -> i32 {
        self.range.map_or(tick, |r| {
            (i64::from(r.offset)
                + (i64::from(tick) - i64::from(r.start)) * i64::from(r.numerator)
                    / i64::from(r.denominator.max(1)))
            .clamp(0, i64::from(i32::MAX)) as i32
        })
    }
    pub fn mode(&self, lane: usize) -> timeline::Interpolation {
        self.interpolation_modes
            .get(lane)
            .copied()
            .unwrap_or(self.interpolation)
    }
}
