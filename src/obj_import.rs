//! Static OBJ import produces ordinary editable mesh assets with UVs and material slots.
use crate::{
    assets,
    mesh::{Document, Face, Group, Slot},
    scene::Material,
};
use std::{collections::BTreeMap, path::Path};
use uuid::Uuid;

fn index(value: &str, count: usize) -> Result<usize, String> {
    let n = value.parse::<i64>().map_err(|_| "Invalid OBJ index")?;
    let resolved = if n < 0 { count as i64 + n } else { n - 1 };
    if n == 0 || resolved < 0 || resolved >= count as i64 {
        Err("OBJ index is outside the preceding vertex/UV list".into())
    } else {
        Ok(resolved as usize)
    }
}
fn number(value: Option<&str>) -> Result<f32, String> {
    let v = value
        .ok_or("Missing OBJ coordinate")?
        .parse::<f32>()
        .map_err(|_| "Invalid OBJ coordinate")?;
    if v.is_finite() {
        Ok(v)
    } else {
        Err("OBJ values must be finite".into())
    }
}
fn named_id(name: &str, slots: &mut Vec<(String, Uuid)>, limit: usize) -> Result<Uuid, String> {
    if let Some((_, id)) = slots.iter().find(|(n, _)| n == name) {
        return Ok(*id);
    }
    if name.is_empty() || name.len() > 128 || slots.len() >= limit {
        return Err("OBJ exceeds group/material name or count limits".into());
    }
    let id = Uuid::new_v4();
    slots.push((name.into(), id));
    Ok(id)
}
/// Ear clipping preserves concave polygons and per-corner UV seams.
fn triangles(points: &[[f32; 3]]) -> Result<Vec<[usize; 3]>, String> {
    if points.len() < 3 || points.len() > 256 {
        return Err("OBJ faces require 3–256 corners".into());
    }
    let mut normal = [0f32; 3];
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let axis = (0..3)
        .max_by(|a, b| normal[*a].abs().total_cmp(&normal[*b].abs()))
        .unwrap();
    let p: Vec<[f32; 2]> = points
        .iter()
        .map(|p| [p[(axis + 1) % 3], p[(axis + 2) % 3]])
        .collect();
    let area = (0..p.len())
        .map(|i| p[i][0] * p[(i + 1) % p.len()][1] - p[(i + 1) % p.len()][0] * p[i][1])
        .sum::<f32>();
    if area.abs() < 1e-8 {
        return Err("OBJ polygon is degenerate or self-intersecting".into());
    }
    let cross = |a: usize, b: usize, c: usize| {
        ((p[b][0] - p[a][0]) * (p[c][1] - p[a][1]) - (p[b][1] - p[a][1]) * (p[c][0] - p[a][0]))
            * area.signum()
    };
    let mut remaining = (0..p.len()).collect::<Vec<_>>();
    let mut out = vec![];
    while remaining.len() > 3 {
        let mut ear = None;
        for i in 0..remaining.len() {
            let a = remaining[(i + remaining.len() - 1) % remaining.len()];
            let b = remaining[i];
            let c = remaining[(i + 1) % remaining.len()];
            if cross(a, b, c) <= 1e-8 {
                continue;
            }
            if remaining.iter().any(|&v| {
                v != a
                    && v != b
                    && v != c
                    && cross(a, b, v) >= -1e-8
                    && cross(b, c, v) >= -1e-8
                    && cross(c, a, v) >= -1e-8
            }) {
                continue;
            }
            ear = Some((i, [a, b, c]));
            break;
        }
        let (i, t) = ear.ok_or(
            "OBJ polygon cannot be triangulated; remove duplicate corners or self-intersections",
        )?;
        out.push(t);
        remaining.remove(i);
    }
    out.push([remaining[0], remaining[1], remaining[2]]);
    Ok(out)
}
pub fn parse(
    source: &str,
    scale: f32,
    materials: &BTreeMap<String, Material>,
) -> Result<Document, String> {
    if source.len() > 8 * 1024 * 1024 || !scale.is_finite() || !(0.0001..=10000.).contains(&scale) {
        return Err("OBJ limit is 8 MiB; scale must be positive and finite".into());
    }
    let mut doc = Document {
        version: 1,
        vertices: vec![],
        faces: vec![],
        groups: vec![],
        materials: vec![],
    };
    let mut uv = Vec::<[f32; 2]>::new();
    let mut groups = vec![];
    let mut slots = vec![];
    let mut group = named_id("Geometry", &mut groups, 256)?;
    let mut material = named_id("Surface", &mut slots, 64)?;
    for (line_no, line) in source.lines().enumerate() {
        let line = line.split('#').next().unwrap().trim();
        let mut words = line.split_whitespace();
        let Some(kind) = words.next() else {
            continue;
        };
        let result = (|| -> Result<(), String> {
            match kind {
                "v" => {
                    if doc.vertices.len() >= 16384 {
                        return Err("OBJ exceeds 16384 vertices".into());
                    }
                    let p = [
                        number(words.next())?,
                        number(words.next())?,
                        number(words.next())?,
                    ];
                    let w = words
                        .next()
                        .map(|v| number(Some(v)))
                        .transpose()?
                        .unwrap_or(1.);
                    if w == 0. {
                        return Err("OBJ homogeneous vertex weight cannot be zero".into());
                    }
                    doc.vertices.push(p.map(|v| v * scale / w));
                }
                "vt" => {
                    if uv.len() >= 65536 {
                        return Err("OBJ exceeds 65536 UV coordinates".into());
                    }
                    let u = number(words.next())?;
                    let v = number(words.next())?;
                    if !(0. ..=1.).contains(&u) || !(0. ..=1.).contains(&v) {
                        return Err("Normalize OBJ UVs to 0..1 or split texture tiles before importing to PSX".into());
                    }
                    uv.push([u, 1. - v]);
                }
                "g" | "o" => {
                    let name = words.collect::<Vec<_>>().join(" ");
                    group = named_id(
                        if name.is_empty() { "Geometry" } else { &name },
                        &mut groups,
                        256,
                    )?;
                }
                "usemtl" => {
                    material = named_id(&words.collect::<Vec<_>>().join(" "), &mut slots, 64)?;
                }
                "f" => {
                    let mut corners = vec![];
                    for word in words {
                        let mut parts = word.split('/');
                        let v = index(parts.next().unwrap(), doc.vertices.len())?;
                        let uv = parts
                            .next()
                            .filter(|s| !s.is_empty())
                            .map(|s| index(s, uv.len()).map(|i| uv[i]))
                            .transpose()?
                            .unwrap_or([0., 0.]);
                        corners.push((v, uv));
                        if corners.len() > 256 {
                            return Err("OBJ face exceeds 256 corners".into());
                        }
                    }
                    let points = corners
                        .iter()
                        .map(|(v, _)| doc.vertices[*v])
                        .collect::<Vec<_>>();
                    for t in triangles(&points)? {
                        if doc.faces.len() >= 3500 {
                            return Err("OBJ exceeds 3500 faces after triangulation".into());
                        }
                        let corners = [corners[t[0]], corners[t[1]], corners[t[2]], corners[t[2]]];
                        doc.faces.push(Face {
                            id: Uuid::new_v4(),
                            vertices: corners.map(|c| c.0 as u32),
                            uv: corners.map(|c| c.1),
                            group,
                            material,
                        });
                    }
                }
                "vn" | "s" | "mtllib" => {}
                "l" | "p" | "vp" | "curv" | "surf" => {
                    return Err(
                        "OBJ curves, lines and points are unsupported; export polygon faces".into(),
                    );
                }
                _ => return Err(format!("Unsupported OBJ statement {kind}")),
            }
            Ok(())
        })();
        result.map_err(|e| format!("OBJ line {}: {e}", line_no + 1))?;
    }
    if doc.faces.is_empty() {
        return Err("OBJ contains no polygon faces".into());
    }
    doc.groups = groups
        .into_iter()
        .map(|(name, id)| Group {
            id,
            name,
            parent: None,
        })
        .collect();
    doc.materials = slots
        .into_iter()
        .map(|(name, id)| Slot {
            id,
            material: materials.get(&name).cloned().unwrap_or_default(),
            name,
        })
        .collect();
    doc.compact();
    doc.validate()?;
    Ok(doc)
}
fn linked(directory: &Path, parent: &Path, value: &str) -> Result<std::path::PathBuf, String> {
    let path = parent
        .join(value)
        .canonicalize()
        .map_err(|e| format!("Missing OBJ dependency {value}: {e}"))?;
    if !path.starts_with(directory) {
        return Err("OBJ dependencies must stay inside the source folder or project assets".into());
    }
    Ok(path)
}
fn read_materials(
    root: &Path,
    path: &Path,
    source: &str,
    destination: &str,
) -> Result<BTreeMap<String, Material>, String> {
    let mut out = BTreeMap::<String, Material>::new();
    let index = assets::scan(root, &mut Default::default());
    let project = root.canonicalize().map_err(|e| e.to_string())?;
    let project_assets = project.join("assets");
    let external = !path.starts_with(&project_assets);
    let directory = if external {
        path.parent().ok_or("Missing OBJ source folder")?
    } else {
        &project_assets
    };
    for line in source.lines() {
        let line = line.split('#').next().unwrap().trim();
        let Some(value) = line.strip_prefix("mtllib ") else {
            continue;
        };
        let mtl = linked(directory, path.parent().unwrap(), value.trim())?;
        let data = std::fs::read_to_string(&mtl).map_err(|e| e.to_string())?;
        let mut current = None::<String>;
        for line in data.lines() {
            let mut words = line.split('#').next().unwrap().split_whitespace();
            let Some(kind) = words.next() else {
                continue;
            };
            if kind == "newmtl" {
                let name = words.collect::<Vec<_>>().join(" ");
                if name.is_empty() || out.len() >= 64 {
                    return Err("Invalid or excessive MTL material definitions".into());
                }
                out.insert(name.clone(), Material::default());
                current = Some(name);
                continue;
            }
            let Some(material) = current.as_ref().and_then(|n| out.get_mut(n)) else {
                continue;
            };
            match kind {
                "Kd" => {
                    material.color = [
                        number(words.next())?,
                        number(words.next())?,
                        number(words.next())?,
                    ]
                }
                "illum" => material.unlit = words.next() == Some("0"),
                "map_Kd" => {
                    let file = words.collect::<Vec<_>>().join(" ");
                    if file.starts_with('-') {
                        return Err(
                            "MTL map_Kd options are unsupported; bake transforms into UVs".into(),
                        );
                    }
                    let file = linked(directory, mtl.parent().unwrap(), &file)?;
                    let imported = if external {
                        project
                            .join(
                                Path::new(destination)
                                    .parent()
                                    .ok_or("Missing mesh destination folder")?,
                            )
                            .join(file.strip_prefix(directory).map_err(|e| e.to_string())?)
                    } else {
                        file
                    };
                    let relative = assets::path_string(&project, &imported);
                    let matches = index
                        .usable()
                        .filter(|r| {
                            r.meta.kind == assets::Kind::Texture
                                && r.meta.source.replace('\\', "/") == relative
                        })
                        .collect::<Vec<_>>();
                    if matches.len() != 1 {
                        return Err(format!(
                            "Import {relative} as one Texture asset before importing the OBJ"
                        ));
                    }
                    material.texture = Some(matches[0].meta.id);
                }
                "d" => {
                    if number(words.next())? < 1. {
                        material.blend = crate::texture::BlendMode::Average;
                    }
                }
                "Tr" if number(words.next())? > 0. => {
                    material.blend = crate::texture::BlendMode::Average;
                }
                _ => {}
            }
        }
    }
    Ok(out)
}
pub fn import(root: &Path, source: &str, destination: &str, scale: f32) -> Result<Uuid, String> {
    let path = assets::inside(root, source)?;
    import_file(root, &path, destination, scale)
}

/// Import from the Project browser; material libraries are read next to the
/// source while the new project-owned package contains the editable geometry.
pub fn import_file(
    root: &Path,
    source: &Path,
    destination: &str,
    scale: f32,
) -> Result<Uuid, String> {
    if !source
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("obj"))
        || !destination.ends_with(".epokasset")
    {
        return Err("Select an .obj source and a new .epokasset destination".into());
    }
    let path = source.canonicalize().map_err(|e| e.to_string())?;
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 8 * 1024 * 1024 {
        return Err("OBJ limit is 8 MiB".into());
    }
    let source = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let materials = read_materials(root, &path, &source, destination)?;
    let document = parse(&source, scale, &materials)?;
    crate::mesh::create(root, destination, &document)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_indices_uv_seams_materials_and_concave_faces() {
        let obj = "v 0 0 0\nv 2 0 0\nv 2 2 0\nv 1 1 0\nv 0 2 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0.5 0.5\nvt 0 1\ng Wall\nusemtl Red\nf -5/1 -4/2 -3/3 -2/4 -1/5\n";
        let materials = BTreeMap::from([(
            "Red".into(),
            Material {
                color: [1., 0., 0.],
                ..Default::default()
            },
        )]);
        let doc = parse(obj, 1., &materials).unwrap();
        assert_eq!(doc.faces.len(), 3);
        assert!(
            doc.materials
                .iter()
                .any(|s| s.name == "Red" && s.material.color == [1., 0., 0.])
        );
        let area = doc
            .faces
            .iter()
            .map(|f| {
                let p = doc.points(f);
                crate::mesh::cross(
                    crate::lighting::sub(p[1], p[0]),
                    crate::lighting::sub(p[2], p[0]),
                )[2] / 2.
            })
            .sum::<f32>();
        assert!((area - 3.).abs() < 0.001);
        assert!(doc.faces.iter().any(|f| f.uv.contains(&[0., 1.])));
    }
    #[test]
    fn invalid_indices_uvs_and_geometry_fail() {
        for data in [
            "v 0 0 0\nf 0 1 1",
            "v 0 0 0\nf 1 2 3",
            "vt -1 0",
            "v NaN 0 0",
            "v 0 0 0\nv 1 1 0\nv 2 2 0\nf 1 2 3",
        ] {
            assert!(parse(data, 1., &BTreeMap::new()).is_err(), "{data}");
        }
    }
    #[test]
    fn new_assets_are_transactional_and_runtime_uvs_survive() {
        let root = std::env::temp_dir().join(format!("epok-obj-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("assets/quad.obj"),"v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nf 1/1 2/2 3/3 4/4\n").unwrap();
        let id = import(&root, "assets/quad.obj", "assets/quad.epokasset", 1.).unwrap();
        assert!(import(&root, "assets/quad.obj", "assets/quad.epokasset", 1.).is_err());
        let index = assets::scan(&root, &mut Default::default());
        let doc = crate::mesh::document(index.resolve(id).unwrap()).unwrap();
        assert_eq!(doc.faces.len(), 2);
        let mut e = crate::scene::Actor::cube("Imported".into());
        e.editable_mesh = Some(crate::mesh::Component {
            asset: id,
            materials: Default::default(),
            document: Some(std::sync::Arc::new(doc)),
            error: None,
        });
        let generated = crate::mesh_compile::header(&e, 0).unwrap();
        assert!(generated.contains("4096"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
