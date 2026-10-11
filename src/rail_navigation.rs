//! Player-driven, explicitly linked camera rails. No source-game identifiers.
use crate::{scene::Scene, visibility_track::Track};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub from_end: bool,
    pub route: u16,
    pub to_end: bool,
    pub enabled: bool,
    /// Discontinuous portal/teleport: retain the displayed frame while loading
    /// the destination, rather than requiring both zones resident together.
    #[serde(default)]
    pub load_cut: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visibility_track::Sample;
    #[test]
    fn graph_pairs_include_disabled_links_and_preserve_reverse_endpoints() {
        let mut scene = Scene::default();
        scene.actors.truncate(2);
        let mut track = Track {
            camera: scene.actors[0].id,
            sample_ms: 100,
            cull_backfaces: false, static_world: false,
            faces: vec![],
            samples: (0..5)
                .map(|_| Sample {
                    position: [0.; 3],
                    rotation: [0.; 3],
                    fov: 90.,
                    visible: vec![],
                    cut: true,
                })
                .collect(),
            navigation: None,
        };
        let nav = Navigation {
            target: scene.actors[1].id,
            start_route: 1,
            routes: vec![
                Route {
                    first: 0,
                    count: 3,
                    automatic: false,
                    links: vec![Link {
                        from_end: true,
                        route: 1,
                        to_end: true,
                        enabled: false,
                        load_cut: false,
                    }],
                },
                Route {
                    first: 3,
                    count: 2,
                    automatic: true,
                    links: vec![Link {
                        from_end: false,
                        route: 0,
                        to_end: true,
                        enabled: true,
                        load_cut: false,
                    }],
                },
            ],
            parameters: vec![
                Parameter {
                    axis: [0., 1.],
                    limits: [0., 1.],
                    speed: 30.
                };
                5
            ],
            aim: None,
        };
        assert!(nav.validate(&scene, &track).is_ok());
        assert_eq!(nav.pairs(), vec![(0, 1), (1, 2), (2, 4), (3, 2), (3, 4)]);
        let mut cut = nav.clone();
        cut.routes[0].links[0].load_cut = true;
        assert_eq!(cut.pairs(), vec![(0, 1), (1, 2), (3, 2), (3, 4)]);
        assert!(cut.validate(&scene, &track).is_ok());
        let mut bad = nav.clone();
        bad.routes[1].first = 2;
        assert!(bad.validate(&scene, &track).is_err());
        bad = nav.clone();
        bad.routes[0].links[0].route = 5;
        assert!(bad.validate(&scene, &track).is_err());
        bad = nav.clone();
        bad.parameters[0].axis = [0., 0.];
        assert!(bad.validate(&scene, &track).is_err());
        bad = nav.clone();
        bad.parameters[0].speed = f32::NAN;
        assert!(bad.validate(&scene, &track).is_err());
        bad = nav.clone();
        bad.target = track.camera;
        assert!(bad.validate(&scene, &track).is_err());
        let original = nav.target;
        track.navigation = Some(nav);
        scene.visibility_track = Some(track);
        scene.duplicate_identities();
        let track = scene.visibility_track.as_ref().unwrap();
        assert_ne!(track.navigation.as_ref().unwrap().target, original);
        track.validate(&scene).unwrap();
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub first: u16,
    pub count: u16,
    pub automatic: bool,
    pub links: Vec<Link>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    /// Projection axis in world X/Z, independent of the rendered camera roll.
    pub axis: [f32; 2],
    pub limits: [f32; 2],
    /// Samples per second. Signed only for automatic routes.
    pub speed: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Navigation {
    pub target: Uuid,
    pub start_route: u16,
    pub routes: Vec<Route>,
    pub parameters: Vec<Parameter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aim: Option<crate::rail_aim::Aim>,
}
impl Navigation {
    pub fn validate(&self, scene: &Scene, track: &Track) -> Result<(), String> {
        if let Some(aim)=&self.aim {aim.validate(track.samples.len())?;}
        if self.target == track.camera
            || !scene.actors.iter().any(|a| {
                a.id == self.target
                    && a.parent.is_none()
                    && a.kind != "Camera"
                    && crate::actor_components::domain(a)
                        == crate::reflection_schema::Domain::World3D
            })
        {
            return Err("Rail navigation needs a distinct root target actor".into());
        }
        if self.routes.is_empty()
            || self.routes.len() > 65535
            || usize::from(self.start_route) >= self.routes.len()
            || self.parameters.len() != track.samples.len()
            || self.routes.iter().map(|r| r.links.len()).sum::<usize>() > 512
        {
            return Err("Invalid rail route, parameter or link count".into());
        }
        let mut covered = vec![false; track.samples.len()];
        for route in &self.routes {
            let first = usize::from(route.first);
            let end = first + usize::from(route.count);
            if route.count == 0 || end > covered.len() {
                return Err("Rail sample range out of bounds".into());
            }
            for value in &mut covered[first..end] {
                if *value {
                    return Err("Rail sample ranges overlap".into());
                }
                *value = true;
            }
            for link in &route.links {
                if usize::from(link.route) >= self.routes.len() {
                    return Err("Rail link destination out of bounds".into());
                }
            }
        }
        if covered.iter().any(|v| !v) {
            return Err("Rail routes must cover every track sample".into());
        }
        for p in &self.parameters {
            let length = p.axis[0].hypot(p.axis[1]);
            if p.axis
                .iter()
                .chain(&p.limits)
                .chain(std::iter::once(&p.speed))
                .any(|v| !v.is_finite())
                || !(0.99..=1.01).contains(&length)
                || p.limits[0] > p.limits[1]
                || p.limits.iter().any(|v| v.abs() > 32767.)
                || p.speed.abs() > 1000.
            {
                return Err("Invalid rail axis, follow limits or speed".into());
            }
        }
        Ok(())
    }
    pub fn pairs(&self) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        for route in &self.routes {
            let first = usize::from(route.first);
            let last = first + usize::from(route.count) - 1;
            result.extend((first..last).map(|i| (i, i + 1)));
            for link in &route.links {
                if link.load_cut {
                    continue;
                } // Destination views are budgeted separately.
                let target = &self.routes[usize::from(link.route)];
                let to = usize::from(target.first)
                    + if link.to_end {
                        usize::from(target.count) - 1
                    } else {
                        0
                    };
                // Disabled links can be enabled by gameplay; budget them too.
                result.push((if link.from_end { last } else { first }, to));
            }
        }
        result.sort_unstable();
        result.dedup();
        result
    }
    pub fn header(&self, scene: &Scene, words: usize) -> String {
        let target = scene
            .actors
            .iter()
            .position(|a| a.id == self.target)
            .unwrap();
        let mut offset = 0;
        let mut routes = Vec::new();
        let mut links = Vec::new();
        for r in &self.routes {
            routes.push(format!(
                "{{{},{},{},{},{}}}",
                r.first,
                r.count,
                offset,
                r.links.len(),
                r.automatic
            ));
            for l in &r.links {
                links.push(format!(
                    "{{{},{},{},{},{}}}",
                    l.route, l.from_end, l.to_end, l.enabled, l.load_cut
                ));
            }
            offset += r.links.len();
        }
        let mut axes = std::collections::BTreeMap::new();
        let mut settings = std::collections::BTreeMap::new();
        let params = self
            .parameters
            .iter()
            .map(|p| {
                let axis = p.axis.map(|v| (v * 4096.).round() as i16);
                let setting = [
                    (p.limits[0] * 4096.).round() as i32,
                    (p.limits[1] * 4096.).round() as i32,
                    (p.speed * 4096.).round() as i32,
                ];
                let next = axes.len();
                let a = *axes.entry(axis).or_insert(next);
                let next = settings.len();
                let s = *settings.entry(setting).or_insert(next);
                format!("{{{a},{s}}}")
            })
            .collect::<Vec<_>>()
            .join(",");
        let mut axes = axes.into_iter().collect::<Vec<_>>();
        axes.sort_by_key(|(_, i)| *i);
        let axes = axes
            .iter()
            .map(|(v, _)| format!("{{{},{}}}", v[0], v[1]))
            .collect::<Vec<_>>()
            .join(",");
        let mut settings = settings.into_iter().collect::<Vec<_>>();
        settings.sort_by_key(|(_, i)| *i);
        let settings = settings
            .iter()
            .map(|(v, _)| format!("{{{},{},{}}}", v[0], v[1], v[2]))
            .collect::<Vec<_>>()
            .join(",");
        let aim_header=self.aim.as_ref().map_or_else(String::new,|aim|aim.header());
        let aim_pointer=if self.aim.is_some() {",&rail_aim_data"} else {""};
        format!(
            "{aim_header}inline uint32_t rail_anchor[{}]={{}};\ninline constexpr RailRoute rail_routes[]={{{}}};\ninline constexpr RailLink rail_links[]={{{}}};\ninline constexpr PackedRailParameter rail_parameters[]={{{params}}};\ninline constexpr int16_t rail_axes[][2]={{{axes}}};\ninline constexpr int32_t rail_settings[][3]={{{settings}}};\ninline constexpr RailNavigationData rail_data={{rail_routes,rail_links,nullptr,rail_anchor,{},{},{},{},rail_parameters,rail_axes,rail_settings{aim_pointer}}};\n",
            words.max(1),
            routes.join(","),
            if links.is_empty() {
                "{}".into()
            } else {
                links.join(",")
            },
            self.routes.len(),
            links.len(),
            target,
            self.start_route
        )
    }
}
