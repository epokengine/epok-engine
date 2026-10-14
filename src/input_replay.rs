//! Explicit, bounded virtual-pad replay for headless emulator verification.
#[derive(Debug)]
pub struct Replay(Vec<(u64, u16)>);
impl Replay {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut steps = Vec::new();
        for step in text.split(',') {
            let (at, mask) = step
                .split_once(':')
                .ok_or("Expected milliseconds:buttons pairs")?;
            let at = at.parse::<u64>().map_err(|_| "Invalid replay time")?;
            let mask = mask.parse::<u16>().map_err(|_| "Invalid 16-bit pad mask")?;
            if at > 3_600_000 || steps.last().is_some_and(|&(previous, _)| previous >= at) {
                return Err("Replay times must increase and fit within one hour".into());
            }
            steps.push((at, mask));
            if steps.len() > 256 {
                return Err("Replay exceeds 256 steps".into());
            }
        }
        if steps.last().is_none_or(|&(_, mask)| mask != 0) {
            return Err("Replay must end by releasing all buttons (mask 0)".into());
        }
        Ok(Self(steps))
    }
    /// When the last step happens, in milliseconds.
    pub fn end(&self) -> u64 {
        self.0.last().map_or(0, |&(at, _)| at)
    }
    pub fn buttons(&self, milliseconds: u64) -> u16 {
        self.0
            .iter()
            .take_while(|&&(at, _)| at <= milliseconds)
            .last()
            .map_or(0, |&(_, mask)| mask)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_is_bounded_ordered_and_releases() {
        let replay = Replay::parse("100:16,200:64,300:0").unwrap();
        for (time, expected) in [
            (0, 0),
            (99, 0),
            (100, 16),
            (199, 16),
            (200, 64),
            (300, 0),
            (999, 0),
        ] {
            assert_eq!(replay.buttons(time), expected);
        }
        for invalid in [
            "",
            "0:65536",
            "0:16",
            "10:0,10:0",
            "20:0,10:0",
            "3600001:0",
            "a:0",
        ] {
            assert!(Replay::parse(invalid).is_err());
        }
    }
}
