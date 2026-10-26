//! Host actuator output. GilRs is owned by the platform loop; workers publish
//! bounded snapshots and never open another controller context.
use gilrs::ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder, Repeat, Ticks};
use std::time::{Duration, Instant};

pub fn write_emulator_config(
    root: &std::path::Path,
    build: &std::path::Path,
) -> Result<(), String> {
    let required = std::fs::read_to_string(build.join("sources.mk"))
        .map_err(|e| e.to_string())?
        .contains("-DEPOK_RUMBLE");
    let map = std::fs::read_to_string(build.join("epok.map")).map_err(|e| e.to_string())?;
    let address = map.lines().find_map(|line| {
        let words: Vec<_> = line.split_whitespace().collect();
        (words.last() == Some(&"epok_rumble_control"))
            .then(|| {
                words.iter().find_map(|word| {
                    word.strip_prefix("0x")
                        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                })
            })
            .flatten()
    });
    if required && address.is_none() {
        return Err("Rumble build is missing its desktop cancellation symbol".into());
    }
    let path = root.join(".epok/emulator/rumble.lua");
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(
        path,
        format!(
            "return {{version=1,required={required},mailbox={}}}\n",
            address.unwrap_or(0)
        ),
    )
    .map_err(|e| e.to_string())
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Command {
    pub revision: u32,
    pub large: u8,
    pub small: bool,
    pub remaining_us: u32,
    pub identity: u32,
}
#[derive(Clone)]
pub struct Snapshot {
    pub session: u64,
    pub generation: u32,
    pub commands: [Command; 4],
    pub received: Instant,
}
type Key = (u64, u32, u32, u32);
struct Motors {
    strong: Effect,
    weak: Effect,
}
impl Motors {
    fn new(gilrs: &mut gilrs::Gilrs, id: gilrs::GamepadId) -> Result<Self, String> {
        let mut make = |kind| {
            EffectBuilder::new()
                .add_effect(BaseEffect {
                    kind,
                    scheduling: gilrs::ff::Replay {
                        play_for: Ticks::from_ms(250),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .gamepads(&[id])
                .repeat(Repeat::For(Ticks::from_ms(250)))
                .finish(gilrs)
                .map_err(|error| error.to_string())
        };
        Ok(Self {
            strong: make(BaseEffectType::Strong {
                magnitude: u16::MAX,
            })?,
            weak: make(BaseEffectType::Weak {
                magnitude: u16::MAX,
            })?,
        })
    }
    fn stop(&self) {
        let _ = self.strong.stop();
        let _ = self.weak.stop();
    }
    fn play(&self, large: u8, small: bool, lease_ms: u32) -> Result<(), String> {
        for (effect, gain) in [
            (&self.strong, f32::from(large) / 255.),
            (&self.weak, if small { 1. } else { 0. }),
        ] {
            if gain == 0. {
                effect.stop().map_err(|e| e.to_string())?;
            } else {
                effect
                    .set_gain(gain)
                    .and_then(|_| effect.set_repeat(Repeat::For(Ticks::from_ms(lease_ms))))
                    .and_then(|_| effect.play())
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
impl Drop for Motors {
    fn drop(&mut self) {
        self.stop();
    }
}
#[derive(Default)]
struct Slot {
    device: Option<usize>,
    identity: u32,
    motors: Option<Motors>,
    key: Option<Key>,
    blocked: Option<Key>,
    refreshed: Option<Instant>,
    value: (u8, bool),
    failed: bool,
}
impl Slot {
    fn stop(&mut self) {
        if self.refreshed.is_some() {
            if let Some(motors) = &self.motors {
                motors.stop();
            }
        }
        self.refreshed = None;
        self.value = (0, false);
    }
}
#[derive(Default)]
pub struct Driver {
    slots: [Slot; 4],
    test: Option<(usize, u32, Instant)>,
    pub error: Option<String>,
}
impl Driver {
    pub fn stop(&mut self) {
        for slot in &mut self.slots {
            slot.stop();
            slot.blocked = slot.key;
        }
        self.test = None;
    }
    pub fn update(
        &mut self,
        gilrs: &mut gilrs::Gilrs,
        input: &mut crate::controls::HostInput,
        snapshot: Option<&Snapshot>,
        play_allowed: bool,
        test_allowed: bool,
    ) {
        let now = Instant::now();
        let test_requested = input.rumble_test.take();
        if let Some(port) = test_requested {
            self.test = test_allowed.then_some((
                port,
                input.identity(port),
                now + Duration::from_millis(150),
            ));
            if test_allowed && port < 4 {
                self.slots[port].stop();
                self.slots[port].motors = None;
                self.slots[port].failed = false;
                input.rumble_errors[port] = None;
            }
        }
        if !test_allowed
            || self
                .test
                .is_some_and(|(port, _, end)| now >= end || input.rumble_panel_port != Some(port))
        {
            self.test = None;
        }
        let fresh = snapshot
            .filter(|s| now.saturating_duration_since(s.received) <= Duration::from_millis(250));
        for port in 0..4 {
            let assigned = input.assignment(port);
            let identity = input.identity(port);
            let slot = &mut self.slots[port];
            if slot.device != assigned || slot.identity != identity {
                slot.stop();
                slot.motors = None;
                slot.device = assigned;
                slot.identity = identity;
                slot.key = None;
                slot.blocked = None;
                slot.failed = false;
                input.rumble_errors[port] = None;
            }
            let command = fresh.map(|s| s.commands[port]);
            let key = fresh.map(|s| {
                (
                    s.session,
                    s.generation,
                    s.commands[port].revision,
                    s.commands[port].identity,
                )
            });
            // Losing focus cancels this command. A later frame with its same
            // revision cannot resurrect it before the runtime sees cancellation.
            if !play_allowed {
                slot.blocked = key.or(slot.key);
            }
            let command_changed = key != slot.key;
            if command_changed {
                slot.key = key;
            }
            let mut wanted = (0, false, 0);
            if play_allowed && key != slot.blocked {
                if let (Some(s), Some(command)) = (fresh, command) {
                    let age = now
                        .saturating_duration_since(s.received)
                        .as_micros()
                        .min(u128::from(u32::MAX)) as u32;
                    let remaining = command.remaining_us.saturating_sub(age);
                    if command.identity == input.identity(port) && remaining != 0 {
                        wanted = (command.large, command.small, remaining);
                    }
                }
            }
            if let Some((selected, identity, end)) = self.test {
                if selected == port && identity == input.identity(port) {
                    wanted = (
                        192,
                        true,
                        end.saturating_duration_since(now)
                            .as_micros()
                            .min(u128::from(u32::MAX)) as u32,
                    );
                }
            }
            if slot.failed || wanted.2 == 0 || (wanted.0 == 0 && !wanted.1) {
                slot.stop();
                continue;
            }
            let Some((id, _)) = gilrs.gamepads().find(|(id, pad)| {
                usize::from(*id) == assigned.unwrap_or(usize::MAX) && pad.is_ff_supported()
            }) else {
                slot.stop();
                continue;
            };
            if slot.motors.is_none() {
                match Motors::new(gilrs, id) {
                    Ok(motors) => slot.motors = Some(motors),
                    Err(error) => {
                        input.rumble_errors[port] = Some(error.clone());
                        self.error = Some(error);
                        slot.failed = true;
                        continue;
                    }
                }
            }
            let changed = command_changed
                || test_requested == Some(port)
                || slot.value != (wanted.0, wanted.1);
            let refresh = slot.refreshed.is_none_or(|last| {
                now.saturating_duration_since(last) >= Duration::from_millis(100)
            });
            if changed || refresh {
                let lease = wanted.2.div_ceil(1000).min(250);
                // Even a partially submitted two-motor update must stop on error.
                slot.refreshed = Some(now);
                if let Err(error) = slot
                    .motors
                    .as_ref()
                    .unwrap()
                    .play(wanted.0, wanted.1, lease)
                {
                    slot.stop();
                    slot.failed = true;
                    input.rumble_errors[port] = Some(error.clone());
                    self.error = Some(error);
                } else {
                    slot.value = (wanted.0, wanted.1);
                    slot.refreshed = Some(now);
                }
            }
        }
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        self.stop();
    }
}
