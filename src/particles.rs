//! Bounded deterministic particle simulation shared by authoring preview and export.
use crate::{sprites::Sprite, transform::Matrix};
use serde::{Deserialize, Serialize};
pub const GLOBAL_LIMIT: usize = 256;
pub const EMITTER_LIMIT: usize = 64;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Emitter {
    pub enabled: bool,
    pub play_on_start: bool,
    pub continuous: bool,
    pub rate: f32,
    pub burst: u16,
    pub max_particles: u16,
    pub lifetime: f32,
    pub velocity: [f32; 3],
    pub spread: [f32; 3],
    pub gravity: [f32; 3],
    pub start_size: f32,
    pub end_size: f32,
    pub start_color: [f32; 3],
    pub end_color: [f32; 3],
    pub local_space: bool,
    pub seed: u32,
    pub sprite: Sprite,
    /// Horizontal cells beginning at sprite.region; zero/one disables flipbook.
    pub frames: u16,
    pub frame_columns: u16,
    pub frame_duration: f32,
}
impl Default for Emitter {
    fn default() -> Self {
        Self {
            enabled: true,
            play_on_start: true,
            continuous: true,
            rate: 8.,
            burst: 8,
            max_particles: 32,
            lifetime: 1.,
            velocity: [0., 1., 0.],
            spread: [0.3, 0.2, 0.3],
            gravity: [0., -0.2, 0.],
            start_size: 0.25,
            end_size: 0.05,
            start_color: [1., 0.6, 0.15],
            end_color: [0.15, 0.05, 0.],
            local_space: false,
            seed: 1,
            sprite: Sprite {
                unlit: true,
                orientation: crate::sprites::Orientation::Spherical,
                ..Sprite::default()
            },
            frames: 1,
            frame_columns: 1,
            frame_duration: 0.1,
        }
    }
}
pub fn validate(scene: &crate::scene::Scene) -> Result<(), String> {
    if scene
        .actors
        .iter()
        .filter(|e| e.particle_emitter.is_some())
        .count()
        > EMITTER_LIMIT
    {
        return Err("Maximum 64 particle emitters per scene".into());
    }
    for e in &scene.actors {
        if let Some(p) = &e.particle_emitter {
            validate_emitter(p)?;
            crate::texture::validate_region(p.sprite.texture, p.sprite.region, scene)?;
            if p.frames > 1 {
                let mut end = p.sprite.region;
                let last = p.frames - 1;
                end[0] += (p.frame_columns.min(p.frames) - 1) * end[2];
                end[1] += (last / p.frame_columns) * end[3];
                crate::texture::validate_region(p.sprite.texture, end, scene)?;
            }
        }
    }
    Ok(())
}
pub fn validate_emitter(p: &Emitter) -> Result<(), String> {
    crate::sprites::validate_sprite(&p.sprite)?;
    if p.max_particles == 0
        || p.max_particles > 128
        || p.burst > 128
        || !p.rate.is_finite()
        || !(0. ..=512.).contains(&p.rate)
        || !p.lifetime.is_finite()
        || !(1. / 60. ..=60.).contains(&p.lifetime)
        || [p.start_size, p.end_size]
            .iter()
            .any(|v| !v.is_finite() || !(0. ..=32.).contains(v))
        || p.velocity
            .iter()
            .chain(&p.spread)
            .chain(&p.gravity)
            .any(|v| !v.is_finite() || v.abs() > 128.)
        || p.spread.iter().any(|v| *v < 0.)
        || p.start_color
            .iter()
            .chain(&p.end_color)
            .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
        || p.frames == 0
        || p.frames > 256
        || p.frame_columns == 0
        || p.frame_columns > 256
        || !p.frame_duration.is_finite()
        || !(1. / 60. ..=60.).contains(&p.frame_duration)
    {
        return Err("Invalid particle limits, lifetime, velocity, size, color or flipbook".into());
    }
    if p.frames > 1 {
        let [x, y, w, h] = p.sprite.region;
        let cols = p.frame_columns.min(p.frames) as u32;
        let rows = (p.frames as u32).div_ceil(p.frame_columns as u32);
        if w == 0 || h == 0 || x as u32 + w as u32 * cols > 256 || y as u32 + h as u32 * rows > 256
        {
            return Err("Particle flipbook cells must fit a 256×256 atlas".into());
        }
    }
    Ok(())
}
#[derive(Clone, Debug)]
pub struct Particle {
    pub owner: usize,
    pub age: f32,
    pub lifetime: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub config: Emitter,
}
#[derive(Default)]
pub struct Pool {
    pub particles: Vec<Particle>,
    pub dropped: u32,
    states: std::collections::BTreeMap<usize, (f32, u32, bool)>,
}
impl Pool {
    pub fn burst(&mut self, owner: usize, p: &Emitter, world: Matrix, count: u16) {
        let state = self
            .states
            .entry(owner)
            .or_insert((0., p.seed.max(1), false));
        let current = self.particles.iter().filter(|v| v.owner == owner).count();
        let room = (p.max_particles as usize)
            .saturating_sub(current)
            .min(GLOBAL_LIMIT.saturating_sub(self.particles.len()));
        let accepted = (count as usize).min(room);
        self.dropped = self.dropped.saturating_add(count as u32 - accepted as u32);
        for _ in 0..accepted {
            let mut velocity = p.velocity;
            for (i, v) in velocity.iter_mut().enumerate() {
                state.1 = state.1.wrapping_mul(1664525).wrapping_add(1013904223);
                let unit = ((state.1 >> 16) & 65535) as f32 / 32768. - 1.;
                *v += unit * p.spread[i];
            }
            let position = if p.local_space {
                [0.; 3]
            } else {
                world.point([0.; 3])
            };
            if !p.local_space {
                velocity = world.vector(velocity);
            }
            self.particles.push(Particle {
                owner,
                age: 0.,
                lifetime: p.lifetime,
                position,
                velocity,
                config: p.clone(),
            });
        }
    }
    pub fn advance(&mut self, scene: &crate::scene::Scene, dt: f32) {
        if !dt.is_finite() || dt <= 0. {
            return;
        }
        let dt = dt.min(0.1);
        self.particles.retain_mut(|v| {
            if scene
                .actors
                .get(v.owner)
                .and_then(|e| e.particle_emitter.as_ref())
                .is_none_or(|p| !p.enabled)
            {
                return false;
            }
            v.age += dt;
            if v.age >= v.lifetime {
                return false;
            }
            for i in 0..3 {
                v.velocity[i] += v.config.gravity[i] * dt;
                v.position[i] += v.velocity[i] * dt;
            }
            true
        });
        for (i, e) in scene.actors.iter().enumerate() {
            if !scene.is_active(i) {
                continue;
            }
            let Some(p) = e
                .particle_emitter
                .as_ref()
                .filter(|p| p.enabled && p.play_on_start)
            else {
                continue;
            };
            let s = self.states.entry(i).or_insert((0., p.seed.max(1), false));
            let mut count = 0;
            if !s.2 {
                s.2 = true;
                if !p.continuous {
                    count = p.burst;
                }
            }
            if p.continuous {
                s.0 += p.rate * dt;
                let due = s.0.floor();
                s.0 -= due;
                count = due.min(512.) as u16;
            }
            self.burst(i, p, scene.world_matrix(i), count);
        }
    }
    pub fn quads(
        &self,
        scene: &crate::scene::Scene,
        camera: [[f32; 3]; 3],
    ) -> Vec<crate::sprites::PreviewQuad> {
        self.particles
            .iter()
            .filter(|p| scene.is_active(p.owner))
            .map(|p| {
                let t = (p.age / p.lifetime).clamp(0., 1.);
                let mut sprite = p.config.sprite.clone();
                let size = p.config.start_size + (p.config.end_size - p.config.start_size) * t;
                sprite.size = sprite.size.map(|v| v * size);
                sprite.color = std::array::from_fn(|i| {
                    sprite.color[i]
                        * (p.config.start_color[i]
                            + (p.config.end_color[i] - p.config.start_color[i]) * t)
                });
                if p.config.frames > 1 {
                    let frame = ((p.age / p.config.frame_duration) as u16).min(p.config.frames - 1);
                    sprite.region[0] += (frame % p.config.frame_columns) * sprite.region[2];
                    sprite.region[1] += (frame / p.config.frame_columns) * sprite.region[3];
                }
                let local = Matrix::trs(p.position, [0.; 3], [1.; 3]);
                let world = if p.config.local_space {
                    scene.world_matrix(p.owner).compose(local)
                } else {
                    local
                };
                crate::sprites::PreviewQuad {
                    owner: p.owner,
                    points: crate::sprites::corners(&sprite, world, camera),
                    sprite,
                }
            })
            .collect()
    }
}
/// Authoring preview loops on a bounded eight-second timeline, exactly 60 simulation steps/s.
pub fn preview(
    scene: &crate::scene::Scene,
    camera: [[f32; 3]; 3],
    seconds: f32,
) -> Vec<crate::sprites::PreviewQuad> {
    if !scene.actors.iter().enumerate().any(|(index, actor)| {
        scene.is_active(index)
            && actor
                .particle_emitter
                .as_ref()
                .is_some_and(|emitter| emitter.enabled && emitter.play_on_start)
    }) {
        return vec![];
    }
    struct Cache {
        scene: crate::scene::Scene,
        pool: Pool,
        step: usize,
    }
    thread_local! {static CACHE:std::cell::RefCell<Option<Cache>>=const {std::cell::RefCell::new(None)};}
    let step = if seconds.is_finite() {
        (seconds.max(0.).rem_euclid(8.) * 60.).floor() as usize
    } else {
        0
    };
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let reset = cache
            .as_ref()
            .is_none_or(|old| old.scene != *scene || step < old.step);
        if reset {
            *cache = Some(Cache {
                scene: scene.clone(),
                pool: Pool::default(),
                step: 0,
            });
        }
        let cache = cache.as_mut().unwrap();
        // Rebuild only after a scene edit or timeline wrap. Normal viewport frames
        // update at most eight ticks and camera motion merely recomputes quads.
        let ticks = if reset {
            step
        } else {
            step.saturating_sub(cache.step).min(8)
        };
        for _ in 0..ticks {
            cache.pool.advance(scene, 1. / 60.);
        }
        cache.step = step;
        cache.pool.quads(scene, camera)
    })
}
pub fn generate(scene: &crate::scene::Scene, ids: &[uuid::Uuid]) -> String {
    let mut out = String::new();
    for (i, e) in scene.actors.iter().enumerate() {
        if let Some(p) = &e.particle_emitter {
            let texture = p
                .sprite
                .texture
                .and_then(|id| ids.iter().position(|v| *v == id))
                .map_or(-1, |v| v as i32);
            out += &cpp_emitter(p, &format!("objects[{i}].particle_emitter"), texture);
        }
    }
    out
}
/// The same typed initializer serves scene emitters and immutable effect layers.
pub fn cpp_emitter(p: &Emitter, target: &str, texture: i32) -> String {
    let fixed = |v: f32| format!("Fixed({},Fixed::RAW)", (v * 4096.).round() as i32);
    let vec = |v: [f32; 3]| v.map(fixed).join(",");
    let col = |v: [f32; 3]| v.map(|v| ((v * 255.).round() as u8).to_string()).join(",");
    let mut out = format!(
        "{{auto& p={target};p.enabled={};p.playing={};p.continuous={};p.rate={};p.burst_count={};p.max_particles={};p.lifetime={};p.start_size={};p.end_size={};p.local_space={};p.seed={};p.sprite={};p.frames={};p.frame_columns={};p.frame_duration={};",
        p.enabled,
        p.play_on_start,
        p.continuous,
        fixed(p.rate),
        p.burst,
        p.max_particles,
        fixed(p.lifetime),
        fixed(p.start_size),
        fixed(p.end_size),
        p.local_space,
        p.seed,
        crate::sprites::cpp_sprite(&p.sprite, texture),
        p.frames,
        p.frame_columns,
        fixed(p.frame_duration)
    );
    out += &format!(
        "Fixed velocity[]={{{}}},spread[]={{{}}},gravity[]={{{}}};uint8_t start[]={{{}}},end[]={{{}}};for(int c=0;c<3;++c){{p.velocity[c]=velocity[c];p.spread[c]=spread[c];p.gravity[c]=gravity[c];p.start_color[c]=start[c];p.end_color[c]=end[c];}}}}\n",
        vec(p.velocity),
        vec(p.spread),
        vec(p.gravity),
        col(p.start_color),
        col(p.end_color)
    );
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pool_limits_recycle_and_seed_is_repeatable() {
        let mut a = Pool::default();
        let mut b = Pool::default();
        let p = Emitter {
            max_particles: 4,
            ..Emitter::default()
        };
        a.burst(0, &p, Matrix::IDENTITY, 10);
        b.burst(0, &p, Matrix::IDENTITY, 10);
        assert_eq!(a.particles.len(), 4);
        assert_eq!(a.dropped, 6);
        assert_eq!(a.particles[0].velocity, b.particles[0].velocity);
        let mut scene = crate::scene::Scene::default();
        scene.actors[0].particle_emitter = Some(Emitter {
            play_on_start: false,
            ..p
        });
        for _ in 0..61 {
            a.advance(&scene, 1. / 60.);
        }
        assert!(a.particles.is_empty());
    }
    #[test]
    fn global_budget_and_world_space_emission() {
        let mut pool = Pool::default();
        let p = Emitter {
            max_particles: 128,
            spread: [0.; 3],
            ..Emitter::default()
        };
        for i in 0..4 {
            pool.burst(i, &p, Matrix::trs([3., 4., 5.], [0.; 3], [2.; 3]), 128);
        }
        assert_eq!(pool.particles.len(), GLOBAL_LIMIT);
        assert_eq!(pool.dropped, 256);
        assert_eq!(pool.particles[0].position, [3., 4., 5.]);
        assert_eq!(pool.particles[0].velocity, [0., 2., 0.]);
    }
    #[test]
    fn preview_advances_once_per_tick_and_respects_parent_activation() {
        let mut scene = crate::scene::Scene::default();
        scene.actors[1].particle_emitter = Some(Emitter {
            rate: 60.,
            lifetime: 2.,
            ..Emitter::default()
        });
        let camera = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        let first = preview(&scene, camera, 0.5);
        let again = preview(&scene, camera, 0.5);
        assert_eq!(first.len(), 30);
        assert_eq!(first[0].points, again[0].points);
        scene.actors[1].parent = Some(3);
        scene.actors[3].active = false;
        assert!(preview(&scene, camera, 0.5).is_empty());
        scene.actors[3].active = true;
        assert_eq!(preview(&scene, camera, 0.5).len(), 30);
    }
}
