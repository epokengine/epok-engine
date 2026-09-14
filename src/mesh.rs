//! Authored geometry. UUID packages own the editable source; scene references only own overrides.
use crate::{
    assets,
    scene::{Actor, Material, Scene},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub asset: Uuid,
    #[serde(default)]
    pub materials: BTreeMap<Uuid, Material>,
    #[serde(skip)]
    pub document: Option<Arc<Document>>,
    #[serde(skip)]
    pub error: Option<String>,
}
impl Component {
    pub fn new(asset: Uuid) -> Self {
        Self {
            asset,
            materials: BTreeMap::new(),
            document: None,
            error: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub parent: Option<Uuid>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Slot {
    pub id: Uuid,
    pub name: String,
    pub material: Material,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Face {
    pub id: Uuid,
    pub vertices: [u32; 4],
    pub group: Uuid,
    pub material: Uuid,
    #[serde(default = "default_uv")]
    pub uv: [[f32; 2]; 4],
}
fn default_uv() -> [[f32; 2]; 4] {
    [[0., 0.], [1., 0.], [1., 1.], [0., 1.]]
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub version: u32,
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<Face>,
    pub groups: Vec<Group>,
    pub materials: Vec<Slot>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            vertices: vec![],
            faces: vec![],
            groups: vec![Group {
                id: Uuid::new_v4(),
                name: "Geometry".into(),
                parent: None,
            }],
            materials: vec![Slot {
                id: Uuid::new_v4(),
                name: "Surface".into(),
                material: Material::default(),
            }],
        }
    }
}
pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn face_normal(p: [[f32; 3]; 4]) -> [f32; 3] {
    crate::lighting::unit(cross(
        crate::lighting::sub(p[1], p[0]),
        crate::lighting::sub(p[2], p[0]),
    ))
}
fn name(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 128 && !s.chars().any(char::is_control)
}
impl Document {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let doc: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        doc.validate()?;
        Ok(doc)
    }
    pub fn points(&self, f: &Face) -> [[f32; 3]; 4] {
        f.vertices.map(|v| self.vertices[v as usize])
    }
    pub fn contains_group(&self, child: Uuid, parent: Uuid) -> bool {
        let mut current = Some(child);
        for _ in 0..=self.groups.len() {
            let Some(id) = current else {
                return false;
            };
            if id == parent {
                return true;
            }
            current = self
                .groups
                .iter()
                .find(|g| g.id == id)
                .and_then(|g| g.parent);
        }
        false
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.vertices.len() > 16384
            || self.faces.len() > 3500
            || self.groups.is_empty()
            || self.groups.len() > 256
            || self.materials.is_empty()
            || self.materials.len() > 64
        {
            return Err("Mesh limits: 16384 vertices, 3500 faces, 256 groups and 64 materials; version 1 required".into());
        }
        let mut ids = BTreeSet::new();
        for id in self
            .groups
            .iter()
            .map(|g| g.id)
            .chain(self.materials.iter().map(|m| m.id))
            .chain(self.faces.iter().map(|f| f.id))
        {
            if id.is_nil() || !ids.insert(id) {
                return Err("Mesh IDs must be unique and nonzero".into());
            }
        }
        for g in &self.groups {
            if !name(&g.name) {
                return Err("Invalid group name".into());
            }
            if let Some(p) = g.parent
                && (!self.groups.iter().any(|g| g.id == p) || self.contains_group(p, g.id))
            {
                return Err("Missing group parent or hierarchy cycle".into());
            }
        }
        for s in &self.materials {
            crate::texture::validate_material(&s.material)?;
            if !name(&s.name)
                || s.material
                    .color
                    .iter()
                    .any(|c| !c.is_finite() || !(0. ..=1.).contains(c))
            {
                return Err("Invalid mesh material".into());
            }
        }
        if self
            .vertices
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 128.)
        {
            return Err("Editable vertices must stay within local ±128 units".into());
        }
        let mut compiled = 0;
        for f in &self.faces {
            if f.vertices
                .iter()
                .any(|v| *v as usize >= self.vertices.len())
                || !self.groups.iter().any(|g| g.id == f.group)
                || !self.materials.iter().any(|m| m.id == f.material)
                || f.uv
                    .iter()
                    .flatten()
                    .any(|u| !u.is_finite() || u.abs() > 4096.)
            {
                return Err("Invalid face vertex, group, material or UV reference".into());
            }
            let p = self.points(f);
            let quantized = p.map(|v| v.map(|v| (v * 4096.).round() / 4096.));
            if crate::lighting::dot(face_normal(quantized), face_normal(quantized)) < 0.5 {
                return Err("A face collapses at PSX Q12 precision; increase its size".into());
            }
            let n = face_normal(p);
            let span = (0..3)
                .map(|c| {
                    p.iter().map(|v| v[c]).fold(f32::NEG_INFINITY, f32::max)
                        - p.iter().map(|v| v[c]).fold(f32::INFINITY, f32::min)
                })
                .fold(0_f32, f32::max);
            compiled += ((span / 4.).ceil().max(1.) as usize).pow(2);
            if compiled > 3500 {
                return Err(
                    "Compiled geometry exceeds 7000 triangles after large-surface subdivision"
                        .into(),
                );
            }
            if crate::lighting::dot(n, n) < 0.5 {
                return Err("Degenerate face: vertices are collinear or repeated".into());
            }
            if crate::lighting::dot(crate::lighting::sub(p[3], p[0]), n).abs() > 0.001 {
                return Err("Faces must be planar. Split the face before moving individual vertices out of its plane.".into());
            }
            if f.vertices[2] != f.vertices[3] {
                for i in 0..4 {
                    if crate::lighting::dot(
                        cross(
                            crate::lighting::sub(p[(i + 1) % 4], p[i]),
                            crate::lighting::sub(p[(i + 2) % 4], p[(i + 1) % 4]),
                        ),
                        n,
                    ) < 1e-7
                    {
                        return Err("Faces must be convex with consistent winding".into());
                    }
                }
            }
        }
        Ok(())
    }
    pub fn add_face(&mut self, p: [[f32; 3]; 4], group: Uuid, material: Uuid) -> Uuid {
        let vertices = p.map(|p| {
            self.vertices
                .iter()
                .position(|v| *v == p)
                .unwrap_or_else(|| {
                    self.vertices.push(p);
                    self.vertices.len() - 1
                }) as u32
        });
        let id = Uuid::new_v4();
        self.faces.push(Face {
            id,
            vertices,
            group,
            material,
            uv: default_uv(),
        });
        id
    }
    pub fn primitive(
        &mut self,
        shape: &str,
        origin: [f32; 3],
        size: [f32; 3],
        steps: usize,
        group: Uuid,
        slot: Uuid,
    ) {
        let transform = |p: [f32; 3]| std::array::from_fn(|c| origin[c] + p[c] * size[c]);
        if shape == "Plane" {
            self.add_face(
                [
                    [-0.5, 0., -0.5],
                    [-0.5, 0., 0.5],
                    [0.5, 0., 0.5],
                    [0.5, 0., -0.5],
                ]
                .map(transform),
                group,
                slot,
            );
            return;
        }
        if shape == "Stairs" {
            for i in 0..steps {
                let mut o = origin;
                let mut s = size;
                s[2] /= steps as f32;
                s[1] *= (i + 1) as f32 / steps as f32;
                o[2] += -size[2] * 0.5 + s[2] * (i as f32 + 0.5);
                o[1] += -size[1] * 0.5 + s[1] * 0.5;
                self.primitive("Box", o, s, 1, group, slot);
            }
            return;
        }
        let mut corners = crate::lighting::CORNERS;
        if shape == "Ramp" {
            corners[2][1] = -0.5;
            corners[3][1] = -0.5;
        }
        for indices in crate::lighting::FACES {
            let p = [indices[0], indices[3], indices[2], indices[1]].map(|i| transform(corners[i]));
            let mut unique = vec![];
            for v in p {
                if !unique.contains(&v) {
                    unique.push(v);
                }
            }
            if unique.len() == 3 {
                unique.push(unique[2]);
            }
            if unique.len() == 4 {
                self.add_face(unique.try_into().unwrap(), group, slot);
            }
        }
    }
    pub fn extrude(&mut self, selected: &BTreeSet<Uuid>, distance: f32) -> Result<(), String> {
        if !distance.is_finite() || distance.abs() < 0.001 {
            return Err("Choose a nonzero extrusion distance".into());
        }
        let faces = self
            .faces
            .iter()
            .filter(|f| selected.contains(&f.id))
            .cloned()
            .collect::<Vec<_>>();
        for f in faces {
            let p = self.points(&f);
            let n = face_normal(p);
            let top = p.map(|v| std::array::from_fn(|c| v[c] + n[c] * distance));
            let offset = self.vertices.len() as u32;
            self.vertices.extend(top);
            let face = self.faces.iter_mut().find(|v| v.id == f.id).unwrap();
            face.vertices = [
                offset,
                offset + 1,
                offset + 2,
                if p[2] == p[3] { offset + 2 } else { offset + 3 },
            ];
            for i in 0..4 {
                let j = (i + 1) % 4;
                if p[i] != p[j] {
                    self.add_face([p[i], p[j], top[j], top[i]], f.group, f.material);
                }
            }
        }
        self.validate()
    }
    pub fn split(&mut self, selected: &BTreeSet<Uuid>) {
        let faces = self
            .faces
            .iter()
            .filter(|f| selected.contains(&f.id))
            .cloned()
            .collect::<Vec<_>>();
        for f in faces {
            let p = self.points(&f);
            let count = if f.vertices[2] == f.vertices[3] { 3 } else { 4 };
            let center = std::array::from_fn(|c| {
                p[..count].iter().map(|v| v[c]).sum::<f32>() / count as f32
            });
            let uv_center: [f32; 2] = std::array::from_fn(|c| {
                f.uv[..count].iter().map(|v| v[c]).sum::<f32>() / count as f32
            });
            let mid: Vec<_> = (0..count)
                .map(|i| std::array::from_fn(|c| (p[i][c] + p[(i + 1) % count][c]) * 0.5))
                .collect();
            self.faces.retain(|v| v.id != f.id);
            for i in 0..count {
                self.add_face(
                    [p[i], mid[i], center, mid[(i + count - 1) % count]],
                    f.group,
                    f.material,
                );
                self.faces.last_mut().unwrap().uv = [
                    f.uv[i],
                    std::array::from_fn(|c| (f.uv[i][c] + f.uv[(i + 1) % count][c]) * 0.5),
                    uv_center,
                    std::array::from_fn(|c| (f.uv[i][c] + f.uv[(i + count - 1) % count][c]) * 0.5),
                ];
            }
        }
    }
    pub fn selected_vertices(&self, faces: &BTreeSet<Uuid>) -> BTreeSet<u32> {
        self.faces
            .iter()
            .filter(|f| faces.contains(&f.id))
            .flat_map(|f| f.vertices)
            .collect()
    }
    pub fn subset(&self, faces: &BTreeSet<Uuid>) -> Self {
        let mut d = self.clone();
        d.faces.retain(|f| faces.contains(&f.id));
        d.compact();
        d
    }
    pub fn compact(&mut self) {
        let used = self
            .faces
            .iter()
            .flat_map(|f| f.vertices)
            .collect::<BTreeSet<_>>();
        let map = used
            .iter()
            .enumerate()
            .map(|(i, v)| (*v, i as u32))
            .collect::<BTreeMap<_, _>>();
        self.vertices = used.iter().map(|i| self.vertices[*i as usize]).collect();
        for f in &mut self.faces {
            f.vertices = f.vertices.map(|v| map[&v]);
        }
    }
}
pub fn create(root: &Path, path: &str, document: &Document) -> Result<Uuid, String> {
    document.validate()?;
    let id = Uuid::new_v4();
    let source = serde_json::to_vec(document).map_err(|e| e.to_string())?;
    let package = assets::Package {
        meta: assets::Metadata {
            version: 1,
            id,
            kind: assets::Kind::EditableMesh,
            importer_version: 1,
            source: path.into(),
            source_hash: assets::hash(&source),
            settings: crate::import_settings::Settings::Authored,
            extra: Default::default(),
        },
        source,
    };
    assets::atomic_write(&assets::inside(root, path)?, &package.bytes()?, None)?;
    Ok(id)
}
pub fn document(record: &assets::Record) -> Result<Document, String> {
    if record.meta.kind != assets::Kind::EditableMesh {
        return Err("Select an EditableMesh asset".into());
    }
    Document::parse(&assets::Package::load(&record.path)?.source)
}
pub fn save(record: &assets::Record, doc: &Document) -> Result<String, String> {
    doc.validate()?;
    let mut package = assets::Package::load(&record.path)?;
    if package.meta.kind != assets::Kind::EditableMesh || package.meta.id != record.meta.id {
        return Err("The mesh asset was replaced; reload before editing".into());
    }
    package.source = serde_json::to_vec(doc).map_err(|e| e.to_string())?;
    package.meta.source_hash = assets::hash(&package.source);
    let bytes = package.bytes()?;
    assets::atomic_write(&record.path, &bytes, Some(&record.revision))?;
    Ok(assets::hash(&bytes))
}
pub fn resolve(scene: &mut Scene, index: &assets::Index) -> Result<(), String> {
    let mut errors = vec![];
    let mut loaded = BTreeMap::new();
    for e in &mut scene.actors {
        if let Some(m) = &mut e.editable_mesh {
            let result = loaded
                .entry(m.asset)
                .or_insert_with(|| index.resolve(m.asset).and_then(document).map(Arc::new));
            match result {
                Ok(doc) => {
                    m.document = Some(doc.clone());
                    m.error = None;
                }
                Err(error) => {
                    m.document = None;
                    m.error = Some(error.clone());
                    errors.push(format!("{}: {error}", e.name));
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
pub fn material(e: &Actor, slot: Uuid) -> Material {
    let Some(m) = &e.editable_mesh else {
        return e.material.clone();
    };
    m.materials
        .get(&slot)
        .cloned()
        .or_else(|| {
            m.document
                .as_ref()?
                .materials
                .iter()
                .find(|s| s.id == slot)
                .map(|s| s.material.clone())
        })
        .unwrap_or_default()
}
pub fn hit(origin: [f32; 3], direction: [f32; 3], p: [[f32; 3]; 3]) -> Option<f32> {
    let e1 = crate::lighting::sub(p[1], p[0]);
    let e2 = crate::lighting::sub(p[2], p[0]);
    let h = cross(direction, e2);
    let det = crate::lighting::dot(e1, h);
    if det.abs() < 1e-8 {
        return None;
    }
    let s = crate::lighting::sub(origin, p[0]);
    let u = crate::lighting::dot(s, h) / det;
    if !(0. ..=1.).contains(&u) {
        return None;
    }
    let q = cross(s, e1);
    let v = crate::lighting::dot(direction, q) / det;
    if v < 0. || u + v > 1. {
        return None;
    }
    let t = crate::lighting::dot(e2, q) / det;
    (t >= 0.).then_some(t)
}
pub fn pick(
    doc: &Document,
    origin: [f32; 3],
    direction: [f32; 3],
    visible: impl Fn(&Face) -> bool,
) -> Option<(Uuid, f32)> {
    doc.faces
        .iter()
        .filter(|f| visible(f))
        .filter_map(|f| {
            let p = doc.points(f);
            [
                hit(origin, direction, [p[0], p[1], p[2]]),
                hit(origin, direction, [p[0], p[2], p[3]]),
            ]
            .into_iter()
            .flatten()
            .min_by(f32::total_cmp)
            .map(|t| (f.id, t))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn shape(shape: &str) -> Document {
        let mut doc = Document::default();
        doc.primitive(
            shape,
            [0.; 3],
            [2.; 3],
            4,
            doc.groups[0].id,
            doc.materials[0].id,
        );
        doc
    }
    #[test]
    fn shapes_are_welded_outward_and_remain_editable() {
        for kind in ["Box", "Plane", "Ramp", "Stairs"] {
            let doc = shape(kind);
            doc.validate().unwrap_or_else(|e| panic!("{kind}: {e}"));
            if kind == "Box" {
                assert_eq!(doc.vertices.len(), 8);
                for f in &doc.faces {
                    let p = doc.points(f);
                    let center = std::array::from_fn(|c| p.iter().map(|p| p[c]).sum::<f32>() / 4.);
                    assert!(crate::lighting::dot(center, face_normal(p)) > 0.9);
                }
            }
            for face in &doc.faces {
                let selection = BTreeSet::from([face.id]);
                let mut extruded = doc.clone();
                extruded
                    .extrude(&selection, 0.5)
                    .unwrap_or_else(|e| panic!("Extrude {kind}: {e}"));
                assert!(extruded.faces.iter().any(|f| f.id == face.id));
                let mut split = doc.clone();
                split.split(&selection);
                split
                    .validate()
                    .unwrap_or_else(|e| panic!("Split {kind}: {e}"));
                assert!(
                    split
                        .faces
                        .iter()
                        .all(|f| f.material == face.material && f.group == face.group)
                );
            }
        }
    }
    #[test]
    fn validation_rejects_cycles_concavity_bad_ids_and_build_expansion() {
        let original = shape("Box");
        let mut doc = original.clone();
        doc.groups[0].parent = Some(doc.groups[0].id);
        assert!(doc.validate().is_err());
        let mut doc = original.clone();
        doc.faces[1].id = doc.faces[0].id;
        assert!(doc.validate().is_err());
        let mut doc = original.clone();
        doc.vertices[0][1] += 0.25;
        assert!(doc.validate().is_err());
        let mut doc = shape("Plane");
        for v in &mut doc.vertices {
            v[0] *= 128.;
            v[2] *= 128.;
        }
        assert!(doc.validate().unwrap_err().contains("7000"));
        let mut doc = original;
        doc.faces[0].vertices[0] = u32::MAX;
        assert!(doc.validate().is_err());
    }
    #[test]
    fn asset_move_copy_stale_write_and_missing_reference() {
        let root = std::env::temp_dir().join(format!("epok-mesh-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let doc = shape("Ramp");
        let id = create(&root, "assets/Ramp.epokasset", &doc).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap();
        let mut edited = doc.clone();
        edited.groups[0].name = "Walls".into();
        save(record, &edited).unwrap();
        assert!(
            save(record, &doc).is_err(),
            "Stale write must not overwrite external edits"
        );
        let renamed = root.join("assets/Moved.epokasset");
        std::fs::rename(&record.path, &renamed).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        let record = index.resolve(id).unwrap();
        assert_eq!(document(record).unwrap(), edited);
        let copy = assets::duplicate(record, &root.join("assets/Copy.epokasset")).unwrap();
        assert_ne!(copy, id);
        let mut scene = Scene::default();
        scene.actors[1].editable_mesh = Some(Component::new(id));
        resolve(&mut scene, &index).unwrap();
        let serialized = serde_json::to_string(&scene).unwrap();
        assert!(!serialized.contains("\"vertices\""));
        std::fs::remove_file(renamed).unwrap();
        let index = assets::scan(&root, &mut Default::default());
        assert!(resolve(&mut scene, &index).is_err());
        assert_eq!(scene.actors[1].editable_mesh.as_ref().unwrap().asset, id);
        assert!(
            scene.actors[1]
                .editable_mesh
                .as_ref()
                .unwrap()
                .document
                .is_none()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn compiler_preserves_material_ids_and_bake_offsets_across_chunks() {
        let mut doc = shape("Plane");
        let slot = Uuid::new_v4();
        doc.materials.push(Slot {
            id: slot,
            name: "Walls".into(),
            material: Material {
                color: [1., 0., 0.],
                unlit: true,
                ..Default::default()
            },
        });
        doc.primitive(
            "Ramp",
            [20., 0., 0.],
            [8., 4., 8.],
            1,
            doc.groups[0].id,
            slot,
        );
        doc.validate().unwrap();
        let mut e = Actor::cube("Room".into());
        let mut m = Component::new(Uuid::new_v4());
        m.materials.insert(
            slot,
            Material {
                color: [0., 0., 1.],
                unlit: true,
                ..Default::default()
            },
        );
        doc.materials.reverse();
        m.document = Some(Arc::new(doc));
        e.editable_mesh = Some(m);
        let quads = crate::lighting::quads(&e);
        let chunks = crate::mesh_compile::chunks(&quads).unwrap();
        assert!(chunks.len() > 1);
        let mut offsets = BTreeSet::new();
        for c in chunks {
            assert!(c.vertices.len() <= 384);
            for (indices, i) in c.faces {
                assert!(offsets.insert(i));
                for (k, vertex) in indices.iter().enumerate() {
                    let actual: [f32; 3] = std::array::from_fn(|a| {
                        c.origin[a] + c.vertices[*vertex as usize][a] as f32 / 4096.
                    });
                    assert_eq!(actual, quads[i].points[k]);
                    for axis in 0..3 {
                        assert!(
                            (i32::from(c.vertices[*vertex as usize][axis]) - c.center[axis]).abs()
                                <= c.extent[axis]
                        );
                    }
                }
            }
        }
        assert_eq!(offsets.len(), quads.len());
        assert!(quads.iter().any(|q| q.material.color == [0., 0., 1.]));
        assert!(!quads.iter().any(|q| q.material.color == [1., 0., 0.]));
        let header = crate::mesh_compile::header(&e, 0).unwrap();
        assert!(header.contains("{0,0,255}"));
    }
}
