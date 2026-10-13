//! Skeleton2D records (World2D initiative, phase 4): cut-out skeletons whose bones are
//! ordinary 2D actors, pose-key animations and two-bone IK chains. Mirrors
//! `runtime/world2d_skeleton.hpp`; sampling here matches the runtime so the editor's
//! scrubbing and preview show what the console plays.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_BONES: usize = 24;
pub const MAX_ANIMATIONS: usize = 16;
pub const MAX_KEYS: usize = 64;
pub const MAX_IK: usize = 2;

/// One bone: an actor of the map and its rest pose (rotation in degrees, local x, y).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Bone {
    pub actor: Option<Uuid>,
    pub rest: [f32; 3],
}
/// A pose at `time` seconds: (rotation, x, y) for every bone, in bone order.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Key {
    pub time: f32,
    pub pose: Vec<[f32; 3]>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Animation {
    pub name: String,
    pub length: f32,
    pub looping: bool,
    pub keys: Vec<Key>,
}
impl Default for Animation {
    fn default() -> Self {
        Self {
            name: "idle".into(),
            length: 1.,
            looping: true,
            keys: Vec::new(),
        }
    }
}
/// A two-bone IK chain: `upper` and `lower` index the bones; `tip` is the actor at the end
/// of the lower bone and `target` the actor it reaches for.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Ik {
    pub upper: usize,
    pub lower: usize,
    pub tip: Option<Uuid>,
    pub target: Option<Uuid>,
    pub bend_positive: bool,
    pub enabled: bool,
}
impl Default for Ik {
    fn default() -> Self {
        Self {
            upper: 0,
            lower: 1,
            tip: None,
            target: None,
            bend_positive: true,
            enabled: true,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Skeleton2D {
    pub bones: Vec<Bone>,
    pub animations: Vec<Animation>,
    /// The animation that starts playing with the map, if any.
    pub autoplay: Option<usize>,
    pub speed: f32,
    pub ik: Vec<Ik>,
}
impl Default for Skeleton2D {
    fn default() -> Self {
        Self {
            bones: Vec::new(),
            animations: Vec::new(),
            autoplay: None,
            speed: 1.,
            ik: Vec::new(),
        }
    }
}

/// An angle in degrees brought into (-180, 180].
pub fn wrap_degrees(value: f32) -> f32 {
    let mut v = value % 360.;
    if v > 180. {
        v -= 360.;
    }
    if v <= -180. {
        v += 360.;
    }
    v
}

impl Skeleton2D {
    pub fn validate(&self) -> Result<(), String> {
        if self.bones.len() > MAX_BONES {
            return Err(format!("a Skeleton2D holds at most {MAX_BONES} bones"));
        }
        if self.animations.len() > MAX_ANIMATIONS {
            return Err(format!(
                "a Skeleton2D holds at most {MAX_ANIMATIONS} animations"
            ));
        }
        if self.ik.len() > MAX_IK {
            return Err(format!("a Skeleton2D holds at most {MAX_IK} IK chains"));
        }
        for a in &self.animations {
            if a.keys.len() > MAX_KEYS {
                return Err(format!(
                    "animation {:?} has more than {MAX_KEYS} keys",
                    a.name
                ));
            }
            if a.length.is_nan() || a.length <= 0. || a.length > 600. {
                return Err(format!(
                    "animation {:?} must last between 0 and 600 seconds",
                    a.name
                ));
            }
        }
        if let Some(i) = self.autoplay
            && i >= self.animations.len()
        {
            return Err("the autoplay animation does not exist".into());
        }
        for chain in &self.ik {
            if chain.upper >= self.bones.len() || chain.lower >= self.bones.len() {
                return Err("an IK chain names a bone the skeleton does not have".into());
            }
            if chain.upper == chain.lower {
                return Err("an IK chain needs two different bones".into());
            }
        }
        Ok(())
    }
    /// Keys sorted by time and inside the animation, every pose one entry per bone (a
    /// missing entry takes the bone's rest pose). Run after every edit.
    pub fn normalize(&mut self) {
        let rest: Vec<[f32; 3]> = self.bones.iter().map(|b| b.rest).collect();
        for a in &mut self.animations {
            a.length = a.length.clamp(0.05, 600.);
            for key in &mut a.keys {
                key.time = key.time.clamp(0., a.length);
                key.pose.truncate(rest.len());
                while key.pose.len() < rest.len() {
                    key.pose.push(rest[key.pose.len()]);
                }
            }
            a.keys.sort_by(|x, y| x.time.total_cmp(&y.time));
            a.keys.dedup_by(|x, y| (x.time - y.time).abs() < 1e-4);
        }
        if self.autoplay.is_some_and(|i| i >= self.animations.len()) {
            self.autoplay = None;
        }
    }
    /// The pose of animation `index` at `time`, exactly as the runtime blends it.
    pub fn sample(&self, index: usize, time: f32) -> Option<Vec<[f32; 3]>> {
        let a = self.animations.get(index)?;
        let first = a.keys.first()?;
        let last = a.keys.last()?;
        let time = if a.looping {
            time.rem_euclid(a.length)
        } else {
            time.clamp(0., a.length)
        };
        if a.keys.len() == 1 {
            return Some(first.pose.clone());
        }
        let (from, to, span, into) = if time < first.time {
            if !a.looping {
                return Some(first.pose.clone());
            }
            (
                last,
                first,
                a.length - last.time + first.time,
                a.length - last.time + time,
            )
        } else if let Some(k) = a.keys.windows(2).position(|w| time < w[1].time) {
            let (x, y) = (&a.keys[k], &a.keys[k + 1]);
            (x, y, y.time - x.time, time - x.time)
        } else {
            if !a.looping {
                return Some(last.pose.clone());
            }
            (
                last,
                first,
                a.length - last.time + first.time,
                time - last.time,
            )
        };
        let f = if span > 0. {
            (into / span).clamp(0., 1.)
        } else {
            0.
        };
        Some(
            from.pose
                .iter()
                .zip(&to.pose)
                .map(|(p, q)| {
                    [
                        wrap_degrees(p[0] + wrap_degrees(q[0] - p[0]) * f),
                        p[1] + (q[1] - p[1]) * f,
                        p[2] + (q[2] - p[2]) * f,
                    ]
                })
                .collect(),
        )
    }
    /// Records `pose` as the key at `time` of animation `index`, replacing a key within a
    /// frame of it. Returns the key's index.
    pub fn key_pose(&mut self, index: usize, time: f32, pose: Vec<[f32; 3]>) -> Option<usize> {
        let a = self.animations.get_mut(index)?;
        let time = time.clamp(0., a.length);
        if let Some(i) = a
            .keys
            .iter()
            .position(|k| (k.time - time).abs() < 1. / 120.)
        {
            a.keys[i].pose = pose;
            return Some(i);
        }
        if a.keys.len() >= MAX_KEYS {
            return None;
        }
        a.keys.push(Key { time, pose });
        a.keys.sort_by(|x, y| x.time.total_cmp(&y.time));
        a.keys.iter().position(|k| (k.time - time).abs() < 1e-6)
    }
}

impl Skeleton2D {
    /// Removes bone `index` from the skeleton, its keys and its IK chains.
    pub fn remove_bone(&mut self, index: usize) {
        if index >= self.bones.len() {
            return;
        }
        self.bones.remove(index);
        for a in &mut self.animations {
            for key in &mut a.keys {
                if index < key.pose.len() {
                    key.pose.remove(index);
                }
            }
        }
        self.ik.retain(|c| c.upper != index && c.lower != index);
        for c in &mut self.ik {
            c.upper -= usize::from(c.upper > index);
            c.lower -= usize::from(c.lower > index);
        }
    }
}

fn angle(v: [f32; 2]) -> f32 {
    v[1].atan2(v[0]).to_degrees()
}
fn rotate_about(p: [f32; 2], centre: [f32; 2], degrees: f32) -> [f32; 2] {
    let (s, c) = degrees.to_radians().sin_cos();
    let (x, y) = (p[0] - centre[0], p[1] - centre[1]);
    [centre[0] + c * x - s * y, centre[1] + s * x + c * y]
}
/// The rotations (degrees) to add to the upper and the lower bone of a two-bone chain so
/// its tip reaches `target`, as the runtime solves it. Positions are world pixels; `None`
/// when a bone has no length.
pub fn ik_turns(
    upper: [f32; 2],
    lower: [f32; 2],
    tip: [f32; 2],
    target: [f32; 2],
    bend_positive: bool,
) -> Option<(f32, f32)> {
    let length = |a: [f32; 2], b: [f32; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
    let (a, b) = (length(upper, lower), length(lower, tip));
    if a < 1e-3 || b < 1e-3 {
        return None;
    }
    let d = length(upper, target).clamp((a - b).abs() + 1e-4, a + b - 1e-4);
    let opening = ((a * a + d * d - b * b) / (2. * a * d))
        .clamp(-1., 1.)
        .acos()
        .to_degrees();
    let base = angle([target[0] - upper[0], target[1] - upper[1]]);
    let wanted = base + if bend_positive { opening } else { -opening };
    let first = wrap_degrees(wanted - angle([lower[0] - upper[0], lower[1] - upper[1]]));
    let (lower, tip) = (
        rotate_about(lower, upper, first),
        rotate_about(tip, upper, first),
    );
    let second = wrap_degrees(
        angle([target[0] - lower[0], target[1] - lower[1]])
            - angle([tip[0] - lower[0], tip[1] - lower[1]]),
    );
    Some((first, second))
}

/// Generated constants and setter statements for one Skeleton2D component.
pub fn cook(
    skeleton: &Skeleton2D,
    symbol: &str,
    actor: &dyn Fn(Option<Uuid>) -> Option<usize>,
    fixed: &dyn Fn(f32) -> String,
    label: &str,
) -> Result<(String, String), String> {
    let mut skeleton = skeleton.clone();
    skeleton.normalize();
    skeleton.validate().map_err(|e| format!("{label}: {e}"))?;
    let list =
        |values: &mut dyn Iterator<Item = f32>| values.map(fixed).collect::<Vec<_>>().join(",");
    let mut statics = String::new();
    let mut body = String::new();
    let mut bones = 0;
    for (i, bone) in skeleton.bones.iter().enumerate() {
        let slot = actor(bone.actor)
            .ok_or_else(|| format!("{label}: bone {i} has no actor in this map"))?;
        body += &format!("c->bones[{i}]=actors[{slot}];");
        bones += 1;
    }
    body += &format!("c->bone_count={bones};");
    if bones > 0 {
        statics += &format!(
            "inline constexpr epok::Fixed {symbol}_rest[]={{{}}};\n",
            list(&mut skeleton.bones.iter().flat_map(|b| b.rest))
        );
        body += &format!("c->rest={symbol}_rest;");
    }
    let mut clips = Vec::new();
    for (n, a) in skeleton.animations.iter().enumerate() {
        if a.keys.is_empty() || bones == 0 {
            clips.push(format!(
                "epok::SkeletonClip2D{{nullptr,nullptr,0,{},{}}}",
                a.looping,
                fixed(a.length)
            ));
            continue;
        }
        statics += &format!(
            "inline constexpr epok::Fixed {symbol}_a{n}_times[]={{{}}};\ninline constexpr epok::Fixed {symbol}_a{n}_poses[]={{{}}};\n",
            list(&mut a.keys.iter().map(|k| k.time)),
            list(&mut a.keys.iter().flat_map(|k| k.pose.iter().flat_map(|p| *p)))
        );
        clips.push(format!(
            "epok::SkeletonClip2D{{{symbol}_a{n}_times,{symbol}_a{n}_poses,{},{},{}}}",
            a.keys.len(),
            a.looping,
            fixed(a.length)
        ));
    }
    if !clips.is_empty() {
        statics += &format!(
            "inline constexpr epok::SkeletonClip2D {symbol}_clips[]={{{}}};\n",
            clips.join(",")
        );
        body += &format!("c->clips={symbol}_clips;c->clip_count={};", clips.len());
    }
    let mut chains = 0;
    for chain in &skeleton.ik {
        let (Some(tip), Some(target)) = (actor(chain.tip), actor(chain.target)) else {
            continue;
        };
        body += &format!(
            "c->ik[{chains}]=epok::SkeletonIk2D{{{},{},actors[{tip}],actors[{target}],{},{}}};",
            chain.upper, chain.lower, chain.bend_positive, chain.enabled
        );
        chains += 1;
    }
    body += &format!("c->ik_count={chains};c->speed={};", fixed(skeleton.speed));
    // Autoplay starts at the first simulation step, once every bone actor exists.
    if let Some(i) = skeleton.autoplay.filter(|_| bones > 0) {
        body += &format!("c->clip={i};c->playing=true;");
    }
    Ok((statics, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk() -> Skeleton2D {
        let mut s = Skeleton2D {
            bones: vec![
                Bone {
                    actor: Some(Uuid::from_u128(1)),
                    rest: [0., 10., 0.],
                },
                Bone {
                    actor: Some(Uuid::from_u128(2)),
                    rest: [0., 0., 8.],
                },
            ],
            animations: vec![Animation {
                name: "walk".into(),
                length: 2.,
                ..Default::default()
            }],
            ..Default::default()
        };
        s.key_pose(0, 0., vec![[0., 10., 0.], [170., 0., 8.]]);
        s.key_pose(0, 1., vec![[90., 10., 4.], [-170., 0., 8.]]);
        s
    }

    #[test]
    fn sampling_blends_poses_along_the_shorter_arc_and_loops() {
        let s = walk();
        let mid = s.sample(0, 0.5).unwrap();
        assert!((mid[0][0] - 45.).abs() < 1e-4 && (mid[0][2] - 2.).abs() < 1e-4);
        assert!((mid[1][0].abs() - 180.).abs() < 1e-3);
        // After the last key a loop blends back into the first.
        let back = s.sample(0, 1.5).unwrap();
        assert!((back[0][0] - 45.).abs() < 1e-4);
        assert_eq!(s.sample(0, 2.5), s.sample(0, 0.5));
    }

    #[test]
    fn keys_replace_within_a_frame_and_stay_sorted() {
        let mut s = walk();
        assert_eq!(s.key_pose(0, 1.004, vec![[30., 0., 0.]; 2]), Some(1));
        assert_eq!(s.animations[0].keys.len(), 2);
        assert_eq!(s.key_pose(0, 0.5, vec![[10., 0., 0.]; 2]), Some(1));
        let times: Vec<f32> = s.animations[0].keys.iter().map(|k| k.time).collect();
        assert_eq!(times, vec![0., 0.5, 1.]);
        // A bone added later fills old keys from its rest pose.
        s.bones.push(Bone {
            actor: Some(Uuid::from_u128(3)),
            rest: [5., 1., 2.],
        });
        s.normalize();
        assert!(
            s.animations[0]
                .keys
                .iter()
                .all(|k| k.pose.len() == 3 && k.pose[2] == [5., 1., 2.])
        );
    }

    #[test]
    fn validation_bounds_bones_animations_and_chains() {
        let mut s = walk();
        assert!(s.validate().is_ok());
        s.ik.push(Ik {
            upper: 0,
            lower: 0,
            ..Default::default()
        });
        assert!(s.validate().is_err());
        s.ik[0].lower = 5;
        assert!(s.validate().is_err());
        s.ik.clear();
        s.autoplay = Some(3);
        assert!(s.validate().is_err());
        s.autoplay = None;
        s.bones = vec![Bone::default(); MAX_BONES + 1];
        assert!(s.validate().is_err());
    }

    #[test]
    fn removing_a_bone_trims_keys_and_chains() {
        let mut s = walk();
        s.bones.push(Bone::default());
        s.normalize();
        s.ik = vec![
            Ik {
                upper: 1,
                lower: 2,
                ..Default::default()
            },
            Ik {
                upper: 0,
                lower: 1,
                ..Default::default()
            },
        ];
        s.remove_bone(0);
        assert_eq!(s.bones.len(), 2);
        assert!(s.animations[0].keys.iter().all(|k| k.pose.len() == 2));
        assert_eq!(s.animations[0].keys[0].pose[0], [170., 0., 8.]);
        assert_eq!(s.ik.len(), 1);
        assert_eq!((s.ik[0].upper, s.ik[0].lower), (0, 1));
    }

    #[test]
    fn ik_turns_put_the_tip_on_the_target_on_either_side() {
        let (upper, lower, tip) = ([0., 0.], [10., 0.], [20., 0.]);
        for bend in [true, false] {
            let (first, second) = ik_turns(upper, lower, tip, [10., 10.], bend).unwrap();
            let lower2 = rotate_about(lower, upper, first);
            let tip2 = rotate_about(rotate_about(tip, upper, first), lower2, second);
            assert!((tip2[0] - 10.).abs() < 0.01 && (tip2[1] - 10.).abs() < 0.01);
            // Positive bends the elbow to the right of the shoulder-target line (+Y side here).
            assert_eq!(lower2[1] - lower2[0] > 0., bend, "{lower2:?}");
        }
        // Out of reach: straight at the target.
        let (first, second) = ik_turns(upper, lower, tip, [0., 100.], true).unwrap();
        // Within half a degree: the chain stops a hair short of fully straight.
        assert!(
            (first - 90.).abs() < 0.5 && second.abs() < 0.5,
            "{first} {second}"
        );
        assert!(ik_turns(upper, upper, tip, [1., 1.], true).is_none());
    }

    #[test]
    fn cooking_emits_bones_rest_clips_and_chains() {
        let mut s = walk();
        s.autoplay = Some(0);
        s.ik.push(Ik {
            tip: Some(Uuid::from_u128(3)),
            target: Some(Uuid::from_u128(4)),
            ..Default::default()
        });
        let slot = |id: Option<Uuid>| id.map(|id| id.as_u128() as usize + 10);
        let fixed = |v: f32| format!("F({})", (v * 4096.).round() as i32);
        let (statics, body) = cook(&s, "w2d_1_2", &slot, &fixed, "test").unwrap();
        assert!(body.contains("c->bones[0]=actors[11];c->bones[1]=actors[12];c->bone_count=2;"));
        assert!(statics.contains("w2d_1_2_rest[]={F(0),F(40960),F(0),F(0),F(0),F(32768)}"));
        assert!(statics.contains("w2d_1_2_a0_times[]={F(0),F(4096)}"));
        assert!(
            statics
                .contains("epok::SkeletonClip2D{w2d_1_2_a0_times,w2d_1_2_a0_poses,2,true,F(8192)}")
        );
        assert!(body.contains(
            "c->ik[0]=epok::SkeletonIk2D{0,1,actors[13],actors[14],true,true};c->ik_count=1;"
        ));
        assert!(body.ends_with("c->clip=0;c->playing=true;"));
        // A bone without an actor in the map is an error, not a silent gap.
        s.bones[1].actor = None;
        assert!(cook(&s, "x", &slot, &fixed, "test").is_err());
    }
}
