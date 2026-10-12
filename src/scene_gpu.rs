//! Scene is an editor view rendered on the host GPU. Game remains the emulator's output.
use crate::scene::Scene;
use wgpu::util::DeviceExt;

pub const SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 960,
    height: 600,
    depth_or_array_layers: 1,
};
pub const NATIVE_PLAY_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 640,
    height: 480,
    depth_or_array_layers: 1,
};
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
// The default WebGPU device supports 256 array layers. Reserve white layer 0
// in every bank; additional banks remove any scene-wide texture count limit.
const TEXTURES_PER_BANK: usize = 255;
type DrawRange = (u32, u32, usize, usize); // first vertex, count, pipeline, bank

fn mirrored_pipeline(mode: usize, mirror_x: bool) -> usize {
    mode + if mirror_x && mode >= 5 { 5 } else { 0 }
}

fn texture_slot(index: usize) -> (usize, u32) {
    (
        index / TEXTURES_PER_BANK,
        (index % TEXTURES_PER_BANK + 1) as u32,
    )
}
fn texture_bank_sizes(count: usize) -> Vec<u32> {
    (0..count.div_ceil(TEXTURES_PER_BANK).max(1))
        .map(|bank| {
            (count
                .saturating_sub(bank * TEXTURES_PER_BANK)
                .min(TEXTURES_PER_BANK)
                + 1) as u32
        })
        .collect()
}
struct TextureBank {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}
impl TextureBank {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        camera: &wgpu::Buffer,
        sampler: &wgpu::Sampler,
        layers: u32,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Scene texture bank"),
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene texture bank"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        let bank = Self {
            texture,
            bind_group,
        };
        bank.write(queue, 0, &vec![255; 256 * 256 * 4]);
        bank
    }
    fn write(&self, queue: &wgpu::Queue, layer: u32, bytes: &[u8]) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
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
}
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
    size: wgpu::Extent3d,
    preview_started: std::time::Instant,
    mesh_pipelines: Vec<wgpu::RenderPipeline>,
    draw_ranges: Vec<DrawRange>,
    texture_banks: Vec<TextureBank>,
    texture_layout: wgpu::BindGroupLayout,
    texture_sampler: wgpu::Sampler,
    cached_angles: [f32; 2],
    cached_view_dependent_geometry: bool,
    shadow_pipeline: wgpu::RenderPipeline,
    shadows: Vertices,
    line_pipeline: wgpu::RenderPipeline,
    background_pipeline: wgpu::RenderPipeline,
    camera: wgpu::Buffer,
    depth: wgpu::TextureView,
    mesh: Vertices,
    static_mesh: Vertices,
    edges: Vertices,
    grid: Vertices,
    cached_scene: Option<Scene>,
    cached_revision: Option<u64>,
    cached_bake_revision: u64,
    cached_textures: Option<Vec<(uuid::Uuid, usize, usize)>>,
    static_draw_ranges: Vec<DrawRange>,
    static_mask: Option<Vec<bool>>,
    cached_skeletal_poses: SkeletalPoseKey,
    cached_selected: Option<usize>,
    cached_wire: bool,
    cached_phase: f32,
    cached_fog_center: [f32; 3],
    cached_fog_distance: f32,
    cached_mesh_revision: u64,
    navigation_preview: crate::navigation::Preview,
    navigation_fill: Vertices,
    navigation_edges: Vertices,
    cached_terrain_revision: u64,
    brush_fill: Vertices,
    brush_edges: Vertices,
}
struct RenderInput<'a> {
    scene: &'a Scene,
    view: &'a crate::viewport::View,
    // None keeps the free editor observer's square-pixel projection. Game
    // rendering uses the runtime's normalized focal ratio, not target w/h.
    projection_aspect: Option<f32>,
    selected: Option<usize>,
    mesh: &'a crate::mesh_editor::State,
    terrain: &'a crate::terrain_editor::State,
    wire: bool,
    grid: bool,
    effect: Option<&'a [crate::particle_effect_preview::Quad]>,
    background: Option<[f32; 3]>,
    /// With `background`: draw the camera's sky/horizon/ground gradient.
    sky_gradient: Option<crate::scene::SkyGradient>,
    revision: Option<u64>,
    split_static: bool,
    baked_selection: Option<&'a crate::baked_preview::Selection>,
    baked_revision: u64,
}
/// Runtime-only skeletal state that changes the generated vertex buffer.
///
/// `Scene` equality deliberately serializes the document before comparing it,
/// so its transient `skeletal_mesh.time` field is omitted. Keep the sampled
/// pose separate from the document cache: animation can then refresh the
/// Scene View without becoming an authored scene edit.
type SkeletalPoseKey = Vec<Option<(usize, Option<uuid::Uuid>, Option<usize>)>>;

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
            Some((
                std::sync::Arc::as_ptr(model) as usize,
                component.clip,
                frame,
            ))
        })
        .collect()
}

// Scene's document equality serializes every actor to JSON. That is useful for
// authoring semantics, but costs a full allocation-heavy document conversion
// on every camera frame. The preview already holds a snapshot, so compare its
// render inputs directly. Keep the document equality used by undo unchanged.
fn same_render_scene(a: &Scene, b: &Scene) -> bool {
    a.version == b.version
        && a.name == b.name
        && a.fog == b.fog
        && a.hud_budget == b.hud_budget
        && a.environment == b.environment
        && a.bake == b.bake
        && a.scene_script == b.scene_script
        && a.navigation == b.navigation
        && a.textures.len() == b.textures.len()
        && a.textures.iter().all(|(id, data)| {
            b.textures
                .get(id)
                .is_some_and(|other| std::sync::Arc::ptr_eq(data, other))
        })
        && a.actors.len() == b.actors.len()
        && a.actors.iter().zip(&b.actors).all(|(left, right)| {
            left.id == right.id
                && left.class == right.class
                && left.name == right.name
                && left.active == right.active
                && left.logical_parent == right.logical_parent
                && left.attach == right.attach
                && left.components == right.components
                && left.properties == right.properties
                && left.overrides == right.overrides
                && left.data == right.data
        })
}

fn view_dependent_geometry(scene: &Scene, ranges: &[DrawRange]) -> bool {
    scene.fog.enabled
        || ranges.iter().any(|&(_, _, mode, _)| mode % 5 != 0)
        || scene.actors.iter().enumerate().any(|(index, actor)| {
            scene.is_active(index)
                && (actor.sprite.as_ref().is_some_and(|sprite| sprite.enabled)
                    || actor
                        .particle_emitter
                        .as_ref()
                        .is_some_and(|emitter| emitter.enabled))
        })
}

fn texture_key(scene: &Scene, seconds: f32) -> Vec<(uuid::Uuid, usize, usize)> {
    crate::texture::ids(scene)
        .into_iter()
        .map(|id| {
            let shift = scene.actors.iter().enumerate().find_map(|(index, actor)| {
                actor.palette_animator.as_ref().and_then(|animator| {
                    (scene.is_active(index) && animator.enabled && animator.texture == Some(id))
                        .then(|| crate::palette::offset(animator, seconds))
                })
            });
            (
                id,
                scene
                    .textures
                    .get(&id)
                    .map_or(0, |data| std::sync::Arc::as_ptr(data) as usize),
                shift.unwrap_or(0),
            )
        })
        .collect()
}

impl SceneGpu {
    pub fn navigation_status(&self) -> Option<Result<usize, String>> {
        self.navigation_preview.status()
    }
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
        Self::new_with_size(device, queue, SIZE)
    }
    pub fn new_with_size(device: &wgpu::Device, queue: &wgpu::Queue, size: wgpu::Extent3d) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Scene GPU shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scene_gpu.wgsl").into()),
        });
        let camera = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene camera"),
            contents: &[0; 112],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
                    // The background fragment reads the sky gradient colours.
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
        let texture_banks = vec![TextureBank::new(
            device, queue, &layout, &camera, &sampler, 1,
        )];
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Scene pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let make_pipeline = |topology, background, shadow: bool, blend: usize, cull: usize| {
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
                    front_face: if cull == 2 {
                        wgpu::FrontFace::Ccw
                    } else {
                        wgpu::FrontFace::Cw
                    },
                    cull_mode: (cull != 0).then_some(wgpu::Face::Back),
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
                size,
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
            size,
            preview_started: std::time::Instant::now(),
            mesh_pipelines: (0..15)
                .map(|b| {
                    make_pipeline(
                        wgpu::PrimitiveTopology::TriangleList,
                        false,
                        false,
                        b % 5,
                        b / 5,
                    )
                })
                .collect(),
            draw_ranges: vec![],
            texture_banks,
            texture_layout: layout,
            texture_sampler: sampler,
            cached_angles: [f32::NAN; 2],
            cached_view_dependent_geometry: false,
            line_pipeline: make_pipeline(wgpu::PrimitiveTopology::LineList, false, false, 0, 0),
            background_pipeline: make_pipeline(
                wgpu::PrimitiveTopology::TriangleList,
                true,
                false,
                0,
                0,
            ),
            camera,
            depth,
            mesh: Vertices::new(device),
            static_mesh: Vertices::new(device),
            shadows: Vertices::new(device),
            shadow_pipeline: make_pipeline(
                wgpu::PrimitiveTopology::TriangleList,
                false,
                true,
                0,
                0,
            ),
            edges: Vertices::new(device),
            grid,
            cached_scene: None,
            cached_revision: None,
            cached_bake_revision: 0,
            cached_textures: None,
            static_draw_ranges: vec![],
            static_mask: None,
            cached_skeletal_poses: vec![],
            cached_selected: None,
            cached_wire: false,
            cached_phase: 0.,
            cached_fog_center: [0.; 3],
            cached_fog_distance: 12.,
            cached_mesh_revision: 0,
            navigation_preview: Default::default(),
            navigation_fill: Vertices::new(device),
            navigation_edges: Vertices::new(device),
            cached_terrain_revision: 0,
            brush_fill: Vertices::new(device),
            brush_edges: Vertices::new(device),
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
        let scene = editor
            .timeline_editor
            .scene_preview
            .scene
            .as_ref()
            .filter(|_| editor.timeline_editor.open && !editor.playing)
            .unwrap_or(&editor.scene);
        self.render_input(
            device,
            queue,
            encoder,
            target,
            RenderInput {
                scene,
                view: &editor.view,
                projection_aspect: None,
                selected: editor.selected,
                mesh: &editor.mesh_editor,
                terrain: &editor.terrain_editor,
                wire: editor.wire,
                grid: editor.grid,
                effect: None,
                background: active_camera_sky(scene),
                sky_gradient: active_camera(scene).and_then(|actor| actor.camera_sky_gradient),
                revision: None,
                split_static: false,
                baked_selection: editor.baked_preview.filter(),
                baked_revision: editor.baked_preview.revision,
            },
        );
    }
    pub fn render_game(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: &crate::native_play::Frame,
    ) {
        let view = frame.view();
        self.render_input(
            device,
            queue,
            encoder,
            target,
            RenderInput {
                scene: &frame.scene,
                view: &view,
                projection_aspect: frame.camera
                    .filter(|&index| index < frame.scene.actors.len())
                    .map(|_| crate::native_play::GAME_PROJECTION_ASPECT),
                selected: None,
                mesh: &Default::default(),
                terrain: &Default::default(),
                wire: false,
                grid: false,
                effect: None,
                background: frame
                    .camera
                    .and_then(|index| frame.scene.actors.get(index))
                    .map(|actor| actor.camera_sky_color),
                sky_gradient: frame
                    .camera
                    .and_then(|index| frame.scene.actors.get(index))
                    .and_then(|actor| actor.camera_sky_gradient),
                revision: Some(frame.number),
                split_static: true,
                baked_selection: None,
                baked_revision: 0,
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
                projection_aspect: None,
                selected: None,
                mesh: &Default::default(),
                terrain: &Default::default(),
                wire: false,
                grid: false,
                effect: Some(&preview.quads),
                background: Some(preview.background),
                sky_gradient: None,
                revision: None,
                split_static: false,
                baked_selection: None,
                baked_revision: 0,
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
                projection_aspect: None,
                selected: None,
                mesh: &Default::default(),
                terrain: &Default::default(),
                wire: false,
                grid: false,
                effect: None,
                background: Some([0.10, 0.11, 0.12]),
                sky_gradient: None,
                revision: None,
                split_static: false,
                baked_selection: None,
                baked_revision: 0,
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
        let scene_changed = if let Some(revision) = input.revision {
            self.cached_revision != Some(revision)
        } else {
            self.cached_revision.is_some()
                || self
                    .cached_scene
                    .as_ref()
                    .is_none_or(|cached| !same_render_scene(cached, scene))
        };
        if input.effect.is_some()
            || self.cached_bake_revision != input.baked_revision
            || scene_changed
            || self.cached_skeletal_poses != skeletal_poses
            || (self.cached_view_dependent_geometry && self.cached_angles != [view.yaw, view.pitch])
            || self.cached_selected != selected
            || self.cached_wire != input.wire
            || self.cached_phase != preview_time
            || (scene.fog.enabled
                && (self.cached_fog_center != view.center
                    || self.cached_fog_distance != view.distance))
            || self.cached_mesh_revision != input.mesh.revision
            || self.cached_terrain_revision != input.terrain.revision
        {
            self.navigation_preview.update(scene, selected);
            let mut nav_fill = Vec::new();
            for quad in &self.navigation_preview.patches {
                for index in [0, 1, 2, 0, 2, 3] {
                    // The existing average-alpha pipeline gives a translucent
                    // unlit green overlay without modifying scene materials.
                    for f in quad[index]
                        .into_iter()
                        .chain([0.12, 0.85, 0.28])
                        .chain([0., 0., 0., 1.])
                    {
                        nav_fill.extend_from_slice(&f.to_le_bytes());
                    }
                }
            }
            let mut nav_lines = Vec::new();
            for &(a, b, color) in &self.navigation_preview.lines {
                line(&mut nav_lines, a, b, color);
            }
            self.navigation_fill.upload(device, queue, &nav_fill);
            self.navigation_edges.upload(device, queue, &nav_lines);
            let pass = if let Some(selection) = input.baked_selection {
                GeometryPass::Baked(selection)
            } else if input.split_static {
                if self.static_mask.is_none() {
                    let mask = static_mesh_mask(scene);
                    let (mesh, _, ranges) = geometry_with_effects(
                        scene,
                        selected,
                        preview_time,
                        input.mesh,
                        view,
                        &[],
                        false,
                        GeometryPass::Static(&mask),
                    );
                    self.static_mesh.upload(device, queue, &mesh);
                    self.static_draw_ranges = ranges;
                    self.static_mask = Some(mask);
                }
                GeometryPass::Dynamic(self.static_mask.as_deref().unwrap())
            } else {
                GeometryPass::All
            };
            let (mesh, edges, ranges) = geometry_with_effects(
                scene,
                selected,
                preview_time,
                input.mesh,
                view,
                input.effect.unwrap_or_default(),
                input.wire,
                pass,
            );
            self.cached_fog_center = view.center;
            self.cached_bake_revision = input.baked_revision;
            self.cached_fog_distance = view.distance;
            self.draw_ranges = ranges;
            self.cached_angles = [view.yaw, view.pitch];
            // Camera motion changes only the uniform for opaque static meshes.
            // Rebuild vertices when billboard sprites, particles, fog, or
            // translucent sorting actually depend on the view direction.
            self.cached_view_dependent_geometry = view_dependent_geometry(scene, &self.draw_ranges);
            let texture_key = texture_key(scene, preview_time);
            if self.cached_textures.as_ref() != Some(&texture_key) {
                let sizes = texture_bank_sizes(texture_key.len());
                let reallocated = self
                    .texture_banks
                    .iter()
                    .map(|bank| bank.texture.depth_or_array_layers())
                    .ne(sizes.iter().copied());
                if reallocated {
                    self.texture_banks = sizes
                        .into_iter()
                        .map(|layers| {
                            TextureBank::new(
                                device,
                                queue,
                                &self.texture_layout,
                                &self.camera,
                                &self.texture_sampler,
                                layers,
                            )
                        })
                        .collect();
                }
                for (index, &(id, _, _)) in texture_key.iter().enumerate() {
                    // A palette animation or reimport uploads only the affected
                    // layer, not every texture in a large authoring scene.
                    if !reallocated
                        && self.cached_textures.as_ref().and_then(|old| old.get(index))
                            == texture_key.get(index)
                    {
                        continue;
                    }
                    let mut bytes = vec![0; 256 * 256 * 4];
                    if let Some(t) = scene.textures.get(&id) {
                        let palette_rgba = crate::palette::preview_gpu_rgba(scene, id, preview_time, t);
                        for y in 0..t.height as usize {
                            let len = t.width as usize * 4;
                            bytes[y * 1024..y * 1024 + len]
                                .copy_from_slice(&palette_rgba[y * len..y * len + len]);
                        }
                    }
                    let (bank, layer) = texture_slot(index);
                    self.texture_banks[bank].write(queue, layer, &bytes);
                }
                self.cached_textures = Some(texture_key);
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
            self.cached_scene = input.revision.is_none().then(|| scene.clone());
            self.cached_revision = input.revision;
            self.cached_skeletal_poses = skeletal_poses;
            self.cached_selected = selected;
            self.cached_wire = input.wire;
            self.cached_phase = preview_time;
            self.cached_mesh_revision = input.mesh.revision;
            self.cached_terrain_revision = input.terrain.revision;
        }
        // The brush cursor moves without the scene changing, so it lives
        // outside the cache: a few hundred vertices rebuilt per render.
        let (brush_fill, brush_ring) = brush_overlay(scene, input.terrain);
        self.brush_fill.upload(device, queue, &brush_fill);
        self.brush_edges.upload(device, queue, &brush_ring);
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
            input.projection_aspect
                .unwrap_or(self.size.width as f32 / self.size.height as f32),
            if view.mirror_x { -1. } else { 1. },
            0.,
        ]
        .into_iter()
        .chain(sky_uniform(input.background, input.sky_gradient, self.size))
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
        pass.set_bind_group(0, &self.texture_banks[0].bind_group, &[]);
        if input.background.is_none() || input.sky_gradient.is_some() {
            pass.set_pipeline(&self.background_pipeline);
            pass.draw(0..3, 0..1);
        }
        if !input.wire {
            if input.split_static && self.static_mesh.count > 0 {
                pass.set_vertex_buffer(0, self.static_mesh.buffer.slice(..));
                for &(start, count, mode, bank) in &self.static_draw_ranges {
                    pass.set_bind_group(0, &self.texture_banks[bank].bind_group, &[]);
                    pass.set_pipeline(&self.mesh_pipelines[mirrored_pipeline(mode, view.mirror_x)]);
                    pass.draw(start..start + count, 0..1);
                }
            }
            if self.mesh.count > 0 {
                pass.set_vertex_buffer(0, self.mesh.buffer.slice(..));
                for &(start, count, mode, bank) in &self.draw_ranges {
                    pass.set_bind_group(0, &self.texture_banks[bank].bind_group, &[]);
                    pass.set_pipeline(&self.mesh_pipelines[mirrored_pipeline(mode, view.mirror_x)]);
                    pass.draw(start..start + count, 0..1);
                }
            }
        }
        if !input.wire && self.shadows.count > 0 {
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_vertex_buffer(0, self.shadows.buffer.slice(..));
            pass.draw(0..self.shadows.count, 0..1);
        }
        if self.navigation_fill.count > 0 {
            pass.set_pipeline(&self.mesh_pipelines[1]);
            pass.set_vertex_buffer(0, self.navigation_fill.buffer.slice(..));
            pass.draw(0..self.navigation_fill.count, 0..1);
        }
        if self.brush_fill.count > 0 {
            pass.set_pipeline(&self.mesh_pipelines[1]);
            pass.set_vertex_buffer(0, self.brush_fill.buffer.slice(..));
            pass.draw(0..self.brush_fill.count, 0..1);
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
        if self.navigation_edges.count > 0 {
            pass.set_vertex_buffer(0, self.navigation_edges.buffer.slice(..));
            pass.draw(0..self.navigation_edges.count, 0..1);
        }
        if self.brush_edges.count > 0 {
            pass.set_vertex_buffer(0, self.brush_edges.buffer.slice(..));
            pass.draw(0..self.brush_edges.count, 0..1);
        }
    }
}

/// Sky, horizon and ground colours (w = 1 when the gradient is on) and the
/// target size, for the background shader.
fn sky_uniform(
    background: Option<[f32; 3]>,
    gradient: Option<crate::scene::SkyGradient>,
    size: wgpu::Extent3d,
) -> [f32; 16] {
    let on = background.is_some() && gradient.is_some();
    let sky = background.unwrap_or([0.; 3]);
    let g = gradient.unwrap_or(crate::scene::SkyGradient {
        horizon: [0.; 3],
        ground: [0.; 3],
    });
    [
        sky[0],
        sky[1],
        sky[2],
        if on { 1. } else { 0. },
        g.horizon[0],
        g.horizon[1],
        g.horizon[2],
        0.,
        g.ground[0],
        g.ground[1],
        g.ground[2],
        0.,
        size.width as f32,
        size.height as f32,
        0.,
        0.,
    ]
}
fn active_camera(scene: &crate::scene::Scene) -> Option<&crate::scene::Actor> {
    scene
        .actors
        .iter()
        .find(|actor| actor.kind == "Camera" && actor.active)
}
fn active_camera_sky(scene: &crate::scene::Scene) -> Option<[f32; 3]> {
    active_camera(scene).map(|actor| actor.camera_sky_color)
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
/// Brush cursor projected onto the terrain: a disc tessellated in rings so
/// every vertex sits on the sampled surface, and a bright rim. Brightness
/// follows the brush falloff, so the disc shows where the stroke bites.
fn brush_overlay(scene: &Scene, state: &crate::terrain_editor::State) -> (Vec<u8>, Vec<u8>) {
    const SEGMENTS: usize = 48;
    const RINGS: usize = 6;
    /// Lift above the surface so the decal wins the depth test against the
    /// terrain it is drawn on, without floating visibly.
    const LIFT: f32 = 0.06;
    let mut fill = Vec::new();
    let mut ring = Vec::new();
    let (Some(index), Some(local), Some(doc)) =
        (state.target, state.cursor_local, state.doc.as_ref())
    else {
        return (fill, ring);
    };
    if !state.open || index >= scene.actors.len() {
        return (fill, ring);
    }
    let world = scene.world_matrix(index);
    let radius = state.brush.radius.max(0.05);
    let at = |r: f32, theta: f32| -> [f32; 3] {
        let x = local[0] + r * theta.cos();
        let z = local[2] + r * theta.sin();
        world.point([x, doc.sample(x, z) + LIFT, z])
    };
    let shade = |r: f32| -> [f32; 3] {
        let k = 0.35 + 0.65 * state.brush.falloff.weight(r / radius);
        [0.30 * k, 1.00 * k, 0.45 * k]
    };
    let push = |out: &mut Vec<u8>, p: [f32; 3], color: [f32; 3]| {
        // Mode 1 is the translucent blend the navigation overlay uses.
        for v in p.into_iter().chain(color).chain([0., 0., 0., 1.]) {
            out.extend_from_slice(&v.to_le_bytes());
        }
    };
    let angle = |s: usize| s as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
    for k in 0..RINGS {
        let r0 = radius * k as f32 / RINGS as f32;
        let r1 = radius * (k + 1) as f32 / RINGS as f32;
        let (c0, c1) = (shade(r0), shade(r1));
        for s in 0..SEGMENTS {
            let (t0, t1) = (angle(s), angle(s + 1));
            let (a, b, c, d) = (at(r0, t0), at(r1, t0), at(r1, t1), at(r0, t1));
            for (p, color) in [(a, c0), (b, c1), (c, c1), (a, c0), (c, c1), (d, c0)] {
                push(&mut fill, p, color);
            }
        }
    }
    for s in 0..SEGMENTS {
        line(
            &mut ring,
            at(radius, angle(s)),
            at(radius, angle(s + 1)),
            [120, 255, 150],
        );
    }
    (fill, ring)
}
type GeometryBuffers = (Vec<u8>, Vec<u8>, Vec<DrawRange>);
struct Draw {
    points: [[f32; 3]; 4],
    colors: [[u8; 3]; 4],
    uv: [[f32; 2]; 4],
    material: crate::scene::Material,
    cull: bool,
    gpu_colors: bool,
}
#[derive(Clone, Copy)]
enum GeometryPass<'a> {
    All,
    Static(&'a [bool]),
    Dynamic(&'a [bool]),
    Baked(&'a crate::baked_preview::Selection),
}
fn static_mesh_mask(scene: &Scene) -> Vec<bool> {
    if scene.fog.enabled {
        return vec![false; scene.actors.len()];
    }
    scene
        .actors
        .iter()
        .map(|actor| {
            actor.kind == "Mesh"
                && actor.lighting.static_geometry
                && actor.skeletal_mesh.is_none()
                && crate::lighting::quads(actor).iter().all(|quad| {
                    quad.material.blend == crate::texture::BlendMode::Cutout
                        && quad.material.uv_scroll == [0.; 2]
                        && (crate::lighting::baked(actor) || quad.material.unlit)
                })
        })
        .collect()
}
#[cfg(test)]
fn geometry(
    scene: &Scene,
    selected: Option<usize>,
    phase: f32,
    state: &crate::mesh_editor::State,
    view: &crate::viewport::View,
) -> GeometryBuffers {
    geometry_with_effects(
        scene,
        selected,
        phase,
        state,
        view,
        &[],
        false,
        GeometryPass::All,
    )
}
fn geometry_with_effects(
    scene: &Scene,
    selected: Option<usize>,
    phase: f32,
    state: &crate::mesh_editor::State,
    view: &crate::viewport::View,
    effects: &[crate::particle_effect_preview::Quad],
    wire: bool,
    pass: GeometryPass<'_>,
) -> GeometryBuffers {
    let (yaw, pitch, view_center) = (view.yaw, view.pitch, view.center);
    let mut draws = Vec::<Draw>::new();
    let mut edges = Vec::new();
    let lighting = crate::lighting::Lighting::unshadowed(scene);
    let mut shared_skeletal_quads = std::collections::HashMap::<
        (usize, Option<uuid::Uuid>, Option<usize>),
        std::sync::Arc<Vec<crate::lighting::Quad>>,
    >::new();
    for (index, e) in scene.actors.iter().enumerate().filter(|(i, e)| {
        e.kind == "Mesh"
            && scene.is_active(*i)
            && match pass {
                GeometryPass::All | GeometryPass::Baked(_) => true,
                GeometryPass::Static(mask) => mask.get(*i).copied().unwrap_or(false),
                GeometryPass::Dynamic(mask) => !mask.get(*i).copied().unwrap_or(false),
            }
    }) {
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
        let quads = if let Some(component) = &e.skeletal_mesh
            && let Some(model) = &component.model
        {
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
            shared_skeletal_quads
                .entry((
                    std::sync::Arc::as_ptr(model) as usize,
                    component.clip,
                    frame,
                ))
                .or_insert_with(|| std::sync::Arc::new(crate::lighting::quads(e)))
                .clone()
        } else {
            std::sync::Arc::new(crate::lighting::quads(e))
        };
        let cached = scene
            .bake
            .as_ref()
            .and_then(|b| b.preview_colors(e.id, quads.len() * 4));
        for (qi, q) in quads.iter().enumerate() {
            if let GeometryPass::Baked(selection) = pass {
                if selection
                    .get(&e.id)
                    .is_some_and(|faces| !faces.contains(&qi))
                {
                    continue;
                }
            }
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
                    if e.editable_mesh.is_some() || e.terrain.is_some() || e.skeletal_mesh.is_some()
                    {
                        lighting.sample(center, n, index, false, false)
                    } else {
                        // Only the built-in cube indexes the six precomputed
                        // face colours; an authored surface carries its own
                        // normal and has to be sampled with it.
                        face_colors[q.face]
                    }
                };
                let color = crate::lighting::modulate(rgb, q.material.color);
                let color = q.vertex_colors.map_or(color, |colors| crate::vertex_colors::tint(colors[v], color));
                if state.open
                    && state.target == Some(index)
                    && q.id.is_some_and(|id| state.selected.contains(&id))
                {
                    let highlight = if q.vertex_colors.is_some() && q.material.texture.is_some() { [128, 90, 28] } else { [255, 180, 55] };
                    std::array::from_fn(|c| ((color[c] as u16 + highlight[c]) / 2) as u8)
                } else {
                    color
                }
            });
            draws.push(Draw {
                gpu_colors: q.vertex_colors.is_some(),
                points: p,
                colors,
                uv: q.uv,
                material: q.material.clone(),
                cull: (e.editable_mesh.is_some()
                    || e.terrain.is_some()
                    || e.skeletal_mesh.is_some())
                    && !q.material.double_sided,
            });
        }
        for (qi, q) in quads.iter().enumerate().filter(|_| {
            wire || selected == Some(index) || (state.open && state.target == Some(index))
        }) {
            if let GeometryPass::Baked(selection) = pass {
                if selection
                    .get(&e.id)
                    .is_some_and(|faces| !faces.contains(&qi))
                {
                    continue;
                }
            }
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
    if !matches!(pass, GeometryPass::Static(_)) {
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
                gpu_colors: false,
                cull: false,
                colors: [crate::lighting::modulate(light, sprite.color); 4],
                uv: crate::sprites::uv(sprite, size.0, size.1),
                material: crate::scene::Material {
                    uv_scroll: [0.; 2],
                    color: sprite.color,
                    unlit: sprite.unlit,
                    double_sided: true,
                    texture: sprite.texture,
                    blend: sprite.blend,
                    depth_bias: sprite.depth_bias,
                },
            });
        }
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
    let (triangles, ranges) = encode_draws(
        &draws,
        scene,
        phase,
        view_center,
        view.distance,
        [sy, cy, sp, cp],
    );
    (triangles, edges, ranges)
}

fn encode_draws(
    draws: &[Draw],
    scene: &Scene,
    phase: f32,
    view_center: [f32; 3],
    view_distance: f32,
    [sy, cy, sp, cp]: [f32; 4],
) -> (Vec<u8>, Vec<DrawRange>) {
    let ids = crate::texture::ids(scene);
    let mut triangles = Vec::new();
    let mut ranges = Vec::<DrawRange>::new();
    let opaque_texels: std::collections::BTreeSet<_> = scene.textures.iter()
        .filter_map(|(&id, t)| t.has_opaque_texels().then_some(id)).collect();
    for draw in draws {
        let mode = draw.material.blend as usize;
        let split_stp = mode != 0 && draw.material.texture.is_some_and(|id| opaque_texels.contains(&id));
        let (bank, layer) = draw
            .material
            .texture
            .and_then(|id| ids.iter().position(|candidate| *candidate == id))
            .map_or((0, 0), texture_slot);
        let size = draw
            .material
            .texture
            .and_then(|id| scene.textures.get(&id))
            .map_or((256, 256), |texture| (texture.width, texture.height));
        let neutral = if draw.gpu_colors && layer != 0 { 128. } else { 255. };
        let mut emitted = Vec::new();
        for triple in [[0, 1, 2], [0, 2, 3]] {
            let input = triple.map(|index| {
                let point: [f32; 3] =
                    std::array::from_fn(|axis| draw.points[index][axis] - view_center[axis]);
                let depth = view_distance + point[0] * sy * cp + point[2] * cy * cp - point[1] * sp;
                crate::effects::Vertex {
                    point: draw.points[index],
                    color: crate::effects::fog_color_domain(draw.colors[index], depth, &scene.fog, neutral),
                    uv: draw.uv[index],
                }
            });
            emitted.extend(crate::effects::scroll_triangle(
                input,
                if layer == 0 {
                    [0.; 2]
                } else {
                    draw.material.uv_scroll
                },
                phase,
            ));
        }
        // Mixed native CLUTs need two mutually exclusive fragment masks. Keep
        // both passes adjacent: regrouping by texture would reorder blending.
        // Legacy PNGs emit exactly the same single pass/vertex count as before.
        for pass in [8, mode].into_iter().skip(usize::from(!split_stp)) {
            let pipeline = (if pass == 8 { 0 } else { mode }) + if draw.cull { 5 } else { 0 };
            let start = (triangles.len() / 40) as u32;
            let count = (emitted.len() * 3) as u32;
            if let Some(last) = ranges.last_mut()
                .filter(|range| range.2 == pipeline && range.3 == bank) {
                last.1 += count;
            } else { ranges.push((start, count, pipeline, bank)); }
        for vertex in emitted.iter().flatten() {
            let uv = if layer == 0 {
                [0., 0.]
            } else {
                [
                    (vertex.uv[0].clamp(0., 1.) * size.0.saturating_sub(1) as f32 + 0.5) / 256.,
                    (vertex.uv[1].clamp(0., 1.) * size.1.saturating_sub(1) as f32 + 0.5) / 256.,
                ]
            };
            for value in vertex
                .point
                .into_iter()
                .chain(vertex.color.map(|channel| channel as f32 / neutral))
                .chain([uv[0], uv[1], layer as f32, pass as f32])
            {
                triangles.extend_from_slice(&value.to_le_bytes());
            }
        }
        }
    }
    (triangles, ranges)
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
    fn projection_frame(size: [u16; 2]) -> crate::native_play::Frame {
        let mut camera = crate::scene::Actor::cube("Camera".into());
        camera.kind = "Camera".into();
        camera.position = [0.; 3];
        camera.rotation = [0.; 3];
        camera.camera_sky_color = [0.; 3];
        crate::native_play::Frame {
            scene: crate::scene::Scene {
                actors: vec![camera],
                display_size: size,
                ..Default::default()
            },
            number: 0,
            buttons: 0,
            actor_count: 1,
            camera: Some(0),
            hud_rgba: vec![],
            hud_size: size.map(u32::from),
            stats: [0; 5],
            audio_voices: 0,
            audio_pending: 0, track_sample: None,
        }
    }

    #[test]
    fn native_projection_scale_matches_runtime_across_ntsc_modes() {
        for (width, height) in crate::settings::Rendering::MODES {
            let mut frame = projection_frame([width, height]);
            for fov in [-10_f32, 25., 60., 90., 120., 160.] {
                frame.scene.actors[0].camera_fov = fov;
                let focal = 1. / (fov.clamp(25., 120.).to_radians() / 2.).tan();
                let view = frame.view();
                let focal_x = f32::from(width) / 2. * 2.9
                    / crate::native_play::GAME_PROJECTION_ASPECT * view.zoom;
                let focal_y = f32::from(height) / 2. * 2.9 * view.zoom;
                assert!((focal_x - f32::from(width / 2) * focal).abs() < 0.001);
                assert!((focal_y - f32::from(height * 2 / 3) * focal).abs() < 0.001);
                assert_eq!(view.center, [0., 0., 1.]);
                assert_eq!(view.distance, 1.);
            }
        }
    }

    #[test]
    #[ignore = "requires a graphics adapter; no window or desktop capture"]
    fn native_game_projection_matches_runtime_focals_in_every_video_mode() {
        use super::*;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: if cfg!(windows) { wgpu::Backends::DX12 } else { wgpu::Backends::PRIMARY },
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut checked = 0;
        for (width, height) in crate::settings::Rendering::MODES {
            let size = wgpu::Extent3d {
                width: u32::from(width), height: u32::from(height), depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Native game focal regression"), size, mip_level_count: 1,
                sample_count: 1, dimension: wgpu::TextureDimension::D2, format: FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let target = texture.create_view(&Default::default());
            let mut renderer = SceneGpu::new_with_size(&device, &queue, size);
            let mut frame = projection_frame([width, height]);
            let mut doc = crate::mesh::Document::default();
            doc.materials[0].material.color = [1., 0., 0.];
            doc.materials[0].material.unlit = true;
            doc.add_face(
                [[1., 0.5, 16.], [1., 1.5, 16.], [3., 1.5, 16.], [3., 0.5, 16.]],
                doc.groups[0].id, doc.materials[0].id,
            );
            let mut marker = crate::scene::Actor::cube("Projected marker".into());
            marker.position = [0.; 3];
            marker.editable_mesh = Some(crate::mesh::Component {
                document: Some(std::sync::Arc::new(doc)),
                ..crate::mesh::Component::new(uuid::Uuid::new_v4())
            });
            frame.scene.actors.push(marker);
            frame.actor_count = 2;
            for fov in [-10_f32, 25., 60., 90., 120., 160.] {
                for mirror in [false, true] {
                    frame.number += 1;
                    frame.scene.actors[0].camera_fov = fov;
                    frame.scene.actors[0].camera_mirror_x = mirror;
                    let mut encoder = device.create_command_encoder(&Default::default());
                    renderer.render_game(&device, &queue, &mut encoder, &target, &frame);
                    queue.submit(Some(encoder.finish()));
                    let png = crate::platform::capture_png(&device, &queue, &texture).unwrap();
                    let mut reader = png::Decoder::new(std::io::Cursor::new(png)).read_info().unwrap();
                    let mut pixels = vec![0; reader.output_buffer_size()];
                    let info = reader.next_frame(&mut pixels).unwrap();
                    assert_eq!([info.width, info.height], [u32::from(width), u32::from(height)]);
                    let mut bounds = [u32::MAX, u32::MAX, 0, 0];
                    let mut red_pixels = 0;
                    for (index, pixel) in pixels.chunks_exact(4).enumerate() {
                        if pixel[0] > 200 && pixel[1] < 20 && pixel[2] < 20 {
                            let x = index as u32 % u32::from(width);
                            let y = index as u32 / u32::from(width);
                            bounds[0] = bounds[0].min(x); bounds[1] = bounds[1].min(y);
                            bounds[2] = bounds[2].max(x + 1); bounds[3] = bounds[3].max(y + 1);
                            red_pixels += 1;
                        }
                    }
                    assert!(red_pixels > 0, "marker absent: {width}x{height} FOV {fov} mirror {mirror}");
                    // Independent reference: camera_project in both PSX and
                    // native C++ uses w/2 and h*2/3, not the editor's 2.9 scale.
                    let focal = 1. / (fov.clamp(25., 120.).to_radians() / 2.).tan();
                    let sx = |x| f32::from(width / 2)
                        + x * focal * f32::from(width / 2) / 16. * if mirror { -1. } else { 1. };
                    let sy = |y| f32::from(height / 2) - y * focal * f32::from(height * 2 / 3) / 16.;
                    let expected = [sx(1.).min(sx(3.)), sy(1.5), sx(1.).max(sx(3.)), sy(0.5)];
                    for (actual, expected) in bounds.into_iter().zip(expected) {
                        assert!((actual as f32 - expected).abs() <= 1.,
                            "{width}x{height} FOV {fov} mirror {mirror}: {bounds:?}, boundary {expected}");
                    }
                    checked += 1;
                }
            }
        }
        println!("Native game projection: {checked} GPU readbacks, ten video modes, six FOV inputs, both mirror states.");
    }

    fn texture_scene(count: usize) -> crate::scene::Scene {
        let mut scene = crate::scene::Scene::default();
        scene.actors.clear();
        for index in 0..count {
            let mut actor = crate::scene::Actor::cube(format!("Texture {index}"));
            actor.active = false;
            actor.material.texture = Some(uuid::Uuid::from_u128(index as u128 + 1));
            actor.material.unlit = true;
            actor.material.double_sided = true;
            actor.material.color = [1.; 3];
            scene.actors.push(actor);
        }
        scene
    }

    #[test]
    fn authored_vertex_colors_preserve_packet_gain_through_preview_scroll_and_fog() {
        use super::*;
        let mut scene = texture_scene(1);
        let mut draw = Draw {
            gpu_colors: true,
            points: [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]],
            colors: [[185, 128, 64]; 4],
            uv: [[0., 0.], [1., 0.], [1., 1.], [0., 1.]],
            material: scene.actors[0].material.clone(),
            cull: false,
        };
        let check = |draw: &Draw, scene: &Scene, expected: [f32; 3]| {
            let (bytes, _) = encode_draws(std::slice::from_ref(draw), scene, 0.5,
                [0.; 3], 8., [0., 1., 0., 1.]);
            assert!(!bytes.is_empty());
            for vertex in bytes.chunks_exact(40) {
                for c in 0..3 {
                    let value = f32::from_le_bytes(vertex[12+c*4..16+c*4].try_into().unwrap());
                    assert!((value - expected[c]).abs() < 1e-6, "channel {c}: {value}");
                }
            }
        };
        check(&draw, &scene, [185./128., 1., 0.5]);
        draw.material.uv_scroll = [0.5, -0.75];
        check(&draw, &scene, [185./128., 1., 0.5]);
        scene.fog = crate::effects::Fog { enabled: true, start: 0., end: 4., color: [1., 0.5, 0.] };
        check(&draw, &scene, [1., 0.5, 0.]);
        scene.fog.enabled = false;
        draw.material.texture = None;
        check(&draw, &scene, [185./255., 128./255., 64./255.]);
        draw.material = scene.actors[0].material.clone();
        draw.gpu_colors = false;
        check(&draw, &scene, [185./255., 128./255., 64./255.]);
    }

    #[test]
    fn texture_banks_cover_every_slot_and_preserve_draw_order() {
        use super::*;
        assert_eq!(texture_bank_sizes(0), [1]);
        assert_eq!(texture_bank_sizes(32), [33]);
        assert_eq!(texture_bank_sizes(255), [256]);
        assert_eq!(texture_bank_sizes(511), [256, 256, 2]);
        let scene = texture_scene(511);
        let indices = [32, 255, 254, 510, 32];
        let draws: Vec<_> = indices
            .iter()
            .map(|&i| Draw {
                gpu_colors: false,
                points: [[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]],
                colors: [[255; 3]; 4],
                uv: [[0.; 2]; 4],
                material: crate::scene::Material {
                    blend: crate::texture::BlendMode::Average,
                    ..scene.actors[i].material.clone()
                },
                cull: false,
            })
            .collect();
        let (vertices, ranges) = encode_draws(&draws, &scene, 0., [0.; 3], 8., [0., 1., 0., 1.]);
        assert_eq!(
            ranges,
            [
                (0, 6, 1, 0),
                (6, 6, 1, 1),
                (12, 6, 1, 0),
                (18, 6, 1, 2),
                (24, 6, 1, 0)
            ],
            "never regroup transparent draws by texture bank"
        );
        for (i, &index) in indices.iter().enumerate() {
            let offset = i * 6 * 40 + 32;
            assert_eq!(
                f32::from_le_bytes(vertices[offset..offset + 4].try_into().unwrap()),
                texture_slot(index).1 as f32
            );
        }
        let key = texture_key(&scene, 0.);
        assert_eq!(
            key.len(),
            511,
            "unresolved texture slots must retain their identity too"
        );
        let mut changed = scene.clone();
        changed.actors[510].material.texture = Some(uuid::Uuid::from_u128(800));
        assert_ne!(texture_key(&changed, 0.), key);
    }

    #[test]
    fn native_stp_passes_are_adjacent_and_legacy_png_is_single_pass() {
        use super::*;
        let mut scene = texture_scene(1);
        let id = scene.actors[0].material.texture.unwrap();
        let mut palette = [0;16]; palette[0]=0x001f; palette[1]=0x801f;
        let tim = crate::texture::tests::tim_fixture(4,&palette,1,1,&[0x10,0x01]);
        let data = crate::texture::decode(&tim).unwrap();
        assert_eq!(crate::palette::preview_gpu_rgba(&scene,id,0.,&data),
            [255,0,0,128,255,0,0,255,255,0,0,255,255,0,0,128]);
        assert!(crate::palette::preview_rgba(&scene,id,0.,&data).chunks_exact(4).all(|p| p[3]==255));
        scene.textures.insert(id,std::sync::Arc::new(data));
        let mut draw=Draw { points:[[-1.,-1.,0.],[1.,-1.,0.],[1.,1.,0.],[-1.,1.,0.]],
            gpu_colors:false,cull:false,colors:[[255;3];4],uv:[[0.;2];4],material:scene.actors[0].material.clone() };
        for cull in [false,true] { for blend in [crate::texture::BlendMode::Cutout,
            crate::texture::BlendMode::Average,crate::texture::BlendMode::Add,
            crate::texture::BlendMode::Subtract,crate::texture::BlendMode::AddQuarter] {
            draw.material.blend=blend; draw.cull=cull;
            let (vertices,ranges)=encode_draws(std::slice::from_ref(&draw),&scene,0.,[0.;3],8.,[0.,1.,0.,1.]);
            let c=if cull {5} else {0};
            if blend==crate::texture::BlendMode::Cutout { assert_eq!(ranges,[(0,6,c,0)]); }
            else {
                assert_eq!(ranges,[(0,6,c,0),(6,6,c+blend as usize,0)]);
                for v in vertices[..6*40].chunks_exact(40) {
                    assert_eq!(f32::from_le_bytes(v[36..40].try_into().unwrap()),8.);
                }
            }
        }}
        scene.textures.insert(id,std::sync::Arc::new(crate::texture::decode(&crate::texture::tests::png()).unwrap()));
        let (_,ranges)=encode_draws(&[draw],&scene,0.,[0.;3],8.,[0.,1.,0.,1.]);
        assert_eq!(ranges,[(0,6,9,0)]);
    }

    #[test]
    #[ignore = "requires a graphics adapter; reads back the sky gradient background"]
    fn gpu_sky_gradient_shows_sky_above_and_ground_below_the_horizon() {
        use super::*;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: if cfg!(windows) {
                wgpu::Backends::DX12
            } else {
                wgpu::Backends::PRIMARY
            },
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Sky gradient"),
            size: SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let mut scene = crate::scene::Scene::default();
        for actor in &mut scene.actors {
            if actor.kind != "Camera" {
                actor.active = false;
            }
        }
        let gradient = crate::scene::SkyGradient {
            horizon: [1., 1., 1.],
            ground: [0., 1., 0.],
        };
        let mut renderer = SceneGpu::new(&device, &queue);
        let mut row = |pitch: f32, y: usize| {
            let view = crate::viewport::View {
                yaw: 0.,
                pitch,
                distance: 8.,
                zoom: 0.85,
                ..Default::default()
            };
            let mut encoder = device.create_command_encoder(&Default::default());
            renderer.render_input(
                &device,
                &queue,
                &mut encoder,
                &target.create_view(&Default::default()),
                RenderInput {
                    scene: &scene,
                    view: &view,
                    projection_aspect: None,
                    selected: None,
                    mesh: &Default::default(),
                    terrain: &Default::default(),
                    wire: false,
                    grid: false,
                    effect: None,
                    background: Some([0., 0., 1.]),
                    sky_gradient: Some(gradient),
                    revision: None,
                    split_static: false,
                    baked_selection: None,
                    baked_revision: 0,
                },
            );
            queue.submit(Some(encoder.finish()));
            let png = crate::platform::capture_png(&device, &queue, &target).unwrap();
            let mut reader = png::Decoder::new(std::io::Cursor::new(png))
                .read_info()
                .unwrap();
            let mut pixels = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut pixels).unwrap();
            let at = (y * info.width as usize + info.width as usize / 2) * 4;
            [pixels[at], pixels[at + 1], pixels[at + 2]]
        };
        let height = SIZE.height as usize;
        // Level camera: the horizon is the middle row; sky at the top, ground at the bottom.
        let top = row(0., 2);
        let middle = row(0., height / 2);
        let bottom = row(0., height - 2);
        assert!(top[2] > top[0] && top[2] > top[1], "{top:?}");
        assert!(middle.iter().all(|&c| c > 200), "{middle:?}");
        assert!(
            bottom[1] > 200 && bottom[0] < 40 && bottom[2] < 40,
            "{bottom:?}"
        );
        // Looking down 60 degrees: the whole view is below the horizon.
        let down = row(1.05, 2);
        assert!(down[1] > 200 && down[0] < 40, "{down:?}");
    }

    #[test]
    #[ignore = "requires a graphics adapter; reads back native per-texel STP in all blend modes"]
    fn gpu_native_stp_keeps_opaque_texels_cutout_and_visible_black() {
        use super::*;
        let mut scene=texture_scene(1);
        let id=scene.actors[0].material.texture.unwrap();
        let mut palette=[0;16]; palette[..4].copy_from_slice(&[0x001f,0x801f,0,0x8000]);
        let tim=crate::texture::tests::tim_fixture(4,&palette,1,1,&[0x10,0x32]);
        scene.textures.insert(id,std::sync::Arc::new(crate::texture::decode(&tim).unwrap()));
        scene.actors[0].active=true;
        let instance=wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends:if cfg!(windows) {wgpu::Backends::DX12} else {wgpu::Backends::PRIMARY},..Default::default() });
        let adapter=pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device,queue)=pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let target=device.create_texture(&wgpu::TextureDescriptor {
            label:Some("Native STP regression"),size:SIZE,mip_level_count:1,sample_count:1,
            dimension:wgpu::TextureDimension::D2,format:FORMAT,
            usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[] });
        let view=crate::viewport::View {yaw:0.,pitch:0.,distance:8.,zoom:0.85,..Default::default()};
        let mut renderer=SceneGpu::new(&device,&queue);
        let srgb=|v:f32| { let v=v.clamp(0.,1.); ((if v<=0.0031308 {v*12.92} else {1.055*v.powf(1./2.4)-0.055})*255.).round() as u8 };
        for blend in [crate::texture::BlendMode::Cutout,crate::texture::BlendMode::Average,
            crate::texture::BlendMode::Add,crate::texture::BlendMode::Subtract,crate::texture::BlendMode::AddQuarter] {
            let mut doc=crate::mesh::Document::default();
            doc.materials[0].material=scene.actors[0].material.clone();
            doc.materials[0].material.blend=blend;
            doc.add_face([[-2.,-1.,0.],[2.,-1.,0.],[2.,1.,0.],[-2.,1.,0.]],doc.groups[0].id,doc.materials[0].id);
            doc.faces[0].uv=[[0.,0.],[1.,0.],[1.,1.],[0.,1.]];
            scene.actors[0].editable_mesh=Some(crate::mesh::Component {document:Some(std::sync::Arc::new(doc)),
                ..crate::mesh::Component::new(uuid::Uuid::new_v4())});
            let mut encoder=device.create_command_encoder(&Default::default());
            renderer.render_input(&device,&queue,&mut encoder,&target.create_view(&Default::default()),RenderInput {
                scene:&scene,view:&view,projection_aspect:None,selected:None,mesh:&Default::default(),terrain:&Default::default(),wire:false,
                grid:false,effect:None,background:Some([0.5;3]),sky_gradient:None,revision:None,split_static:false,baked_selection:None,baked_revision:0 });
            queue.submit(Some(encoder.finish()));
            let png=crate::platform::capture_png(&device,&queue,&target).unwrap();
            let mut reader=png::Decoder::new(std::io::Cursor::new(png)).read_info().unwrap();
            let mut pixels=vec![0;reader.output_buffer_size()]; let info=reader.next_frame(&mut pixels).unwrap();
            for (index,x) in [340,430,530,620].into_iter().enumerate() {
                let expected: [u8;3]=std::array::from_fn(|channel| {
                    // RenderInput background is a linear attachment clear;
                    // scene preview blending is linear, not PSX integer RGB5.
                    let f=if index<2 && channel==0 {1.} else {0.}; let b=0.5;
                    if index==2 {return srgb(b);}
                    srgb(if index==0 {f} else {match blend {
                        crate::texture::BlendMode::Cutout=>f,crate::texture::BlendMode::Average=>(b+f)*0.5,
                        crate::texture::BlendMode::Add=>b+f,crate::texture::BlendMode::Subtract=>b-f,
                        crate::texture::BlendMode::AddQuarter=>b+f*0.25 }})
                });
                let start=(300*960+x)*info.color_type.samples();
                for channel in 0..3 {assert!(pixels[start+channel].abs_diff(expected[channel])<=2,
                    "{blend:?} texel {index}, channel {channel}: actual {}, expected {}",pixels[start+channel],expected[channel]);}
            }
        }
    }

    #[test]
    #[ignore = "requires a graphics adapter; validates actual texture-bank sampling"]
    fn gpu_samples_textures_beyond_32_and_across_banks() {
        use super::*;
        let mut scene = texture_scene(256);
        for (column, (index, color)) in [
            (32, [255, 0, 0, 255]),
            (255, [0, 255, 0, 255]),
            (254, [0, 0, 255, 255]),
        ]
        .into_iter()
        .enumerate()
        {
            let actor = &mut scene.actors[index];
            actor.active = true;
            actor.position = [column as f32 * 2. - 2., 0., 0.];
            scene.textures.insert(
                actor.material.texture.unwrap(),
                std::sync::Arc::new(crate::texture::Data {
                    width: 1,
                    height: 1,
                    rgba: color.to_vec(),
                    words: vec![],
                    palette: vec![],
                }),
            );
        }
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: if cfg!(windows) {
                wgpu::Backends::DX12
            } else {
                wgpu::Backends::PRIMARY
            },
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture-bank regression"),
            size: SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = crate::viewport::View {
            yaw: 0.,
            pitch: 0.,
            distance: 8.,
            zoom: 0.85,
            ..Default::default()
        };
        let mut renderer = SceneGpu::new(&device, &queue);
        for pass in 0..4 {
            // Verify reuse and resource shrinkage after a scene switch too.
            if pass == 2 {
                let id = scene.actors[32].material.texture.unwrap();
                let mut data = (**scene.textures.get(&id).unwrap()).clone();
                data.rgba = vec![255, 0, 255, 255];
                scene.textures.insert(id, std::sync::Arc::new(data));
            }
            if pass == 3 {
                scene.actors.truncate(33);
            }
            let mut encoder = device.create_command_encoder(&Default::default());
            renderer.render_input(
                &device,
                &queue,
                &mut encoder,
                &target.create_view(&Default::default()),
                RenderInput {
                    scene: &scene,
                    view: &view,
                    projection_aspect: None,
                    selected: None,
                    mesh: &Default::default(),
                    terrain: &Default::default(),
                    wire: false,
                    grid: false,
                    effect: None,
                    background: Some([0.; 3]),
                    sky_gradient: None,
                    revision: None,
                    split_static: false,
                    baked_selection: None,
                    baked_revision: 0,
                },
            );
            queue.submit(Some(encoder.finish()));
            let png = crate::platform::capture_png(&device, &queue, &target).unwrap();
            let mut reader = png::Decoder::new(std::io::Cursor::new(png))
                .read_info()
                .unwrap();
            let mut pixels = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut pixels).unwrap();
            for (x, expected) in [(295, [255, 0, 0]), (480, [0, 255, 0]), (665, [0, 0, 255])] {
                let offset = (300 * 960 + x) * info.color_type.samples();
                let expected = if pass == 3 && x != 295 {
                    [0; 3]
                } else if pass >= 2 && x == 295 {
                    [255, 0, 255]
                } else {
                    expected
                };
                assert_eq!(
                    &pixels[offset..offset + 3],
                    &expected,
                    "pass {pass}, pixel {x}"
                );
            }
            assert_eq!(renderer.texture_banks.len(), if pass == 3 { 1 } else { 2 });
        }
    }

    #[test]
    fn baked_preview_filters_exact_cooked_quads_without_mutating_authoring() {
        let mut scene = crate::scene::Scene::default();
        scene.actors.truncate(2);
        let actor = scene.actors[1].id;
        let state = crate::mesh_editor::State::default();
        let view = crate::viewport::View::default();
        let all = super::geometry(&scene, None, 0., &state, &view);
        let filter =
            crate::baked_preview::Selection::from([(actor, std::collections::BTreeSet::new())]);
        let hidden = super::geometry_with_effects(
            &scene,
            None,
            0.,
            &state,
            &view,
            &[],
            false,
            super::GeometryPass::Baked(&filter),
        );
        assert!(!all.0.is_empty());
        assert!(hidden.0.is_empty());
        let filter =
            crate::baked_preview::Selection::from([(actor, std::collections::BTreeSet::from([0]))]);
        let one = super::geometry_with_effects(
            &scene,
            None,
            0.,
            &state,
            &view,
            &[],
            false,
            super::GeometryPass::Baked(&filter),
        );
        assert!(!one.0.is_empty() && one.0.len() < all.0.len());
        assert_eq!(super::geometry(&scene, None, 0., &state, &view), all);
    }
    #[test]
    fn authoring_preview_ignores_runtime_visibility_track() {
        let mut doc = crate::mesh::Document::default();
        doc.add_face(
            [[-1., -1., 0.], [-1., 1., 0.], [1., 1., 0.], [1., -1., 0.]],
            doc.groups[0].id,
            doc.materials[0].id,
        );
        let face = doc.faces[0].id;
        let mut actor = crate::scene::Actor::cube("Tracked surface".into());
        actor.editable_mesh = Some(crate::mesh::Component {
            document: Some(std::sync::Arc::new(doc)),
            ..crate::mesh::Component::new(uuid::Uuid::new_v4())
        });
        let mut scene = crate::scene::Scene::default();
        scene.actors.push(actor);
        let view = crate::viewport::View::default();
        let state = crate::mesh_editor::State::default();
        let expected = super::geometry(&scene, None, 0., &state, &view);
        assert!(!expected.0.is_empty());
        scene.visibility_track = Some(crate::visibility_track::Track { path: Vec::new(),
            navigation: None,
            camera: scene.actors[0].id,
            sample_ms: 100,
            cull_backfaces: false, static_world: false,
            faces: vec![face],
            samples: vec![crate::visibility_track::Sample {
                position: [1000.; 3],
                rotation: [90.; 3],
                fov: 30.,
                visible: vec![],
                cut: true,
            }],
        });
        assert_eq!(super::geometry(&scene, None, 0., &state, &view), expected);
        assert!(super::same_render_scene(&scene, &{
            let mut untracked = scene.clone();
            untracked.visibility_track = None;
            untracked
        }));
    }

    /// Reproducible offscreen capture of the actual authoring renderer, not the
    /// runtime/PVS. Set EPOK_PREVIEW_SCENE, EPOK_PREVIEW_OUTPUT and optionally a
    /// JSON EPOK_PREVIEW_VIEW. Does not change the open editor or project files.
    #[test]
    #[ignore = "requires a graphics adapter and an explicit local scene/output"]
    fn capture_authoring_preview() {
        use super::*;
        let path = std::env::var("EPOK_PREVIEW_SCENE").expect("EPOK_PREVIEW_SCENE");
        let output = std::env::var("EPOK_PREVIEW_OUTPUT").expect("EPOK_PREVIEW_OUTPUT");
        let mut scene = Scene::load(std::path::Path::new(&path)).unwrap();
        if std::env::var_os("EPOK_PREVIEW_FLAT").is_some() {
            let flatten = |material: &mut crate::scene::Material| {
                material.texture = None;
                material.unlit = true;
                material.color = [0.8; 3];
            };
            for actor in &mut scene.actors {
                flatten(&mut actor.material);
                if let Some(mesh) = &mut actor.editable_mesh {
                    for material in mesh.materials.values_mut() {
                        flatten(material);
                    }
                    if let Some(doc) = &mut mesh.document {
                        for slot in &mut std::sync::Arc::make_mut(doc).materials {
                            flatten(&mut slot.material);
                        }
                    }
                }
            }
        }
        let mut bake = crate::baked_preview::State::default();
        if let Ok(sample) = std::env::var("EPOK_PREVIEW_BAKED_SAMPLE") {
            let root = std::path::Path::new(&path)
                .ancestors()
                .find(|p| p.join("assets").is_dir())
                .unwrap();
            bake.enabled = true;
            bake.sample = sample.parse().unwrap();
            bake.load(root, &scene);
            assert!(bake.error.is_none(), "{:?}", bake.error);
            assert!(bake.filter().is_some());
            println!(
                "Baked sample {}: {} selected faces, {} pages",
                bake.sample,
                bake.selection.values().map(|s| s.len()).sum::<usize>(),
                bake.bake.as_ref().unwrap().pages(bake.sample).len()
            );
        }
        let mut view = std::env::var("EPOK_PREVIEW_VIEW")
            .ok()
            .map(|v| serde_json::from_str::<crate::viewport::View>(&v).unwrap())
            .unwrap_or_else(|| crate::scene_view_state::framed(&scene, [960., 600.], 5.));
        if let Ok(sample) = std::env::var("EPOK_PREVIEW_CAMERA_SAMPLE") {
            let sample =
                &scene.visibility_track.as_ref().unwrap().samples[sample.parse::<usize>().unwrap()];
            view.yaw = sample.rotation[1].to_radians();
            view.pitch = sample.rotation[0].to_radians();
            view.zoom = 480. / (870. * (sample.fov.to_radians() * 0.5).tan());
            let forward = view.basis()[2];
            view.center = std::array::from_fn(|i| sample.position[i] + forward[i] * view.distance);
        }
        println!("Capture view: {}", serde_json::to_string(&view).unwrap());
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: if cfg!(windows) {
                wgpu::Backends::DX12
            } else {
                wgpu::Backends::PRIMARY
            },
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Authoring preview capture"),
            size: SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let mut renderer = SceneGpu::new(&device, &queue);
        // Diagnostic capture only; never changes the normal editor pipelines.
        if std::env::var_os("EPOK_PREVIEW_TWO_SIDED").is_some() {
            for i in 0..5 {
                renderer.mesh_pipelines[5 + i] = renderer.mesh_pipelines[i].clone();
            }
        }
        let mut encoder = device.create_command_encoder(&Default::default());
        renderer.render_input(
            &device,
            &queue,
            &mut encoder,
            &texture.create_view(&Default::default()),
            RenderInput {
                scene: &scene,
                view: &view,
                projection_aspect: None,
                selected: None,
                mesh: &crate::mesh_editor::State::default(),
                terrain: &crate::terrain_editor::State::default(),
                wire: false,
                grid: false,
                effect: None,
                background: None,
                sky_gradient: None,
                revision: None,
                split_static: false,
                baked_selection: bake.filter(),
                baked_revision: bake.revision,
            },
        );
        queue.submit(Some(encoder.finish()));
        println!(
            "Texture slots: {}, banks: {}, vertices: {}",
            renderer.cached_textures.as_ref().unwrap().len(),
            renderer.texture_banks.len(),
            renderer.mesh.count
        );
        std::fs::write(
            output,
            crate::platform::capture_png(&device, &queue, &texture).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn camera_rotation_reuses_opaque_geometry_but_not_billboards_or_transparency() {
        let mut scene = crate::scene::Scene::default();
        assert!(!super::view_dependent_geometry(
            &scene,
            &[(0, 6, 0, 0), (6, 6, 5, 0)]
        ));
        assert!(super::view_dependent_geometry(&scene, &[(0, 6, 1, 0)]));

        scene.actors[1].sprite = Some(Default::default());
        assert!(super::view_dependent_geometry(&scene, &[(0, 6, 0, 0)]));
        scene.actors[1].sprite = None;
        scene.fog.enabled = true;
        assert!(super::view_dependent_geometry(&scene, &[(0, 6, 0, 0)]));
    }

    #[test]
    fn render_cache_detects_authored_transform_and_material_changes() {
        let scene = crate::scene::Scene::default();
        let mut changed = scene.clone();
        assert!(super::same_render_scene(&scene, &changed));

        changed.actors[1].position[0] += 1.;
        assert!(!super::same_render_scene(&scene, &changed));
        changed = scene.clone();
        changed.actors[1].material.color[0] = 0.25;
        assert!(!super::same_render_scene(&scene, &changed));
        changed = scene.clone();
        changed.environment.ambient[0] = 0.25;
        assert!(!super::same_render_scene(&scene, &changed));
    }

    #[test]
    fn skeletal_pose_cache_tracks_transient_animation_time() {
        use std::sync::Arc;

        let skeleton = uuid::Uuid::new_v4();
        let clip = uuid::Uuid::new_v4();
        let alternate = uuid::Uuid::new_v4();
        let model = Arc::new(crate::skeletal::Model {
            mesh: crate::skeletal::Mesh {
                skeleton,
                vertices: vec![],
                triangles: vec![],
                materials: vec![],
                clips: vec![clip, alternate],
                animation_storage: Default::default(),
            },
            skeleton: crate::skeletal::Skeleton { bones: vec![] },
            clips: vec![
                (
                    clip,
                    crate::skeletal::Clip {
                        skeleton,
                        name: "Move".into(),
                        fps: 30,
                        frames: 2,
                        tracks: vec![],
                    },
                ),
                (
                    alternate,
                    crate::skeletal::Clip {
                        skeleton,
                        name: "Alternate".into(),
                        fps: 30,
                        frames: 2,
                        tracks: vec![],
                    },
                ),
            ],
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
        let mut switched = before.clone();
        switched.actors[0].skeletal_mesh.as_mut().unwrap().clip = Some(alternate);
        assert_ne!(
            super::skeletal_pose_key(&before),
            super::skeletal_pose_key(&switched),
            "switching clips at the same sampled frame must rebuild geometry"
        );
    }

    /// Diagnostic, not a gate: how long one preview rebuild takes for a scene
    /// like the sample arena with a 32x32 terrain. Run explicitly:
    /// `cargo test -- --ignored --nocapture profile_terrain_preview_rebuild`.
    #[test]
    #[ignore = "timing diagnostic; prints, asserts nothing"]
    fn profile_terrain_preview_rebuild() {
        use super::geometry;
        use crate::scene::Scene;
        let build = |cells: u16, baked: bool| {
            let mut doc = crate::terrain::Document::new([cells, cells], 128. / f32::from(cells));
            for j in 0..=cells {
                for i in 0..=cells {
                    doc.set_height(
                        i,
                        j,
                        ((f32::from(i) * 0.4).sin() + (f32::from(j) * 0.3).cos()) * 2.,
                    );
                }
            }
            let mut terrain = crate::scene::Actor::cube("Terrain".into());
            terrain.position = [0.; 3];
            let mut component = crate::terrain::Component::new(uuid::Uuid::new_v4());
            component.document = Some(std::sync::Arc::new(doc));
            terrain.terrain = Some(component);
            terrain.lighting.receive = if baked {
                crate::lighting::Receive::Baked
            } else {
                crate::lighting::Receive::Realtime
            };
            let mut actors = vec![terrain];
            for k in 0..12 {
                let mut b = crate::scene::Actor::cube(format!("Box {k}"));
                b.position = [k as f32 * 3. - 18., 1., (k % 3) as f32 * 4.];
                b.scale = [2., 2., 2.];
                actors.push(b);
            }
            let mut sun = crate::scene::Actor::cube("Sun".into());
            sun.kind = "Light".into();
            sun.light = Some(Default::default());
            actors.push(sun);
            Scene {
                actors,
                ..Default::default()
            }
        };
        let view = crate::viewport::View::default();
        let state = crate::mesh_editor::State::default();
        for (cells, baked) in [(16, true), (32, true), (32, false), (48, true)] {
            let scene = build(cells, baked);
            let t = std::time::Instant::now();
            let quads: usize = scene.actors.iter().map(crate::lighting::quad_count).sum();
            let quads_ms = t.elapsed().as_secs_f64() * 1000.;
            let t = std::time::Instant::now();
            let (mesh, _, _) = geometry(&scene, Some(0), 0., &state, &view);
            let rebuild_ms = t.elapsed().as_secs_f64() * 1000.;
            println!(
                "cells {cells:>2}x{cells:<2} baked={baked:<5} quads={quads:>5}  quads()={quads_ms:>7.2} ms  rebuild={rebuild_ms:>8.2} ms  ({} verts)",
                mesh.len() / 40
            );
        }
    }
    #[test]
    fn brush_decal_hugs_the_terrain_and_vanishes_when_idle() {
        use super::brush_overlay;
        use crate::scene::Scene;

        let mut doc = crate::terrain::Document::new([8, 8], 2.);
        // A plateau under the cursor, so a decal that ignored the surface
        // would sit visibly below it.
        for j in 2..=6 {
            for i in 2..=6 {
                doc.set_height(i, j, 5.);
            }
        }
        let mut actor = crate::scene::Actor::cube("Terrain".into());
        actor.position = [0., 1., 0.];
        let mut component = crate::terrain::Component::new(uuid::Uuid::new_v4());
        component.document = Some(std::sync::Arc::new(doc.clone()));
        actor.terrain = Some(component);
        let scene = Scene {
            actors: vec![actor],
            ..Default::default()
        };
        let mut state = crate::terrain_editor::State::default();
        // Closed tool, no cursor: nothing is drawn.
        assert_eq!(brush_overlay(&scene, &state), (vec![], vec![]));
        state.open = true;
        state.target = Some(0);
        state.doc = Some(doc);
        state.brush.radius = 2.;
        state.cursor_local = Some([0., 5., 0.]);
        let (fill, ring) = brush_overlay(&scene, &state);
        assert!(!fill.is_empty() && !ring.is_empty());
        assert_eq!(fill.len() % 40, 0);
        assert_eq!(ring.len() % 40, 0);
        // Every vertex sits just above the sampled surface in world space:
        // plateau at 5 plus the actor's 1 unit, plus a small lift.
        let heights: Vec<f32> = fill
            .chunks_exact(40)
            .map(|v| f32::from_le_bytes(v[4..8].try_into().unwrap()))
            .collect();
        assert!(
            heights.iter().all(|y| (*y - 6.).abs() < 0.2),
            "{:?}",
            &heights[..4]
        );
        // Mode 1 is the translucent blend; the centre is brighter than the rim.
        let mode = f32::from_le_bytes(fill[36..40].try_into().unwrap());
        assert_eq!(mode, 1.);
        let green: Vec<f32> = fill
            .chunks_exact(40)
            .map(|v| f32::from_le_bytes(v[16..20].try_into().unwrap()))
            .collect();
        let brightest = green.iter().cloned().fold(0., f32::max);
        let dimmest = green.iter().cloned().fold(1., f32::min);
        assert!(
            brightest > dimmest + 0.3,
            "falloff not visible: {brightest} vs {dimmest}"
        );
        // Closing the tool hides it again even with a stale cursor.
        state.open = false;
        assert_eq!(brush_overlay(&scene, &state), (vec![], vec![]));
    }
    #[test]
    fn terrain_reaches_the_preview_like_any_authored_surface() {
        use super::geometry;
        use crate::scene::Scene;

        // A missing terrain branch in a preview predicate does not fail a
        // build: the ground silently vanishes, or is shaded from a cube face.
        // Compare it against an editable mesh covering the same quads.
        let view = crate::viewport::View {
            distance: 64.,
            ..Default::default()
        };
        let state = crate::mesh_editor::State::default();
        let mut terrain_actor = crate::scene::Actor::cube("Terrain".into());
        terrain_actor.position = [0.; 3];
        let mut component = crate::terrain::Component::new(uuid::Uuid::new_v4());
        component.document = Some(std::sync::Arc::new(crate::terrain::Document::new(
            [4, 4],
            2.,
        )));
        terrain_actor.terrain = Some(component);
        let terrain_scene = Scene {
            actors: vec![terrain_actor],
            ..Default::default()
        };
        let (mesh, _, _) = geometry(&terrain_scene, None, 0., &state, &view);
        assert!(!mesh.is_empty(), "terrain produced no preview geometry");

        let mut doc = crate::mesh::Document::default();
        for j in 0..4_i32 {
            for i in 0..4_i32 {
                let (x, z) = (i as f32 * 2. - 4., j as f32 * 2. - 4.);
                doc.add_face(
                    [
                        [x, 0., z],
                        [x, 0., z + 2.],
                        [x + 2., 0., z + 2.],
                        [x + 2., 0., z],
                    ],
                    doc.groups[0].id,
                    doc.materials[0].id,
                );
            }
        }
        let mut mesh_actor = crate::scene::Actor::cube("Blockout".into());
        mesh_actor.position = [0.; 3];
        mesh_actor.editable_mesh = Some(crate::mesh::Component {
            document: Some(std::sync::Arc::new(doc)),
            ..crate::mesh::Component::new(uuid::Uuid::new_v4())
        });
        let mesh_scene = Scene {
            actors: vec![mesh_actor],
            ..Default::default()
        };
        let (reference, _, _) = geometry(&mesh_scene, None, 0., &state, &view);
        // Same surface, same quads: the same preview vertices, and the same
        // per-actor shading rather than a cube face colour.
        assert_eq!(mesh, reference);
    }
    #[test]
    #[ignore = "requires a graphics adapter"]
    fn scene_preview_culls_reversed_faces_and_keeps_shaded_surfaces_clean() {
        use super::*;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: if cfg!(windows) {
                wgpu::Backends::DX12
            } else {
                wgpu::Backends::PRIMARY
            },
            ..Default::default()
        });
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
        let mut previous_centroid: Option<f64> = None;
        for (reverse, active, double_sided, expected_red, mirror_x) in [
            (false, true, false, true),
            (true, true, false, false),
            (false, false, false, false),
            (true, true, true, true),
            (false, true, true, true),
            (true, false, true, false),
        ]
        .into_iter()
        .flat_map(|(r, a, d, e)| [false, true].map(|m| (r, a, d, e, m)))
        {
            let view = crate::viewport::View { mirror_x, ..view };
            let mut doc = crate::mesh::Document::default();
            doc.materials[0].material.color = [1., 0., 0.];
            doc.materials[0].material.unlit = true;
            doc.materials[0].material.double_sided = double_sided;
            let mut points = [[-1., -1., 0.], [-1., 1., 0.], [1., 1., 0.], [1., -1., 0.]];
            for point in &mut points {
                point[0] += 0.4;
            }
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
                    projection_aspect: None,
                    selected: None,
                    mesh: &state,
                    terrain: &crate::terrain_editor::State::default(),
                    wire: false,
                    grid: false,
                    effect: None,
                    background: Some([0.; 3]),
                    sky_gradient: None,
                    revision: None,
                    split_static: false,
                    baked_selection: None,
                    baked_revision: 0,
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
                "{reverse}/{active}/{double_sided}/{mirror_x}: {pixel:?}"
            );
            if expected_red {
                let xs: Vec<_> = bytes
                    .chunks_exact(4)
                    .enumerate()
                    .filter(|(_, p)| p[0] > 200 && p[1] < 20 && p[2] < 20)
                    .map(|(i, _)| (i % 960) as f64)
                    .collect();
                assert!(!xs.is_empty());
                let centroid = xs.iter().sum::<f64>() / xs.len() as f64;
                if mirror_x {
                    assert!((centroid + previous_centroid.unwrap() - 959_f64).abs() < 1.);
                } else {
                    previous_centroid = Some(centroid);
                }
            }
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
