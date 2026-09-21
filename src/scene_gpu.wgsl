struct Camera { values: vec4<f32>, center:vec4<f32>, orbit:vec4<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var textures:texture_2d_array<f32>;
@group(0) @binding(2) var nearest_sampler:sampler;
struct Vertex { @location(0) position: vec3<f32>, @location(1) color: vec3<f32>, @location(2) tex:vec4<f32> };
struct Output { @builtin(position) position: vec4<f32>, @location(0) @interpolate(linear) color: vec3<f32>, @location(1) @interpolate(linear) uv:vec2<f32>, @location(2) @interpolate(flat) layer:i32, @location(3) @interpolate(flat) mode:i32 };
fn linear(c:vec3<f32>) -> vec3<f32> {
    return select(c/12.92, pow((c+0.055)/1.055,vec3<f32>(2.4)),c>vec3<f32>(0.04045));
}
@vertex fn vertex(v:Vertex) -> Output {
    let s=camera.values.x; let c=camera.values.y; let sp=camera.values.z; let cp=camera.values.w;
    let zoom=camera.center.w; let p=v.position-camera.center.xyz;
    let x=p.x*c-p.z*s;
    let z=p.x*s+p.z*c;
    let y=p.y*cp+z*sp;
    let depth=camera.orbit.x+z*cp-p.y*sp;
    let near=1.0; let far=20000.0;
    // Same camera and 960x600 reference coordinates as viewport::project / gizmos.
    let clip=vec4<f32>(x*(2.9/camera.orbit.y)*zoom,y*2.9*zoom,(depth-near)*far/(far-near),depth);
    return Output(clip,v.color,v.tex.xy,i32(v.tex.z),i32(v.tex.w));
}
@fragment fn fragment(v:Output) -> @location(0) vec4<f32> {
    let tex=textureSample(textures,nearest_sampler,v.uv,v.layer);
    if tex.a<0.5 {discard;}
    let alpha=select(select(1.0,0.25,v.mode==4),0.5,v.mode==1);
    return vec4<f32>(linear(v.color*tex.rgb),alpha);
}
@vertex fn background_vertex(@builtin(vertex_index) i:u32) -> @builtin(position) vec4<f32> {
    let p=array<vec2<f32>,3>(vec2<f32>(-1,-1),vec2<f32>(3,-1),vec2<f32>(-1,3));
    return vec4<f32>(p[i],0.0,1.0);
}
@fragment fn background_fragment(@builtin(position) p:vec4<f32>) -> @location(0) vec4<f32> {
    let shade=(68.0-p.y/600.0*6.0)/255.0;
    return vec4<f32>(linear(vec3<f32>(shade)),1.0);
}
