//! Bounded, non-mutating proposal generation for an over-budget PSX music bank.
//!
//! The optimizer changes only the maximum sample rate in a copied recipe. It
//! deliberately returns a proposal for the caller to review and apply; it never
//! writes settings, source snapshots, caches, or staged output.
use crate::{
    psx_library::{self, Prepared, Report},
    psx_music_settings::{Preset, Recipe},
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

pub const MAX_CANDIDATES: usize = 32;

#[derive(Clone, Debug, Serialize)]
pub struct Candidate {
    /// Evaluation order, including the baseline as zero.
    pub ordinal: usize,
    pub max_sample_rate: u32,
    pub sample_spu_bytes: usize,
    pub available_bank_bytes: u32,
    pub fits_sample_budget: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Proposal {
    pub original_recipe: Recipe,
    pub proposed_recipe: Option<Recipe>,
    pub original_report: Report,
    pub proposed_report: Option<Report>,
    /// Every fully cooked candidate, in deterministic evaluation order.
    pub candidates: Vec<Candidate>,
    /// Present when no changed recipe can be proposed (and when the baseline
    /// already fits, so callers do not mistake absence for a failed search).
    pub reason: Option<String>,
}

/// Evaluates the original recipe and, only when it does not fit and lowering is
/// explicitly allowed, proposes a lower maximum sample rate. The rate ladder is
/// a deterministic binary search between the configured minimum and the original
/// rate. It is bounded by `Optimization::max_candidates` (and `MAX_CANDIDATES`),
/// so it is not a claim of globally optimal bank compression.
pub fn propose(
    prepared: &Prepared,
    recipe: &Recipe,
    cancelled: &AtomicBool,
) -> Result<Proposal, String> {
    propose_with(recipe, cancelled, |candidate, cancelled| {
        psx_library::cook(prepared, candidate, cancelled).map(|cooked| cooked.report)
    })
}

fn propose_with<F>(
    recipe: &Recipe,
    cancelled: &AtomicBool,
    mut evaluate: F,
) -> Result<Proposal, String>
where
    F: FnMut(&Recipe, &AtomicBool) -> Result<Report, String>,
{
    recipe.validate()?;
    check_cancelled(cancelled)?;
    let limit = usize::from(recipe.optimization.max_candidates).min(MAX_CANDIDATES);
    let baseline = evaluate(recipe, cancelled)?;
    check_cancelled(cancelled)?;
    let mut candidates = vec![candidate(0, recipe.max_sample_rate, &baseline)];
    if baseline.fits_sample_budget {
        return Ok(Proposal {
            original_recipe: recipe.clone(),
            proposed_recipe: None,
            original_report: baseline,
            proposed_report: None,
            candidates,
            reason: Some("The original recipe already fits the combined PSX SPU budget".into()),
        });
    }
    if !recipe.optimization.allow_lower_rate {
        return no_proposal(
            recipe,
            baseline,
            candidates,
            "The original recipe exceeds the combined PSX SPU budget and lower sample rates are disabled",
        );
    }
    let minimum = recipe.optimization.minimum_sample_rate;
    if minimum >= recipe.max_sample_rate {
        return no_proposal(
            recipe,
            baseline,
            candidates,
            "The configured minimum sample rate leaves no lower rate to propose",
        );
    }
    if limit == 1 {
        return no_proposal(
            recipe,
            baseline,
            candidates,
            "The optimizer candidate limit permits the baseline only",
        );
    }

    let lowest_recipe = lowered_recipe(recipe, minimum);
    let lowest = evaluate(&lowest_recipe, cancelled)?;
    check_cancelled(cancelled)?;
    candidates.push(candidate(1, minimum, &lowest));
    if !lowest.fits_sample_budget {
        return no_proposal(
            recipe,
            baseline,
            candidates,
            &format!(
                "The minimum allowed rate {minimum} Hz still exceeds the combined PSX SPU budget; no tested rate fits"
            ),
        );
    }

    // `low` is the best fitting rate observed. `high` is an untested-or-failing
    // upper bound. ADPCM size is normally rate-monotone but rounding can make
    // edge cases differ, so this finds the best rate on this declared ladder,
    // not a global optimum over every integer rate.
    let mut low = minimum;
    let mut high = recipe.max_sample_rate - 1;
    let mut best = (lowest_recipe, lowest);
    while candidates.len() < limit && low < high {
        check_cancelled(cancelled)?;
        let mid = low + (high - low + 1) / 2;
        let candidate_recipe = lowered_recipe(recipe, mid);
        let report = evaluate(&candidate_recipe, cancelled)?;
        check_cancelled(cancelled)?;
        candidates.push(candidate(candidates.len(), mid, &report));
        if report.fits_sample_budget {
            low = mid;
            best = (candidate_recipe, report);
        } else {
            high = mid - 1;
        }
    }
    Ok(Proposal {
        original_recipe: recipe.clone(),
        proposed_recipe: Some(best.0),
        original_report: baseline,
        proposed_report: Some(best.1),
        candidates,
        reason: None,
    })
}

fn candidate(ordinal: usize, rate: u32, report: &Report) -> Candidate {
    Candidate {
        ordinal,
        max_sample_rate: rate,
        sample_spu_bytes: report.sample_spu_bytes,
        available_bank_bytes: report.available_bank_bytes,
        fits_sample_budget: report.fits_sample_budget,
    }
}

fn lowered_recipe(recipe: &Recipe, max_sample_rate: u32) -> Recipe {
    let mut proposed = recipe.clone();
    proposed.preset = Preset::Custom;
    proposed.max_sample_rate = max_sample_rate;
    proposed
}

fn no_proposal(
    recipe: &Recipe,
    baseline: Report,
    candidates: Vec<Candidate>,
    reason: &str,
) -> Result<Proposal, String> {
    Ok(Proposal {
        original_recipe: recipe.clone(),
        proposed_recipe: None,
        original_report: baseline,
        proposed_report: None,
        candidates,
        reason: Some(reason.into()),
    })
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("PSX music optimization cancelled".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_recipe() -> Recipe {
        let mut recipe = Recipe::default();
        recipe.preset = Preset::Custom;
        recipe.bank_budget_bytes = 12_000;
        recipe.optimization.minimum_sample_rate = 8_000;
        recipe.optimization.max_candidates = 32;
        recipe
    }

    fn report(recipe: &Recipe, bytes: usize) -> Report {
        Report {
            reverb_spu_bytes: recipe.reverb_bytes(),
            profile: "test".into(),
            recipe: recipe.clone(),
            note_events: 0,
            regions: 0,
            samples: 0,
            song_peak_layers_per_note: 0,
            sample_spu_bytes: bytes,
            other_resident_bytes: recipe.other_resident_bytes,
            available_bank_bytes: recipe.available_bytes(),
            fits_sample_budget: bytes <= recipe.available_bytes() as usize,
            adaptations: vec![],
            squared_error: 0,
            encoded_input_frames: 0,
            maximum_loop_step: 0.,
            accounting: Default::default(), loops: vec![],
        }
    }

    #[test]
    fn baseline_that_fits_is_reported_without_a_changed_recipe() {
        let recipe = test_recipe();
        let cancelled = AtomicBool::new(false);
        let proposal = propose_with(&recipe, &cancelled, |candidate, _| {
            Ok(report(candidate, 10_000))
        })
        .unwrap();
        assert!(proposal.proposed_recipe.is_none());
        assert_eq!(proposal.candidates.len(), 1);
        assert!(proposal.reason.unwrap().contains("already fits"));
    }

    #[test]
    fn ladder_proposes_the_highest_tested_fitting_rate_and_preserves_inputs() {
        let recipe = test_recipe();
        let original = recipe.clone();
        let cancelled = AtomicBool::new(false);
        let proposal = propose_with(&recipe, &cancelled, |candidate, _| {
            Ok(report(candidate, candidate.max_sample_rate as usize))
        })
        .unwrap();
        let proposed = proposal.proposed_recipe.unwrap();
        assert_eq!(recipe, original);
        assert_eq!(proposed.preset, Preset::Custom);
        assert_eq!(proposed.max_sample_rate, 12_000);
        assert!(proposal.candidates.len() <= MAX_CANDIDATES);
        assert!(proposal
            .candidates
            .windows(2)
            .all(|pair| pair[0].ordinal + 1 == pair[1].ordinal));
    }

    #[test]
    fn disabled_search_and_candidate_failure_do_not_make_an_implicit_proposal() {
        let mut recipe = test_recipe();
        recipe.optimization.allow_lower_rate = false;
        let cancelled = AtomicBool::new(false);
        let disabled = propose_with(&recipe, &cancelled, |candidate, _| {
            Ok(report(candidate, 99_999))
        })
        .unwrap();
        assert!(disabled.proposed_recipe.is_none());
        assert_eq!(disabled.candidates.len(), 1);

        let recipe = test_recipe();
        let failed = propose_with(&recipe, &cancelled, |candidate, _| {
            if candidate.max_sample_rate == recipe.optimization.minimum_sample_rate {
                Err("synthetic cooker failure".into())
            } else {
                Ok(report(candidate, 99_999))
            }
        });
        assert!(failed.unwrap_err().contains("synthetic cooker failure"));
    }

    #[test]
    fn cancellation_prevents_the_first_cook() {
        let cancelled = AtomicBool::new(true);
        let result = propose_with(&test_recipe(), &cancelled, |_, _| {
            panic!("cancelled proposal must not cook")
        });
        assert!(result.unwrap_err().contains("cancelled"));
    }
}
