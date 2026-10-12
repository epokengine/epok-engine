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
    /// Depth step of the generated draw order, in world units. Packets whose
    /// depths fall in the same step keep the previous sample's order, so
    /// consecutive lists stay similar and their deltas small.
    pub order_step: f32,
    /// Spatial cell (world units) used to keep a generated packet order's
    /// projection blocks compact; see `ordered_program`.
    pub order_cell: f32,
    /// A face hidden for at most this many consecutive samples of a continuous
    /// stretch, and visible on both sides, stays listed through the gap: fewer
    /// list changes (smaller resident deltas) for a few extra faces.
    pub fill_gaps: u32,
    /// Widen what each recorded camera sees by this fraction in both
    /// directions (0.1 = 10%), covering poses between recorded sessions.
    pub view_margin: f32,
    /// The same for the track's own path poses. Recorded cameras already
    /// follow the real camera, so by default only they are widened.
    pub path_margin: f32,
    /// World units per runtime camera-space unit along the view (the track
    /// camera's scale): `near` and `far` are runtime camera units. Set by
    /// `bake` from the scene.
    pub depth_scale: f32,
    /// List a face only if, in at least one of a sample's poses, it covers at
    /// least this many display pixels. 0 lists every face that shows at all;
    /// 1 drops faces that never cover a whole pixel (dust that costs a packet).
    pub min_pixels: f32,
    /// Samples finished so far, for a progress display.
    pub progress: Option<std::sync::Arc<std::sync::atomic::AtomicUsize>>,
}
impl Default for Options {
    fn default() -> Self {
        // Near and far match the runtime's mesh camera units (0.25 and 128 world
        // units); the far limit is also the default geometry draw distance.
        Self {
            width: 512,
            height: 240,
            scale: 4,
            substeps: 4,
            near: 0.25,
            far: 128.,
            see_through: 0.05,
            refine_pixels: 2.,
            refine_depth: 6,
            order_step: 0.5,
            order_cell: 16.,
            fill_gaps: 2,
            view_margin: 0.,
            path_margin: 0.,
            depth_scale: 1.,
            min_pixels: 1.,
            progress: None,
        }
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
            position: std::array::from_fn(|c| {
                self.position[c] + (next.position[c] - self.position[c]) * t
            }),
            rotation: std::array::from_fn(|c| angle(self.rotation[c], next.rotation[c])),
            fov: self.fov + (next.fov - self.fov) * t,
        }
    }
    /// Camera-from-world rows and projection scales, as the uniform takes them.
    fn uniform(self, options: &Options) -> Result<[f32; 16], String> {
        let view =
            crate::transform::Matrix::trs(self.position, self.rotation, [1.; 3]).inverse()?;
        let fov = self.fov.clamp(25., 120.).to_radians();
        let horizontal = 1. / (fov / 2.).tan();
        let mut out = [0.; 16];
        for r in 0..3 {
            out[r * 4..r * 4 + 4].copy_from_slice(&view.0[r]);
        }
        let widen = 1. + options.path_margin.max(0.);
        out[12] = horizontal / widen;
        out[13] = horizontal * 4. / 3. / widen;
        out[14] = options.near * options.depth_scale;
        out[15] = options.far * options.depth_scale;
        Ok(out)
    }
}

/// Poses of a track's samples, and for each sample the following sample it
/// moves on to continuously (none across a cut, or past a route's end).
pub fn poses(track: &Track) -> (Vec<Pose>, Vec<Option<usize>>) {
    let poses = track
        .samples
        .iter()
        .map(|s| Pose {
            position: s.position,
            rotation: s.rotation,
            fov: s.fov,
        })
        .collect::<Vec<_>>();
    let route_end = |i: usize| {
        track.navigation.as_ref().is_some_and(|n| {
            n.routes
                .iter()
                .any(|r| usize::from(r.first) + usize::from(r.count) == i + 1)
        })
    };
    let next = (0..poses.len())
        .map(|i| {
            (i + 1 < poses.len() && !track.samples[i + 1].cut && !route_end(i)).then_some(i + 1)
        })
        .collect();
    (poses, next)
}

/// Fraction of the texels a face's UV triangles cover that are transparent
/// (alpha 0). Only those texels matter: textures keep transparent texels
/// elsewhere. An unresolved texture counts as fully see-through.
fn transparent_fraction(
    scene: &Scene,
    texture: uuid::Uuid,
    uv: [[f32; 2]; 4],
    triangles: &[[usize; 3]],
) -> f32 {
    let Some(t) = scene.textures.get(&texture) else {
        return 1.;
    };
    let (w, h) = (i32::from(t.width), i32::from(t.height));
    if w == 0 || h == 0 {
        return 1.;
    }
    let (mut covered, mut clear) = (0u32, 0u32);
    let texel = |c: [f32; 2]| {
        [
            c[0].clamp(0., 1.) * (w - 1) as f32,
            c[1].clamp(0., 1.) * (h - 1) as f32,
        ]
    };
    let alpha = |x: i32, y: i32| {
        t.rgba
            .get(((y * w + x) * 4 + 3) as usize)
            .copied()
            .unwrap_or(0)
    };
    for tri in triangles {
        let p = tri.map(|i| texel(uv[i]));
        let (x0, x1) = (
            p.iter().map(|q| q[0]).fold(f32::MAX, f32::min).floor() as i32,
            p.iter().map(|q| q[0]).fold(f32::MIN, f32::max).ceil() as i32,
        );
        let (y0, y1) = (
            p.iter().map(|q| q[1]).fold(f32::MAX, f32::min).floor() as i32,
            p.iter().map(|q| q[1]).fold(f32::MIN, f32::max).ceil() as i32,
        );
        let edge = |a: [f32; 2], b: [f32; 2], c: [f32; 2]| {
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
        };
        let area = edge(p[0], p[1], p[2]);
        for y in y0.max(0)..=y1.min(h - 1) {
            for x in x0.max(0)..=x1.min(w - 1) {
                // Texels whose centre is inside, or within half a texel of an edge.
                let c = [x as f32, y as f32];
                let inside = if area.abs() < 1e-6 {
                    true
                } else {
                    let s = area.signum();
                    let tol = 0.75 * area.abs().sqrt().max(1.);
                    (0..3).all(|i| s * edge(p[i], p[(i + 1) % 3], c) >= -tol)
                };
                if inside {
                    covered += 1;
                    clear += u32::from(alpha(x, y) == 0);
                }
            }
        }
    }
    if covered == 0 {
        0.
    } else {
        clear as f32 / covered as f32
    }
}

/// Triangles of every active mesh actor in world space: tracked faces carry
/// their index in `track.faces`; others only occlude. Faces that can be seen
/// through are drawn but never occlude.
fn geometry(scene: &Scene, track: &Track, see_through: f32) -> (Vec<Vertex>, Vec<Vertex>) {
    let index: HashMap<uuid::Uuid, u32> = track
        .faces
        .iter()
        .enumerate()
        .map(|(i, f)| (*f, i as u32))
        .collect();
    let (mut occluders, mut all) = (Vec::new(), Vec::new());
    for (actor, e) in scene.actors.iter().enumerate() {
        if e.kind != "Mesh" || !scene.is_active(actor) {
            continue;
        }
        let transform = scene.world_matrix(actor);
        for q in crate::lighting::quads(e) {
            let Quad {
                points,
                material,
                id,
                uv,
                ..
            } = &q;
            let face = id.and_then(|id| index.get(&id).copied()).unwrap_or(NO_FACE);
            let corners = points.map(|p| transform.point(p));
            let triangles: &[[usize; 3]] = if points[2] == points[3] {
                &[[0, 1, 2]]
            } else {
                &[[0, 1, 2], [0, 2, 3]]
            };
            let opaque = material.blend == BlendMode::Cutout
                && material
                    .texture
                    .is_none_or(|t| transparent_fraction(scene, t, *uv, triangles) <= see_through);
            for t in triangles {
                for &c in t {
                    let v = Vertex {
                        position: corners[c],
                        face,
                    };
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
    fn new(
        options: &Options,
        faces: usize,
        occluders: &[Vertex],
        all: &[Vertex],
    ) -> Result<Self, String> {
        use wgpu::util::DeviceExt;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: if cfg!(windows) {
                wgpu::Backends::DX12
            } else {
                wgpu::Backends::PRIMARY
            },
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .map_err(|e| format!("No GPU adapter for the visibility bake: {e}"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))
            .map_err(|e| format!("No GPU device for the visibility bake: {e}"))?;
        let size = wgpu::Extent3d {
            width: options.width * options.scale,
            height: options.height * options.scale,
            depth_or_array_layers: 1,
        };
        let target = |format, usage| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Visibility bake"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let depth_view = target(
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let color_view = target(
            wgpu::TextureFormat::R8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Visibility bake"),
            source: wgpu::ShaderSource::Wgsl(include_str!("visibility_bake.wgsl").into()),
        });
        let uniform_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Visibility bake depth"),
            entries: &[uniform_entry],
        });
        let mark_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Visibility bake mark"),
            entries: &[
                uniform_entry,
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
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
            format: wgpu::TextureFormat::R8Unorm,
            blend: None,
            write_mask: wgpu::ColorWrites::empty(),
        })];
        let pipeline = |layout: &wgpu::BindGroupLayout, depth: bool| {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Visibility bake"),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(if depth {
                    "Visibility bake occluders"
                } else {
                    "Visibility bake faces"
                }),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: std::slice::from_ref(&vertex_layout),
                },
                // Tracks draw both sides: never cull.
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: depth.then(|| wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: (!depth).then(|| wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("mark"),
                    compilation_options: Default::default(),
                    targets: &mark_targets,
                }),
                multiview: None,
                cache: None,
            })
        };
        let depth_pipeline = pipeline(&depth_layout, true);
        let mark_pipeline = pipeline(&mark_layout, false);
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visibility bake view"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let flags = (faces.max(1) * 4) as u64;
        let visible = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visibility bake faces"),
            size: flags,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visibility bake readback"),
            size: flags,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let depth_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Visibility bake depth"),
            layout: &depth_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let mark_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Visibility bake mark"),
            layout: &mark_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: visible.as_entire_binding(),
                },
            ],
        });
        let buffer = |vertices: &[Vertex]| {
            (
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Visibility bake geometry"),
                    contents: if vertices.is_empty() {
                        &[0; 16]
                    } else {
                        bytes(vertices)
                    },
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                vertices.len() as u32,
            )
        };
        Ok(Self {
            occluders: buffer(occluders),
            all: buffer(all),
            device,
            queue,
            depth_pipeline,
            mark_pipeline,
            depth_bind,
            mark_bind,
            uniform,
            visible,
            readback,
            depth_view,
            color_view,
        })
    }
    /// Marks the faces visible from `pose` in the GPU flags (not cleared).
    fn render(&self, pose: Pose, options: &Options, clear: bool) -> Result<(), String> {
        self.render_uniform(pose.uniform(options)?, clear)
    }
    fn render_uniform(&self, uniform: [f32; 16], clear: bool) -> Result<(), String> {
        self.queue.write_buffer(&self.uniform, 0, bytes(&uniform));
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
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.),
                        store: wgpu::StoreOp::Store,
                    }),
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
                    view: &self.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Discard,
                    },
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
        Ok(self
            .read_pixels(faces)?
            .into_iter()
            .map(|(f, _)| f)
            .collect())
    }
    /// Visible faces with the pixels they cover at the bake's resolution.
    fn read_pixels(&self, faces: usize) -> Result<Vec<(u16, u32)>, String> {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.visible, 0, &self.readback, 0, self.readback.size());
        self.queue.submit(Some(encoder.finish()));
        let slice = self.readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::Wait)
            .map_err(|e| e.to_string())?;
        rx.recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let data = slice.get_mapped_range();
        let out = data
            .chunks_exact(4)
            .take(faces)
            .enumerate()
            .map(|(i, w)| (i as u16, u32::from_le_bytes([w[0], w[1], w[2], w[3]])))
            .filter(|(_, n)| *n != 0)
            .collect();
        drop(data);
        self.readback.unmap();
        Ok(out)
    }
}

/// Visible face indices (into `track.faces`) for every sample: the faces seen
/// from the sample's pose or any of `substeps` poses toward the next sample.
pub fn bake(scene: &Scene, track: &Track, options: &Options) -> Result<Vec<Vec<u16>>, String> {
    bake_with_cameras(scene, track, options, &[])
}
/// As `bake`, adding for each sample what the recorded cameras assigned to
/// it saw (for example camera poses recorded while playing).
/// The track camera's scale along its view axis (see `Options::depth_scale`).
pub fn camera_depth_scale(scene: &Scene, track: &Track) -> f32 {
    scene
        .actors
        .iter()
        .position(|a| a.id == track.camera)
        .map_or(1., |i| {
            let m = scene.world_matrix(i);
            (0..3)
                .map(|r| m.0[r][2] * m.0[r][2])
                .sum::<f32>()
                .sqrt()
                .max(1e-6)
        })
}
pub fn bake_with_cameras(
    scene: &Scene,
    track: &Track,
    options: &Options,
    recorded: &[CameraCheck],
) -> Result<Vec<Vec<u16>>, String> {
    let options = &Options {
        depth_scale: camera_depth_scale(scene, track),
        ..options.clone()
    };
    let mut by_sample: HashMap<usize, Vec<&CameraCheck>> = HashMap::new();
    for camera in recorded {
        by_sample.entry(camera.sample).or_default().push(camera);
    }
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
        // Largest coverage (bake pixels) of each face over the sample's poses.
        let mut union: HashMap<u16, u32> = HashMap::new();
        let merge = |union: &mut HashMap<u16, u32>, seen: &HashMap<u16, u32>| {
            for (f, n) in seen {
                let e = union.entry(*f).or_insert(0);
                *e = (*e).max(*n);
            }
        };
        let first = at(*pose)?;
        merge(&mut union, &first);
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
                    if depth >= options.refine_depth {
                        continue;
                    }
                    let find = |t: f32, seen: &Vec<(f32, HashMap<u16, u32>)>| {
                        seen.iter().position(|(x, _)| *x == t).unwrap()
                    };
                    let (sa, sb) = (find(a, &seen), find(b, &seen));
                    if !differs(&seen[sa].1, &seen[sb].1) {
                        continue;
                    }
                    let m = (a + b) / 2.;
                    let middle = at(pose.lerp(poses[n], m))?;
                    merge(&mut union, &middle);
                    seen.push((m, middle));
                    stack.push((a, m, depth + 1));
                    stack.push((m, b, depth + 1));
                }
                merge(&mut union, &current);
                previous = (t, current);
            }
        }
        for camera in by_sample.get(&i).into_iter().flatten() {
            gpu.render_uniform(camera.uniform(options), true)?;
            merge(&mut union, &gpu.read_pixels(faces)?.into_iter().collect());
        }
        if let Some(progress) = &options.progress {
            progress.store(i + 1, std::sync::atomic::Ordering::Relaxed);
        }
        let keep = (options.min_pixels * (options.scale * options.scale) as f32).max(1.) as u32;
        let mut list: Vec<u16> = union
            .into_iter()
            .filter(|(_, n)| *n >= keep)
            .map(|(f, _)| f)
            .collect();
        list.sort_unstable();
        out.push(list);
    }
    fill_gaps(&mut out, &next, options.fill_gaps as usize);
    Ok(out)
}

/// See `Options::fill_gaps`. Gaps never cross a cut or a route end.
fn fill_gaps(lists: &mut [Vec<u16>], next: &[Option<usize>], max_gap: usize) {
    if max_gap == 0 {
        return;
    }
    let sets: Vec<std::collections::BTreeSet<u16>> =
        lists.iter().map(|l| l.iter().copied().collect()).collect();
    let mut added: Vec<std::collections::BTreeSet<u16>> = vec![Default::default(); lists.len()];
    for i in 0..lists.len() {
        // Faces visible at i and not at i+1: look for them again within the gap.
        let Some(first) = next[i] else { continue };
        for &face in sets[i].difference(&sets[first]) {
            let mut path = vec![first];
            while path.len() <= max_gap {
                let last = *path.last().unwrap();
                let Some(n) = next[last] else { break };
                if sets[n].contains(&face) {
                    for &k in &path {
                        added[k].insert(face);
                    }
                    break;
                }
                path.push(n);
            }
        }
    }
    for (list, extra) in lists.iter_mut().zip(added) {
        if extra.is_empty() {
            continue;
        }
        let mut set: std::collections::BTreeSet<u16> = list.iter().copied().collect();
        set.extend(extra);
        *list = set.into_iter().collect();
    }
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
        for f in reference.difference(&bake) {
            *missing_by_face.entry(*f).or_default() += 1;
        }
        existing += reference.len();
        automatic += bake.len();
        missing += lost;
        extra += added;
        rows.push(
            serde_json::json!({"sample": sample, "existing": reference.len(), "baked": bake.len(),
            "only_existing": lost, "only_baked": added}),
        );
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
pub fn diagnose(
    scene: &Scene,
    track: &Track,
    sample: usize,
    options: &Options,
) -> Result<String, String> {
    let s = &track.samples[sample];
    let pose = Pose {
        position: s.position,
        rotation: s.rotation,
        fov: s.fov,
    };
    let u = pose.uniform(options)?;
    let listed: std::collections::HashSet<uuid::Uuid> = s
        .visible
        .iter()
        .map(|&i| track.faces[usize::from(i)])
        .collect();
    let (mut front, mut inside, mut behind, mut total) = (0, 0, 0, 0);
    let mut zs = Vec::new();
    for (actor, e) in scene.actors.iter().enumerate() {
        if e.kind != "Mesh" {
            continue;
        }
        let transform = scene.world_matrix(actor);
        for q in crate::lighting::quads(e) {
            if !q.id.is_some_and(|id| listed.contains(&id)) {
                continue;
            }
            total += 1;
            let c = q.points.iter().fold([0f32; 3], |a, p| {
                let w = transform.point(*p);
                [a[0] + w[0] / 4., a[1] + w[1] / 4., a[2] + w[2] / 4.]
            });
            let cam: [f32; 3] = std::array::from_fn(|r| {
                u[r * 4] * c[0] + u[r * 4 + 1] * c[1] + u[r * 4 + 2] * c[2] + u[r * 4 + 3]
            });
            zs.push(cam);
            if cam[2] > 0. {
                front += 1;
                if (u[12] * cam[0] / cam[2]).abs() <= 1. && (u[13] * cam[1] / cam[2]).abs() <= 1. {
                    inside += 1;
                }
            } else {
                behind += 1;
            }
        }
    }
    let (occluders, all) = geometry(scene, track, options.see_through);
    Ok(format!(
        "occluder triangles {} of {}; sample {sample}: listed faces {total}, centroid in front {front}, inside frustum {inside}, behind {behind}; pose {:?}; first camera points {:?}",
        occluders.len() / 3,
        all.len() / 3,
        pose,
        &zs[..zs.len().min(4)]
    ))
}

/// Debug aid: classify where faces only one of the two lists holds sit, for
/// one sample (distance from the camera, screen position).
pub fn diagnose_differences(
    scene: &Scene,
    track: &Track,
    sample: usize,
    options: &Options,
) -> Result<String, String> {
    let mut single = options.clone();
    single.substeps = 0;
    let s = &track.samples[sample];
    let pose = Pose {
        position: s.position,
        rotation: s.rotation,
        fov: s.fov,
    };
    let one = Track {
        samples: vec![s.clone()],
        navigation: None,
        ..track.clone()
    };
    let baked: std::collections::HashSet<u16> =
        bake(scene, &one, &single)?.remove(0).into_iter().collect();
    let listed: std::collections::HashSet<u16> = s.visible.iter().copied().collect();
    let u = pose.uniform(options)?;
    let index: HashMap<uuid::Uuid, u16> = track
        .faces
        .iter()
        .enumerate()
        .map(|(i, f)| (*f, i as u16))
        .collect();
    let mut out = String::new();
    for (name, set, other) in [
        ("only baked", &baked, &listed),
        ("only listed", &listed, &baked),
    ] {
        let mut dist = Vec::new();
        let (mut edge, mut offscreen, mut n) = (0, 0, 0);
        for (actor, e) in scene.actors.iter().enumerate() {
            if e.kind != "Mesh" {
                continue;
            }
            let transform = scene.world_matrix(actor);
            for q in crate::lighting::quads(e) {
                let Some(i) = q.id.and_then(|id| index.get(&id)) else {
                    continue;
                };
                if !set.contains(i) || other.contains(i) {
                    continue;
                }
                n += 1;
                let cams: Vec<[f32; 3]> = q
                    .points
                    .iter()
                    .map(|p| {
                        let w = transform.point(*p);
                        std::array::from_fn(|r| {
                            u[r * 4] * w[0]
                                + u[r * 4 + 1] * w[1]
                                + u[r * 4 + 2] * w[2]
                                + u[r * 4 + 3]
                        })
                    })
                    .collect();
                let near = cams.iter().map(|c| c[2]).fold(f32::MAX, f32::min);
                dist.push(near);
                let ndc: Vec<[f32; 2]> = cams
                    .iter()
                    .filter(|c| c[2] > 0.)
                    .map(|c| [u[12] * c[0] / c[2], u[13] * c[1] / c[2]])
                    .collect();
                let all_out = !ndc.is_empty()
                    && (ndc.iter().all(|v| v[0] < -1.)
                        || ndc.iter().all(|v| v[0] > 1.)
                        || ndc.iter().all(|v| v[1] < -1.)
                        || ndc.iter().all(|v| v[1] > 1.));
                if all_out {
                    offscreen += 1;
                } else if ndc.iter().any(|v| v[0].abs() > 0.9 || v[1].abs() > 0.9) {
                    edge += 1;
                }
            }
        }
        dist.sort_by(f32::total_cmp);
        let q = |f: f32| {
            dist.get(((dist.len() as f32 - 1.) * f) as usize)
                .copied()
                .unwrap_or(0.)
        };
        out += &format!(
            "{name}: {n} faces, nearest-corner depth p10 {:.1} p50 {:.1} p90 {:.1} max {:.1}; off screen {offscreen}, touching edge {edge}\n",
            q(0.1),
            q(0.5),
            q(0.9),
            q(1.)
        );
    }
    Ok(out)
}

/// Hole check between samples: at poses part way to the next continuous sample
/// (which a sample's list must cover while the camera moves), the faces truly
/// visible there that each set of lists lacks.
pub fn holes(
    scene: &Scene,
    track: &Track,
    baked: &[Vec<u16>],
    options: &Options,
    fractions: &[f32],
) -> Result<serde_json::Value, String> {
    let options = &Options {
        depth_scale: camera_depth_scale(scene, track),
        ..options.clone()
    };
    let (occluders, all) = geometry(scene, track, options.see_through);
    let gpu = Gpu::new(options, track.faces.len(), &occluders, &all)?;
    let (poses, next) = poses(track);
    let (mut truth_total, mut missing_baked, mut missing_existing, mut checks) =
        (0usize, 0usize, 0usize, 0usize);
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
            let pixels = truth
                .iter()
                .filter(|(f, _)| !b.contains(f))
                .map(|(_, n)| f64::from(*n) / area)
                .collect::<Vec<_>>();
            missing_pixels += pixels.iter().sum::<f64>();
            let largest = pixels.iter().copied().fold(0., f64::max);
            if largest > worst_pixels {
                worst_pixels = largest;
                worst_case = (
                    i,
                    t,
                    truth
                        .iter()
                        .filter(|(f, _)| !b.contains(f))
                        .max_by_key(|(_, n)| *n)
                        .map(|(f, _)| *f),
                );
            }
            truth_total += truth.len();
            missing_baked += mb;
            missing_existing += me;
            checks += 1;
            if mb > worst_baked.1 {
                worst_baked = (i, mb);
            }
            if me > worst_existing.1 {
                worst_existing = (i, me);
            }
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
pub(crate) mod tests {
    use super::*;
    fn cube_actor(
        name: &str,
        centre: [f32; 3],
        size: f32,
        blend: BlendMode,
    ) -> crate::scene::Actor {
        let mut doc = crate::mesh::Document::default();
        doc.materials[0].material.unlit = true;
        doc.materials[0].material.blend = blend;
        doc.primitive(
            "Cube",
            [0.; 3],
            [size; 3],
            1,
            doc.groups[0].id,
            doc.materials[0].id,
        );
        let mut e = crate::scene::Actor::cube(name.into());
        let mut component = crate::mesh::Component::new(uuid::Uuid::new_v4());
        component.document = Some(std::sync::Arc::new(doc));
        e.editable_mesh = Some(component);
        e.position = centre;
        e
    }
    pub(crate) fn scene_with(front_blend: BlendMode) -> (Scene, Track, usize) {
        // A large cube right in front of the camera hides a small one behind it.
        let front = cube_actor("Front", [0., 0., 6.], 6., front_blend);
        let back = cube_actor("Back", [0., 0., 14.], 1., BlendMode::Cutout);
        let ids = |e: &crate::scene::Actor| {
            e.editable_mesh
                .as_ref()
                .unwrap()
                .document
                .as_ref()
                .unwrap()
                .faces
                .iter()
                .map(|f| f.id)
                .collect::<Vec<_>>()
        };
        let front_faces = ids(&front).len();
        let faces: Vec<_> = ids(&front).into_iter().chain(ids(&back)).collect();
        let camera = Scene::default().actors[0].clone();
        let track = Track { path: Vec::new(),
            camera: camera.id,
            sample_ms: 100,
            cull_backfaces: false,
            static_world: false,
            navigation: None,
            samples: vec![crate::visibility_track::Sample {
                position: [0.; 3],
                rotation: [0.; 3],
                fov: 60.,
                visible: vec![],
                cut: true,
            }],
            faces,
        };
        (
            Scene {
                actors: vec![front, back, camera],
                ..Default::default()
            },
            track,
            front_faces,
        )
    }
    #[test]
    fn opaque_faces_hide_what_is_behind_and_blended_ones_do_not() {
        let options = Options {
            scale: 1,
            substeps: 0,
            ..Default::default()
        };
        let (scene, track, front_faces) = scene_with(BlendMode::Cutout);
        let visible = bake(&scene, &track, &options).unwrap().remove(0);
        // Only the front cube's face towards the camera shows.
        assert_eq!(visible.len(), 1, "{visible:?}");
        assert!(usize::from(visible[0]) < front_faces);
        let (scene, track, front_faces) = scene_with(BlendMode::Average);
        let visible = bake(&scene, &track, &options).unwrap().remove(0);
        assert!(
            visible.iter().any(|&f| usize::from(f) >= front_faces),
            "{visible:?}"
        );
        // Blended faces never occlude, so the far side of the front cube shows too.
        assert!(
            visible
                .iter()
                .filter(|&&f| usize::from(f) < front_faces)
                .count()
                > 1
        );
    }
    #[test]
    fn recorded_cameras_load_from_json_lines_and_match_the_track_projection() {
        // A recorded camera equal to the track pose bakes the same list.
        let (scene, track, _) = scene_with(BlendMode::Cutout);
        let options = Options {
            scale: 1,
            substeps: 0,
            ..Default::default()
        };
        let cot = 1. / (60f32.to_radians() / 2.).tan();
        let line = serde_json::json!({"sample": 0, "frame": 3, "basis": [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]],
            "origin": [0, 0, 0], "row_scale": [(cot * 256. / 160. * 4096.).round(), (cot * 4096.).round()]});
        let path =
            std::env::temp_dir().join(format!("epok-cameras-{}.jsonl", uuid::Uuid::new_v4()));
        std::fs::write(&path, format!("{line}\n{line}\n")).unwrap();
        let cameras = CameraCheck::load(&path, &options).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(cameras.len(), 2);
        let from_track = bake(&scene, &track, &options).unwrap();
        let mut empty = track.clone();
        empty.samples[0].visible.clear();
        let check = check_cameras(&scene, &empty, &from_track, &cameras, &options).unwrap();
        assert_eq!(check["lists"]["missing_faces"], 0, "{check}");
    }
    #[test]
    fn poses_stop_at_cuts_and_interpolate_angles_the_short_way() {
        let a = Pose {
            position: [0.; 3],
            rotation: [350., 0., 0.],
            fov: 60.,
        };
        let b = Pose {
            position: [2., 0., 0.],
            rotation: [10., 0., 0.],
            fov: 80.,
        };
        let m = a.lerp(b, 0.5);
        assert_eq!(m.position, [1., 0., 0.]);
        assert!((m.rotation[0] - 360.).abs() < 1e-3 && (m.fov - 70.).abs() < 1e-6);
    }
}

/// A tracked triangle for the prepared packet order: its source identity,
/// world Q12 corners and the attributes two halves of a packet must share.
struct Triangle {
    reference: crate::ordered_draw::FaceRef,
    track_index: u16,
    points: [[i32; 3]; 3],
    uv: [[f32; 2]; 3],
    colors: Option<[[u8; 3]; 3]>,
    material: crate::scene::Material,
}

fn triangles(scene: &Scene, track: &Track) -> Result<Vec<Triangle>, String> {
    let index: HashMap<uuid::Uuid, u16> = track
        .faces
        .iter()
        .enumerate()
        .map(|(i, f)| (*f, i as u16))
        .collect();
    let mut out = Vec::new();
    for (actor_index, actor) in scene.actors.iter().enumerate() {
        let Some(doc) = actor
            .editable_mesh
            .as_ref()
            .and_then(|m| m.document.as_deref())
        else {
            continue;
        };
        if actor.kind != "Mesh" || !doc.faces.iter().any(|f| index.contains_key(&f.id)) {
            continue;
        }
        let transform = scene.world_matrix(actor_index);
        for face in &doc.faces {
            let Some(&track_index) = index.get(&face.id) else {
                continue;
            };
            if face.vertices[2] != face.vertices[3] {
                return Err("The prepared packet order needs triangle source faces".into());
            }
            let raw = doc.points(face);
            let mut points = [[0; 3]; 3];
            for c in 0..3 {
                points[c] = crate::mesh_compile::world_q12(&transform, raw[c])?;
            }
            out.push(Triangle {
                reference: crate::ordered_draw::FaceRef {
                    actor: actor.id,
                    face: face.id,
                },
                track_index,
                points,
                uv: [face.uv[0], face.uv[1], face.uv[2]],
                colors: face.vertex_colors.map(|c| [c[0], c[1], c[2]]),
                material: crate::mesh::material(actor, face.material),
            });
        }
    }
    Ok(out)
}

/// Pairs neighbouring triangles into GPU quads: same material and colour
/// domain, and an edge whose corners agree in position, UV and colour. The
/// packet's corners 1 and 2 are the shared edge; 0 and 3 the other corners.
/// Triangles (and the corners of the edge) sharing each quantized edge.
type Edges = HashMap<([i32; 3], [i32; 3]), Vec<(usize, usize, usize)>>;

fn pair(tris: &[Triangle]) -> Vec<crate::ordered_draw::Primitive> {
    let mut edges = Edges::new();
    for (t, tri) in tris.iter().enumerate() {
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            let key = if tri.points[a] <= tri.points[b] {
                (tri.points[a], tri.points[b])
            } else {
                (tri.points[b], tri.points[a])
            };
            edges.entry(key).or_default().push((t, a, b));
        }
    }
    let corner_matches = |x: &Triangle, i: usize, y: &Triangle, j: usize| {
        x.points[i] == y.points[j]
            && x.uv[i] == y.uv[j]
            && x.colors.map(|c| c[i]) == y.colors.map(|c| c[j])
    };
    let mut partner = vec![None::<(usize, usize, usize)>; tris.len()];
    let mut used = vec![false; tris.len()];
    for (t, tri) in tris.iter().enumerate() {
        if used[t] {
            continue;
        }
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            let key = if tri.points[a] <= tri.points[b] {
                (tri.points[a], tri.points[b])
            } else {
                (tri.points[b], tri.points[a])
            };
            let found = edges[&key]
                .iter()
                .find(|&&(u, c, d)| {
                    u != t
                        && !used[u]
                        && tris[u].material == tri.material
                        && tris[u].colors.is_some() == tri.colors.is_some()
                        && ((corner_matches(tri, a, &tris[u], c)
                            && corner_matches(tri, b, &tris[u], d))
                            || (corner_matches(tri, a, &tris[u], d)
                                && corner_matches(tri, b, &tris[u], c)))
                })
                .copied();
            if let Some((u, c, d)) = found {
                used[t] = true;
                used[u] = true;
                partner[t] = Some((u, a, b));
                let _ = (c, d);
                break;
            }
        }
    }
    let mut primitives = Vec::new();
    let mut done = vec![false; tris.len()];
    for (t, tri) in tris.iter().enumerate() {
        if done[t] {
            continue;
        }
        done[t] = true;
        match partner[t] {
            Some((u, a, b)) => {
                done[u] = true;
                let opposite = 3 - a - b;
                let other = &tris[u];
                let opposite_other = (0..3)
                    .find(|&k| other.points[k] != tri.points[a] && other.points[k] != tri.points[b])
                    .unwrap();
                primitives.push(crate::ordered_draw::Primitive {
                    first: tri.reference,
                    second: Some(other.reference),
                    corners: [
                        [0, opposite as u8],
                        [0, a as u8],
                        [0, b as u8],
                        [1, opposite_other as u8],
                    ],
                });
            }
            None => primitives.push(crate::ordered_draw::Primitive {
                first: tri.reference,
                second: None,
                corners: [[0, 0], [0, 1], [0, 2], [0, 2]],
            }),
        }
    }
    primitives
}

/// A prepared packet order for a track whose lists are `lists`: triangles
/// paired into quads, and each sample's packets sorted back to front by their
/// mean camera depth at the sample's pose (the renderer draws a list from its
/// end, keeping this order with a monotone depth cursor).
pub fn ordered_program(
    scene: &Scene,
    track: &Track,
    lists: &[Vec<u16>],
    options: &Options,
) -> Result<crate::ordered_draw::Program, String> {
    let tris = triangles(scene, track)?;
    let primitives = pair(&tris);
    let by_face: HashMap<uuid::Uuid, usize> = tris
        .iter()
        .enumerate()
        .map(|(i, t)| (t.reference.face, i))
        .collect();
    // Track face index -> (primitive, half: 1 first, 2 second).
    let mut owner = vec![(0usize, 0u32); track.faces.len()];
    for (p, primitive) in primitives.iter().enumerate() {
        owner[usize::from(tris[by_face[&primitive.first.face]].track_index)] = (p, 1);
        if let Some(second) = primitive.second {
            owner[usize::from(tris[by_face[&second.face]].track_index)] = (p, 2);
        }
    }
    let (poses, next) = poses(track);
    let mut samples: Vec<Vec<u32>> = Vec::with_capacity(lists.len());
    // Position of each primitive in the previous continuous sample's list.
    let mut previous: HashMap<usize, usize> = HashMap::new();
    for (i, (list, pose)) in lists.iter().zip(&poses).enumerate() {
        let continuous = i > 0 && next[i - 1] == Some(i);
        if !continuous {
            previous.clear();
        }
        let view =
            crate::transform::Matrix::trs(pose.position, pose.rotation, [1.; 3]).inverse()?;
        let mut parts: BTreeMap<usize, u32> = BTreeMap::new();
        for &face in list {
            *parts.entry(owner[usize::from(face)].0).or_default() |= owner[usize::from(face)].1;
        }
        let mut packets: Vec<(f64, u32, usize)> = parts
            .into_iter()
            .map(|(p, halves)| {
                let primitive = &primitives[p];
                let part = if primitive.second.is_none() {
                    0
                } else {
                    halves
                };
                let mut faces = vec![];
                if part != 2 {
                    faces.push(&tris[by_face[&primitive.first.face]]);
                }
                if part >= 2 {
                    faces.push(&tris[by_face[&primitive.second.unwrap().face]]);
                }
                let (mut depth, mut n) = (0f64, 0f64);
                for t in faces {
                    for c in t.points {
                        let w = c.map(|v| v as f32 / 4096.);
                        depth += f64::from(view.point(w)[2]);
                        n += 1.;
                    }
                }
                (depth / n, (p as u32) << 2 | part, p)
            })
            .collect();
        // Far first by depth step; within a step, the previous order, then
        // newly visible packets by exact depth (primitive index last).
        let step = f64::from(options.order_step.max(1e-6));
        packets.sort_by(|a, b| {
            let (sa, sb) = ((a.0 / step).floor(), (b.0 / step).floor());
            sb.total_cmp(&sa)
                .then_with(|| match (previous.get(&a.2), previous.get(&b.2)) {
                    (Some(x), Some(y)) => x.cmp(y),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => b.0.total_cmp(&a.0),
                })
                .then(a.1.cmp(&b.1))
        });
        previous = packets.iter().enumerate().map(|(k, x)| (x.2, k)).collect();
        samples.push(packets.into_iter().map(|(_, token, _)| token).collect());
    }
    // Number primitives so that consecutive ones are close in space: the
    // cooker groups each 64 consecutive primitives into one projection block,
    // whose vertices take the fast compact path only when the whole block
    // lies within the GTE's 16-bit range of the camera. Cells of
    // `order_cell` world units follow a Z-order (Morton) curve; within a cell,
    // primitives keep their order of first appearance in the lists, so
    // packets drawn together also tend to share pages.
    let mut first_seen = vec![usize::MAX; primitives.len()];
    let mut rank = 0;
    for list in &samples {
        for &token in list {
            let p = (token >> 2) as usize;
            if first_seen[p] == usize::MAX {
                first_seen[p] = rank;
                rank += 1;
            }
        }
    }
    let cell = f64::from(options.order_cell.max(1e-3)) * 4096.;
    let morton = |p: usize| -> u64 {
        let t = &tris[by_face[&primitives[p].first.face]];
        let mut key = 0u64;
        let c: [u64; 3] = std::array::from_fn(|a| {
            let centre = t.points.iter().map(|q| f64::from(q[a])).sum::<f64>() / 3.;
            ((centre / cell).floor() as i64 + (1 << 20)).clamp(0, (1 << 21) - 1) as u64
        });
        for bit in (0..21).rev() {
            for axis in c {
                key = key << 1 | (axis >> bit & 1);
            }
        }
        key
    };
    let mut order: Vec<usize> = (0..primitives.len()).collect();
    let keys: Vec<u64> = (0..primitives.len()).map(morton).collect();
    order.sort_by_key(|&p| (keys[p], first_seen[p], p));
    let mut renumber = vec![0u32; primitives.len()];
    for (new, &old) in order.iter().enumerate() {
        renumber[old] = new as u32;
    }
    let primitives: Vec<_> = order.iter().map(|&p| primitives[p].clone()).collect();
    let samples = samples
        .into_iter()
        .map(|list| {
            list.into_iter()
                .map(|t| renumber[(t >> 2) as usize] << 2 | (t & 3))
                .collect()
        })
        .collect();
    Ok(crate::ordered_draw::Program {
        version: 1,
        static_world: true,
        primitives,
        samples,
    })
}

/// Debug aid: distribution of the nearest-corner camera depth of the faces a
/// track's existing lists hold, over all samples (how far the lists reach).
pub fn list_depths(scene: &Scene, track: &Track) -> Result<String, String> {
    let tris = triangles(scene, track)?;
    let by_index: HashMap<u16, &Triangle> = tris.iter().map(|t| (t.track_index, t)).collect();
    let (poses, _) = poses(track);
    let mut depths = Vec::new();
    for (sample, pose) in track.samples.iter().zip(&poses) {
        let view =
            crate::transform::Matrix::trs(pose.position, pose.rotation, [1.; 3]).inverse()?;
        for f in &sample.visible {
            let Some(t) = by_index.get(f) else { continue };
            let z = t
                .points
                .iter()
                .map(|c| view.point(c.map(|v| v as f32 / 4096.))[2])
                .fold(f32::MAX, f32::min);
            depths.push(z);
        }
    }
    depths.sort_by(f32::total_cmp);
    let q = |f: f32| depths[((depths.len() - 1) as f32 * f) as usize];
    let beyond =
        |d: f32| depths.iter().filter(|&&z| z > d).count() as f32 / depths.len() as f32 * 100.;
    Ok(format!(
        "listed faces {}: depth p50 {:.0} p90 {:.0} p99 {:.0} max {:.0}; beyond 64: {:.1}%, 96: {:.1}%, 128: {:.1}%",
        depths.len(),
        q(0.5),
        q(0.9),
        q(0.99),
        q(1.),
        beyond(64.),
        beyond(96.),
        beyond(128.)
    ))
}

/// A camera given directly as runtime camera space: basis rows (right, up,
/// forward), origin, and the projection's NDC scales for x and y.
#[derive(Clone, Copy, Debug)]
pub struct CameraCheck {
    pub sample: usize,
    pub basis: [[f32; 3]; 3],
    pub origin: [f32; 3],
    pub scale: [f32; 2],
}

impl CameraCheck {
    fn uniform(&self, options: &Options) -> [f32; 16] {
        // Depth in world units whatever the basis scale (a recording keeps the
        // runtime's, a converted camera may be unit length): divide by the
        // forward row's length, which leaves x/z and y/z unchanged. Near and
        // far are runtime camera units, converted with the track camera's scale.
        let forward = (0..3)
            .map(|c| self.basis[2][c] * self.basis[2][c])
            .sum::<f32>()
            .sqrt()
            .max(1e-6);
        let mut u = [0f32; 16];
        for r in 0..3 {
            for c in 0..3 {
                u[r * 4 + c] = self.basis[r][c] / forward;
            }
            u[r * 4 + 3] = -(0..3)
                .map(|c| self.basis[r][c] * self.origin[c])
                .sum::<f32>()
                / forward;
        }
        let widen = 1. + options.view_margin.max(0.);
        u[12] = self.scale[0] / widen;
        u[13] = self.scale[1] / widen;
        u[14] = options.near * options.depth_scale;
        u[15] = options.far * options.depth_scale;
        u
    }
    /// Runtime Q12 camera space as JSON {"sample","basis","origin","row_scale"}.
    pub fn from_json(c: &serde_json::Value, options: &Options) -> Self {
        let q12 = |v: &serde_json::Value| v.as_f64().unwrap_or(0.) as f32 / 4096.;
        let focal = (options.height * 2 / 3) as f32;
        Self {
            sample: c["sample"].as_u64().unwrap_or(0) as usize,
            basis: std::array::from_fn(|r| std::array::from_fn(|k| q12(&c["basis"][r][k]))),
            origin: std::array::from_fn(|k| q12(&c["origin"][k])),
            scale: [
                focal * q12(&c["row_scale"][0]) / (options.width / 2) as f32,
                focal * q12(&c["row_scale"][1]) / (options.height / 2) as f32,
            ],
        }
    }
    /// A JSON file {"cameras":[...]}, or JSON lines with one camera each (as
    /// Native PC Play records them with --record-camera).
    pub fn load(path: &std::path::Path, options: &Options) -> Result<Vec<Self>, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text)
            && let Some(cameras) = json["cameras"].as_array()
        {
            return Ok(cameras
                .iter()
                .map(|c| Self::from_json(c, options))
                .collect());
        }
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                serde_json::from_str::<serde_json::Value>(l)
                    .map(|c| Self::from_json(&c, options))
                    .map_err(|e| e.to_string())
            })
            .collect()
    }
}

/// Hole check with explicit cameras (for example the cameras a game actually
/// used): faces truly visible from each camera that the sample's list lacks,
/// in faces and display pixels, for the given lists and the track's own.
pub fn check_cameras(
    scene: &Scene,
    track: &Track,
    lists: &[Vec<u16>],
    cameras: &[CameraCheck],
    options: &Options,
) -> Result<serde_json::Value, String> {
    let (occluders, all) = geometry(scene, track, options.see_through);
    let gpu = Gpu::new(options, track.faces.len(), &occluders, &all)?;
    let area = f64::from(options.scale * options.scale);
    let (mut faces_lists, mut faces_track, mut pixels_lists, mut pixels_track, mut checked) =
        (0usize, 0usize, 0f64, 0f64, 0usize);
    let (mut worst_lists, mut worst_track) = ((0usize, 0f64), (0usize, 0f64));
    // Display pixels each listed face covers at the camera: 0, <1, <4, <16, >=16.
    let mut coverage_lists = [0usize; 5];
    let mut coverage_track = [0usize; 5];
    let bucket = |px: f64| {
        if px <= 0. {
            0
        } else if px < 1. {
            1
        } else if px < 4. {
            2
        } else if px < 16. {
            3
        } else {
            4
        }
    };
    // Checks see exactly what each camera sees: no view margin.
    let exact = Options {
        view_margin: 0.,
        depth_scale: camera_depth_scale(scene, track),
        ..options.clone()
    };
    for camera in cameras {
        if camera.sample >= lists.len() {
            continue;
        }
        gpu.render_uniform(camera.uniform(&exact), true)?;
        let truth = gpu.read_pixels(track.faces.len())?;
        checked += truth.len();
        let pixels: HashMap<u16, f64> = truth
            .iter()
            .map(|(f, n)| (*f, f64::from(*n) / area))
            .collect();
        for f in &lists[camera.sample] {
            coverage_lists[bucket(pixels.get(f).copied().unwrap_or(0.))] += 1;
        }
        for f in &track.samples[camera.sample].visible {
            coverage_track[bucket(pixels.get(f).copied().unwrap_or(0.))] += 1;
        }
        for (list, faces, pixels, worst) in [
            (
                &lists[camera.sample],
                &mut faces_lists,
                &mut pixels_lists,
                &mut worst_lists,
            ),
            (
                &track.samples[camera.sample].visible,
                &mut faces_track,
                &mut pixels_track,
                &mut worst_track,
            ),
        ] {
            let set: std::collections::HashSet<u16> = list.iter().copied().collect();
            let missing: Vec<f64> = truth
                .iter()
                .filter(|(f, _)| !set.contains(f))
                .map(|(_, n)| f64::from(*n) / area)
                .collect();
            *faces += missing.len();
            let sum: f64 = missing.iter().sum();
            *pixels += sum;
            if sum > worst.1 {
                *worst = (camera.sample, sum);
            }
        }
    }
    let n = cameras.len().max(1) as f64;
    Ok(serde_json::json!({
        "cameras": cameras.len(), "visible_faces_checked": checked,
        "coverage_buckets": "display pixels at the camera: 0, <1, <4, <16, >=16",
        "lists_coverage": coverage_lists, "track_coverage": coverage_track,
        "lists": {"missing_faces": faces_lists, "missing_display_pixels_per_camera": pixels_lists / n, "worst": worst_lists},
        "track": {"missing_faces": faces_track, "missing_display_pixels_per_camera": pixels_track / n, "worst": worst_track},
    }))
}
