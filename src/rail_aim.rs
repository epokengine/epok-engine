//! Optional, cooked angular tracking for actor-driven rails.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Parameters {
    pub base_weight: [u16; 3],
    pub target_bias: [i16; 3],
    pub smoothing_weight: u16,
    pub smoothing_limit: u16,
    pub positive_limit: [u16; 3],
    pub negative_limit: [u16; 3],
    pub smooth: bool,
}
impl Default for Parameters {
    fn default() -> Self {
        Self {
            base_weight: [256; 3],
            target_bias: [0; 3],
            smoothing_weight: 256,
            smoothing_limit: 2048,
            positive_limit: [2048; 3],
            negative_limit: [2048; 3],
            smooth: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Aim {
    pub update_hz: u16,
    /// Reflection of the rig's world X and yaw/roll axes, not projection Mirror X.
    pub reflect_x: bool,
    /// Targets outside the base yaw's forward hemisphere use pitch -90 degrees.
    pub front_only: bool,
    pub samples: Vec<Parameters>,
}
impl Default for Aim {
    fn default() -> Self {
        Self {
            update_hz: 30,
            reflect_x: false,
            front_only: false,
            samples: Vec::new(),
        }
    }
}
impl Aim {
    pub fn validate(&self, count: usize) -> Result<(), String> {
        if !(1..=120).contains(&self.update_hz) || self.samples.len() != count {
            return Err("Rail aim requires 1..120 Hz and one parameter set per sample".into());
        }
        for p in &self.samples {
            if p.base_weight.iter().any(|&v| v > 256)
                || p.smoothing_weight > 256
                || p.smoothing_limit > 2048
                || p.positive_limit
                    .iter()
                    .chain(&p.negative_limit)
                    .any(|&v| v > 2048)
            {
                return Err(
                    "Rail aim weights must be 0..256 and angular limits 0..2048 turn12".into(),
                );
            }
        }
        Ok(())
    }
    pub fn header(&self) -> String {
        let mut dictionary = BTreeMap::new();
        let ids = self
            .samples
            .iter()
            .map(|p| {
                let id = dictionary.len();
                *dictionary.entry(p.clone()).or_insert(id)
            })
            .collect::<Vec<_>>();
        let mut entries = dictionary.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|(_, id)| *id);
        fn three<T: std::fmt::Display>(v: &[T; 3]) -> String {
            format!("{{{},{},{}}}", v[0], v[1], v[2])
        }
        let parameters = entries
            .iter()
            .map(|(p, _)| {
                format!(
                    "{{{},{},{},{},{},{}}}",
                    three(&p.base_weight),
                    three(&p.target_bias),
                    p.smoothing_weight,
                    p.smoothing_limit,
                    three(&p.positive_limit),
                    three(&p.negative_limit)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let samples = ids
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let smooth = self
            .samples
            .iter()
            .map(|p| if p.smooth { "1" } else { "0" })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "inline constexpr CameraAimParameters rail_aim_parameters[]={{{parameters}}};\n\
            inline constexpr uint16_t rail_aim_samples[]={{{samples}}};\n\
            inline constexpr uint8_t rail_aim_smooth[]={{{smooth}}};\n\
            inline constexpr RailAimData rail_aim_data={{rail_aim_parameters,rail_aim_samples,rail_aim_smooth,{},{},{}}};\n",
            self.update_hz, self.reflect_x, self.front_only
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parameters_validate_and_cook_with_exact_deduplication() {
        let mut aim = Aim {
            samples: vec![Parameters::default(); 3],
            ..Aim::default()
        };
        aim.samples[1].base_weight = [166, 128, 128];
        aim.samples[1].smooth = false;
        assert!(aim.validate(3).is_ok());
        let header = aim.header();
        assert!(header.contains("rail_aim_samples[]={0,1,0}"));
        assert!(header.contains("rail_aim_smooth[]={1,0,1}"));
        assert!(header.contains("rail_aim_smooth,30,false,false"));
        let json = serde_json::to_value(&aim).unwrap();
        assert_eq!(serde_json::from_value::<Aim>(json).unwrap(), aim);
        assert!(aim.validate(2).is_err());
        aim.update_hz = 0;
        assert!(aim.validate(3).is_err());
        aim.update_hz = 121;
        assert!(aim.validate(3).is_err());
        aim.update_hz = 30;
        aim.samples[1].base_weight[0] = 257;
        assert!(aim.validate(3).is_err());
        aim.samples[1].base_weight[0] = 256;
        aim.samples[1].negative_limit[2] = 2049;
        assert!(aim.validate(3).is_err());
    }
}
