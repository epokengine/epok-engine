//! Host execution of the actual PSX effect/particle/director kernels through a
//! bounded C ABI. Native code is linked into the editor, never hot-reloaded.
use crate::{
    particle_effect::{LayerContent, ParticleEffect},
    sprites::{Orientation, Sprite},
    texture::BlendMode,
    timeline_compile::Compiled,
};
use std::{
    ffi::c_void,
    ptr::NonNull,
    sync::{Mutex, MutexGuard},
};
use uuid::Uuid;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SpriteData {
    texture: i32,
    region: [i32; 4],
    size: [i32; 2],
    pivot: [i32; 2],
    color: [i32; 3],
    enabled: u32,
    flip_x: u32,
    flip_y: u32,
    orientation: u32,
    unlit: u32,
    blend: u32,
    depth_bias: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct LayerData {
    sprite: SpriteData,
    slot: u32,
    emitter: u32,
    enabled: u32,
    playing: u32,
    continuous: u32,
    local_space: u32,
    seed: u32,
    burst: u32,
    max_particles: u32,
    position: [i32; 3],
    velocity: [i32; 3],
    spread: [i32; 3],
    gravity: [i32; 3],
    rate: i32,
    lifetime: i32,
    start_size: i32,
    end_size: i32,
    start_color: [i32; 3],
    end_color: [i32; 3],
    frames: u32,
    columns: u32,
    frame_ticks: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KeyData {
    tick: i32,
    value: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct TrackData {
    property: u64,
    slot: u32,
    field: u32,
    channels: u32,
    additive: u32,
    restore: u32,
    interpolation: u32,
    lengths: [u32; 4],
    keys: [[KeyData; 4]; 4],
}
#[repr(C)]
struct EventData {
    slot: u32,
    function: u32,
    count: u32,
    idempotent: u32,
}
#[repr(C)]
struct SignalData {
    tick: i32,
    index: u32,
    event: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct QuadData {
    layer: u32,
    sprite: SpriteData,
    world: [i32; 12],
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ParticleData {
    pub index: u32,
    pub layer: u32,
    pub age: i32,
    pub lifetime: i32,
    pub position: [i32; 3],
    pub velocity: [i32; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, serde::Serialize, PartialEq, Eq)]
pub struct Stats {
    pub state: u32,
    pub tick: u32,
    pub alive: u32,
    pub spawned: u32,
    pub dropped: u32,
    pub peak: u32,
    pub events: u32,
    pub markers: u32,
    pub skipped_targets: u32,
    pub skipped_events: u32,
    pub dropped_emitters: u32,
    pub diagnostics_dropped: u32,
}
unsafe extern "C" {
    fn epok_preview_create() -> *mut c_void;
    fn epok_preview_destroy(context: *mut c_void);
    fn epok_preview_layer(context: *mut c_void, layer: *const LayerData) -> u32;
    fn epok_preview_track(context: *mut c_void, track: *const TrackData) -> u32;
    fn epok_preview_event(context: *mut c_void, event: *const EventData) -> u32;
    fn epok_preview_start(
        context: *mut c_void,
        id: u64,
        timeline_id: u64,
        duration: i32,
        repeat: u32,
        seed: u32,
        slots: u32,
        required: u32,
        markers: *const u64,
        marker_count: u32,
        signals: *const SignalData,
        signal_count: u32,
    ) -> u32;
    fn epok_preview_step(context: *mut c_void, tick: i32, paused: u32);
    fn epok_preview_stats(context: *mut c_void, stats: *mut Stats);
    fn epok_preview_quads(context: *mut c_void, quads: *mut QuadData, capacity: u32) -> u32;
    fn epok_preview_particles(
        context: *mut c_void,
        particles: *mut ParticleData,
        capacity: u32,
    ) -> u32;
    fn epok_preview_abi(kind: u32) -> u32;
}
static OWNER: Mutex<()> = Mutex::new(());
pub struct Simulation {
    native: NonNull<c_void>,
    _owner: MutexGuard<'static, ()>,
    textures: Vec<Uuid>,
    pub layers: Vec<Uuid>,
    pub resources: crate::scene::Scene,
}
pub struct Quad {
    pub layer: Uuid,
    pub sprite: Sprite,
    pub world: crate::transform::Matrix,
}
/// A bounded, headless trace of the linked production kernels. Q12 values are
/// retained in particle snapshots; floating point is used only for drawing.
pub fn trace(
    root: &std::path::Path,
    path: &std::path::Path,
    steps: u32,
) -> Result<serde_json::Value, String> {
    if steps > 482 {
        return Err("Effect preview is limited to 482 fixed steps (eight seconds)".into());
    }
    let effect = crate::particle_effect::load(path)?;
    let scripts = crate::scripts::catalog(root)?;
    let registry = crate::blueprint::native_registry(root, &scripts)?;
    let index = crate::assets::scan(root, &mut Default::default());
    let mut simulation = Simulation::new(&effect, &registry, &index)?;
    let mut frames = Vec::with_capacity(steps as usize);
    for step in 0..steps {
        simulation.step(68, false);
        let quads = simulation
            .quads()
            .into_iter()
            .map(|q| {
                serde_json::json!({
                    "layer": q.layer, "sprite": q.sprite, "world": q.world.0,
                })
            })
            .collect::<Vec<_>>();
        frames.push(serde_json::json!({"step": step + 1, "stats": simulation.stats(), "particles": simulation.particles(), "quads": quads}));
    }
    Ok(
        serde_json::json!({"version": 1, "asset": effect.id, "signature": effect.semantic_hash(), "tick_q12": 68, "frames": frames}),
    )
}
fn q12(value: f32) -> i32 {
    (value * 4096.).round() as i32
}
fn color(value: f32) -> i32 {
    (value * 255.).round() as i32
}
fn sprite_data(sprite: &Sprite, textures: &[Uuid]) -> SpriteData {
    SpriteData {
        texture: sprite
            .texture
            .and_then(|id| textures.iter().position(|v| *v == id))
            .map_or(-1, |i| i as i32),
        region: sprite.region.map(i32::from),
        size: sprite.size.map(q12),
        pivot: sprite.pivot.map(q12),
        color: sprite.color.map(color),
        enabled: sprite.enabled.into(),
        flip_x: sprite.flip_x.into(),
        flip_y: sprite.flip_y.into(),
        orientation: match sprite.orientation {
            Orientation::Fixed => 0,
            Orientation::Upright => 1,
            Orientation::Spherical => 2,
        },
        unlit: sprite.unlit.into(),
        blend: match sprite.blend {
            BlendMode::Cutout => 0,
            BlendMode::Average => 1,
            BlendMode::Add => 2,
            BlendMode::Subtract => 3,
            BlendMode::AddQuarter => 4,
        },
        depth_bias: i32::from(sprite.depth_bias),
    }
}
fn property_adapter(id: &str) -> Result<u32, String> {
    [
        "af33a70b-5855-4430-8cba-bc2b38119e86",
        "37e43711-dba0-4bca-8499-952a73450d44",
        "775f36b0-2b5b-4920-aa9f-981b166507ef",
        "dbd149e3-2164-4bd8-87bc-92f53bdc0f74",
        "a8ab3325-89dd-416f-a946-082292c297c6",
        "c06779d1-42a0-43ae-8712-fbac4a641bcf",
        "286d0bc0-1d5e-4b47-974d-b87119cb63df",
        "41c3e62b-b9df-4df6-86b8-638e7851c4f3",
    ]
    .iter()
    .position(|v| *v == id)
    .map(|i| i as u32)
    .ok_or_else(|| {
        format!(
            "Property {id} has no linked host-preview adapter; rebuild the editor after SDK changes"
        )
    })
}
impl Simulation {
    pub fn new(
        effect: &ParticleEffect,
        registry: &crate::blueprint::Registry,
        index: &crate::assets::Index,
    ) -> Result<Self, String> {
        let errors = effect.validate(registry);
        if !errors.is_empty() {
            return Err(errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        let compiled = crate::timeline_compile::compile_index(&effect.timeline, registry, index)
            .map_err(|errors| {
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
        let prepared = crate::particle_effect_scene::Prepared {
            source: effect.clone(),
            compiled: compiled.clone(),
            initializers: Default::default(),
        };
        let resources = crate::blueprint_refs::resources(
            &[],
            &[],
            registry,
            index,
            &crate::particle_effect_scene::resources(std::slice::from_ref(&prepared)),
        )?;
        // Apply the production texture/atlas checks before starting a preview.
        crate::particle_effect_scene::header(&prepared, registry, &resources)?;
        Self::from_compiled(effect, &compiled, resources)
    }
    fn from_compiled(
        effect: &ParticleEffect,
        compiled: &Compiled,
        resources: crate::scene::Scene,
    ) -> Result<Self, String> {
        // The SDK owns one typed layer resolver. Never block an editor frame
        // waiting for another preview; independent callers can retry later.
        let owner = OWNER
            .try_lock()
            .map_err(|_| "Another effect preview is active")?;
        for (kind, size) in [
            size_of::<SpriteData>(),
            size_of::<LayerData>(),
            size_of::<TrackData>(),
            size_of::<EventData>(),
            size_of::<SignalData>(),
            size_of::<QuadData>(),
            size_of::<ParticleData>(),
            size_of::<Stats>(),
        ]
        .into_iter()
        .enumerate()
        {
            if unsafe { epok_preview_abi(kind as u32) } as usize != size {
                return Err("Host effect preview ABI mismatch; rebuild the editor".into());
            }
        }
        let native = NonNull::new(unsafe { epok_preview_create() })
            .ok_or("Host effect preview allocation failed")?;
        let mut layers = effect.layers.iter().collect::<Vec<_>>();
        layers.sort_by_key(|l| l.id);
        let mut textures = layers
            .iter()
            .filter_map(|l| match &l.content {
                LayerContent::Sprite { sprite, .. } => sprite.texture,
                LayerContent::Emitter { emitter } => emitter.sprite.texture,
            })
            .collect::<Vec<_>>();
        textures.sort();
        textures.dedup();
        let simulation = Self {
            native,
            _owner: owner,
            textures,
            layers: layers.iter().map(|l| l.id).collect(),
            resources,
        };
        let pointer = simulation.native.as_ptr();
        let slot = |id: Uuid| {
            compiled
                .slots
                .iter()
                .position(|s| s.id == id)
                .map(|i| i as u32)
                .ok_or_else(|| format!("Missing compiled slot {id}"))
        };
        for layer in layers {
            let mut data = LayerData {
                slot: slot(layer.slot)?,
                enabled: layer.enabled.into(),
                playing: 1,
                rate: 8 * 4096,
                velocity: [0, 4096, 0],
                position: layer.position.map(q12),
                frames: 1,
                columns: 1,
                frame_ticks: 410,
                ..Default::default()
            };
            match &layer.content {
                LayerContent::Sprite {
                    sprite,
                    frames,
                    columns,
                    frame_ticks,
                } => {
                    data.sprite = sprite_data(sprite, &simulation.textures);
                    data.frames = (*frames).into();
                    data.columns = (*columns).into();
                    data.frame_ticks = *frame_ticks;
                }
                LayerContent::Emitter { emitter: p } => {
                    data.emitter = 1;
                    data.sprite = sprite_data(&p.sprite, &simulation.textures);
                    data.enabled = (layer.enabled && p.enabled).into();
                    data.playing = p.play_on_start.into();
                    data.continuous = p.continuous.into();
                    data.local_space = p.local_space.into();
                    data.seed = p.seed;
                    data.burst = p.burst.into();
                    data.max_particles = p.max_particles.into();
                    data.velocity = p.velocity.map(q12);
                    data.spread = p.spread.map(q12);
                    data.gravity = p.gravity.map(q12);
                    data.rate = q12(p.rate);
                    data.lifetime = q12(p.lifetime);
                    data.start_size = q12(p.start_size);
                    data.end_size = q12(p.end_size);
                    data.start_color = p.start_color.map(color);
                    data.end_color = p.end_color.map(color);
                    data.frames = p.frames.into();
                    data.columns = p.frame_columns.into();
                    data.frame_ticks = q12(p.frame_duration);
                }
            }
            if unsafe { epok_preview_layer(pointer, &data) } == 0 {
                return Err(format!("Host preview rejected layer {}", layer.id));
            }
        }
        for track in &compiled.tracks {
            let target = slot(track.slot)?;
            let field = if matches!(
                compiled.slots[target as usize].target,
                crate::reflection_schema::Type::EffectLayerRef { .. }
            ) {
                property_adapter(&track.property)?
            } else {
                8
            };
            let mut data = TrackData {
                property: crate::blueprint_refs::compact_id(&track.property),
                slot: target,
                field,
                channels: track.channels.len() as u32,
                additive: (track.blend == crate::timeline::Blend::Additive).into(),
                restore: (track.restore == crate::timeline::Restore::RestoreInitial).into(),
                interpolation: match track.interpolation {
                    crate::timeline::Interpolation::Linear => 0,
                    crate::timeline::Interpolation::Step => 1,
                    crate::timeline::Interpolation::Smoothstep => 2,
                    crate::timeline::Interpolation::EaseIn => 3,
                    crate::timeline::Interpolation::EaseOut => 4,
                },
                ..Default::default()
            };
            for (c, keys) in track.channels.iter().enumerate() {
                data.lengths[c] = keys.len() as u32;
                for (k, (tick, value)) in keys.iter().enumerate() {
                    data.keys[c][k] = KeyData {
                        tick: *tick,
                        value: *value,
                    };
                }
            }
            if unsafe { epok_preview_track(pointer, &data) } == 0 {
                return Err(format!("Host preview rejected track {}", track.id));
            }
        }
        for event in &compiled.events {
            let target = slot(event.slot)?;
            let function = if matches!(
                compiled.slots[target as usize].target,
                crate::reflection_schema::Type::EffectLayerRef { .. }
            ) {
                match event.function.as_str() {
                    "a5cd1cfd-e3a5-4b6e-96d0-c8c5bb5f43f5" => 0,
                    "52f8c306-ee5f-4a2b-957b-bca9d99d9ec3" => 1,
                    "c2650304-c940-408c-a8a8-05641c2a0489" => 2,
                    _ => {
                        return Err(format!(
                            "Function {} needs a linked host-preview adapter",
                            event.function
                        ));
                    }
                }
            } else {
                3
            };
            let data = EventData {
                slot: slot(event.slot)?,
                function,
                count: event.arguments.first().map_or(0, |a| a.lanes[0] as u32),
                idempotent: (event.call
                    == crate::reflection_schema::TimelineCall::IdempotentAction)
                    .into(),
            };
            if unsafe { epok_preview_event(pointer, &data) } == 0 {
                return Err(format!("Host preview rejected event {}", event.key));
            }
        }
        let markers = compiled
            .markers
            .iter()
            .map(|(_, id)| crate::blueprint_refs::compact_id(&id.to_string()))
            .collect::<Vec<_>>();
        let signals = compiled
            .ordered_signals()
            .into_iter()
            .map(|(tick, _, event, index)| SignalData {
                tick,
                index: index as u32,
                event: event.into(),
            })
            .collect::<Vec<_>>();
        let required = compiled
            .slots
            .iter()
            .enumerate()
            .fold(0, |mask, (i, slot)| mask | ((slot.required as u32) << i));
        if unsafe {
            epok_preview_start(
                pointer,
                crate::blueprint_refs::compact_id(&effect.id.to_string()),
                crate::blueprint_refs::compact_id(&compiled.asset.to_string()),
                compiled.duration_ticks,
                (compiled.loop_mode == crate::timeline::LoopMode::Repeat).into(),
                effect.seed,
                compiled.slots.len() as u32,
                required,
                markers.as_ptr(),
                markers.len() as u32,
                signals.as_ptr(),
                signals.len() as u32,
            )
        } == 0
        {
            return Err("Effect preview could not start; required external scene bindings need the PSX Game view".into());
        }
        Ok(simulation)
    }
    pub fn step(&mut self, tick: i32, paused: bool) {
        unsafe { epok_preview_step(self.native.as_ptr(), tick, paused.into()) };
    }
    pub fn stats(&self) -> Stats {
        let mut stats = Stats::default();
        unsafe { epok_preview_stats(self.native.as_ptr(), &mut stats) };
        stats
    }
    pub fn particles(&self) -> Vec<ParticleData> {
        let mut output = vec![ParticleData::default(); 256];
        let count =
            unsafe { epok_preview_particles(self.native.as_ptr(), output.as_mut_ptr(), 256) };
        output.truncate(count as usize);
        output
    }
    pub fn quads(&self) -> Vec<Quad> {
        let mut output = vec![QuadData::default(); 264];
        let count = unsafe { epok_preview_quads(self.native.as_ptr(), output.as_mut_ptr(), 264) };
        output.truncate(count as usize);
        output
            .into_iter()
            .filter_map(|quad| {
                let raw = quad.sprite;
                let sprite = Sprite {
                    texture: usize::try_from(raw.texture)
                        .ok()
                        .and_then(|i| self.textures.get(i).copied()),
                    enabled: raw.enabled != 0,
                    region: raw.region.map(|v| v as u16),
                    size: raw.size.map(|v| v as f32 / 4096.),
                    pivot: raw.pivot.map(|v| v as f32 / 4096.),
                    color: raw.color.map(|v| v as f32 / 255.),
                    flip_x: raw.flip_x != 0,
                    flip_y: raw.flip_y != 0,
                    orientation: match raw.orientation {
                        0 => Orientation::Fixed,
                        1 => Orientation::Upright,
                        _ => Orientation::Spherical,
                    },
                    unlit: raw.unlit != 0,
                    blend: match raw.blend {
                        1 => BlendMode::Average,
                        2 => BlendMode::Add,
                        3 => BlendMode::Subtract,
                        4 => BlendMode::AddQuarter,
                        _ => BlendMode::Cutout,
                    },
                    depth_bias: raw.depth_bias as i16,
                };
                Some(Quad {
                    layer: *self.layers.get(quad.layer as usize)?,
                    sprite,
                    world: crate::transform::Matrix(std::array::from_fn(|r| {
                        std::array::from_fn(|c| quad.world[r * 4 + c] as f32 / 4096.)
                    })),
                })
            })
            .collect()
    }
}
impl Drop for Simulation {
    fn drop(&mut self) {
        unsafe { epok_preview_destroy(self.native.as_ptr()) };
    }
}

/// Editor transport; seeking replays bounded fixed steps from the authored seed.
/// It never reverses the production runtime or dispatches game-side callbacks.
pub struct View {
    pub simulation: Option<Simulation>,
    signature: String,
    error: String,
    pub camera: crate::viewport::View,
    pub background: [f32; 3],
    pub texture: Option<imgui::TextureId>,
    paused: bool,
    step: i32,
    fraction: f32,
    pub quads: Vec<Quad>,
}
impl Default for View {
    fn default() -> Self {
        Self {
            simulation: None,
            signature: String::new(),
            error: String::new(),
            camera: crate::viewport::View {
                yaw: 0.,
                pitch: 0.15,
                distance: 7.,
                center: [0., 1., 0.],
                ..Default::default()
            },
            background: [0.015, 0.02, 0.03],
            texture: None,
            paused: false,
            step: 0,
            fraction: 0.,
            quads: vec![],
        }
    }
}
impl View {
    pub fn clear(&mut self) {
        self.simulation = None;
        self.quads.clear();
        self.signature.clear();
    }
    pub fn draw(
        &mut self,
        ui: &imgui::Ui,
        effect: &ParticleEffect,
        registry: &crate::blueprint::Registry,
        index: &crate::assets::Index,
        stale: Option<&str>,
    ) {
        if let Some(error) = stale {
            self.clear();
            ui.text_colored(
                [1., 0.7, 0.3, 1.],
                format!("Effect preview unavailable: {error}"),
            );
            return;
        }
        let signature = crate::assets::hash(
            &serde_json::to_vec(&(
                effect.semantic_hash(),
                &registry.classes,
                index.fingerprint(),
            ))
            .expect("serializable preview dependencies"),
        );
        let mut restart = signature != self.signature;
        if ui.button(if self.paused {
            "Play preview"
        } else {
            "Pause preview"
        }) {
            self.paused = !self.paused;
        }
        ui.same_line();
        restart |= ui.button("Restart preview");
        ui.same_line();
        ui.text_disabled("Eight-second loop / Q12 fixed steps");
        let mut requested = self.step;
        let seek = ui.slider("Preview step", 0, 482, &mut requested);
        if seek {
            self.paused = true;
            restart = true;
        }
        if restart {
            self.clear();
            self.signature = signature;
            self.step = 0;
            self.fraction = 0.;
            match Simulation::new(effect, registry, index) {
                Ok(simulation) => {
                    self.simulation = Some(simulation);
                    self.error.clear();
                }
                Err(error) => self.error = error,
            }
            if seek && let Some(simulation) = &mut self.simulation {
                for _ in 0..requested {
                    simulation.step(68, false);
                }
                self.step = requested;
            }
        }
        if !self.error.is_empty() {
            ui.text_wrapped(&self.error);
            return;
        }
        if let Some(simulation) = &mut self.simulation {
            if !self.paused {
                // Drop wall-clock catch-up beyond six steps; preview never stalls
                // a frame and every simulated step still has the PSX Q12 duration.
                self.fraction += ui.io().delta_time.min(0.1) * 60.;
                let steps = (self.fraction.floor() as i32).min(6).min(482 - self.step);
                self.fraction -= steps as f32;
                for _ in 0..steps {
                    simulation.step(68, false);
                }
                self.step += steps;
            }
            self.quads = simulation.quads();
            let s = simulation.stats();
            ui.text(format!(
                "Particles {} / 256 | Peak {} | Spawned {} | Dropped {} | Events {} | Markers {}",
                s.alive, s.peak, s.spawned, s.dropped, s.events, s.markers
            ));
            if s.skipped_targets + s.skipped_events + s.dropped_emitters + s.diagnostics_dropped > 0
            {
                ui.text_colored(
                    [1., 0.7, 0.3, 1.],
                    format!(
                        "Skipped targets {} | Events {} | Emitters {} | Dropped diagnostics {}",
                        s.skipped_targets,
                        s.skipped_events,
                        s.dropped_emitters,
                        s.diagnostics_dropped
                    ),
                );
            }
        }
        if ui.collapsing_header(
            "Preview camera and background",
            imgui::TreeNodeFlags::empty(),
        ) {
            ui.slider(
                "Yaw",
                -std::f32::consts::PI,
                std::f32::consts::PI,
                &mut self.camera.yaw,
            );
            ui.slider("Pitch", -1.4, 1.4, &mut self.camera.pitch);
            ui.slider("Distance", 1.1, 30., &mut self.camera.distance);
            crate::gui::Drag::new("Center")
                .speed(0.02)
                .build_array(ui, &mut self.camera.center);
            ui.color_edit3("Background", &mut self.background);
        }
        if let Some(texture) = self.texture {
            let height = (ui.content_region_avail()[1] * 0.45).clamp(140., 300.);
            let width = ui.content_region_avail()[0].clamp(1., height * 960. / 600.);
            imgui::Image::new(texture, [width, width * 600. / 960.]).build(ui);
        }
        if self.step >= 482 && !self.paused {
            self.signature.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_bridge_pause_replay_and_required_bindings() {
        let mut effect = ParticleEffect::new("Preview contract".into());
        effect
            .add_layer(
                "Emitter".into(),
                LayerContent::Emitter {
                    emitter: Box::new(crate::particles::Emitter {
                        continuous: false,
                        burst: 8,
                        lifetime: 1.,
                        ..Default::default()
                    }),
                },
            )
            .unwrap();
        let mut compiled = Compiled {
            asset: effect.timeline.id,
            duration_ticks: 4096,
            loop_mode: crate::timeline::LoopMode::Once,
            slots: effect.timeline.slots.clone(),
            tracks: vec![],
            events: vec![],
            markers: vec![],
            dependencies: Default::default(),
            signature: String::new(),
        };
        let mut simulation =
            Simulation::from_compiled(&effect, &compiled, Default::default()).unwrap();
        for _ in 0..12 {
            simulation.step(68, false);
        }
        let particles = simulation.particles();
        let stats = simulation.stats();
        assert_eq!(particles.len(), 8);
        assert_eq!(simulation.quads().len(), 8);
        for _ in 0..8 {
            simulation.step(68, true);
        }
        assert_eq!(simulation.particles(), particles);
        assert_eq!(simulation.stats(), stats);
        assert_eq!(
            simulation.quads().len(),
            8,
            "Paused particles remain visible"
        );
        assert!(Simulation::from_compiled(&effect, &compiled, Default::default()).is_err());
        drop(simulation);
        let mut replay = Simulation::from_compiled(&effect, &compiled, Default::default()).unwrap();
        for _ in 0..12 {
            replay.step(68, false);
        }
        assert_eq!(replay.particles(), particles);
        assert_eq!(replay.stats(), stats);
        drop(replay);
        compiled.slots.push(crate::timeline::Slot {
            id: Uuid::new_v4(),
            name: "External".into(),
            target: crate::reflection_schema::Type::ObjectRef { class: None },
            required: true,
            extra: Default::default(),
        });
        assert!(Simulation::from_compiled(&effect, &compiled, Default::default()).is_err());
        compiled.slots.last_mut().unwrap().required = false;
        compiled
            .tracks
            .push(crate::timeline_compile::CompiledTrack {
                id: Uuid::new_v4(),
                slot: compiled.slots.last().unwrap().id,
                property: Uuid::new_v4().to_string(),
                class: "External".into(),
                field: "value".into(),
                priority: 0,
                restore: crate::timeline::Restore::LeaveFinal,
                channels: vec![vec![(0, 0), (4096, 4096)]],
                value_type: crate::reflection_schema::Type::Fixed,
                interpolation: crate::timeline::Interpolation::Linear,
                blend: crate::timeline::Blend::Absolute,
                key_ids: vec![],
            });
        let mut optional =
            Simulation::from_compiled(&effect, &compiled, Default::default()).unwrap();
        optional.step(68, false);
        assert_eq!(optional.stats().alive, 8);
        assert!(optional.stats().skipped_targets > 0);
    }
}
