//! Layer authoring around the existing TimelineAsset controls and emitter UI.
use crate::{
    particle_effect::{LayerContent, ParticleEffect},
    particles::Emitter,
    sprites::{Orientation, Sprite},
    texture::BlendMode,
    timeline,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub enum Preset {
    Fire,
    Smoke,
    Sparks,
    Impact,
    Projectile,
    Aura,
    Rune,
}
impl Preset {
    pub fn parse(name: &str) -> Result<Self, String> {
        Self::ALL.into_iter().find(|p|p.name().eq_ignore_ascii_case(name)).ok_or_else(||format!("Unknown effect preset {name}; choose Fire, Smoke, Sparks, Impact, Projectile, Aura or Rune"))
    }
    pub const ALL: [Self; 7] = [
        Self::Fire,
        Self::Smoke,
        Self::Sparks,
        Self::Impact,
        Self::Projectile,
        Self::Aura,
        Self::Rune,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Fire => "Fire",
            Self::Smoke => "Smoke",
            Self::Sparks => "Sparks",
            Self::Impact => "Impact",
            Self::Projectile => "Projectile",
            Self::Aura => "Aura",
            Self::Rune => "Rune",
        }
    }
}
/// Presets produce ordinary emitter/sprite payloads and shared timeline tracks.
/// They do not introduce a preset runtime interpreter or particle-only curves.
pub fn add_preset(effect: &mut ParticleEffect, preset: Preset) -> Result<Uuid, String> {
    if effect.timeline.duration_ticks < 8 {
        return Err("Preset requires a timeline duration of at least eight Q12 ticks".into());
    }
    let burst = match preset {
        Preset::Sparks => 48,
        Preset::Impact => 24,
        _ => 0,
    };
    if effect.timeline.tracks.len() + effect.timeline.events.len() + 1 + usize::from(burst > 0)
        > timeline::TRACK_LIMIT
    {
        return Err("Preset would exceed sixteen timeline tracks".into());
    }
    if burst > 0
        && effect
            .timeline
            .events
            .iter()
            .map(|t| t.keys.len())
            .sum::<usize>()
            >= timeline::EVENT_LIMIT
    {
        return Err("Preset would exceed sixty-four timeline events".into());
    }
    let sprite = Sprite {
        unlit: true,
        orientation: Orientation::Spherical,
        blend: BlendMode::Add,
        ..Default::default()
    };
    let content = if matches!(preset, Preset::Projectile | Preset::Rune) {
        LayerContent::Sprite {
            sprite: Sprite {
                size: if matches!(preset, Preset::Rune) {
                    [2., 2.]
                } else {
                    [0.5, 0.5]
                },
                color: if matches!(preset, Preset::Rune) {
                    [0.3, 0.65, 1.]
                } else {
                    [1., 0.4, 0.05]
                },
                ..sprite
            },
            frames: 1,
            columns: 1,
            frame_ticks: 410,
        }
    } else {
        let mut emitter = Emitter {
            sprite,
            ..Default::default()
        };
        match preset {
            Preset::Fire => {
                emitter.rate = 40.;
                emitter.max_particles = 64;
                emitter.lifetime = 0.5;
                emitter.velocity = [0., 1.5, 0.];
                emitter.start_size = 0.3;
                emitter.end_size = 0.05;
            }
            Preset::Smoke => {
                emitter.rate = 12.;
                emitter.max_particles = 32;
                emitter.lifetime = 1.5;
                emitter.velocity = [0., 0.6, 0.];
                emitter.gravity = [0.; 3];
                emitter.start_size = 0.2;
                emitter.end_size = 0.8;
                emitter.start_color = [0.35; 3];
                emitter.end_color = [0.05; 3];
                emitter.sprite.blend = BlendMode::Average;
            }
            Preset::Sparks | Preset::Impact => {
                emitter.continuous = false;
                emitter.play_on_start = false;
                emitter.burst = burst;
                emitter.max_particles = 64;
                emitter.lifetime = 0.6;
                emitter.velocity = [0., 1., 0.];
                emitter.spread = [2., 2., 2.];
                emitter.gravity = [0., -3., 0.];
                emitter.start_size = 0.09;
                emitter.end_size = 0.;
            }
            Preset::Aura => {
                emitter.rate = 16.;
                emitter.max_particles = 32;
                emitter.lifetime = 1.;
                emitter.velocity = [0., 0.5, 0.];
                emitter.gravity = [0.; 3];
                emitter.spread = [0.4, 0.1, 0.4];
                emitter.start_size = 0.08;
                emitter.end_size = 0.;
                emitter.start_color = [0.3, 1., 0.5];
                emitter.end_color = [0., 0.2, 0.05];
                emitter.local_space = true;
            }
            _ => unreachable!(),
        }
        LayerContent::Emitter {
            emitter: Box::new(emitter),
        }
    };
    let id = effect.add_layer(preset.name().into(), content)?;
    let slot = effect.layers.last().unwrap().slot;
    let end = effect.timeline.duration_ticks;
    effect.timeline.tracks.push(timeline::Track {
        sections: vec![],
        id: Uuid::new_v4(),
        name: format!("{} opacity", preset.name()),
        slot,
        property: "775f36b0-2b5b-4920-aa9f-981b166507ef".into(),
        value_type: crate::reflection_schema::Type::Fixed,
        priority: 0,
        blend: timeline::Blend::Absolute,
        restore: timeline::Restore::LeaveFinal,
        interpolation: timeline::Interpolation::Linear,
        keys: [
            (0, 0.),
            (end / 8, 1.),
            ((i64::from(end) * 3 / 4) as i32, 1.),
            (end, 0.),
        ]
        .into_iter()
        .map(|(tick, value)| timeline::Key {
            id: Uuid::new_v4(),
            tick,
            value: serde_json::json!(value),
            extra: Default::default(),
        })
        .collect(),
        extra: Default::default(),
    });
    if burst > 0 {
        effect.timeline.events.push(timeline::EventTrack {
            id: Uuid::new_v4(),
            name: format!("{} burst", preset.name()),
            slot,
            function: "c2650304-c940-408c-a8a8-05641c2a0489".into(),
            keys: vec![timeline::EventKey {
                id: Uuid::new_v4(),
                tick: 0,
                arguments: std::collections::BTreeMap::from([(
                    "count".into(),
                    timeline::Argument::Literal {
                        value_type: crate::reflection_schema::Type::UInt32,
                        value: serde_json::json!(burst),
                    },
                )]),
                extra: Default::default(),
            }],
            extra: Default::default(),
        });
    }
    Ok(id)
}
pub fn remove_layer(effect: &mut ParticleEffect, id: Uuid) {
    let Some(index) = effect.layers.iter().position(|l| l.id == id) else {
        return;
    };
    let slot = effect.layers.remove(index).slot;
    effect.timeline.slots.retain(|s| s.id != slot);
    effect.timeline.tracks.retain(|t| t.slot != slot);
    effect.timeline.events.retain(|t| t.slot != slot);
}
/// Artist controls edit the existing source payload and shared event keys.
/// There is no second parameter state to serialize or evaluate at runtime.
fn expressive_controls(
    ui: &imgui::Ui,
    layer: &mut crate::particle_effect::Layer,
    events: &mut [timeline::EventTrack],
) {
    let sprite = match &mut layer.content {
        LayerContent::Sprite { sprite, .. } => {
            let old = sprite.size.into_iter().fold(0_f32, f32::max);
            let mut scale = old;
            let minimum = 0.001 * old
                / sprite
                    .size
                    .into_iter()
                    .fold(f32::INFINITY, f32::min)
                    .max(0.001);
            if crate::gui::Drag::new("Scale")
                .range(minimum.max(0.001), 128.)
                .speed(0.01)
                .build(ui, &mut scale)
            {
                sprite.size = if old > 0. {
                    sprite.size.map(|v| (v * scale / old).max(0.001))
                } else {
                    [scale; 2]
                };
            }
            sprite
        }
        LayerContent::Emitter { emitter } => {
            if emitter.continuous {
                crate::gui::Drag::new("Density / second")
                    .range(0., 512.)
                    .speed(0.25)
                    .build(ui, &mut emitter.rate);
            } else {
                let mut event_density = false;
                for event in events.iter_mut().filter(|e| {
                    e.slot == layer.slot && e.function == "c2650304-c940-408c-a8a8-05641c2a0489"
                }) {
                    for key in &mut event.keys {
                        if let Some(timeline::Argument::Literal {
                            value_type: crate::reflection_schema::Type::UInt32,
                            value,
                        }) = key.arguments.get_mut("count")
                            && let Some(count) = value.as_u64()
                        {
                            let mut density = count.min(256) as u32;
                            let _id = ui.push_id(key.id.to_string());
                            if crate::gui::Drag::new(format!(
                                "Density at {:.2}s",
                                key.tick as f32 / 4096.
                            ))
                            .range(0, 256)
                            .build(ui, &mut density)
                            {
                                *value = serde_json::json!(density);
                            }
                            event_density = true;
                        }
                    }
                }
                if !event_density {
                    crate::gui::Drag::new("Density / startup burst")
                        .range(0, 128)
                        .build(ui, &mut emitter.burst);
                }
            }
            let speed = emitter.velocity.iter().map(|v| v * v).sum::<f32>().sqrt();
            let mut violence = speed;
            let direction = if speed > 0. {
                emitter.velocity.map(|v| v / speed)
            } else {
                [0., 1., 0.]
            };
            if crate::gui::Drag::new("Violence / speed")
                .range(0., 128.)
                .speed(0.05)
                .build(ui, &mut violence)
            {
                emitter.velocity = direction.map(|v| v * violence);
            }
            let mut yaw = direction[0].atan2(direction[2]).to_degrees();
            let mut pitch = direction[1].clamp(-1., 1.).asin().to_degrees();
            let changed = crate::gui::Drag::new("Direction yaw")
                .range(-180., 180.)
                .speed(0.5)
                .build(ui, &mut yaw);
            if crate::gui::Drag::new("Direction elevation")
                .range(-90., 90.)
                .speed(0.5)
                .build(ui, &mut pitch)
                || changed
            {
                let (sy, cy) = yaw.to_radians().sin_cos();
                let (sp, cp) = pitch.to_radians().sin_cos();
                emitter.velocity = [sy * cp, sp, cy * cp].map(|v| v * violence.min(128.));
            }
            let old = emitter.spread.into_iter().fold(0_f32, f32::max);
            let mut chaos = old;
            if crate::gui::Drag::new("Chaos / spread")
                .range(0., 128.)
                .speed(0.02)
                .build(ui, &mut chaos)
            {
                emitter.spread = if old > 0. {
                    emitter.spread.map(|v| v * chaos / old)
                } else {
                    [chaos; 3]
                };
            }
            let old = emitter.start_size.max(emitter.end_size);
            let mut scale = old;
            if crate::gui::Drag::new("Scale")
                .range(0., 32.)
                .speed(0.01)
                .build(ui, &mut scale)
            {
                if old > 0. {
                    emitter.start_size *= scale / old;
                    emitter.end_size *= scale / old;
                } else {
                    emitter.start_size = scale;
                }
            }
            &mut emitter.sprite
        }
    };
    let old = sprite.color.into_iter().fold(0_f32, f32::max);
    let mut brightness = old;
    if ui.slider("Brightness", 0., 1., &mut brightness) {
        sprite.color = if old > 0. {
            sprite.color.map(|v| v * brightness / old)
        } else {
            [brightness; 3]
        };
    }
    ui.text_disabled("Tracks override these starting values while playing.");
}

/// Preserve every key/marker ID when changing artistic duration. Reject lossy
/// collapse of distinct property keys instead of silently dropping animation.
fn retime(asset: &mut timeline::TimelineAsset, duration: i32) -> Result<(), String> {
    if duration <= 0 || asset.duration_ticks <= 0 {
        return Err("Duration must be positive".into());
    }
    let mut next = asset.clone();
    let tick =
        |v: i32| (i64::from(v) * i64::from(duration) / i64::from(asset.duration_ticks)) as i32;
    for track in &mut next.tracks {
        for key in &mut track.keys {
            key.tick = tick(key.tick);
        }
        let unique = track
            .keys
            .iter()
            .map(|k| k.tick)
            .collect::<std::collections::BTreeSet<_>>();
        if unique.len() != track.keys.len() {
            return Err("Duration is too short to preserve distinct curve keys".into());
        }
        for section in &mut track.sections {
            section.start_tick = tick(section.start_tick);
            section.end_tick = tick(section.end_tick);
            section.source_offset_tick = tick(section.source_offset_tick);
            if section.end_tick <= section.start_tick {
                return Err("Duration is too short to preserve a timeline section".into());
            }
            for channel in &mut section.channels {
                for key in &mut channel.keys {
                    key.tick = tick(key.tick);
                }
                let unique = channel
                    .keys
                    .iter()
                    .map(|k| k.tick)
                    .collect::<std::collections::BTreeSet<_>>();
                if unique.len() != channel.keys.len() {
                    return Err("Duration is too short to preserve distinct channel keys".into());
                }
            }
        }
    }
    for event in &mut next.events {
        for key in &mut event.keys {
            key.tick = tick(key.tick);
        }
    }
    for marker in &mut next.markers {
        marker.tick = tick(marker.tick);
    }
    next.duration_ticks = duration;
    *asset = next;
    Ok(())
}
pub fn layers(
    ui: &imgui::Ui,
    effect: &mut ParticleEffect,
    registry: &crate::blueprint::Registry,
    _scene: &crate::scene::Scene,
    index: &crate::assets::Index,
) {
    ui.input_text("Effect name", &mut effect.name).build();
    crate::gui::Drag::new("Effect seed").build(ui, &mut effect.seed);
    let mut duration = effect.timeline.duration_ticks as f32 / 4096.;
    if crate::gui::Drag::new("Duration / retime effect")
        .range(1. / 4096., 3600.)
        .speed(0.01)
        .build(ui, &mut duration)
        && let Err(error) = retime(&mut effect.timeline, (duration * 4096.).round() as i32)
    {
        ui.text_wrapped(error);
    }
    ui.text_disabled(format!(
        "{} / 8 layers | shared 256 particles / 64 emitters / 2048 sprite triangles",
        effect.layers.len()
    ));
    if crate::timeline_editor::button(ui, "Add Fire layer")
        && let Err(error) = add_preset(effect, Preset::Fire)
    {
        ui.text_wrapped(error);
    }
    if let Some(_combo) = ui.begin_combo("Add preset layer", "Choose a preset") {
        for preset in Preset::ALL {
            if ui.selectable(preset.name())
                && let Err(error) = add_preset(effect, preset)
            {
                ui.text_wrapped(error);
            }
        }
    }
    let mut remove = None;
    let mut reorder = None;
    ui.child_window("Effect layers")
        .size([0., 240.])
        .border(true)
        .build(|| {
            let length = effect.layers.len();
            for (i, layer) in effect.layers.iter_mut().enumerate() {
                let _id = ui.push_id(layer.id.to_string());
                if !ui.collapsing_header(
                    format!("{}###layer", layer.name),
                    imgui::TreeNodeFlags::empty(),
                ) {
                    continue;
                }
                ui.input_text("Layer name", &mut layer.name).build();
                if let Some(slot) = effect
                    .timeline
                    .slots
                    .iter_mut()
                    .find(|s| s.id == layer.slot)
                {
                    slot.name = layer.name.clone();
                }
                ui.checkbox("Layer enabled", &mut layer.enabled);
                crate::gui::Drag::new("Local offset")
                    .speed(0.01)
                    .build_array(ui, &mut layer.position);
                expressive_controls(ui, layer, &mut effect.timeline.events);
                if ui.collapsing_header(
                    "Advanced sprite and emission",
                    imgui::TreeNodeFlags::empty(),
                ) {
                    match &mut layer.content {
                        LayerContent::Emitter { emitter } => {
                            crate::sprites_editor::emitter(ui, index, emitter)
                        }
                        LayerContent::Sprite {
                            sprite,
                            frames,
                            columns,
                            frame_ticks,
                        } => {
                            crate::sprites_editor::sprite(ui, index, sprite);
                            crate::gui::Drag::new("Flipbook frames")
                                .range(1, 256)
                                .build(ui, frames);
                            crate::gui::Drag::new("Flipbook columns")
                                .range(1, 256)
                                .build(ui, columns);
                            let mut seconds = *frame_ticks as f32 / 4096.;
                            if crate::gui::Drag::new("Seconds per cell")
                                .range(1. / 4096., 60.)
                                .speed(0.01)
                                .build(ui, &mut seconds)
                            {
                                *frame_ticks = (seconds * 4096.).round() as i32;
                            }
                        }
                    }
                }
                if i > 0 && ui.small_button("Move up") {
                    reorder = Some((i, i - 1));
                }
                if i + 1 < length && ui.small_button("Move down") {
                    reorder = Some((i, i + 1));
                }
                if ui.small_button("Remove layer and its tracks") {
                    remove = Some(layer.id);
                }
            }
        });
    if let Some((a, b)) = reorder {
        effect.layers.swap(a, b);
    }
    if let Some(id) = remove {
        remove_layer(effect, id);
    }
    for error in effect.validate(registry) {
        ui.text_colored([1., 0.5, 0.3, 1.], error.to_string());
    }
    ui.separator();
    ui.text("Embedded Timeline");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn artistic_duration_preserves_identity_and_rejects_collapsed_keys() {
        let mut effect = ParticleEffect::new("Retime".into());
        add_preset(&mut effect, Preset::Sparks).unwrap();
        effect.timeline.markers.push(timeline::Marker {
            id: Uuid::new_v4(),
            name: "Impact".into(),
            tick: 2048,
            extra: Default::default(),
        });
        let before = effect.timeline.clone();
        retime(&mut effect.timeline, 8192).unwrap();
        assert_eq!(
            before.identities().collect::<Vec<_>>(),
            effect.timeline.identities().collect::<Vec<_>>()
        );
        assert_eq!(effect.timeline.markers[0].tick, 4096);
        retime(&mut effect.timeline, 4096).unwrap();
        assert_eq!(effect.timeline, before);
        assert!(retime(&mut effect.timeline, 1).is_err());
        assert_eq!(effect.timeline, before);
    }
    #[test]
    fn presets_use_shared_tracks_and_respect_particle_budgets() {
        let mut effect = ParticleEffect::new("Presets".into());
        for preset in Preset::ALL {
            add_preset(&mut effect, preset).unwrap();
            if let LayerContent::Emitter { emitter } = &effect.layers.last().unwrap().content {
                crate::particles::validate_emitter(emitter).unwrap();
            }
        }
        assert_eq!(effect.layers.len(), 7);
        assert_eq!(effect.timeline.tracks.len(), 7);
        assert_eq!(effect.timeline.events.len(), 2);
        let before = effect.clone();
        effect.layers.reverse();
        effect.timeline.slots.reverse();
        assert_eq!(before.semantic_hash(), effect.semantic_hash());
        let slot = effect.layers[0].slot;
        let layer = effect.layers[0].id;
        remove_layer(&mut effect, layer);
        assert!(!effect.timeline.slots.iter().any(|s| s.id == slot));
        assert!(!effect.timeline.tracks.iter().any(|t| t.slot == slot));
        assert_eq!(effect.timeline.id, before.timeline.id);
    }
}
