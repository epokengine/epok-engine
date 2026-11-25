// Included inside namespace epok, after the public SplinePose value.
namespace spline_path {
struct Ring {
    int32_t distance,center[3],half_width;
    int16_t right[3],up[3],rotation[3]; // Unit vectors Q12, rotation in 1/64 degrees.
    int16_t bend; // Signed lateral curvature in Q12 inverse world units.
    uint8_t marker;
};
struct Path {
    const Ring* rings;
    uint16_t count;
    uint8_t channel;
    bool closed;
};
inline const Path* paths=nullptr;
inline unsigned count=0;
inline void bind(const Path* value,unsigned size){paths=value;count=size;}

inline int32_t blend(int32_t a,int32_t b,int32_t t){
    return a+int32_t(((int64_t(b)-a)*t)>>12);
}
inline SplinePose sample(uint32_t channel,Fixed distance,Fixed lateral,Fixed height){
    SplinePose pose{};
    const Path* path=nullptr;
    for(unsigned i=0;i<count;++i)if(paths[i].channel==channel){path=&paths[i];break;}
    if(!path)return pose;
    const auto* rings=path->rings;
    unsigned lo=0,hi=path->count-1;
    const int32_t length=rings[hi].distance;
    int32_t d=distance.raw();
    if(path->closed && (d<0 || d>=length)){d%=length;if(d<0)d+=length;}
    else {if(d<0)d=0;if(d>length)d=length;}
    while(hi-lo>1){const auto mid=(lo+hi)/2;if(rings[mid].distance<=d)lo=mid;else hi=mid;}
    const auto& a=rings[lo];const auto& b=rings[hi];
    const auto t=int32_t(int64_t(d-a.distance)*4096/(b.distance-a.distance));
    Fixed pos[3],rot[3];
    for(unsigned k=0;k<3;++k){
        const auto right=Fixed(blend(a.right[k],b.right[k],t),Fixed::RAW);
        const auto up=Fixed(blend(a.up[k],b.up[k],t),Fixed::RAW);
        pos[k]=Fixed(blend(a.center[k],b.center[k],t),Fixed::RAW)+right*lateral+up*height;
        int32_t delta=b.rotation[k]-a.rotation[k];
        if(delta>180*64)delta-=360*64;
        if(delta< -180*64)delta+=360*64;
        rot[k]=Fixed(blend(a.rotation[k]*64,(a.rotation[k]+delta)*64,t),Fixed::RAW);
    }
    pose.valid=true;pose.marker=a.marker;pose.bend=Fixed(a.bend,Fixed::RAW);pose.length=Fixed(length,Fixed::RAW);
    pose.half_width=Fixed(blend(a.half_width,b.half_width,t),Fixed::RAW);
    pose.position={pos[0],pos[1],pos[2]};pose.rotation={rot[0],rot[1],rot[2]};
    return pose;
}
}
