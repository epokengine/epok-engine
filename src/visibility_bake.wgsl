// Visibility bake: which faces show at least one pixel from a camera.
// Pass 1 writes the depth of opaque occluders. Pass 2 draws every face and
// marks it visible where it is not behind that depth, so faces seen through
// cut-out or blended faces, and coplanar decals, are kept as well.
struct View {
    rows: array<vec4<f32>, 3>, // camera-from-world, row major (xyz, translation)
    projection: vec4<f32>,     // horizontal and vertical cot(fov/2) scales, near, far
}
@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var occluder_depth: texture_depth_2d;
// Pixels (at the bake's resolution) each face covers, summed over the poses.
@group(0) @binding(2) var<storage, read_write> visible: array<atomic<u32>>;

struct Out {
    @invariant @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) face: u32,
}

@vertex
fn vertex(@location(0) world: vec3<f32>, @location(1) face: u32) -> Out {
    let camera = vec3<f32>(
        dot(view.rows[0].xyz, world) + view.rows[0].w,
        dot(view.rows[1].xyz, world) + view.rows[1].w,
        dot(view.rows[2].xyz, world) + view.rows[2].w);
    let near = view.projection.z;
    let far = view.projection.w;
    var out: Out;
    out.position = vec4<f32>(view.projection.x * camera.x, view.projection.y * camera.y,
        (camera.z - near) * far / (far - near), camera.z);
    out.face = face;
    return out;
}

@fragment
fn mark(input: Out) -> @location(0) vec4<f32> {
    let occluder = textureLoad(occluder_depth, vec2<i32>(input.position.xy), 0);
    if (input.face != 0xffffffffu && input.position.z <= occluder + 1e-6) {
        atomicAdd(&visible[input.face], 1u);
    }
    return vec4<f32>(0.0);
}
