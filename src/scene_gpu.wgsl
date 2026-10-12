struct Camera { values: vec4<f32>, center:vec4<f32>, orbit:vec4<f32>, sky:vec4<f32>, horizon:vec4<f32>, ground:vec4<f32>, extent:vec4<f32> };
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
    let x=(p.x*c-p.z*s)*camera.orbit.z;
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
    // Native CLUT: STP-clear texels overwrite; STP-set texels use material ABR.
    // Mode 8 is the opaque half of a mixed primitive; mode 0 ignores STP.
    if v.mode==8 && tex.a>0.75 {discard;}
    if v.mode>0 && v.mode<8 && tex.a<0.75 {discard;}
    let alpha=select(select(1.0,0.25,v.mode==4),0.5,v.mode==1);
    return vec4<f32>(linear(v.color*tex.rgb),alpha);
}
@vertex fn background_vertex(@builtin(vertex_index) i:u32) -> @builtin(position) vec4<f32> {
    let p=array<vec2<f32>,3>(vec2<f32>(-1,-1),vec2<f32>(3,-1),vec2<f32>(-1,3));
    return vec4<f32>(p[i],0.0,1.0);
}
@fragment fn background_fragment(@builtin(position) p:vec4<f32>) -> @location(0) vec4<f32> {
    if camera.sky.w>0.5 {
        // Sky gradient, as the runtime draws it: the elevation of the view ray
        // through this pixel picks sky (45 degrees up), horizon or ground (15
        // degrees down). Colours are written like the flat sky clear.
        let ndc=vec2<f32>(p.x/camera.extent.x*2.0-1.0,1.0-p.y/camera.extent.y*2.0);
        let zoom=camera.center.w; let sp=camera.values.z; let cp=camera.values.w;
        let ray=vec3<f32>(ndc.x*camera.orbit.y/(2.9*zoom),ndc.y/(2.9*zoom),1.0);
        let elevation=(ray.y*cp-sp)/length(ray);
        var color=camera.horizon.rgb;
        if elevation>=0.0 {color=mix(camera.horizon.rgb,camera.sky.rgb,min(elevation/0.7,1.0));}
        else {color=mix(camera.horizon.rgb,camera.ground.rgb,min(-elevation/0.25,1.0));}
        return vec4<f32>(color,1.0);
    }
    let shade=(68.0-p.y/600.0*6.0)/255.0;
    return vec4<f32>(linear(vec3<f32>(shade)),1.0);
}
