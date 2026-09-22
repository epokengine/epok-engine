//! PSX conversion controls. Analysis is bounded, cancellable and never publishes assets.
use crate::{assets, psx_music_settings::*};
use imgui::Ui;
use std::{
    cell::RefCell,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

#[derive(Default)]
struct HelpTimer {
    key: String,
    since: f64,
    frame: i32,
}
impl HelpTimer {
    fn ready(&mut self, key: &str, time: f64, frame: i32) -> bool {
        if self.key != key || self.frame < frame - 1 || time < self.since {
            self.key = key.into();
            self.since = time;
        }
        self.frame = frame;
        time - self.since >= 2.0
    }
}
thread_local! {static HELP:RefCell<HelpTimer>=RefCell::new(HelpTimer::default());}
pub fn help(ui: &Ui, key: &str, text: &str) {
    let min = ui.item_rect_min();
    let max = ui.item_rect_max();
    ui.same_line();
    let clicked = ui.small_button(format!("?##music-help-{key}"));
    let hovered = ui.is_mouse_hovering_rect(
        min,
        [
            ui.item_rect_max()[0].max(max[0]),
            ui.item_rect_max()[1].max(max[1]),
        ],
    );
    if hovered || clicked || ui.is_item_focused() {
        let ready = HELP.with(|timer| {
            timer
                .borrow_mut()
                .ready(key, ui.time(), unsafe { imgui::sys::igGetFrameCount() })
        });
        if ready || clicked || ui.is_item_focused() {
            ui.tooltip(|| {
                let _wrap = ui.push_text_wrap_pos_with_pos(390.);
                ui.text(text);
            });
        }
    }
}
pub fn integer(ui: &Ui, label: &str, value: &mut u32, min: u32, max: u32, text: &str) {
    let mut input = *value as i32;
    if ui.input_int(label, &mut input).build() {
        *value = input.clamp(min as i32, max as i32) as u32;
    }
    help(ui, label, text);
}
fn small_integer(ui: &Ui, label: &str, value: &mut u16, min: u32, max: u32, text: &str) {
    let mut n = u32::from(*value);
    integer(ui, label, &mut n, min, max, text);
    *value = n as u16;
}
fn choice<T: Copy + PartialEq>(
    ui: &Ui,
    label: &str,
    value: &mut T,
    options: &[(T, &str)],
    text: &str,
) {
    let mut n = options.iter().position(|(v, _)| v == value).unwrap_or(0);
    let labels = options.iter().map(|(_, s)| *s).collect::<Vec<_>>();
    if ui.combo_simple_string(label, &mut n, &labels) {
        *value = options[n].0;
    }
    help(ui, label, text);
}

struct Analysis {
    summary: String,
    details: String,
    proposed: Option<Recipe>,
}
struct Job {
    key: String,
    cancel: Arc<AtomicBool>,
    receive: mpsc::Receiver<Result<Analysis, String>>,
}
#[derive(Default)]
pub struct State {
    key: String,
    job: Option<Job>,
    result: Option<Analysis>,
    error: Option<String>,
    pub messages: Vec<String>,
}
impl Drop for State {
    fn drop(&mut self) {
        self.cancel();
    }
}
impl State {
    pub fn cancel(&mut self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }
    fn poll(&mut self, key: String) {
        if self.key != key {
            self.cancel();
            self.key = key;
            self.result = None;
            self.error = None;
        }
        let received = self
            .job
            .as_ref()
            .and_then(|job| match job.receive.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Music conversion worker stopped".into()))
                }
            });
        if let Some(result) = received {
            let job = self.job.take().unwrap();
            if job.key != self.key || job.cancel.load(Ordering::Relaxed) {
                return;
            }
            match result {
                Ok(result) => {
                    self.result = Some(result);
                    self.error = None;
                }
                Err(error) => {
                    self.messages.push(format!("PSX music conversion: {error}"));
                    self.error = Some(error);
                }
            }
        }
    }
}

pub fn controls(
    ui: &Ui,
    settings: &mut crate::sequence::Settings,
    root: &Path,
    index: &assets::Index,
    source: &str,
    snapshot: Option<&assets::Record>,
    state: &mut State,
) {
    ui.separator();
    ui.text("PSX music conversion");
    help(
        ui,
        "target",
        "Prepares music for the PSX SPU: 24 shared physical voices and 512 KiB of SPU RAM. Source MIDI and instrument-library snapshots remain authoritative.",
    );
    let mut recipe = match Recipe::from_settings(settings) {
        Ok(r) => r,
        Err(error) => {
            ui.text_wrapped(error);
            return;
        }
    };
    let mut preset = recipe.preset;
    choice(
        ui,
        "Quality preset",
        &mut preset,
        &[
            (Preset::Compact, "Compact"),
            (Preset::Balanced, "Balanced"),
            (Preset::High, "High"),
            (Preset::Custom, "Custom"),
        ],
        "Compact, Balanced and High start at 11025, 22050 and 44100 Hz. They preserve required notes and layers. Larger samples use more SPU memory; a preset may exceed your budget. Editing conversion values selects Custom.",
    );
    if preset != recipe.preset {
        recipe.select_preset(preset);
    }
    let before = recipe.clone();
    choice(
        ui,
        "Music driver",
        &mut recipe.driver,
        &[
            (
                crate::psx_music_settings::Driver::NativeSpu,
                "Epok Pulse (recommended)",
            ),
            (
                crate::psx_music_settings::Driver::SoftwareReference,
                "SoundFont software reference",
            ),
        ],
        "Epok Pulse compiles MIDI controls in the editor and uses hardware ADSR. Envelope rates/sustain are quantized; delay is omitted and hold joins decay. Target Preview auditions this adaptation. Software reference preserves the previous envelope behavior at a higher CPU cost.",
    );
    if ui.collapsing_header("Advanced conversion", imgui::TreeNodeFlags::empty()) {
        integer(
            ui,
            "Maximum sample rate (Hz)",
            &mut recipe.max_sample_rate,
            400,
            44100,
            "A lower maximum saves sample memory and may reduce brightness. Sources below this rate are not upsampled. Tempo and note tuning stay unchanged.",
        );
        choice(
            ui,
            "Encoder effort",
            &mut recipe.encoder_effort,
            &[
                (crate::spu_encoder::Effort::Fast, "Fast"),
                (crate::spu_encoder::Effort::Thorough, "Thorough"),
            ],
            "Thorough searches more ADPCM encodings and can reduce error, at a higher conversion cost. SPU ADPCM has a fixed storage rate, so this option does not increase playback quality by allocating more bits.",
        );
        ui.text_disabled("Sample channels: mono with voice pan");
        help(
            ui,
            "channels",
            "Samples are downmixed to mono; instrument and MIDI pan control their stereo placement. A stereo sample profile consuming two hardware voices is not implemented.",
        );
        choice(
            ui,
            "Instrument selection",
            &mut recipe.selection,
            &[
                (Selection::Reachable, "Used regions"),
                (Selection::FullLibrary, "Entire library"),
            ],
            "Used regions retains every layer, key and velocity region reached by this MIDI. The authoring library is never pruned. Entire library can exceed the target's bounded region, sample or memory limits.",
        );
        ui.text_disabled("Mappings: automatic bank / program / drum key; layers preserved");
        help(
            ui,
            "mapping",
            "The converter resolves programs and drum keys against the selected SoundBank. Missing mappings are errors. It does not substitute a generic instrument or silently discard layers.",
        );
        choice(
            ui,
            "Instrument sample loops",
            &mut recipe.sample_loops,
            &[
                (SampleLoops::AlignOutward, "Align to ADPCM blocks"),
                (SampleLoops::ExactBlocks, "Require exact blocks"),
            ],
            "These are instrument sustain loops, separate from the song loop. SPU ADPCM blocks contain 28 samples. Alignment changes are reported; Require exact blocks rejects incompatible loop endpoints.",
        );
        small_integer(
            ui,
            "Loop crossfade (frames)",
            &mut recipe.loop_crossfade_frames,
            0,
            256,
            "Blends this many frames at the aligned sample-loop seam. Zero preserves the seam. This changes instrument timbre and is reported; it does not crossfade the song's beginning and end.",
        );
        let mut capped = recipe.maximum_release_ms.is_some();
        if ui.checkbox("Limit instrument release", &mut capped) {
            recipe.maximum_release_ms = if capped { Some(450) } else { None };
        }
        help(
            ui,
            "release-policy",
            "Preserve uses authored release envelopes, which may need many simultaneous voices. An explicit cap shortens full-scale release tails in the target. It preserves notes and the original library; Source Preview keeps the authored tails.",
        );
        if let Some(limit) = &mut recipe.maximum_release_ms {
            small_integer(
                ui,
                "Maximum release (ms)",
                limit,
                10,
                30000,
                "Maximum full-scale volume-envelope release in milliseconds. Shorter tails reduce overlapping voices. Every adapted region is reported. The tested Ironwood recipe uses 450 ms and 21 music voices.",
            );
        }
        choice(
            ui,
            "Filter policy",
            &mut recipe.filter_policy,
            &[
                (FilterPolicy::BakeSustain, "Bake at sustain"),
                (FilterPolicy::RequireStatic, "Require static"),
            ],
            "The SPU cannot reproduce arbitrary SoundFont filters. Bake at sustain precalculates a reported constant filter at its sustain/reference-velocity state; transposing the sample also transposes that filter. Animated filters outside this policy remain errors. Source Preview retains the original filter.",
        );
        choice(
            ui,
            "Effects",
            &mut recipe.effects,
            &[(Effects::Dry, "Dry"), (Effects::Room, "PSX Room")],
            "Dry disables target effects explicitly and reports the adaptation. Room uses one shared PSX reverb resource and reserves 9920 SPU bytes. Its per-voice send is binary, so source send depths are adapted. Room does not implement chorus; unsupported effects remain errors in that profile.",
        );
        ui.disabled(recipe.effects!=Effects::Room,||small_integer(ui,"Room output (permille)",&mut recipe.reverb_depth_permille,0,1000,
            "Global Room output gain: 1000 is full scale. Multiple active banks must agree on the shared reverb configuration. Host Target Preview does not render the wet reverb; use PSX playback to hear it."));
        small_integer(
            ui,
            "Headroom (0.1 dB)",
            &mut recipe.headroom_centibels,
            0,
            960,
            "Shared attenuation before mixing; 60 means 6 dB. More headroom leaves room for instruments and SFX to sum without clipping. The same attenuation is used for Source/Target comparison and preserves relative instrument levels.",
        );
        integer(
            ui,
            "Bank sample budget (bytes)",
            &mut recipe.bank_budget_bytes,
            1,
            crate::audio_import::SPU_BUDGET as u32,
            "Upper limit for this bank's SPU samples. The effective budget also subtracts capture, reverb and the other resident samples below. Main RAM and file sizes are separate costs.",
        );
        integer(
            ui,
            "Other resident samples (bytes)",
            &mut recipe.other_resident_bytes,
            0,
            crate::audio_import::SPU_BUDGET as u32 - 1,
            "Reserve the project's other banks and SFX here. Analysis uses this explicit reservation; the full build independently checks the complete included asset set. A single-song report does not prove the entire game fits.",
        );
        ui.checkbox(
            "Automatically fit sample budget on import",
            &mut recipe.optimization.allow_lower_rate,
        );
        help(
            ui,
            "optimizer-rate",
            "When enabled, importing or reimporting a sequence with a SoundFont automatically saves the highest tested rate that fits its PSX sample budget. It never deletes notes, layers or instruments, and does not change residency. Disable it to make an over-budget conversion fail before publishing.",
        );
        integer(
            ui,
            "Minimum proposed rate (Hz)",
            &mut recipe.optimization.minimum_sample_rate,
            400,
            44100,
            "Lowest rate the bounded automatic fit may try. It is a quality floor: if no candidate at or above it fits, import stops before publishing an unusable target recipe.",
        );
        small_integer(
            ui,
            "Maximum candidates",
            &mut recipe.optimization.max_candidates,
            1,
            32,
            "Bounds conversion work, including the original recipe. More candidates refine the deterministic rate search but take longer. This is the best tested candidate, not a guarantee of a globally optimal encoding.",
        );
    }
    if recipe != before {
        recipe.preset = Preset::Custom;
    }
    if let Err(error) = recipe.store(settings) {
        ui.text_wrapped(error);
        return;
    }
    ui.text(format!(
        "Effective sample budget: {} bytes; maximum {} Hz",
        recipe.available_bytes(),
        recipe.max_sample_rate
    ));
    let bank = crate::sequence::resolve_bank(root, settings, index).cloned();
    let source_stamp = std::fs::metadata(root.join(source))
        .ok()
        .map(|m| (m.len(), m.modified().ok()));
    let key = format!(
        "{}:{source}:{source_stamp:?}:{}:{}",
        root.display(),
        serde_json::to_string(settings).unwrap_or_default(),
        format_args!(
            "{:?}:{:?}",
            snapshot.map(|r| &r.revision),
            bank.as_ref().ok().map(|r| &r.revision)
        )
    );
    state.poll(key);
    let mut request = None;
    ui.disabled(state.job.is_some(),||{
        if ui.button("Analyze conversion"){request=Some(false);}
        help(ui,"analyze","Converts a read-only draft in a cancellable worker and reports exact cooked sizes and adaptations. It publishes no sequence or bank. Voice peaks and PSX timing still require playback measurements.");
        ui.same_line();if ui.button("Optimize to budget"){request=Some(true);}
        help(ui,"optimize","Previews the automatic import decision without publishing it. Adopting the proposal updates this editable draft; Save settings / Reimport publishes it.");
    });
    if state.job.is_some() {
        ui.text("Converting draft...");
        ui.same_line();
        if ui.button("Cancel conversion") {
            state.cancel();
        }
        help(
            ui,
            "cancel-conversion",
            "Stops the worker at the next bounded conversion checkpoint. Cancelled or stale results are discarded and cannot publish assets.",
        );
    }
    if let Some(optimize) = request {
        let root = root.to_owned();
        let source = source.to_string();
        let snapshot = snapshot.cloned();
        let settings = settings.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (send, receive) = mpsc::channel();
        state.error = None;
        state.result = None;
        std::thread::spawn(move || {
            let result = (|| {
                let bank = bank?;
                if bank.meta.settings.sound_bank()?.library.is_none() {
                    return Err("Select a SoundFont instrument library to use these conversion options. Legacy portable banks retain their own zone settings.".into());
                }
                let bytes = if let Some(record) = &snapshot {
                    assets::Package::load(&record.path)?.source
                } else {
                    assets::read_bounded(&assets::inside(&root, &source)?)?
                };
                let source_hash = assets::hash(&bytes);
                let ir = crate::sequence::decode_source(&bytes, &settings)?;
                settings.validate_playback(&ir)?;
                let library = assets::Package::load(&bank.path)?;
                let prepared = crate::psx_library::prepare_selection(
                    &library.source,
                    &ir,
                    &settings.instrument_mappings,
                    recipe.selection,
                    &worker_cancel,
                )?;
                let (report, proposed, reason) = if optimize {
                    let proposal =
                        crate::psx_music_optimizer::propose(&prepared, &recipe, &worker_cancel)?;
                    (
                        proposal.proposed_report.unwrap_or(proposal.original_report),
                        proposal.proposed_recipe,
                        proposal.reason,
                    )
                } else {
                    (
                        crate::psx_library::cook(&prepared, &recipe, &worker_cancel)?.report,
                        None,
                        None,
                    )
                };
                let (_, payload) =
                    crate::psx_sequence::library_payload(&ir, &settings, bank.meta.id)?;
                crate::psx_library_asset::verify_record(&bank, &worker_cancel)?;
                if let Some(record) = &snapshot {
                    crate::psx_library_asset::verify_record(record, &worker_cancel)?;
                } else if assets::hash(&assets::read_bounded(&assets::inside(&root, &source)?)?)
                    != source_hash
                {
                    return Err("MIDI source changed during conversion; analyze again".into());
                }
                Ok(Analysis {
                    summary: format!(
                        "{:.3} s; {} sequence bytes; {} sample SPU bytes / {} available; {} adaptations. {}{}",
                        ir.duration_micros as f64 / 1_000_000.,
                        payload.len(),
                        report.sample_spu_bytes,
                        report.recipe.available_bytes(),
                        report.adaptations.len(),
                        if report.fits_sample_budget {
                            "Samples fit. "
                        } else {
                            "Samples exceed budget. "
                        },
                        reason.unwrap_or_default()
                    ),
                    details: serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
                    proposed,
                })
            })();
            let _ = send.send(result);
        });
        state.job = Some(Job {
            key: state.key.clone(),
            cancel,
            receive,
        });
    }
    if let Some(result) = &state.result {
        ui.text_wrapped(&result.summary);
        if let Some(proposed) = &result.proposed {
            ui.text(format!(
                "Proposal: {} Hz, {} bytes available",
                proposed.max_sample_rate,
                proposed.available_bytes()
            ));
            if ui.button("Adopt proposed recipe") {
                if let Err(error) = proposed.store(settings) {
                    state.error = Some(error);
                }
            }
            help(
                ui,
                "adopt-proposal",
                "Updates the editable recipe only. Import/reimport also applies this same fit automatically when the automatic-fit setting is enabled.",
            );
        }
        if ui.collapsing_header("Conversion details", imgui::TreeNodeFlags::empty()) {
            ui.child_window("music-conversion-report")
                .size([0., 180.])
                .build(|| ui.text(&result.details));
        }
    }
    if let Some(error) = &state.error {
        ui.text_wrapped(error);
    }
    ui.text_wrapped("Save settings, then compare Source Preview and PSX Target Preview in Explorer or Inspector. Source uses the original library; Target uses cooked samples and sequence. Host Target Preview uses linear interpolation and omits wet reverb.");
    help(
        ui,
        "preview-modes",
        "The existing preview controls keep one audition active and cancel stale work. Saving invalidates derived data. Compare at the same headroom; actual SPU interpolation and Room reverb require PSX playback. Draft audition before saving is pending.",
    );
}

pub fn reference_candidate(root: &Path) -> Result<assets::Candidate, String> {
    const BYTES: &[u8] = include_bytes!("../resources/audio/FluidR3Mono_GM.sf3");
    let source = "assets/AudioLibraries/FluidR3Mono_GM.sf3";
    let destination = "assets/AudioLibraries/FluidR3Mono_GM.epokasset";
    if assets::hash(BYTES) != crate::soundfont_asset::REFERENCE_HASH {
        return Err("Bundled library checksum mismatch".into());
    }
    let path = assets::inside(root, source)?;
    if path.exists() {
        if assets::hash(&assets::read_soundfont_bounded(&path)?)
            != crate::soundfont_asset::REFERENCE_HASH
        {
            return Err("Reference-library source destination contains another file; move it or import your own library".into());
        }
    } else {
        assets::atomic_write(&path, BYTES, None)?;
    }
    let index = assets::scan(root, &mut Default::default());
    let existing = index.usable().find(|r| r.path == root.join(destination));
    if existing.is_some_and(|r| r.meta.source_hash != crate::soundfont_asset::REFERENCE_HASH) {
        return Err(
            "Reference-library asset destination is occupied; no asset was overwritten".into(),
        );
    }
    let mut settings = existing
        .map(|r| r.meta.settings.sound_bank().cloned())
        .transpose()?
        .unwrap_or_default();
    if settings.provenance.is_empty() {
        settings.provenance = format!(
            "FluidR3Mono GM 2.315 (Debian 2.315-7). SHA-256 {}.\n{}",
            crate::soundfont_asset::REFERENCE_HASH,
            crate::soundfont_asset::REFERENCE_LICENSE
        );
    }
    crate::soundfont_asset::prepare(root, source, destination, settings, existing, false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn tooltip_delay_identity_and_frame_gap() {
        let mut timer = super::HelpTimer::default();
        assert!(!timer.ready("rate", 0., 1));
        assert!(!timer.ready("rate", 1.9, 2));
        assert!(timer.ready("rate", 2.1, 3));
        assert!(!timer.ready("release", 2.2, 4));
        assert!(!timer.ready("release", 8., 6));
    }
}
