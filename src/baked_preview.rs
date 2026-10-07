//! Host diagnostics for the *actual* streaming bake. No visibility selection is
//! inferred from the observer camera, and none of this state is saved in a map.
use crate::scene::Scene;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use uuid::Uuid;

pub const FILE: &str = "scene-stream-bake.json";
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub actor: Uuid,
    pub page: u32,
    pub word: usize,
    pub quads: Vec<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bake {
    pub version: u32,
    pub signature: String,
    pub pool_pages: usize,
    pub chunks: Vec<Chunk>,
    pub samples: Vec<BTreeSet<usize>>,
    pub edges: Vec<(usize, usize)>,
    /// Version 3: render-page sets may differ from query-mesh chunks and
    /// include streamed order blocks. Selection still uses source face bits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_pages: Option<Vec<BTreeSet<u32>>>,
}
pub type Selection = BTreeMap<Uuid, BTreeSet<usize>>;

pub fn signature(scene: &Scene) -> String {
    // Geometry documents are resolved but deliberately omitted from authored
    // map serialization. Include them explicitly, plus world transforms and
    // texture data. A transform/material/asset/track edit must invalidate bake.
    let geometry: Vec<_> = scene
        .actors
        .iter()
        .enumerate()
        .filter(|(_, a)| a.kind == "Mesh")
        .map(|(i, a)| {
            serde_json::json!({"id":a.id,"active":a.active,
            "world":scene.world_matrix(i).0,"material":a.material,
            "mesh_component":a.editable_mesh,
            "mesh":a.editable_mesh.as_ref().and_then(|m|m.document.as_deref()),
            "terrain":a.terrain})
        })
        .collect();
    let textures: Vec<_> = scene
        .textures
        .iter()
        .map(|(id, t)| (id, t.width, t.height, crate::assets::hash(&t.rgba)))
        .collect();
    let base = crate::assets::hash(
        &serde_json::to_vec(&(
            2, // lossless authored topology and per-chunk Q12 vertex width
            geometry,
            textures,
            &scene.visibility_track,
            &scene.environment,
            &scene.fog,
        ))
        .unwrap(),
    );
    // Preserve existing bake signatures for ordinary scenes. A prepared program
    // adds its own identity without invalidating unrelated version-2 bakes.
    scene.ordered_draw.as_ref().map_or(base.clone(), |program|
        crate::assets::hash(&serde_json::to_vec(&(3,base,program)).unwrap()))
}
impl Bake {
    pub fn pages(&self, sample: usize) -> BTreeSet<u32> {
        if let Some(pages)=&self.sample_pages { return pages[sample].clone(); }
        self.chunks
            .iter()
            .filter(|c| {
                self.samples[sample]
                    .range(c.word * 32..c.word * 32 + c.quads.len())
                    .next()
                    .is_some()
            })
            .map(|c| c.page)
            .collect()
    }
    pub fn working_pages(&self, sample: usize, next: Option<usize>) -> BTreeSet<u32> {
        let mut pages = self.pages(sample);
        if let Some(next) = next {
            pages.extend(self.pages(next));
        }
        pages
    }
    pub fn neighbours(&self, sample: usize) -> BTreeSet<usize> {
        self.edges
            .iter()
            .filter_map(|&(a, b)| {
                if a == sample {
                    Some(b)
                } else if b == sample {
                    Some(a)
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn selection(&self, sample: Option<usize>) -> Selection {
        let mut result = Selection::new();
        for c in &self.chunks {
            let faces = result.entry(c.actor).or_default();
            if let Some(sample) = sample {
                for (q, &index) in c.quads.iter().enumerate() {
                    if self.samples[sample].contains(&(c.word * 32 + q)) {
                        faces.insert(index);
                    }
                }
            }
        }
        result
    }
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.version,2|3)
            || (self.version==3)!=self.sample_pages.is_some()
            || self.sample_pages.as_ref().is_some_and(|s|s.len()!=self.samples.len())
            || self.samples.is_empty()
            || self.pool_pages == 0
            || self.pool_pages > 8
        {
            return Err("Unsupported or invalid streaming bake".into());
        }
        let mut known = BTreeSet::new();
        let mut mapped = BTreeSet::new();
        for c in &self.chunks {
            if c.word >= 2048
                || c.quads.is_empty()
                || c.quads.len() > 96
                || c.word * 32 + c.quads.len() > 65536
            {
                return Err("Bake has invalid chunk bounds".into());
            }
            for (q, &index) in c.quads.iter().enumerate() {
                if !known.insert(c.word * 32 + q) || !mapped.insert((c.actor, index)) {
                    return Err("Bake has overlapping face mappings".into());
                }
            }
        }
        if self.samples.iter().any(|s| !s.is_subset(&known))
            || (0..self.samples.len()).any(|i| self.pages(i).len() > self.pool_pages)
        {
            return Err("Bake has invalid face references or residency budget".into());
        }
        for &(a, b) in &self.edges {
            if a >= self.samples.len()
                || b >= self.samples.len()
                || self.working_pages(a, Some(b)).len() > self.pool_pages
            {
                return Err("Bake has an invalid continuous transition".into());
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct State {
    pub enabled: bool,
    pub streaming: bool,
    pub sample: usize,
    pub next: Option<usize>,
    pub bake: Option<Bake>,
    pub error: Option<String>,
    pub selection: Selection,
    pub resident: BTreeSet<u32>,
    pub requested: BTreeSet<u32>,
    pub committed: Option<usize>,
    pub committed_next: Option<usize>,
    pub revision: u64,
}
impl State {
    pub fn invalidate(&mut self) {
        if self.bake.take().is_some() {
            self.error = Some("Bake is stale. Build PSX and reload it before simulating.".into());
            self.selection.clear();
            self.revision += 1;
        }
    }
    pub fn load(&mut self, root: &Path, scene: &Scene) {
        let result = (|| {
            let bytes = std::fs::read(root.join(".epok/build").join(FILE))
                .map_err(|_| "No bake for this map. Build PSX first.".to_string())?;
            let bakes: Vec<Bake> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            let signature = signature(scene);
            let bake = bakes
                .into_iter()
                .find(|b| b.signature == signature)
                .ok_or("Bake is stale or belongs to another map. Build PSX first.")?;
            bake.validate()?;
            Ok::<_, String>(bake)
        })();
        self.resident.clear();
        self.requested.clear();
        self.committed = None;
        self.committed_next = None;
        self.selection.clear();
        match result {
            Ok(b) => {
                self.sample = self.sample.min(b.samples.len() - 1);
                self.bake = Some(b);
                self.error = None;
                self.refresh();
            }
            Err(e) => {
                self.bake = None;
                self.error = Some(e);
            }
        }
        self.revision += 1;
    }
    pub fn refresh(&mut self) {
        self.revision += 1;
        let Some(bake) = &self.bake else {
            return;
        };
        self.sample = self.sample.min(bake.samples.len() - 1);
        if self
            .next
            .is_some_and(|n| !bake.neighbours(self.sample).contains(&n))
        {
            self.next = None;
        }
        let required = bake.working_pages(self.sample, self.next);
        // Scrubbing is an explicit cut, not free-camera movement. Preserve the
        // last complete draw selection until the entire destination is ready.
        self.requested = required.difference(&self.resident).copied().collect();
        if !self.streaming || self.requested.is_empty() {
            self.committed = Some(self.sample);
            self.committed_next = self.next;
        }
        self.selection = bake.selection(self.committed);
        if let Some(next) = self.committed_next {
            for (actor, quads) in bake.selection(Some(next)) {
                self.selection.entry(actor).or_default().extend(quads);
            }
        }
    }
    pub fn read_page(&mut self) {
        let Some(page) = self.requested.first().copied() else {
            return;
        };
        let bake = self.bake.as_ref().unwrap();
        let needed = bake.working_pages(self.sample, self.next);
        if self.resident.len() >= bake.pool_pages {
            if let Some(old) = self.resident.difference(&needed).next().copied() {
                self.resident.remove(&old);
            } else {
                self.error = Some("Requested working set exceeds the baked pool".into());
                return;
            }
        }
        self.resident.insert(page);
        self.refresh();
    }
    pub fn filter(&self) -> Option<&Selection> {
        (self.enabled && self.bake.is_some() && self.error.is_none()).then_some(&self.selection)
    }
}

pub fn ui(ui: &imgui::Ui, editor: &mut crate::editor::Editor) {
    if editor.scene.visibility_track.is_none() {
        return;
    }
    if ui.checkbox(
        "Simulate baked visibility",
        &mut editor.baked_preview.enabled,
    ) {
        if editor.baked_preview.enabled && editor.baked_preview.bake.is_none() {
            editor.baked_preview.load(&editor.root, &editor.scene);
        }
        editor.baked_preview.revision += 1;
        editor.view_dirty = true;
    }
    if !editor.baked_preview.enabled {
        return;
    }
    ui.same_line();
    if ui.button("Reload bake") {
        editor.baked_preview.load(&editor.root, &editor.scene);
        editor.view_dirty = true;
    }
    ui.same_line();
    if ui.button("Build PSX bake") {
        editor.build(false);
    }
    if let Some(error) = &editor.baked_preview.error {
        ui.text_colored([1., 0.65, 0.25, 1.], error);
        ui.text_disabled("Showing the unrestricted authoring map.");
        return;
    }
    let Some(bake) = &editor.baked_preview.bake else {
        return;
    };
    let mut sample = editor.baked_preview.sample as i32;
    if ui.slider(
        "Baked sample",
        0,
        bake.samples.len() as i32 - 1,
        &mut sample,
    ) {
        editor.baked_preview.sample = sample as usize;
        editor.baked_preview.refresh();
        editor.view_dirty = true;
    }
    let neighbours = editor
        .baked_preview
        .bake
        .as_ref()
        .unwrap()
        .neighbours(editor.baked_preview.sample);
    let mut options = vec!["None (sample / cut)".to_string()];
    options.extend(neighbours.iter().map(|n| format!("Sample {n}")));
    let mut choice = editor
        .baked_preview
        .next
        .and_then(|n| neighbours.iter().position(|&v| v == n))
        .map_or(0, |n| n + 1);
    if ui.combo_simple_string("Continuous edge to", &mut choice, &options) {
        editor.baked_preview.next = choice
            .checked_sub(1)
            .and_then(|n| neighbours.iter().nth(n).copied());
        editor.baked_preview.refresh();
        editor.view_dirty = true;
    }
    if ui.checkbox(
        "Simulate streaming (manual I/O)",
        &mut editor.baked_preview.streaming,
    ) {
        editor.baked_preview.resident.clear();
        editor.baked_preview.committed = None;
        editor.baked_preview.committed_next = None;
        editor.baked_preview.refresh();
        editor.view_dirty = true;
    }
    if editor.baked_preview.streaming {
        ui.same_line();
        if ui.button("Complete one page read") {
            editor.baked_preview.read_page();
            editor.view_dirty = true;
        }
        let state = &editor.baked_preview;
        ui.text(format!(
            "Resident {:?} | Requested {:?} | Displayed {:?} -> {:?}",
            state.resident, state.requested, state.committed, state.committed_next
        ));
        ui.text_disabled("A continuous edge shows both samples. Scrubbing is a cut; the complete working set must be ready.");
        ui.text_disabled("Manual I/O tests residency, not CD latency or PSX FPS.");
    }
    ui.text_disabled("Observer camera is free; it does not select visibility. Disable simulation to edit the entire map.");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Bake {
        Bake {
            version: 2,
            signature: "test".into(),
            pool_pages: 2,
            chunks: vec![
                Chunk {
                    actor: Uuid::nil(),
                    page: 0,
                    word: 0,
                    quads: vec![3, 5],
                },
                Chunk {
                    actor: Uuid::nil(),
                    page: 1,
                    word: 1,
                    quads: vec![8],
                },
                Chunk {
                    actor: Uuid::nil(),
                    page: 2,
                    word: 2,
                    quads: vec![12],
                },
            ],
            samples: vec![BTreeSet::from([1, 32]), BTreeSet::from([64])],
            edges: vec![],
            sample_pages: None,
        }
    }
    #[test]
    fn defaults_are_unrestricted_and_cooked_quad_mapping_is_exact() {
        assert!(State::default().filter().is_none());
        let b = fixture();
        b.validate().unwrap();
        assert_eq!(b.selection(Some(0))[&Uuid::nil()], BTreeSet::from([5, 8]));
        assert_eq!(b.pages(0), BTreeSet::from([0, 1]));
    }
    #[test]
    fn continuous_edge_waits_for_and_displays_the_union() {
        let mut bake = fixture();
        bake.pool_pages = 3;
        bake.edges = vec![(0, 1)];
        bake.validate().unwrap();
        let mut state = State {
            enabled: true,
            streaming: true,
            bake: Some(bake),
            ..Default::default()
        };
        state.refresh();
        state.read_page();
        state.read_page();
        assert_eq!(state.committed, Some(0));
        state.next = Some(1);
        state.refresh();
        assert_eq!(state.committed_next, None);
        state.read_page();
        assert_eq!(state.committed_next, Some(1));
        assert_eq!(state.selection[&Uuid::nil()], BTreeSet::from([5, 8, 12]));
        state.sample = 1;
        state.next = Some(0);
        state.refresh();
        assert_eq!(state.selection[&Uuid::nil()], BTreeSet::from([5, 8, 12]));
        let mut invalid = fixture();
        invalid.edges = vec![(0, 1)];
        assert!(invalid.validate().is_err());
        invalid.edges = vec![(0, usize::MAX)];
        assert!(invalid.validate().is_err());
    }
    #[test]
    fn cold_cut_never_commits_a_partially_resident_sample() {
        let mut state = State {
            enabled: true,
            streaming: true,
            bake: Some(fixture()),
            ..Default::default()
        };
        state.refresh();
        assert_eq!(state.committed, None);
        state.read_page();
        assert_eq!(state.committed, None);
        state.read_page();
        assert_eq!(state.committed, Some(0));
        state.sample = 1;
        state.refresh();
        assert_eq!(state.committed, Some(0));
        state.read_page();
        assert_eq!(state.committed, Some(1));
        assert_eq!(state.resident.len(), 2);
        state.invalidate();
        assert!(state.filter().is_none());
        assert!(state.error.is_some());
    }
    #[test]
    fn malformed_or_oversized_bakes_are_rejected() {
        let mut b = fixture();
        b.samples[0].insert(31);
        assert!(b.validate().is_err());
        let mut b = fixture();
        b.pool_pages = 1;
        assert!(b.validate().is_err());
        let mut b = fixture();
        b.chunks[0].word = usize::MAX;
        assert!(b.validate().is_err());
        let mut b = fixture();
        b.chunks.push(b.chunks[0].clone());
        assert!(b.validate().is_err());
    }
}
