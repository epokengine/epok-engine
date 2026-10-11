//! Automatic camera visibility bake. For every sample of a visibility track,
//! render the scene from the sample's camera (and from intermediate poses up to
//! the next sample) on the GPU and record which tracked faces show at least one
//! pixel. Occlusion only counts opaque faces: a face behind a cut-out or
//! blended face, or coplanar with an occluder, stays visible.
//!
//! The camera reproduces the PSX runtime's projection: horizontal field of view
//! across the display width, and the same vertical focal length for any
//! resolution (2/3 of the display height in pixels).
use crate::{lighting::Quad, scene::Scene, texture::BlendMode, visibility_track::Track};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug)]
pub struct Options {
    /// Render width and height, in PSX display pixels times `scale`.
    pub width: u32,
    pub height: u32,
    pub scale: u32,
    /// Extra poses rendered between a sample and the next continuous one.
    pub substeps: u32,
    pub near: f32,
    pub far: f32,
    /// A cut-out face stops occluding when more than this fraction of the
    /// texels it covers is transparent (fences, foliage). PSX textures often
    /// hold isolated transparent texels inside solid surfaces.
    pub see_through: f32,
    /// Adaptive refinement: between two consecutive poses, a face that shows
    /// in one but not the other with at least this many display pixels gets
    /// a pose half way between, up to `refine_depth` halvings.
    pub refine_pixels: f32,
    pub refine_depth: u32,
}
impl Default for Options {
    fn default() -> Self {
        // Near and far match the runtime's mesh camera units (0.25 and 128 world
        // units); the far limit is also the default geometry draw distance.
        Self { width: 512, height: 240, scale: 4, substeps: 4, near: 0.25, far: 128., see_through: 0.05,
            refine_pixels: 2., refine_depth: 6 }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    face: u32,
}
const NO_FACE: u32 = u32::MAX;

/// A camera pose: position, Euler rotation in degrees and horizontal FOV.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub fov: f32,
}
impl Pose {
    fn lerp(self, next: Self, t: f32) -> Self {
        // Shortest-path Euler interpolation, as the runtime plays a track.
        let angle = |a: f32, b: f32| a + ((b - a + 180.).rem_euclid(360.) - 180.) * t;
        Self {
            position: std::array::from_fn(|c| self.position[c] + (next.position[c] - self.position[c]) * t),
            rotation: std::array::from_fn(|c| angle(self.rotation[c], next.rotation[c])),
            fov: self.fov + (next.fov - self.fov) * t,
        }
    }
    /// Camera-from-world rows and projection scales, as the uniform takes them.
    fn uniform(self, options: &Options) -> Result<[f32; 16], String> {
        let view = crate::transform::Matrix::trs(self.position, self.rotation, [1.; 3]).inverse()?;
        let fov = self.fov.clamp(25., 120.).to_radians();
        let horizontal = 1. / (fov / 2.).tan();
        let mut out = [0.; 16];
        for r in 0..3 {
            out[r * 4..r * 4 + 4].copy_from_slice(&view.0[r]);
        }
        out[12] = horizontal;
        out[13] = horizontal * 4. / 3.;
        out[14] = options.near;
        out[15] = options.far;
        Ok(out)
    }
}

/// Poses of a track's samples, and for each sample the following sample it
/// moves on to continuously (none across a cut, or past a route's end).
pub fn poses(track: &Track) -> (Vec<Pose>, Vec<Option<usize>>) {
    let poses = track.samples.iter()
        .map(|s| Pose { position: s.position, rotation: s.rotation, fov: s.fov }).collect::<Vec<_>>();
    let route_end = |i: usize| track.navigation.as_ref().is_some_and(|n| n.routes.iter()
        .any(|r| usize::from(r.first) + usize::from(r.count) == i + 1));
    let next = (0..poses.len()).map(|i| {
        (i + 1 < poses.len() && !track.samples[i + 1].cut && !route_end(i)).then_some(i + 1)
    }).collect();
    (poses, next)
}

/// Fraction of the texels a face's UV triangles cover that are transparent
/// (alpha 0). Only those texels matter: textures keep transparent texels
/// elsewhere. An unresolved texture counts as fully see-through.
fn transparent_fraction(scene: &Scene, texture: uuid::Uuid, uv: [[f32; 2]; 4], triangles: &[[usize; 3]]) -> f32 {
    let Some(t) = scene.textures.get(&texture) else { return 1. };
    let (w, h) = (i32::from(t.width), i32::from(t.height));
    if w == 0 || h == 0 { return 1.; }
    let (mut covered, mut clear) = (0u32, 0u32);
    let texel = |c: [f32; 2]| [c[0].clamp(0., 1.) * (w - 1) as f32, c[1].clamp(0., 1.) * (h - 1) as f32];
    let alpha = |x: i32, y: i32| t.rgba.get(((y * w + x) * 4 + 3) as usize).copied().unwrap_or(0);
    for tri in triangles {
        let p = tri.map(|i| texel(uv[i]));
        let (x0, x1) = (p.iter().map(|q| q[0]).fold(f32::MAX, f32::min).floor() as i32, p.iter().map(|q| q[0]).fold(f32::MIN, f32::max).ceil() as i32);
        let (y0, y1) = (p.iter().map(|q| q[1]).fold(f32::MAX, f32::min).floor() as i32, p.iter().map(|q| q[1]).fold(f32::MIN, f32::max).ceil() as i32);
        let edge = |a: [f32; 2], b: [f32; 2], c: [f32; 2]| (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        let area = edge(p[0], p[1], p[2]);
        for y in y0.max(0)..=y1.min(h - 1) {
            for x in x0.max(0)..=x1.min(w - 1) {
                // Texels whose centre is inside, or within half a texel of an edge.
                let c = [x as f32, y as f32];
                let inside = if area.abs() < 1e-6 { true } else {
                    let s = area.signum();
                    let tol = 0.75 * area.abs().sqrt().max(1.);
                    (0..3).all(|i| s * edge(p[i], p[(i + 1) % 3], c) >= -tol)
                };
                if inside { covered += 1; clear += u32::from(alpha(x, y) == 0); }
            }
        }
    }
    if covered == 0 { 0. } else { clear as f32 / covered as f32 }
}

/// Triangles of every active mesh actor in world space: tracked faces carry
/// their index in `track.faces`; others only occlude. Faces that can be seen
/// through are drawn but never occlude.
fn geometry(scene: &Scene, track: &Track, see_through: f32) -> (Vec<Vertex>, Vec<Vertex>) {
    let index: HashMap<uuid::Uuid, u32> = track.faces.iter().enumerate().map(|(i, f)| (*f, i as u32)).collect();
    let (mut occluders, mut all) = (Vec::new(), Vec::new());
    for (actor, e) in scene.actors.iter().enumerate() {
        if e.kind != "Mesh" || !scene.is_active(actor) {
            continue;
        }
        let transform = scene.world_matrix(actor);
        for q in crate::lighting::quads(e) {
            let Quad { points, material, id, uv, .. } = &q;
            let face = id.and_then(|id| index.get(&id).copied()).unwrap_or(NO_FACE);
            let corners = points.map(|p| transform.point(p));
            let triangles: &[[usize; 3]] = if points[2] == points[3] { &[[0, 1, 2]] } else { &[[0, 1, 2], [0, 2, 3]] };
            let opaque = material.blend == BlendMode::Cutout
                && material.texture.is_none_or(|t| transparent_fraction(scene, t, *uv, triangles) <= see_through);
            for t in triangles {
                for &c in t {
                    let v = Vertex { position: corners[c], face };
                    all.push(v);
                    if opaque {
                        occluders.push(v);
                    }
                }
            }
        }
    }
    (occluders, all)
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    depth_pipeline: wgpu::RenderPipeline,
    mark_pipeline: wgpu::RenderPipeline,
    depth_bind: wgpu::BindGroup,
    mark_bind: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    visible: wgpu::Buffer,
    readback: wgpu::Buffer,
    depth_view: wgpu::TextureView,
    color_view: wgpu::TextureView,
    occluders: (wgpu::Buffer, u32),
    all: (wgpu::Buffer, u32),
}

fn bytes<T: Copy>(values: &[T]) -> &[u8] {
    // Vertex and f32 are plain old data without padding.
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values)) }
}

impl Gpu {
    fn new(options: &Options, faces: usize, occluders: &[Vertex], all: &[Vertex]) -> Result<Self, String> {
        use wgpu::util::DeviceExt;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: if cfg!(windows) { wgpu::Backends::DX12 } else { wgpu::Backends::PRIMARY },
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })).map_err(|e| format!("No GPU adapter for the visibility bake: {e}"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))
            .map_err(|e| format!("No GPU device for the visibility bake: {e}"))?;
        let size = wgpu::Extent3d {
            width: options.width * options.scale,
            height: options.height * options.scale,
            depth_or_array_layers: 1,
        };
        let target = |format, usage| device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Visibility bake"), size, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format, usage, view_formats: &[],
        }).create_view(&Default::default());
        let depth_view = target(wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING);
        let color_view = target(wgpu::TextureFormat::R8Unorm, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Visibility bake"),
            source: wgpu::ShaderSource::Wgsl(include_str!("visibility_bake.wgsl").into()),
        });
        let uniform_entry = wgpu::BindGroupLayoutEntry {
            binding: 0, visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Visibility bake depth"), entries: &[uniform_entry],
        });
        let mark_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Visibility bake mark"),
            entries: &[
                uniform_entry,
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32],
        };
        let mark_targets = [Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::R8Unorm, blend: None, write_mask: wgpu::ColorWrites::empty(),
        })];
        let pipeline = |layout: &wgpu::BindGroupLayout, depth: bool| {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Visibility bake"), bind_group_layouts: &[layout], push_constant_ranges: &[],
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(if depth { "Visibility bake occluders" } else { "Visibility bake faces" }),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader, entry_point: Some("vertex"), compilation_options: Default::default(),
                    buffers: std::slice::from_ref(&vertex_layout),
                },
                // Tracks draw both sides: never cull.
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: depth.then(|| wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, stencil: Default::default(), bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: (!depth).then(|| wgpu::FragmentState {
                    module: &shader, entry_point: Some("mark"), compilation_options: Default::default(),
                    targets: &mark_targets,
                }),
                multiview: None,
                cache: None,
            })
        };
        let depth_pipeline = pipeline(&depth_layout, true);
        let mark_pipeline = pipeline(&mark_layout, false);
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visibility bake view"), size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
        });
        let flags = (faces.max(1) * 4) as u64;
        let visible = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visibility bake faces"), size: flags,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visibility bake readback"), size: flags,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
        });
        let depth_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Visibility bake depth"), layout: &depth_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }],
        });
        let mark_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Visibility bake mark"), layout: &mark_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&depth_view) },
                wgpu::BindGroupEntry { binding: 2, resource: visible.as_entire_binding() },
            ],
        });
        let buffer = |vertices: &[Vertex]| (
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Visibility bake geometry"),
                contents: if vertices.is_empty() { &[0; 16] } else { bytes(vertices) },
                usage: wgpu::BufferUsages::VERTEX,
            }),
            vertices.len() as u32,
        );
        Ok(Self {
            occluders: buffer(occluders), all: buffer(all), device, queue, depth_pipeline, mark_pipeline,
            depth_bind, mark_bind, uniform, visible, readback, depth_view, color_view,
        })
    }
    /// Marks the faces visible from `pose` in the GPU flags (not cleared).
    fn render(&self, pose: Pose, options: &Options, clear: bool) -> Result<(), String> {
        self.queue.write_buffer(&self.uniform, 0, bytes(&pose.uniform(options)?));
        let mut encoder = self.device.create_command_encoder(&Default::default());
        if clear {
            encoder.clear_buffer(&self.visible, 0, None);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Visibility bake occluders"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&self.depth_pipeline);
            pass.set_bind_group(0, &self.depth_bind, &[]);
            pass.set_vertex_buffer(0, self.occluders.0.slice(..));
            pass.draw(0..self.occluders.1, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Visibility bake faces"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.color_view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Discard },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.mark_pipeline);
            pass.set_bind_group(0, &self.mark_bind, &[]);
            pass.set_vertex_buffer(0, self.all.0.slice(..));
            pass.draw(0..self.all.1, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        Ok(())
    }
    fn read(&self, faces: usize) -> Result<Vec<u16>, String> {
        Ok(self.read_pixels(faces)?.into_iter().map(|(f, _)| f).collect())
    }
    /// Visible faces with the pixels they cover at the bake's resolution.
    fn read_pixels(&self, faces: usize) -> Result<Vec<(u16, u32)>, String> {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.visible, 0, &self.readback, 0, self.readback.size());
        self.queue.submit(Some(encoder.finish()));
        let slice = self.readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
        self.device.poll(wgpu::PollType::Wait).map_err(|e| e.to_string())?;
        rx.recv().map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
        let data = slice.get_mapped_range();
        let out = data.chunks_exact(4).take(faces).enumerate()
            .map(|(i, w)| (i as u16, u32::from_le_bytes([w[0], w[1], w[2], w[3]])))
            .filter(|(_, n)| *n != 0).collect();
        drop(data);
        self.readback.unmap();
        Ok(out)
    }
}

/// Visible face indices (into `track.faces`) for every sample: the faces seen
/// from the sample's pose or any of `substeps` poses toward the next sample.
pub fn bake(scene: &Scene, track: &Track, options: &Options) -> Result<Vec<Vec<u16>>, String> {
    let (occluders, all) = geometry(scene, track, options.see_through);
    let gpu = Gpu::new(options, track.faces.len(), &occluders, &all)?;
    let (poses, next) = poses(track);
    let faces = track.faces.len();
    let threshold = (options.refine_pixels * (options.scale * options.scale) as f32) as u32;
    let at = |pose: Pose| -> Result<HashMap<u16, u32>, String> {
        gpu.render(pose, options, true)?;
        Ok(gpu.read_pixels(faces)?.into_iter().collect())
    };
    // Whether a face that matters appears or disappears between two poses.
    let differs = |a: &HashMap<u16, u32>, b: &HashMap<u16, u32>| {
        a.iter().any(|(f, n)| *n >= threshold && !b.contains_key(f))
            || b.iter().any(|(f, n)| *n >= threshold && !a.contains_key(f))
    };
    let mut out = Vec::with_capacity(poses.len());
    for (i, pose) in poses.iter().enumerate() {
        let mut union: std::collections::BTreeSet<u16> = std::collections::BTreeSet::new();
        let first = at(*pose)?;
        union.extend(first.keys());
        if let Some(n) = next[i] {
            // Evenly spaced poses up to and including the next sample, then
            // halve any interval across which a face of note appears.
            let steps = options.substeps + 1;
            let mut previous = (0f32, first);
            for s in 1..=steps {
                let t = s as f32 / steps as f32;
                let current = at(pose.lerp(poses[n], t))?;
                let mut stack = vec![(previous.0, t, 0u32)];
                let mut seen = vec![(previous.0, previous.1.clone()), (t, current.clone())];
                while let Some((a, b, depth)) = stack.pop() {
                    if depth >= options.refine_depth { continue; }
                    let find = |t: f32, seen: &Vec<(f32, HashMap<u16, u32>)>| seen.iter().position(|(x, _)| *x == t).unwrap();
                    let (sa, sb) = (find(a, &seen), find(b, &seen));
                    if !differs(&seen[sa].1, &seen[sb].1) { continue; }
                    let m = (a + b) / 2.;
                    let middle = at(pose.lerp(poses[n], m))?;
                    union.extend(middle.keys());
                    seen.push((m, middle));
                    stack.push((a, m, depth + 1));
                    stack.push((m, b, depth + 1));
                }
                union.extend(current.keys());
                previous = (t, current);
            }
        }
        out.push(union.into_iter().collect());
    }
    Ok(out)
}

/// Per-sample comparison of a bake with the track's existing lists.
pub fn compare(track: &Track, baked: &[Vec<u16>]) -> serde_json::Value {
    let mut rows = Vec::new();
    let (mut existing, mut automatic, mut missing, mut extra) = (0usize, 0usize, 0usize, 0usize);
    let mut missing_by_face: BTreeMap<u16, usize> = BTreeMap::new();
    for (sample, (s, b)) in track.samples.iter().zip(baked).enumerate() {
        let reference: std::collections::BTreeSet<u16> = s.visible.iter().copied().collect();
        let bake: std::collections::BTreeSet<u16> = b.iter().copied().collect();
        let lost = reference.difference(&bake).count();
        let added = bake.difference(&reference).count();
        for f in reference.difference(&bake) { *missing_by_face.entry(*f).or_default() += 1; }
        existing += reference.len(); automatic += bake.len(); missing += lost; extra += added;
        rows.push(serde_json::json!({"sample": sample, "existing": reference.len(), "baked": bake.len(),
            "only_existing": lost, "only_baked": added}));
    }
    serde_json::json!({
        "samples": baked.len(), "faces": track.faces.len(),
        "existing_total": existing, "baked_total": automatic,
        "only_existing_total": missing, "only_baked_total": extra,
        "faces_only_in_existing_lists": missing_by_face.len(),
        "per_sample": rows,
    })
}

/// Debug aid: for one sample, where the track's listed faces fall in this
/// bake's camera (in front, inside the frustum).
pub fn diagnose(scene: &Scene, track: &Track, sample: usize, options: &Options) -> Result<String, String> {
    let s = &track.samples[sample];
    let pose = Pose { position: s.position, rotation: s.rotation, fov: s.fov };
    let u = pose.uniform(options)?;
    let listed: std::collections::HashSet<uuid::Uuid> = s.visible.iter().map(|&i| track.faces[usize::from(i)]).collect();
    let (mut front, mut inside, mut behind, mut total) = (0, 0, 0, 0);
    let mut zs = Vec::new();
    for (actor, e) in scene.actors.iter().enumerate() {
        if e.kind != "Mesh" { continue; }
        let transform = scene.world_matrix(actor);
        for q in crate::lighting::quads(e) {
            if !q.id.is_some_and(|id| listed.contains(&id)) { continue; }
            total += 1;
            let c = q.points.iter().fold([0f32; 3], |a, p| { let w = transform.point(*p); [a[0] + w[0] / 4., a[1] + w[1] / 4., a[2] + w[2] / 4.] });
            let cam: [f32; 3] = std::array::from_fn(|r| u[r * 4] * c[0] + u[r * 4 + 1] * c[1] + u[r * 4 + 2] * c[2] + u[r * 4 + 3]);
            zs.push(cam);
            if cam[2] > 0. { front += 1; if (u[12] * cam[0] / cam[2]).abs() <= 1. && (u[13] * cam[1] / cam[2]).abs() <= 1. { inside += 1; } } else { behind += 1; }
        }
    }
    let (occluders, all) = geometry(scene, track, options.see_through);
    Ok(format!("occluder triangles {} of {}; sample {sample}: listed faces {total}, centroid in front {front}, inside frustum {inside}, behind {behind}; pose {:?}; first camera points {:?}", occluders.len() / 3, all.len() / 3, pose, &zs[..zs.len().min(4)]))
}

/// Debug aid: classify where faces only one of the two lists holds sit, for
/// one sample (distance from the camera, screen position).
pub fn diagnose_differences(scene: &Scene, track: &Track, sample: usize, options: &Options) -> Result<String, String> {
    let mut single = options.clone();single.substeps = 0;
    let s = &track.samples[sample];
    let pose = Pose { position: s.position, rotation: s.rotation, fov: s.fov };
    let one = Track { samples: vec![s.clone()], navigation: None, ..track.clone() };
    let baked: std::collections::HashSet<u16> = bake(scene, &one, &single)?.remove(0).into_iter().collect();
    let listed: std::collections::HashSet<u16> = s.visible.iter().copied().collect();
    let u = pose.uniform(options)?;
    let index: HashMap<uuid::Uuid, u16> = track.faces.iter().enumerate().map(|(i, f)| (*f, i as u16)).collect();
    let mut out = String::new();
    for (name, set, other) in [("only baked", &baked, &listed), ("only listed", &listed, &baked)] {
        let mut dist = Vec::new();let (mut edge, mut offscreen, mut n) = (0, 0, 0);
        for (actor, e) in scene.actors.iter().enumerate() {
            if e.kind != "Mesh" { continue; }
            let transform = scene.world_matrix(actor);
            for q in crate::lighting::quads(e) {
                let Some(i) = q.id.and_then(|id| index.get(&id)) else { continue };
                if !set.contains(i) || other.contains(i) { continue; }
                n += 1;
                let cams: Vec<[f32; 3]> = q.points.iter().map(|p| { let w = transform.point(*p);
                    std::array::from_fn(|r| u[r * 4] * w[0] + u[r * 4 + 1] * w[1] + u[r * 4 + 2] * w[2] + u[r * 4 + 3]) }).collect();
                let near = cams.iter().map(|c| c[2]).fold(f32::MAX, f32::min);
                dist.push(near);
                let ndc: Vec<[f32; 2]> = cams.iter().filter(|c| c[2] > 0.).map(|c| [u[12] * c[0] / c[2], u[13] * c[1] / c[2]]).collect();
                let all_out = !ndc.is_empty() && (ndc.iter().all(|v| v[0] < -1.) || ndc.iter().all(|v| v[0] > 1.) || ndc.iter().all(|v| v[1] < -1.) || ndc.iter().all(|v| v[1] > 1.));
                if all_out { offscreen += 1; } else if ndc.iter().any(|v| v[0].abs() > 0.9 || v[1].abs() > 0.9) { edge += 1; }
            }
        }
        dist.sort_by(f32::total_cmp);
        let q = |f: f32| dist.get(((dist.len() as f32 - 1.) * f) as usize).copied().unwrap_or(0.);
        out += &format!("{name}: {n} faces, nearest-corner depth p10 {:.1} p50 {:.1} p90 {:.1} max {:.1}; off screen {offscreen}, touching edge {edge}\n", q(0.1), q(0.5), q(0.9), q(1.));
    }
    Ok(out)
}

/// Hole check between samples: at poses part way to the next continuous sample
/// (which a sample's list must cover while the camera moves), the faces truly
/// visible there that each set of lists lacks.
pub fn holes(scene: &Scene, track: &Track, baked: &[Vec<u16>], options: &Options, fractions: &[f32]) -> Result<serde_json::Value, String> {
    let (occluders, all) = geometry(scene, track, options.see_through);
    let gpu = Gpu::new(options, track.faces.len(), &occluders, &all)?;
    let (poses, next) = poses(track);
    let (mut truth_total, mut missing_baked, mut missing_existing, mut checks) = (0usize, 0usize, 0usize, 0usize);
    let (mut worst_baked, mut worst_existing) = ((0usize, 0usize), (0usize, 0usize));
    let (mut missing_pixels, mut worst_pixels) = (0f64, 0f64);
    let mut worst_case = (0usize, 0f32, None::<u16>);
    for (i, pose) in poses.iter().enumerate() {
        let Some(n) = next[i] else { continue };
        let b: std::collections::HashSet<u16> = baked[i].iter().copied().collect();
        let e: std::collections::HashSet<u16> = track.samples[i].visible.iter().copied().collect();
        for &t in fractions {
            gpu.render(pose.lerp(poses[n], t), options, true)?;
            let truth = gpu.read_pixels(track.faces.len())?;
            let mb = truth.iter().filter(|(f, _)| !b.contains(f)).count();
            let me = truth.iter().filter(|(f, _)| !e.contains(f)).count();
            // Coverage in display pixels of the faces the bake lacks.
            let area = (options.scale * options.scale) as f64;
            let pixels = truth.iter().filter(|(f, _)| !b.contains(f)).map(|(_, n)| f64::from(*n) / area).collect::<Vec<_>>();
            missing_pixels += pixels.iter().sum::<f64>();
            let largest = pixels.iter().copied().fold(0., f64::max);
            if largest > worst_pixels { worst_pixels = largest; worst_case = (i, t, truth.iter().filter(|(f, _)| !b.contains(f)).max_by_key(|(_, n)| *n).map(|(f, _)| *f)); }
            truth_total += truth.len(); missing_baked += mb; missing_existing += me; checks += 1;
            if mb > worst_baked.1 { worst_baked = (i, mb); }
            if me > worst_existing.1 { worst_existing = (i, me); }
        }
    }
    // The same check for the existing lists, assuming each covers the stretch
    // leading up to its sample instead (from the previous continuous sample).
    let (mut before_total, mut before_missing) = (0usize, 0usize);
    for (p, pose) in poses.iter().enumerate() {
        let Some(i) = next[p] else { continue };
        let e: std::collections::HashSet<u16> = track.samples[i].visible.iter().copied().collect();
        for &t in fractions {
            gpu.render(pose.lerp(poses[i], t), options, true)?;
            let truth = gpu.read(track.faces.len())?;
            before_total += truth.len();
            before_missing += truth.iter().filter(|f| !e.contains(f)).count();
        }
    }
    Ok(serde_json::json!({
        "existing_as_leading_up": {"visible_faces_checked": before_total, "missing": before_missing},
        "checks": checks, "visible_faces_checked": truth_total,
        "missing_from_baked_display_pixels_total": missing_pixels,
        "missing_from_baked_display_pixels_per_check": missing_pixels / checks.max(1) as f64,
        "largest_missing_face_display_pixels": worst_pixels,
        "largest_missing_case": {"sample": worst_case.0, "fraction": worst_case.1, "face": worst_case.2},
        "missing_from_baked": missing_baked, "missing_from_existing": missing_existing,
        "worst_baked_sample": worst_baked, "worst_existing_sample": worst_existing,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cube_actor(name: &str, centre: [f32; 3], size: f32, blend: BlendMode) -> crate::scene::Actor {
        let mut doc = crate::mesh::Document::default();
        doc.materials[0].material.unlit = true;
        doc.materials[0].material.blend = blend;
        doc.primitive("Cube", [0.; 3], [size; 3], 1, doc.groups[0].id, doc.materials[0].id);
        let mut e = crate::scene::Actor::cube(name.into());
        let mut component = crate::mesh::Component::new(uuid::Uuid::new_v4());
        component.document = Some(std::sync::Arc::new(doc));
        e.editable_mesh = Some(component);
        e.position = centre;
        e
    }
    fn scene_with(front_blend: BlendMode) -> (Scene, Track, usize) {
        // A large cube right in front of the camera hides a small one behind it.
        let front = cube_actor("Front", [0., 0., 6.], 6., front_blend);
        let back = cube_actor("Back", [0., 0., 14.], 1., BlendMode::Cutout);
        let ids = |e: &crate::scene::Actor| e.editable_mesh.as_ref().unwrap().document.as_ref().unwrap()
            .faces.iter().map(|f| f.id).collect::<Vec<_>>();
        let front_faces = ids(&front).len();
        let faces: Vec<_> = ids(&front).into_iter().chain(ids(&back)).collect();
        let camera = Scene::default().actors[0].clone();
        let track = Track {
            camera: camera.id, sample_ms: 100, cull_backfaces: false, static_world: false, navigation: None,
            samples: vec![crate::visibility_track::Sample {
                position: [0.; 3], rotation: [0.; 3], fov: 60., visible: vec![], cut: true,
            }],
            faces,
        };
        (Scene { actors: vec![front, back, camera], ..Default::default() }, track, front_faces)
    }
    #[test]
    fn opaque_faces_hide_what_is_behind_and_blended_ones_do_not() {
        let options = Options { scale: 1, substeps: 0, ..Default::default() };
        let (scene, track, front_faces) = scene_with(BlendMode::Cutout);
        let visible = bake(&scene, &track, &options).unwrap().remove(0);
        // Only the front cube's face towards the camera shows.
        assert_eq!(visible.len(), 1, "{visible:?}");
        assert!(usize::from(visible[0]) < front_faces);
        let (scene, track, front_faces) = scene_with(BlendMode::Average);
        let visible = bake(&scene, &track, &options).unwrap().remove(0);
        assert!(visible.iter().any(|&f| usize::from(f) >= front_faces), "{visible:?}");
        // Blended faces never occlude, so the far side of the front cube shows too.
        assert!(visible.iter().filter(|&&f| usize::from(f) < front_faces).count() > 1);
    }
    #[test]
    fn poses_stop_at_cuts_and_interpolate_angles_the_short_way() {
        let a = Pose { position: [0.; 3], rotation: [350., 0., 0.], fov: 60. };
        let b = Pose { position: [2., 0., 0.], rotation: [10., 0., 0.], fov: 80. };
        let m = a.lerp(b, 0.5);
        assert_eq!(m.position, [1., 0., 0.]);
        assert!((m.rotation[0] - 360.).abs() < 1e-3 && (m.fov - 70.).abs() < 1e-6);
    }
}
