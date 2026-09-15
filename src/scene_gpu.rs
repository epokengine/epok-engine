//! Scene is an editor view rendered on the host GPU. Game remains the emulator's output.
use crate::scene::Scene;
use wgpu::util::DeviceExt;

pub const SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 960,
    height: 600,
    depth_or_array_layers: 1,
};
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
struct Vertices {
    buffer: wgpu::Buffer,
    capacity: u64,
    count: u32,
}
impl Vertices {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Scene vertices"),
                size: 40,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            capacity: 40,
            count: 0,
        }
    }
    fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        if data.len() as u64 > self.capacity {
            self.capacity = (data.len() as u64).next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Cached Scene geometry"),
                size: self.capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        self.count = (data.len() / 40) as u32;
        if !data.is_empty() {
            queue.write_buffer(&self.buffer, 0, data);
        }
    }
}
pub struct SceneGpu {
    preview_started: std::time::Instant,
    mesh_pipelines: Vec<wgpu::RenderPipeline>,
    draw_ranges: Vec<(u32, u32, usize)>,
    textures: wgpu::Texture,
    cached_angles: [f32; 2],
    shadow_pipeline: wgpu::RenderPipeline,
    shadows: Vertices,
    line_pipeline: wgpu::RenderPipeline,
    background_pipeline: wgpu::RenderPipeline,
    camera: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    depth: wgpu::TextureView,
    mesh: Vertices,
    edges: Vertices,
    grid: Vertices,
    cached_scene: Option<Scene>,
    cached_skeletal_poses: SkeletalPoseKey,
    cached_selected: Option<usize>,
    cached_wire: bool,
    cached_phase: f32,
    cached_fog_center: [f32; 3],
    cached_fog_distance: f32,
    cached_mesh_revision: u64,
}
struct RenderInput<'a> {
    scene: &'a Scene,
    view: &'a crate::viewport::View,
    selected: Option<usize>,
    mesh: &'a crate::mesh_editor::State,
    wire: bool,
    grid: bool,
    effect: Option<&'a [crate::particle_effect_preview::Quad]>,
    background: Option<[f32; 3]>,
}
/// Runtime-only skeletal state that changes the generated vertex buffer.
///
/// `Scene` equality deliberately serializes the document before comparing it,
/// so its transient `skeletal_mesh.time` field is omitted. Keep the sampled
/// pose separate from the document cache: animation can then refresh the
/// Scene View without becoming an authored scene edit.
type SkeletalPoseKey = Vec<Option<(usize, Option<usize>)>>;

fn skeletal_pose_key(scene: &Scene) -> SkeletalPoseKey {
    scene
        .actors
        .iter()
        .enumerate()
        .map(|(index, actor)| {
            if !scene.is_active(index) {
                return None;
            }
            let component = actor.skeletal_mesh.as_ref()?;
            let model = component.model.as_ref()?;
            let frame = component
                .clip
                .and_then(|id| model.clips.iter().find(|(key, _)| *key == id))
                .map(|(_, clip)| {
                    let frame =
                        (component.time.max(0.) * clip.fps as f32 + 0.0001).floor() as usize;
                    if component.looping {
                        frame % (clip.frames as usize - 1).max(1)
                    } else {
                        frame.min(clip.frames as usize - 1)
                    }
                });
            Some((std::sync::Arc::as_ptr(model) as usize, frame))
        })
        .collect()
}

impl SceneGpu {
    pub fn animated(scene: &Scene) -> bool {
        crate::effects::animated(scene)
            || scene.actors.iter().enumerate().any(|(index, e)| {
                scene.is_active(index)
                    && (e.sprite_animator.as_ref().is_some_and(|a| a.playing)
                        || e.particle_emitter.as_ref().is_some_and(|p| p.enabled)
                        || e.palette_animator.as_ref().is_some_and(|p| p.enabled))
            })
    }
    pub fn preview_time(&self, scene: &Scene, phase: f32) -> f32 {
        phase
            + if Self::animated(scene) {
                (self.preview_started.elapsed().as_secs_f32() * 60.).floor() / 60.
            } else {
                0.
            }
    }
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Scene GPU shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scene_gpu.wgsl").into()),
        });
        let camera = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene camera"),
            contents: &[0; 48],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let textures = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PSX texture previews"),
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 33,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = textures.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene camera layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene camera"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Scene pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let make_pipeline = |topology, background, shadow: bool, blend: usize, cull: bool| {
            const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
                wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x4];
            let buffers = [wgpu::VertexBufferLayout {
                array_stride: 40,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATTRIBUTES,
            }];
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(if background {
                    "Scene background"
                } else {
                    "Scene geometry"
                }),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(if background {
                        "background_vertex"
                    } else {
                        "vertex"
                    }),
                    compilation_options: Default::default(),
                    buffers: if background { &[] } else { &buffers },
                },
                primitive: wgpu::PrimitiveState {
                    topology,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: cull.then_some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: !background && !shadow && blend == 0,
                    depth_compare: if background {
                        wgpu::CompareFunction::Always
                    } else {
                        wgpu::CompareFunction::LessEqual
                    },
                    stencil: Default::default(),
                    bias: if topology == wgpu::PrimitiveTopology::LineList {
                        wgpu::DepthBiasState {
                            constant: -2,
                            slope_scale: -1.,
                            clamp: 0.,
                        }
                    } else {
                        Default::default()
                    },
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(if background {
                        "background_fragment"
                    } else {
                        "fragment"
                    }),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: FORMAT,
                        blend: if blend > 0 {
                            Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: if blend == 1 || blend == 4 {
                                        wgpu::BlendFactor::SrcAlpha
                                    } else {
                                        wgpu::BlendFactor::One
                                    },
                                    dst_factor: if blend == 1 {
                                        wgpu::BlendFactor::OneMinusSrcAlpha
                                    } else {
                                        wgpu::BlendFactor::One
                                    },
                                    operation: if blend == 3 {
                                        wgpu::BlendOperation::ReverseSubtract
                                    } else {
                                        wgpu::BlendOperation::Add
                                    },
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            })
                        } else if shadow {
                            Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::ReverseSubtract,
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            })
                        } else {
                            None
                        },
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            })
        };
        let depth = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Scene depth"),
                size: SIZE,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default());
        let mut grid = Vertices::new(device);
        let mut data = Vec::new();
        for i in -10..=10 {
            line(
                &mut data,
                [i as f32, -0.26, -10.],
                [i as f32, -0.26, 10.],
                [88, 88, 88],
            );
            line(
                &mut data,
                [-10., -0.26, i as f32],
                [10., -0.26, i as f32],
                [88, 88, 88],
            );
        }
        line(
            &mut data,
            [-10., -0.25, 0.],
            [10., -0.25, 0.],
            [127, 75, 75],
        );
        line(
            &mut data,
            [0., -0.25, -10.],
            [0., -0.25, 10.],
            [71, 102, 139],
        );
        grid.upload(device, queue, &data);
        Self {
            preview_started: std::time::Instant::now(),
            mesh_pipelines: (0..10)
                .map(|b| {
                    make_pipeline(
                        wgpu::PrimitiveTopology::TriangleList,
                        false,
                        false,
                        b % 5,
                        b >= 5,
                    )
                })
                .collect(),
            draw_ranges: vec![],
            textures,
            cached_angles: [f32::NAN; 2],
            line_pipeline: make_pipeline(wgpu::PrimitiveTopology::LineList, false, false, 0, false),
            background_pipeline: make_pipeline(
                wgpu::PrimitiveTopology::TriangleList,
                true,
                false,
                0,
                false,
            ),
            camera,
            bind_group,
            depth,
            mesh: Vertices::new(device),
            shadows: Vertices::new(device),
            shadow_pipeline: make_pipeline(
                wgpu::PrimitiveTopology::TriangleList,
                false,
                true,
                0,
                false,
            ),
            edges: Vertices::new(device),
            grid,
            cached_scene: None,
            cached_skeletal_poses: vec![],
            cached_selected: None,
            cached_wire: false,
            cached_phase: 0.,
            cached_fog_center: [0.; 3],
            cached_fog_distance: 12.,
            cached_mesh_revision: 0,
        }
    }
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        editor: &crate::editor::Editor,
    ) {
        self.render_input(
            device,
            queue,
            encoder,
            target,
            RenderInput {
                scene: editor.timeline_editor.scene_preview.scene.as_ref().filter(|_| editor.timeline_editor.open && !editor.playing).unwrap_or(&editor.scene),
                view: &editor.view,
                selected: editor.selected,
                mesh: &editor.mesh_editor,
                wire: editor.wire,
                grid: editor.grid,
                effect: None,
                background: None,
            },
        );
    }
    pub fn render_effect(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        preview: &crate::particle_effect_preview::View,
    ) {
        let Some(simulation) = &preview.simulation else {
            return;
        };
        self.render_input(
            device,
            queue,
            encoder,
            target,
            RenderInput {
                scene: &simulation.resources,
                view: &preview.camera,
                selected: None,
                mesh: &Default::default(),
                wire: false,
                grid: false,
                effect: Some(&preview.quads),
                background: Some(preview.background),
            },
        );
    }
    pub fn render_asset(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        preview: &crate::asset_inspector::State,
    ) {
        let Some(scene) = preview.details.as_ref().and_then(|d| d.scene.as_ref()) else {
            return;
        };
        self.render_input(
            device,
            queue,
            encoder,
            target,
            RenderInput {
                scene,
                view: &preview.camera,
                selected: None,
                mesh: &Default::default(),
                wire: false,
                grid: false,
                effect: None,
                background: Some([0.10, 0.11, 0.12]),
            },
        );
    }
    fn render_input(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        input: RenderInput<'_>,
    ) {
        let scene = input.scene;
        let view = input.view;
        let selected = input.selected;
        let preview_time = self.preview_time(scene, view.phase);
        let skeletal_poses = skeletal_pose_key(scene);
        if input.effect.is_some()
            || self.cached_scene.as_ref() != Some(scene)
            || self.cached_skeletal_poses != skeletal_poses
            || self.cached_angles != [view.yaw, view.pitch]
            || self.cached_selected != selected
            || self.cached_wire != input.wire
            || self.cached_phase != preview_time
            || (scene.fog.enabled
                && (self.cached_fog_center != view.center
                    || self.cached_fog_distance != view.distance))
            || self.cached_mesh_revision != input.mesh.revision
        {
            let (mesh, edges, ranges) = geometry_with_effects(
                scene,
                selected,
                preview_time,
                input.mesh,
                view,
                input.effect.unwrap_or_default(),
                input.wire,
            );
            self.cached_fog_center = view.center;
            self.cached_fog_distance = view.distance;
            self.draw_ranges = ranges;
            self.cached_angles = [view.yaw, view.pitch];
            let mut layers = vec![(None, vec![255u8; 256 * 256 * 4])];
            for id in crate::texture::ids(scene).into_iter().take(32) {
                let mut bytes = vec![0; 256 * 256 * 4];
                if let Some(t) = scene.textures.get(&id) {
                    let palette_rgba = crate::palette::preview_rgba(scene, id, preview_time, t);
                    for y in 0..t.height as usize {
                        let len = t.width as usize * 4;
                        bytes[y * 1024..y * 1024 + len]
                            .copy_from_slice(&palette_rgba[y * len..y * len + len]);
                    }
                }
                layers.push((Some(id), bytes));
            }
            for (i, (_, bytes)) in layers.iter().enumerate() {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.textures,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    bytes,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(1024),
                        rows_per_image: Some(256),
                    },
                    wgpu::Extent3d {
                        width: 256,
                        height: 256,
                        depth_or_array_layers: 1,
                    },
                );
            }
            self.mesh.upload(device, queue, &mesh);
            self.edges.upload(device, queue, &edges);
            let mut shadows = Vec::new();
            for blob in crate::shadows::blobs(scene) {
                for i in 1..=8 {
                    vertex(&mut shadows, blob.points[0], blob.color);
                    vertex(&mut shadows, blob.points[i], [0; 3]);
                    vertex(
                        &mut shadows,
                        blob.points[if i == 8 { 1 } else { i + 1 }],
                        [0; 3],
                    );
                }
            }
            self.shadows.upload(device, queue, &shadows);
            self.cached_scene = Some(scene.clone());
            self.cached_skeletal_poses = skeletal_poses;
            self.cached_selected = selected;
            self.cached_wire = input.wire;
            self.cached_phase = preview_time;
            self.cached_mesh_revision = input.mesh.revision;
        }
        let (s, c) = view.yaw.sin_cos();
        let (sp, cp) = view.pitch.sin_cos();
        let uniform = [
            s,
            c,
            sp,
            cp,
            view.center[0],
            view.center[1],
            view.center[2],
            view.zoom,
            view.distance,
            0.,
            0.,
            0.,
        ]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect::<Vec<_>>();
        queue.write_buffer(&self.camera, 0, &uniform);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Scene GPU pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(input.background.map_or(wgpu::Color::BLACK, |c| {
                        wgpu::Color {
                            r: c[0] as f64,
                            g: c[1] as f64,
                            b: c[2] as f64,
                            a: 1.,
                        }
                    })),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_bind_group(0, &self.bind_group, &[]);
        if input.background.is_none() {
            pass.set_pipeline(&self.background_pipeline);
            pass.draw(0..3, 0..1);
        }
        if !input.wire && self.mesh.count > 0 {
            pass.set_vertex_buffer(0, self.mesh.buffer.slice(..));
            for &(start, count, mode) in &self.draw_ranges {
                pass.set_pipeline(&self.mesh_pipelines[mode]);
                pass.draw(start..start + count, 0..1);
            }
        }
        if !input.wire && self.shadows.count > 0 {
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_vertex_buffer(0, self.shadows.buffer.slice(..));
            pass.draw(0..self.shadows.count, 0..1);
        }
        pass.set_pipeline(&self.line_pipeline);
        if input.grid {
            pass.set_vertex_buffer(0, self.grid.buffer.slice(..));
            pass.draw(0..self.grid.count, 0..1);
        }
        if self.edges.count > 0 {
            pass.set_vertex_buffer(0, self.edges.buffer.slice(..));
            pass.draw(0..self.edges.count, 0..1);
        }
    }
}
fn vertex(out: &mut Vec<u8>, p: [f32; 3], color: [u8; 3]) {
    for v in p
        .into_iter()
        .chain(color.map(|c| c as f32 / 255.))
        .chain([0., 0., 0., 0.])
    {
        out.extend_from_slice(&v.to_le_bytes());
    }
}
fn line(out: &mut Vec<u8>, a: [f32; 3], b: [f32; 3], color: [u8; 3]) {
    vertex(out, a, color);
    vertex(out, b, color);
}
type GeometryBuffers = (Vec<u8>, Vec<u8>, Vec<(u32, u32, usize)>);
#[cfg(test)]
fn geometry(
    scene: &Scene,
    selected: Option<usize>,
    phase: f32,
    state: &crate::mesh_editor::State,
    view: &crate::viewport::View,
) -> GeometryBuffers {
    geometry_with_effects(scene, selected, phase, state, view, &[], false)
}
fn geometry_with_effects(
    scene: &Scene,
    selected: Option<usize>,
    phase: f32,
    state: &crate::mesh_editor::State,
    view: &crate::viewport::View,
    effects: &[crate::particle_effect_preview::Quad],
    wire: bool,
) -> GeometryBuffers {
    let (yaw, pitch, view_center) = (view.yaw, view.pitch, view.center);
    let mut triangles = Vec::new();
    struct Draw {
        points: [[f32; 3]; 4],
        colors: [[u8; 3]; 4],
        uv: [[f32; 2]; 4],
        material: crate::scene::Material,
        cull: bool,
    }
    let mut draws = Vec::<Draw>::new();
    let mut edges = Vec::new();
    let lighting = crate::lighting::Lighting::unshadowed(scene);
    for (index, e) in scene
        .actors
        .iter()
        .enumerate()
        .filter(|(i, e)| e.kind == "Mesh" && scene.is_active(*i))
    {
        let world = scene.world_matrix(index);
        let offline = crate::lighting::baked(e);
        let center = world.point([0.; 3]);
        let face_colors: [[u8; 3]; 6] = std::array::from_fn(|f| {
            lighting.sample(
                center,
                crate::lighting::normal(world, f),
                index,
                false,
                false,
            )
        });
        let quads = crate::lighting::quads(e);
        let cached = scene
            .bake
            .as_ref()
            .and_then(|b| b.preview_colors(e.id, quads.len() * 4));
        for (qi, q) in quads.iter().enumerate() {
            let p = q.points.map(|v| world.point(v));
            let n = crate::lighting::transform_normal(world, q.normal);
            let colors: [[u8; 3]; 4] = std::array::from_fn(|v| {
                let rgb = if q.material.unlit {
                    [255; 3]
                } else if offline && let Some(colors) = cached {
                    colors[qi * 4 + v]
                } else if offline {
                    lighting.sample(p[v], n, index, true, false)
                } else {
                    if e.editable_mesh.is_some() || e.skeletal_mesh.is_some() {
                        lighting.sample(center, n, index, false, false)
                    } else {
                        face_colors[q.face]
                    }
                };
                let color = crate::lighting::modulate(rgb, q.material.color);
                if state.open
                    && state.target == Some(index)
                    && q.id.is_some_and(|id| state.selected.contains(&id))
                {
                    std::array::from_fn(|c| ((color[c] as u16 + [255, 180, 55][c]) / 2) as u8)
                } else {
                    color
                }
            });
            draws.push(Draw {
                points: p,
                colors,
                uv: q.uv,
                material: q.material.clone(),
                cull: e.editable_mesh.is_some() || e.skeletal_mesh.is_some(),
            });
        }
        for q in quads.iter().filter(|_| {
            wire || selected == Some(index) || (state.open && state.target == Some(index))
        }) {
            let p = q.points.map(|v| world.point(v));
            let color = if state.open
                && state.target == Some(index)
                && q.id.is_some_and(|id| state.selected.contains(&id))
            {
                [255, 225, 90]
            } else if selected == Some(index) {
                [238, 165, 74]
            } else {
                [79, 94, 111]
            };
            for i in 0..4 {
                line(&mut edges, p[i], p[(i + 1) % 4], color);
            }
        }
        if state.open
            && state.mode == crate::mesh_editor::Mode::Vertices
            && state.target == Some(index)
            && let Some(doc) = e.editable_mesh.as_ref().and_then(|m| m.document.as_ref())
        {
            let used = doc
                .faces
                .iter()
                .flat_map(|f| f.vertices)
                .collect::<std::collections::BTreeSet<_>>();
            for i in used {
                let p = world.point(doc.vertices[i as usize]);
                let color = if state.vertices.contains(&i) {
                    [255, 225, 90]
                } else {
                    [180, 220, 255]
                };
                for axis in 0..3 {
                    let mut a = p;
                    let mut b = p;
                    a[axis] -= 0.035;
                    b[axis] += 0.035;
                    line(&mut edges, a, b, color);
                }
            }
        }
        if state.open
            && state.mode == crate::mesh_editor::Mode::Edges
            && state.target == Some(index)
            && let Some(doc) = e.editable_mesh.as_ref().and_then(|m| m.document.as_ref())
        {
            for edge in doc
                .faces
                .iter()
                .flat_map(crate::mesh_ops::edges)
                .collect::<std::collections::BTreeSet<_>>()
            {
                line(
                    &mut edges,
                    world.point(doc.vertices[edge[0] as usize]),
                    world.point(doc.vertices[edge[1] as usize]),
                    if state.edges.contains(&edge) {
                        [255, 225, 90]
                    } else {
                        [180, 220, 255]
                    },
                );
            }
        }
    }
    // Light helpers are editor-only, using the entity's inherited transform.
    for (i, e) in scene.actors.iter().enumerate() {
        if let Some(l) = &e.light {
            let m = scene.world_matrix(i);
            let p = m.point([0.; 3]);
            let color = if selected == Some(i) {
                [255, 190, 70]
            } else {
                [225, 210, 120]
            };
            for axis in 0..3 {
                let mut a = p;
                let mut b = p;
                a[axis] -= 0.15;
                b[axis] += 0.15;
                line(&mut edges, a, b, color);
            }
            if l.kind == crate::lighting::LightType::Directional {
                line(&mut edges, p, m.point([0., 0., 0.8]), color);
            } else if selected == Some(i) {
                for plane in 0..3 {
                    for step in 0..48 {
                        let point = |s: usize| {
                            let angle = s as f32 * std::f32::consts::TAU / 48.;
                            let mut v = p;
                            v[(plane + 1) % 3] += angle.cos() * l.range;
                            v[(plane + 2) % 3] += angle.sin() * l.range;
                            v
                        };
                        line(&mut edges, point(step), point(step + 1), color);
                    }
                }
            }
        }
    }
    if let Some(i) = selected
        && let Some(e) = scene.actors.get(i)
        && let Some(c) = e.collider.as_ref().filter(|c| c.enabled)
    {
        for (a, b) in crate::collision::world_bounds(c, scene.world_matrix(i)).edges() {
            line(
                &mut edges,
                a,
                b,
                if c.trigger {
                    [240, 190, 60]
                } else {
                    [60, 230, 130]
                },
            );
        }
    }
    if let Some(i) = selected
        && let Some(e) = scene.actors.get(i)
        && e.kind == "Camera"
    {
        let world = scene.world_matrix(i);
        let origin = world.point([0.; 3]);
        let half = (e.camera_fov.clamp(25., 120.).to_radians() * 0.5).tan() * 3.;
        let corners = [
            [-half, half * 0.75, 3.],
            [half, half * 0.75, 3.],
            [half, -half * 0.75, 3.],
            [-half, -half * 0.75, 3.],
        ]
        .map(|p| world.point(p));
        for j in 0..4 {
            line(&mut edges, origin, corners[j], [80, 185, 245]);
            line(&mut edges, corners[j], corners[(j + 1) % 4], [80, 185, 245]);
        }
    }
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    let camera = [
        [cy, 0., -sy],
        [sy * sp, cp, cy * sp],
        [sy * cp, -sp, cy * cp],
    ];
    for q in crate::sprites::preview(scene, camera, phase)
        .into_iter()
        .chain(crate::particles::preview(scene, camera, phase))
        .chain(effects.iter().map(|q| crate::sprites::PreviewQuad {
            owner: usize::MAX,
            points: crate::sprites::corners(&q.sprite, q.world, camera),
            sprite: q.sprite.clone(),
        }))
    {
        let sprite = &q.sprite;
        let size = sprite
            .texture
            .and_then(|id| scene.textures.get(&id))
            .map_or((256, 256), |t| (t.width, t.height));
        let n = crate::mesh::face_normal(q.points);
        let center = std::array::from_fn(|c| q.points.iter().map(|p| p[c]).sum::<f32>() / 4.);
        let light = if sprite.unlit || q.owner == usize::MAX {
            [255; 3]
        } else {
            lighting.sample(center, n, q.owner, false, false)
        };
        draws.push(Draw {
            points: q.points,
            cull: false,
            colors: [crate::lighting::modulate(light, sprite.color); 4],
            uv: crate::sprites::uv(sprite, size.0, size.1),
            material: crate::scene::Material {
                uv_scroll: [0.; 2],
                color: sprite.color,
                unlit: sprite.unlit,
                texture: sprite.texture,
                blend: sprite.blend,
                depth_bias: sprite.depth_bias,
            },
        });
    }
    let depth = |d: &Draw| {
        let p: [f32; 3] = std::array::from_fn(|c| d.points.iter().map(|p| p[c]).sum::<f32>() / 4.);
        p[0] * sy * cp + p[2] * cy * cp - p[1] * sp + d.material.depth_bias as f32 * 0.25
    };
    draws.sort_by(|a, b| {
        let a_trans = a.material.blend != crate::texture::BlendMode::Cutout;
        let b_trans = b.material.blend != crate::texture::BlendMode::Cutout;
        a_trans
            .cmp(&b_trans)
            .then_with(|| depth(b).total_cmp(&depth(a)))
    });
    let ids = crate::texture::ids(scene);
    let mut ranges = Vec::<(u32, u32, usize)>::new();
    for d in draws {
        let mode = d.material.blend as usize;
        let pipeline = mode + if d.cull { 5 } else { 0 };
        let start = (triangles.len() / 40) as u32;
        let layer = d
            .material
            .texture
            .and_then(|id| ids.iter().position(|v| *v == id))
            .filter(|i| *i < 32)
            .map_or(0, |i| i + 1);
        let size = d
            .material
            .texture
            .and_then(|id| scene.textures.get(&id))
            .map_or((256, 256), |t| (t.width, t.height));
        let mut emitted = Vec::new();
        for triple in [[0, 1, 2], [0, 2, 3]] {
            let input = triple.map(|i| {
                let p: [f32; 3] = std::array::from_fn(|c| d.points[i][c] - view_center[c]);
                let depth = view.distance + p[0] * sy * cp + p[2] * cy * cp - p[1] * sp;
                crate::effects::Vertex {
                    point: d.points[i],
                    color: crate::effects::fog_color(d.colors[i], depth, &scene.fog),
                    uv: d.uv[i],
                }
            });
            emitted.extend(crate::effects::scroll_triangle(
                input,
                if layer == 0 {
                    [0.; 2]
                } else {
                    d.material.uv_scroll
                },
                phase,
            ));
        }
        let count = (emitted.len() * 3) as u32;
        if let Some(last) = ranges.last_mut().filter(|r| r.2 == pipeline) {
            last.1 += count;
        } else {
            ranges.push((start, count, pipeline));
        }
        for v in emitted.into_iter().flatten() {
            let uv = if layer == 0 {
                [0., 0.]
            } else {
                [
                    (v.uv[0].clamp(0., 1.) * size.0.saturating_sub(1) as f32 + 0.5) / 256.,
                    (v.uv[1].clamp(0., 1.) * size.1.saturating_sub(1) as f32 + 0.5) / 256.,
                ]
            };
            for f in v
                .point
                .into_iter()
                .chain(v.color.map(|c| c as f32 / 255.))
                .chain([uv[0], uv[1], layer as f32, mode as f32])
            {
                triangles.extend_from_slice(&f.to_le_bytes());
            }
        }
    }
    (triangles, edges, ranges)
}

/// Measures end-to-end offscreen submission + GPU completion; excludes UI and presentation.
pub fn profile(project: crate::workspace::Project) -> Result<(), Box<dyn std::error::Error>> {
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
    }))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Scene benchmark"),
        size: SIZE,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target = texture.create_view(&Default::default());
    let mut renderer = SceneGpu::new(&device, &queue);
    let mut editor = crate::editor::Editor::open(project)?;
    let mut samples = Vec::new();
    let mut cpu = Vec::new();
    for i in 0..130 {
        editor.view.yaw += 0.008;
        let start = std::time::Instant::now();
        let mut encoder = device.create_command_encoder(&Default::default());
        renderer.render(&device, &queue, &mut encoder, &target, &editor);
        let submit = start.elapsed().as_secs_f64() * 1000.;
        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::PollType::Wait)?;
        if i >= 10 {
            samples.push(start.elapsed().as_secs_f64() * 1000.);
            cpu.push(submit);
        }
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "GPU Scene ({}): CPU encode {:.3} ms mean; submit + GPU completion median {:.3} ms, p95 {:.3} ms; 120 orbit frames at 960x600",
        adapter.get_info().name,
        cpu.iter().sum::<f64>() / 120.,
        samples[60],
        samples[114]
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn skeletal_pose_cache_tracks_transient_animation_time() {
        use std::sync::Arc;

        let skeleton = uuid::Uuid::new_v4();
        let clip = uuid::Uuid::new_v4();
        let model = Arc::new(crate::skeletal::Model {
            mesh: crate::skeletal::Mesh {
                skeleton,
                vertices: vec![],
                triangles: vec![],
                materials: vec![],
                clips: vec![clip],
                animation_storage: Default::default(),
            },
            skeleton: crate::skeletal::Skeleton { bones: vec![] },
            clips: vec![(
                clip,
                crate::skeletal::Clip {
                    skeleton,
                    name: "Move".into(),
                    fps: 30,
                    frames: 2,
                    tracks: vec![],
                },
            )],
            materials: vec![],
        });
        let mut actor = crate::scene::Actor::cube("Character".into());
        let mut component = crate::skeletal::Component::new(uuid::Uuid::new_v4());
        component.clip = Some(clip);
        component.looping = false;
        component.model = Some(model);
        actor.skeletal_mesh = Some(component);
        let before = crate::scene::Scene {
            actors: vec![actor],
            ..Default::default()
        };
        let mut after = before.clone();
        after.actors[0].skeletal_mesh.as_mut().unwrap().time = 1. / 30.;

        assert_eq!(before, after, "document equality omits preview time");
        assert_ne!(
            super::skeletal_pose_key(&before),
            super::skeletal_pose_key(&after),
            "the GPU cache must still rebuild for a different sampled pose"
        );
    }

    #[test]
    #[ignore = "requires a graphics adapter"]
    fn scene_preview_culls_reversed_faces_and_keeps_shaded_surfaces_clean() {
        use super::*;
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Scene culling regression"),
            size: SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target = texture.create_view(&Default::default());
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene culling pixels"),
            size: 960 * 600 * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut renderer = SceneGpu::new(&device, &queue);
        let view = crate::viewport::View {
            yaw: 0.,
            pitch: 0.,
            center: [0.; 3],
            distance: 4.,
            ..Default::default()
        };
        for (reverse, active, expected_red) in [
            (false, true, true),
            (true, true, false),
            (false, false, false),
        ] {
            let mut doc = crate::mesh::Document::default();
            doc.materials[0].material.color = [1., 0., 0.];
            doc.materials[0].material.unlit = true;
            let mut points = [[-1., -1., 0.], [-1., 1., 0.], [1., 1., 0.], [1., -1., 0.]];
            if reverse {
                points.reverse();
            }
            doc.add_face(points, doc.groups[0].id, doc.materials[0].id);
            let mut actor = crate::scene::Actor::cube("Surface".into());
            actor.active = active;
            actor.editable_mesh = Some(crate::mesh::Component {
                document: Some(std::sync::Arc::new(doc)),
                ..crate::mesh::Component::new(uuid::Uuid::new_v4())
            });
            let scene = Scene {
                actors: vec![actor],
                ..Default::default()
            };
            let state = crate::mesh_editor::State::default();
            assert!(
                geometry(&scene, None, 0., &state, &view).1.is_empty(),
                "Shaded does not draw unsolicited wire edges"
            );
            if active {
                assert!(!geometry(&scene, Some(0), 0., &state, &view).1.is_empty());
            }
            let mut encoder = device.create_command_encoder(&Default::default());
            renderer.render_input(
                &device,
                &queue,
                &mut encoder,
                &target,
                RenderInput {
                    scene: &scene,
                    view: &view,
                    selected: None,
                    mesh: &state,
                    wire: false,
                    grid: false,
                    effect: None,
                    background: Some([0.; 3]),
                },
            );
            encoder.copy_texture_to_buffer(
                texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(960 * 4),
                        rows_per_image: Some(600),
                    },
                },
                SIZE,
            );
            queue.submit(Some(encoder.finish()));
            let (tx, rx) = std::sync::mpsc::channel();
            buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
            device.poll(wgpu::PollType::Wait).unwrap();
            rx.recv().unwrap().unwrap();
            let bytes = buffer.slice(..).get_mapped_range();
            let pixel = &bytes[(300 * 960 + 480) * 4..(300 * 960 + 480) * 4 + 3];
            assert_eq!(
                pixel[0] > 200 && pixel[1] < 20 && pixel[2] < 20,
                expected_red,
                "{reverse}/{active}: {pixel:?}"
            );
            drop(bytes);
            buffer.unmap();
        }
    }

    #[test]
    fn static_toggle_keeps_last_baked_shadows_in_both_renderers() {
        let mut scene = crate::lighting::tests::shadow_scene();
        scene.actors[1].lighting.static_geometry = false;
        scene.actors[1].lighting.receive = crate::lighting::Receive::Realtime;
        assert!(!crate::lighting::valid_bake(&scene));
        let mut expected = scene.clone();
        let cache = expected.bake.as_mut().unwrap();
        cache.colors[1].clear();
        cache.fingerprint = crate::lighting::fingerprint(&scene);
        assert!(crate::lighting::valid_bake(&expected));
        let view = crate::viewport::View::default();
        let state = crate::mesh_editor::State::default();
        assert_eq!(
            super::geometry(&scene, Some(1), 0., &state, &view),
            super::geometry(&expected, Some(1), 0., &state, &view),
            "GPU preview must retain the last saved colors even when outdated"
        );
        assert_eq!(
            crate::viewport::render(&scene, Some(1), &view, false, false, false).pixels,
            crate::viewport::render(&expected, Some(1), &view, false, false, false).pixels,
            "CPU preview must retain the same saved lighting"
        );
        let saved = scene.bake.clone();
        let before = super::geometry(&scene, Some(1), 0., &state, &view);
        scene.bake = Some(crate::lighting::bake(&scene).unwrap());
        assert_ne!(before, super::geometry(&scene, Some(1), 0., &state, &view));
        // An incompatible cache must fall back safely without tracing shadows.
        scene.bake = saved;
        scene.bake.as_mut().unwrap().colors[3].clear();
        super::geometry(&scene, Some(1), 0., &state, &view);
        scene.bake = None;
        super::geometry(&scene, Some(1), 0., &state, &view);
        assert!(scene.bake.is_none());
    }

    #[test]
    #[ignore = "requires a graphics adapter"]
    fn profile_gpu_scene() {
        super::profile(
            crate::workspace::Project::open(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample-game"),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
