#pragma once
#include "epok.hpp"
#include "font_types.hpp"
#include <type_traits>

namespace epok::hud_core {
#ifdef EPOK_HUD_IMAGE_REUSE_DIAGNOSTICS
// Console experiment only: normal builds reserve no diagnostic state.
struct ImageReuseDiagnostics {
    bool enabled=true,invalidate_next=false;
    uint32_t draws=0,cached_draws=0,patched_draws=0,patched_images=0,full_compiles=0,state_bytes=0;
};
inline ImageReuseDiagnostics image_reuse_diagnostics;
#endif
// Shared by the PSX packet writer and the native preview. No GPU, heap, or
// floating-point operations: layout, clipping, UVs, text and budgets agree.
struct Rect { Fixed x=0.0,y=0.0,w=0.0,h=0.0; };
struct Budget { unsigned layouts,rectangles,texts,glyphs,rotated; };
inline int pixel(Fixed v) { auto r=v.raw();return int((int64_t(r)+(r>=0?2048:-2048))/4096); }
inline int clamp(int v,int hi) { return v<0?0:v>hi?hi:v; }
struct ImageRectangle {int x0=0,y0=0,x1=0,y1=0;uint8_t color[3]={};};
// Patch scratch has no default initialization: only accepted commands are read.
struct ImagePatchRectangle {int x0,y0,x1,y1;uint8_t color[3];};
template<class Rectangle> inline bool clipped_rectangle(Rect r,int width,int height,Rectangle& result){
    result.x0=clamp(pixel(r.x),width);result.x1=clamp(pixel(r.x+r.w),width);
    result.y0=clamp(height-pixel(r.y+r.h),height);result.y1=clamp(height-pixel(r.y),height);
    return result.x1>result.x0&&result.y1>result.y0;
}
template<class Fill> void untextured_image(Rect r,const Image& image,Fill fill){
    if(image.borders[0]||image.borders[1]||image.borders[2]||image.borders[3]){
        if(r.w.raw()<=0||r.h.raw()<=0)return;
        Fixed left=Fixed(int32_t(image.borders[0])*4096,Fixed::RAW),top=Fixed(int32_t(image.borders[1])*4096,Fixed::RAW);
        Fixed right=Fixed(int32_t(image.borders[2])*4096,Fixed::RAW),bottom=Fixed(int32_t(image.borders[3])*4096,Fixed::RAW);
        if(left+right>r.w){left=left*r.w/(left+right);right=r.w-left;}
        if(top+bottom>r.h){top=top*r.h/(top+bottom);bottom=r.h-top;}
        fill(Rect{r.x,r.y+r.h-top,r.w,top},image.color);
        fill(Rect{r.x,r.y,r.w,bottom},image.color);
        const Fixed middle=r.h-top-bottom;
        fill(Rect{r.x,r.y+bottom,left,middle},image.color);
        fill(Rect{r.x+r.w-right,r.y+bottom,right,middle},image.color);
    }else fill(r,image.color);
}
// Shared emission/clipping for ordinary builds and uninitialized POD scratch.
// Each callback writes every field of the accepted entry before increasing
// count. A border emits at most four callbacks, so no fifth entry is touched.
template<class Output> void build_image_rectangles(Output& output,Rect rect,const Image& image,int width,int height){
    output.count=0;untextured_image(rect,image,[&](Rect part,const uint8_t* color){
        auto& next=output.values[output.count];
        if(!clipped_rectangle(part,width,height,next))return;
        for(unsigned c=0;c<3;++c)next.color[c]=color[c];++output.count;
    });
}
struct ImageRectangles {
    ImageRectangle values[4];unsigned count=0;
    void build(Rect rect,const Image& image,int width,int height){build_image_rectangles(*this,rect,image,width,height);}
};
struct ImagePatchRectangles {
    ImagePatchRectangle values[4];unsigned count;
    void build(Rect rect,const Image& image,int width,int height){build_image_rectangles(*this,rect,image,width,height);}
};
// psyqo::Trig's table, by the same integer recurrence, but held here as a
// constexpr array: the console, the native preview and the host harness all get
// identical cosines without EASTL storage or the out-of-line generator. 2048
// units make one turn; the entries are Q12 cosines of the first quarter turn.
struct TrigTable {
    int32_t quarter[512]={};
    constexpr TrigTable(){
        constexpr int64_t step=16777137; // 2^24 * cos(2*pi/2048)
        quarter[0]=16777216;quarter[1]=int32_t(step);
        for(int i=2;i<511;++i)quarter[i]=int32_t((step*quarter[i-1])>>23)-quarter[i-2];
        quarter[511]=0;
        for(auto& value:quarter)value>>=12;
    }
};
inline constexpr TrigTable trig_table{};
inline Fixed cosine(int units){
    const int t=units&2047;
    const int32_t r=t<512?trig_table.quarter[t]:t<1024?-trig_table.quarter[1023-t]:t<1536?-trig_table.quarter[t-1024]:trig_table.quarter[2047-t];
    return Fixed(r,Fixed::RAW);
}
inline Fixed sine(int units){return cosine(units-512);}
// Degrees to 1/2048 of a turn, rounded once per rotated node rather than per primitive.
inline int turn_units(Fixed degrees){const int32_t r=degrees.raw();return int((int64_t(r)+(r>=0?360:-360))/720);}
// Rotation-only 2-D affine in Q12: x'=c*x-s*y+tx, y'=s*x+c*y+ty, in HUD space.
struct Affine2 { Fixed c=1.0,s=0.0,tx=0.0,ty=0.0; };
inline bool identity(const Affine2& m){return m.c.raw()==4096&&m.s.raw()==0&&m.tx.raw()==0&&m.ty.raw()==0;}
// `b` applied first, then `a`: the parent's accumulated transform composed onto a child's.
inline Affine2 compose(const Affine2& a,const Affine2& b){
    return {a.c*b.c-a.s*b.s,a.s*b.c+a.c*b.s,a.c*b.tx-a.s*b.ty+a.tx,a.s*b.tx+a.c*b.ty+a.ty};
}
// T(pivot) * R(degrees) * T(-pivot).
inline Affine2 rotation_about(Fixed x,Fixed y,Fixed degrees){
    const int units=turn_units(degrees);const Fixed c=cosine(units),s=sine(units);
    return {c,s,x-(c*x-s*y),y-(s*x+c*y)};
}
// A node may retain packets only at the same offsets in arenas it uses, and only from the
// preceding rebuild of that parity. Skipped/inactive nodes lose ownership of
// their old slots as soon as another rebuild can borrow those slots.
template<size_t Capacity> class NodeCache {
public:
    struct Counts {uint32_t values[6]={};bool operator==(const Counts&)const=default;};
private:
    struct Entry {uint32_t epoch=0,key=0;Counts begin,delta;HudStats stats;};
    Entry entries[2][Capacity];uint32_t epochs[2]={};unsigned parity=0;
    static bool compatible(const Entry& e,const Counts& counts){
        if(e.begin==counts)return true;
        // Text checks its budget even when its geometry emits nothing.
        if(e.stats.dropped||e.begin.values[1]!=counts.values[1]||e.begin.values[2]!=counts.values[2])return false;
        // Rectangle/image packets and the two rotated packet kinds each share
        // a budget. Only groups this node actually wrote constrain its slots.
        if((e.delta.values[0]||e.delta.values[3])&&(e.begin.values[0]!=counts.values[0]||e.begin.values[3]!=counts.values[3]))return false;
        if((e.delta.values[4]||e.delta.values[5])&&(e.begin.values[4]!=counts.values[4]||e.begin.values[5]!=counts.values[5]))return false;
        return true;
    }
    static HudStats subtract(const HudStats& a,const HudStats& b){return {a.rectangles-b.rectangles,a.glyphs-b.glyphs,a.texts-b.texts,a.images-b.images,a.dropped-b.dropped,a.rotated-b.rotated};}
    static void add(HudStats& a,const HudStats& b){a.rectangles+=b.rectangles;a.glyphs+=b.glyphs;a.texts+=b.texts;a.images+=b.images;a.dropped+=b.dropped;a.rotated+=b.rotated;}
public:
    void invalidate(){for(auto& bank:entries)for(auto& e:bank)e.epoch=0;epochs[0]=epochs[1]=0;}
    void invalidate_node(unsigned p,size_t owner){if(p<2&&owner<Capacity)entries[p][owner].epoch=0;}
    void rebuild(unsigned p){parity=p;if(++epochs[p]==0){for(auto& e:entries[p])e.epoch=0;epochs[p]=1;}}
    bool begin(size_t owner,uint32_t key,Counts& counts,HudStats& stats){
        auto& e=entries[parity][owner];const auto epoch=epochs[parity];
        if(e.epoch&&e.epoch==epoch-1&&e.key==key&&compatible(e,counts)){
            for(unsigned i=0;i<6;++i)counts.values[i]+=e.delta.values[i];add(stats,e.stats);e.epoch=epoch;return true;
        }
        e.epoch=epoch;e.key=key;e.begin=counts;e.stats=stats;return false;
    }
    void end(size_t owner,const Counts& counts,const HudStats& stats){
        auto& e=entries[parity][owner];for(unsigned i=0;i<6;++i)e.delta.values[i]=counts.values[i]-e.begin.values[i];e.stats=subtract(stats,e.stats);
    }
};
inline void apply(const Affine2& m,Fixed x,Fixed y,Fixed& out_x,Fixed& out_y){out_x=m.c*x-m.s*y+m.tx;out_y=m.s*x+m.c*y+m.ty;}
inline Rect resolve(Rect parent,const RectTransform& r) {
    Fixed output[4],pos[2]={parent.x,parent.y},size[2]={parent.w,parent.h};
    for(int i=0;i<2;++i){
        Fixed extent=size[i]*(r.anchor_max[i]-r.anchor_min[i])+r.size[i];
        if(extent.raw()<0)extent=0.0;
        output[i+2]=extent;
        output[i]=pos[i]+size[i]*(r.anchor_min[i]+(r.anchor_max[i]-r.anchor_min[i])*r.pivot[i])+r.position[i]-extent*r.pivot[i];
    }
    return {output[0],output[1],output[2],output[3]};
}
// Packet keys are shared with host validation. Nodes without Canvas/Rect do
// not participate in HUD layout or emission.
// This runs for every field of every node, every frame. It only has to notice a
// change, and a rotate and xor always does when one input differs.
inline uint32_t render_key_mix(uint32_t h,uint32_t value){return ((h<<5)|(h>>27))^value;}
inline uint32_t render_node_key(const ActorData& e,uintptr_t textures,uintptr_t fonts,bool ignore_image_appearance=false){
    if(!e.canvas.enabled&&!e.rect.enabled)return 0;
    uint32_t h=2166136261u;
    h=render_key_mix(h,uint32_t(e.alive)|uint32_t(e.active)<<1|uint32_t(e.canvas.enabled)<<2|uint32_t(e.rect.enabled)<<3|uint32_t(e.parent+1)<<8);
    if(!e.alive||!e.active)return h;
    h=render_key_mix(h,uint32_t(uint16_t(e.canvas.focused)));
    h=render_key_mix(h,e.canvas.screen_overlay);
    for(int k=0;k<2;++k){h=render_key_mix(h,uint32_t(e.rect.anchor_min[k].raw()));h=render_key_mix(h,uint32_t(e.rect.anchor_max[k].raw()));h=render_key_mix(h,uint32_t(e.rect.pivot[k].raw()));h=render_key_mix(h,uint32_t(e.rect.position[k].raw()));h=render_key_mix(h,uint32_t(e.rect.size[k].raw()));}
    h=render_key_mix(h,uint32_t(e.rect.rotation.raw()));
    h=render_key_mix(h,uint32_t(e.image.enabled));if(e.image.enabled){if(!ignore_image_appearance)h=render_key_mix(h,uint32_t(e.image.color[0])<<8|uint32_t(e.image.color[1])<<16|uint32_t(e.image.color[2])<<24);h=render_key_mix(h,uint32_t(e.image.texture));h=render_key_mix(h,uint32_t(e.image.tiling));
    for(int k=0;k<4;++k)h=render_key_mix(h,uint32_t(e.image.region[k])|(ignore_image_appearance?0:uint32_t(e.image.borders[k])<<16));
    }
    h=render_key_mix(h,uint32_t(e.progress.enabled));if(e.progress.enabled){h=render_key_mix(h,uint32_t(e.progress.color[0])<<8|uint32_t(e.progress.color[1])<<16|uint32_t(e.progress.color[2])<<24);
    h=render_key_mix(h,uint32_t(e.progress.value.raw()));h=render_key_mix(h,uint32_t(e.progress.background[0])|uint32_t(e.progress.background[1])<<8|uint32_t(e.progress.background[2])<<16);
    h=render_key_mix(h,uint32_t(e.progress.step.raw()));}
    h=render_key_mix(h,uint32_t(e.text.enabled));if(e.text.enabled){h=render_key_mix(h,uint32_t(e.text.wrap)<<1|uint32_t(e.text.color[0])<<8|uint32_t(e.text.color[1])<<16|uint32_t(e.text.color[2])<<24);
    h=render_key_mix(h,uint32_t(e.text.font));h=render_key_mix(h,uint32_t(e.text.align));
#if defined(EPOK_TEXT_REVEAL) || defined(EPOK_REFLECTION) || defined(EPOK_DIALOGUE)
    h=render_key_mix(h,e.text.visible_glyphs);
#endif
    if(e.text.enabled)for(const char* c=e.text.value;*c;++c)h=render_key_mix(h,uint8_t(*c));
    }
    h=render_key_mix(h,uint32_t(e.layout_element.enabled));if(e.layout_element.enabled){
    h=render_key_mix(h,uint32_t(e.layout_element.horizontal)|uint32_t(e.layout_element.vertical)<<8);
    for(int k=0;k<2;++k)h=render_key_mix(h,uint32_t(e.layout_element.minimum[k].raw()));
    h=render_key_mix(h,uint32_t(e.layout_element.stretch.raw()));}
    h=render_key_mix(h,uint32_t(e.layout_container.enabled));if(e.layout_container.enabled){
    h=render_key_mix(h,uint32_t(e.layout_container.kind)|uint32_t(e.layout_container.columns)<<8);
    for(int k=0;k<2;++k)h=render_key_mix(h,uint32_t(e.layout_container.spacing[k].raw()));
    for(int k=0;k<4;++k)h=render_key_mix(h,uint32_t(e.layout_container.padding[k].raw()));}
    h=render_key_mix(h,uint32_t(e.focusable.enabled));if(e.focusable.enabled){
    h=render_key_mix(h,uint32_t(e.focusable.highlight[0])<<16|uint32_t(e.focusable.highlight[1])<<24);
    h=render_key_mix(h,uint32_t(e.focusable.highlight[2])|uint32_t(e.focusable.kind)<<8|uint32_t(e.focusable.checked)<<16|uint32_t(e.focusable.outline)<<24);
    }
    h=render_key_mix(h,uint32_t(textures));h=render_key_mix(h,uint32_t(fonts));
    return h;
}

// A bounded cache for appearance-only leaf Images. Full node keys remain the
// authority for every other control, layout, focus/reveal input and hierarchy.
// The renderer changes existing rectangle packets only when their count stays
// identical, so DMA links, packet budgets and sibling offsets remain untouched.
template<size_t Capacity> class ImageReuseCache {
public:
    struct Entry {uint32_t owner=0,first=0,count=0,base_key=0,full_key=0;Rect rect;};
    // All scratch fields are assigned by prepare before a successful caller
    // consumes them. Keeping this POD avoids clearing8x96B on every miss.
    struct Patch {uint32_t owner,first,cache_index;ImagePatchRectangles rectangles;};
    static_assert(std::is_trivially_default_constructible_v<Patch>);
    static_assert(sizeof(Patch)==96);
    static_assert(sizeof(Entry)==36,"Image reuse entry must stay packed without pointers");
private:
    Entry entries_[2][Capacity];uint32_t stable_key_[2]={};uint8_t count_[2]={};
    uint32_t stable_key(unsigned parity,size_t count,const uint32_t* keys,uintptr_t textures,uintptr_t fonts)const{
        uint32_t key=render_key_mix(2166136261u,uint32_t(count));key=render_key_mix(key,uint32_t(textures));key=render_key_mix(key,uint32_t(fonts));
        // seal sorts only this bounded cache. Preserve actor/hash order while
        // advancing once through the sparse image owners, rather than searching
        // all cached images for every actor (including ordinary world actors).
        unsigned next=0;
        for(size_t i=0;i<count;++i){
            const bool cached=next<count_[parity]&&entries_[parity][next].owner==i;
            key=render_key_mix(key,cached?entries_[parity][next++].base_key:keys[i]);
        }
        return key;
    }
public:
    void invalidate(){count_[0]=count_[1]=0;}
    void begin(unsigned parity){count_[parity]=0;}
    void capture(unsigned parity,size_t owner,const ActorData* entities,size_t count,Rect rect,const Affine2& transform,
                 uint32_t first,uintptr_t textures,uintptr_t fonts){
        const auto& data=entities[owner];
        if(count_[parity]>=Capacity||!data.rect.enabled||data.canvas.enabled||!data.image.enabled||data.image.texture>=0
           ||data.text.enabled||data.progress.enabled||data.focusable.enabled||data.layout_container.enabled||!identity(transform))return;
        for(size_t i=0;i<count;++i)if(entities[i].parent==int(owner))return;
        entries_[parity][count_[parity]++]={uint32_t(owner),first,0,render_node_key(data,textures,fonts,true),0,rect};
    }
    void end(unsigned parity,size_t owner,uint32_t used){
        for(unsigned n=0;n<count_[parity];++n)if(entries_[parity][n].owner==owner){entries_[parity][n].count=used-entries_[parity][n].first;break;}
    }
    void seal(unsigned parity,const ActorData* entities,size_t count,const uint32_t* keys,uintptr_t textures,uintptr_t fonts){
        (void)entities;
        for(unsigned n=0;n<count_[parity];++n)entries_[parity][n].full_key=keys[entries_[parity][n].owner];
        for(unsigned n=1;n<count_[parity];++n){
            const Entry value=entries_[parity][n];unsigned pos=n;
            while(pos&&entries_[parity][pos-1].owner>value.owner){entries_[parity][pos]=entries_[parity][pos-1];--pos;}
            entries_[parity][pos]=value;
        }
        stable_key_[parity]=stable_key(parity,count,keys,textures,fonts);
    }
    bool prepare(unsigned parity,const ActorData* entities,size_t count,const uint32_t* keys,uintptr_t textures,uintptr_t fonts,
                 int width,int height,Patch (&patches)[Capacity],unsigned& patch_count)const{
        patch_count=0;if(!count_[parity])return false;
        for(unsigned n=0;n<count_[parity];++n){const auto& entry=entries_[parity][n];
            if(entry.owner>=count)return false;
            // The ordinary layout traversal already computed full node keys.
            // An unchanged image needs no normalized rehash or packet replay.
            if(entry.full_key!=keys[entry.owner]&&entry.base_key!=render_node_key(entities[entry.owner],textures,fonts,true))return false;
        }
        if(stable_key_[parity]!=stable_key(parity,count,keys,textures,fonts))return false;
        for(unsigned n=0;n<count_[parity];++n){const auto& entry=entries_[parity][n];
            if(entry.full_key==keys[entry.owner])continue;
            auto& patch=patches[patch_count++];patch.owner=entry.owner;patch.first=entry.first;patch.cache_index=n;
            patch.rectangles.build(entry.rect,entities[entry.owner].image,width,height);
            if(patch.rectangles.count!=entry.count)return false;
        }
        return true;
    }
    // Commit only after every prepare check succeeded and this packet was
    // written. Failed prepares remain read-only, including mixed-image failure.
    void applied(unsigned parity,const Patch& patch,uint32_t full_key){entries_[parity][patch.cache_index].full_key=full_key;}
};


template<class Sink> class Compiler {
    Sink& sink; int width,height; Budget budget; unsigned layouts=0; int owner=-1;
    // Caller-owned scratch for the measure and arrange passes; valid for one call.
    Fixed (*measured)[2]=nullptr; Rect* rects=nullptr; Affine2* transforms=nullptr;
    // Per-node emit state: the accumulated transform, whether it is anything but
    // the identity, the focus highlight, and the canvas's current focus index.
    Affine2 current; bool rotated=false; uint8_t tint[3]={255,255,255}; int focused=-1;
    bool primitive_room()const{return stats.rectangles+stats.images<budget.rectangles;}
    bool primitive_available(){if(primitive_room())return true;++stats.dropped;return false;}
    // A rotated primitive comes out of its own pool, so an unrotated scene keeps
    // every rectangle and glyph slot it has today.
    bool rotated_room()const{return stats.rotated<budget.rotated;}
    bool room()const{return rotated?rotated_room():primitive_room();}
    bool available(){if(room())return true;++stats.dropped;return false;}
    // The focused element's highlight, per channel. 255,255,255 is the identity,
    // so an unfocused node hands the sink exactly the authored bytes.
    void shade(const uint8_t* in,uint8_t* out)const{for(int i=0;i<3;++i)out[i]=uint8_t(unsigned(in[i])*tint[i]/255);}
    // Screen-space corners of a HUD rect through the accumulated transform, in
    // the A/B/C/D Z order the quad primitives want.
    void corners(Rect r,int* x,int* y)const{
        const Fixed ax[4]={r.x,r.x+r.w,r.x,r.x+r.w},ay[4]={r.y+r.h,r.y+r.h,r.y,r.y};
        for(int i=0;i<4;++i){Fixed tx,ty;apply(current,ax[i],ay[i],tx,ty);x[i]=pixel(tx);y[i]=height-pixel(ty);}
    }
    void fill(Rect r,const uint8_t* color){
        uint8_t c[3];shade(color,c);
        if(rotated){
            if(r.w.raw()<=0||r.h.raw()<=0||!available())return;
            int x[4],y[4];corners(r,x,y);
            ++stats.rotated;sink.quad(owner,x,y,c);return;
        }
        ImageRectangle rectangle;if(!clipped_rectangle(r,width,height,rectangle)||!primitive_available())return;
        ++stats.rectangles;sink.rectangle(owner,rectangle.x0,rectangle.y0,rectangle.x1,rectangle.y1,c);
    }
    // An optional one-pixel focus frame stays inside the authored control rect.
    // White passes through the current highlight tint without recolouring text.
    void focus_outline(Rect r){
        if(r.w.raw()<=0||r.h.raw()<=0)return;
        static constexpr uint8_t white[3]={255,255,255};
        const Fixed one=Fixed(4096,Fixed::RAW),w=r.w<one?r.w:one,h=r.h<one?r.h:one;
        fill({r.x,r.y,r.w,h},white);
        if(r.h>one)fill({r.x,r.y+r.h-h,r.w,h},white);
        if(r.h>h+h){
            fill({r.x,r.y+h,w,r.h-h-h},white);
            if(r.w>one)fill({r.x+r.w-w,r.y+h,w,r.h-h-h},white);
        }
    }
    // A Toggle's box at the right edge of its rect: a frame, a dark well and, when
    // checked, a mark. Three fills, tinted by the focus highlight like the rest.
    void check_box(Rect r,bool checked){
        static constexpr uint8_t frame[3]={230,230,230},well[3]={31,31,31},mark[3]={64,217,77};
        const int h=pixel(r.h),side=(h-6<6?6:h-6>14?14:h-6);
        const Fixed s=Fixed(side*4096,Fixed::RAW),margin=Fixed(3*4096,Fixed::RAW);
        Rect box{r.x+r.w-s-margin,r.y+(r.h-s)/2,s,s};
        if(box.x<r.x)box.x=r.x;
        fill(box,frame);
        const Fixed one=Fixed(4096,Fixed::RAW),two=Fixed(2*4096,Fixed::RAW);
        Rect inner{box.x+one,box.y+one,box.w-two,box.h-two};
        fill(inner,well);
        if(checked){Rect dot{inner.x+one,inner.y+one,inner.w-two,inner.h-two};fill(dot,mark);}
    }
    // A slider's handle: a light bar across the end of the fill, two pixels taller
    // than the track on each side. Tinted like the rest, so it carries the focus.
    void handle(Rect r,Fixed value){
        static constexpr uint8_t light[3]={230,230,230};
        const int h=pixel(r.h),w=(h/2<3?3:h/2>6?6:h/2);
        const Fixed width=Fixed(w*4096,Fixed::RAW),two=Fixed(2*4096,Fixed::RAW);
        Fixed x=r.x+r.w*value-width/2;
        if(x+width>r.x+r.w)x=r.x+r.w-width;
        if(x<r.x)x=r.x;
        fill(Rect{x,r.y-two,width,r.h+two+two},light);
    }
    void picture(Rect r,const Image& image){
        int texture_width=0,texture_height=0;
        if(!sink.texture_size(image.texture,texture_width,texture_height)){
            // An untextured Image can be a hollow frame. Emit only its edges,
            // sharing the normal clipping, rotation, tint and packet budgets.
            untextured_image(r,image,[&](Rect part,const uint8_t* color){fill(part,color);});
            return;
        }
        int x=pixel(r.x),y=height-pixel(r.y+r.h),w=pixel(r.w),h=pixel(r.h);
        if(w<=0||h<=0)return;
        int x0=clamp(x,width),x1=clamp(x+w,width),y0=clamp(y,height),y1=clamp(y+h,height);
        // A rotated quad is clipped by the drawing area, not by this axis-aligned
        // box, and it is counted against the rotated pool instead.
        if(rotated){if(!available())return;}
        else if(x1<=x0||y1<=y0||!primitive_available())return;
        int u=image.region[0],v=image.region[1];
        int tw=image.region[2]?image.region[2]:texture_width-u,th=image.region[3]?image.region[3]:texture_height-v;
        if(tw<=0||th<=0||u+tw>texture_width||v+th>texture_height)return;
        if(image.borders[0]||image.borders[1]||image.borders[2]||image.borders[3]){
            int left=image.borders[0],top=image.borders[1],right=image.borders[2],bottom=image.borders[3];
            if(left+right>tw||top+bottom>th)return;
            int dl=left,dr=right,dt=top,db=bottom;
            if(dl+dr>w){dl=left*w/(left+right);dr=w-dl;}
            if(dt+db>h){dt=top*h/(top+bottom);db=h-dt;}
            const int sx[4]={0,left,tw-right,tw},sy[4]={0,top,th-bottom,th};
            const int dx[4]={0,dl,w-dr,w},dy[4]={0,dt,h-db,h};
            for(int row=0;row<3;++row)for(int col=0;col<3;++col){
                if(dx[col+1]<=dx[col]||dy[row+1]<=dy[row]||sx[col+1]<=sx[col]||sy[row+1]<=sy[row])continue;
                Image part=image;for(auto& border:part.borders)border=0;
                // Only the centre piece tiles; the corners and edges keep their
                // stretched behaviour so a nine-slice frame stays a frame.
                part.tiling=row==1&&col==1?image.tiling:ImageTiling::None;
                part.region[0]=uint16_t(u+sx[col]);part.region[1]=uint16_t(v+sy[row]);part.region[2]=uint16_t(sx[col+1]-sx[col]);part.region[3]=uint16_t(sy[row+1]-sy[row]);
                picture({Fixed((x+dx[col])*4096,Fixed::RAW),Fixed((height-y-dy[row+1])*4096,Fixed::RAW),Fixed((dx[col+1]-dx[col])*4096,Fixed::RAW),Fixed((dy[row+1]-dy[row])*4096,Fixed::RAW)},part);
            }
            return;
        }
        if(image.tiling!=ImageTiling::None){
            // Tile repeats the source at its native texel size from the rect's
            // bottom-left and clips the last column and row; TileFit rounds the
            // count per axis to at least one and scales the tiles to fill exactly.
            const bool fit=image.tiling==ImageTiling::TileFit;
            int columns=fit?(w+tw/2)/tw:(w+tw-1)/tw,rows=fit?(h+th/2)/th:(h+th-1)/th;
            if(columns<1)columns=1;
            if(rows<1)rows=1;
            for(int row=0;row<rows;++row)for(int col=0;col<columns;++col){
                int dx0,dx1,dy0,dy1,source_v=0,source_w=tw,source_h=th;
                if(fit){dx0=col*w/columns;dx1=(col+1)*w/columns;dy0=row*h/rows;dy1=(row+1)*h/rows;}
                else{
                    dx0=col*tw;dx1=dx0+tw>w?w:dx0+tw;
                    dy1=h-row*th;dy0=dy1-th<0?0:dy1-th;
                    source_w=dx1-dx0;source_h=dy1-dy0;source_v=th-source_h;
                }
                if(dx1<=dx0||dy1<=dy0)continue;
                if(!room()){++stats.dropped;return;}
                Image part=image;part.tiling=ImageTiling::None;
                part.region[0]=uint16_t(u);part.region[1]=uint16_t(v+source_v);part.region[2]=uint16_t(source_w);part.region[3]=uint16_t(source_h);
                picture({Fixed((x+dx0)*4096,Fixed::RAW),Fixed((height-y-dy1)*4096,Fixed::RAW),Fixed((dx1-dx0)*4096,Fixed::RAW),Fixed((dy1-dy0)*4096,Fixed::RAW)},part);
            }
            return;
        }
        uint8_t c[3];shade(image.color,c);
        if(rotated){
            int px[4],py[4];corners(r,px,py);
            const int su[4]={u,u+tw-1,u,u+tw-1},sv[4]={v,v,v+th-1,v+th-1};
            ++stats.rotated;sink.textured_quad(owner,image.texture,px,py,su,sv,c);return;
        }
        ++stats.images;
        sink.image(owner,image.texture,x0,y0,x1,y1,u+(x0-x)*tw/w,v+(y0-y)*th/h,u+(x1-x)*tw/w-1,v+(y1-y)*th/h-1,c);
    }
    // What one codepoint contributes to a line: the atlas cell it draws from,
    // the drawn box, the pen advance, and the offset of that box from the pen
    // and the baseline. `ink` is false for a codepoint the font cannot draw but
    // still advances past. The built-in atlas is uniform 8x16 with no offsets,
    // so it reproduces the fixed-cell walk this replaced exactly.
    struct Placed { int u=0,v=0,width=0,height=0,advance=0,dx=0,dy=0;bool ink=false; };
    static Placed place_glyph(const Font* font,uint32_t c){
        Placed g;
        // The built-in space draws nothing: its cell holds the atlas's CLUT.
        if(!font){const int cell=builtin_cell(c);g={(cell%32)*8,(cell/32)*16,8,16,8,0,0,c!=' '};return g;}
        const GlyphMetric* m=find_glyph(*font,c);
        if(!m)m=find_glyph(*font,'?');
        if(!m){const GlyphMetric* space=find_glyph(*font,' ');g.advance=space?int(space->advance):0;return g;}
        g={m->u,m->v,m->width,m->height,m->advance,m->x_offset,m->y_offset,m->width>0&&m->height>0};
        return g;
    }
    // A font index nothing answers, or one whose table came through empty,
    // falls back to the built-in atlas so a label stays visible while its asset
    // is still unresolved.
    const Font* font_for(int index){const Font* f=index>=0?sink.font(index):nullptr;return f&&f->count?f:nullptr;}
    // Alignment has to know a line's width before its first glyph is placed, so
    // it measures into this table; Left needs no measure and is unbounded.
    static constexpr int measured_rows=32;
    static int hex_digit(char c){return c>='0'&&c<='9'?c-'0':c>='a'&&c<='f'?c-'a'+10:c>='A'&&c<='F'?c-'A'+10:-1;}
    // Inline colour markup: `[color=#rrggbb]` switches the glyph colour and
    // `[/color]` restores the text's own. Consumes one tag at `n`; false otherwise.
    static bool markup(const char* s,size_t& n,uint8_t* color,const uint8_t* base){
        if(s[n]!='[')return false;
        static constexpr char open[]="[color=#",close[]="[/color]";
        size_t k=0;while(close[k]&&n+k<511&&s[n+k]==close[k])++k;
        if(!close[k]){for(int c=0;c<3;++c)color[c]=base[c];n+=k;return true;}
        k=0;while(open[k]&&n+k<511&&s[n+k]==open[k])++k;
        if(open[k]||n+k+7>511)return false;
        uint8_t parsed[3];
        for(int c=0;c<3;++c){const int hi=hex_digit(s[n+k+2*c]),lo=hex_digit(s[n+k+2*c+1]);if(hi<0||lo<0)return false;parsed[c]=uint8_t(hi*16+lo);}
        if(s[n+k+6]!=']')return false;
        for(int c=0;c<3;++c)color[c]=parsed[c];
        n+=k+7;return true;
    }
    void label(Rect r,const Text& text){
        if(stats.texts>=budget.texts){++stats.dropped;return;}
        const Font* font=font_for(text.font);
        const int line=font?(font->line_height?int(font->line_height):1):16,base=font?int(font->baseline):0;
        const int box=pixel(r.w);int rows=pixel(r.h)/line;
        // The built-in font keeps its "at least one 8 px column" rule so an
        // unchanged scene emits exactly the sprites it emitted before.
        if(box/(font?1:8)<=0||rows<=0)return;
        int widths[measured_rows]={};
        if(text.align!=TextAlign::Left){
            if(rows>measured_rows)rows=measured_rows;
            int pen=0,row=0;
            uint8_t ignored[3];
            for(size_t n=0;n<511&&text.value[n];){
                if(markup(text.value,n,ignored,text.color))continue;
                const uint32_t c=utf8_scalar(text.value,n);
                if(c==10){pen=0;if(++row>=rows)break;continue;}
                const Placed g=place_glyph(font,c);
                if(text.wrap&&pen+g.advance>box){pen=0;if(++row>=rows)break;}
                if(pen+g.advance<=box)widths[row]=pen+g.advance;
                pen+=g.advance;
            }
        }
        ++stats.texts;sink.begin_text(text.font);
        const int left=pixel(r.x),top=height-pixel(r.y+r.h);
        int pen=0,row=0;
#ifdef EPOK_TEXT_REVEAL
        uint32_t revealed=0;
#endif
        uint8_t ink[3]={text.color[0],text.color[1],text.color[2]};
        for(size_t n=0;n<511&&text.value[n];){
            if(markup(text.value,n,ink,text.color))continue;
            const uint32_t c=utf8_scalar(text.value,n);
            if(c==10){pen=0;++row;continue;}
#ifdef EPOK_TEXT_REVEAL
            if(revealed++>=text.visible_glyphs)break;
#endif
            const Placed g=place_glyph(font,c);
            if(text.wrap&&pen+g.advance>box){pen=0;++row;}
            if(row>=rows)break;
            const int start=pen;pen+=g.advance;
            // Past the right edge the glyph is dropped, not clipped: the fixed
            // column cap did the same before advances varied per glyph.
            if(start+g.advance>box||!g.ink)continue;
            const int offset=text.align==TextAlign::Left?0:text.align==TextAlign::Center?(box-widths[row])/2:box-widths[row];
            const int x=left+offset+start+g.dx,y=top+row*line+base+g.dy;
            if(rotated){
                // The screen box a rotated cell lands in is not this one, so the
                // line cap alone prunes here and the drawing area clips the rest.
                if(!rotated_room()){++stats.dropped;break;}
                const Rect cell{Fixed(x*4096,Fixed::RAW),Fixed((height-y-g.height)*4096,Fixed::RAW),Fixed(g.width*4096,Fixed::RAW),Fixed(g.height*4096,Fixed::RAW)};
                int px[4],py[4];corners(cell,px,py);
                const int su[4]={g.u,g.u+g.width,g.u,g.u+g.width},sv[4]={g.v,g.v,g.v+g.height,g.v+g.height};
                ++stats.rotated;sink.glyph_quad(owner,text.font,px,py,su,sv,ink);
                continue;
            }
            if(x>=width||y>=height||x+g.width<=0||y+g.height<=0)continue;
            if(stats.glyphs>=budget.glyphs){++stats.dropped;break;}
            int x0=clamp(x,width),y0=clamp(y,height),x1=clamp(x+g.width,width),y1=clamp(y+g.height,height);
            ++stats.glyphs;sink.glyph(owner,text.font,g.u+(x0-x),g.v+(y0-y),x0,y0,x1,y1,ink);
        }
    }
    static bool root_of_layout(const ActorData& e){return (e.canvas.enabled||e.rect.enabled)&&e.parent<0;}
    // A top-level Canvas fills the display. A reusable nested Canvas owns its
    // authored rect and focus scope within its parent's layout.
    static bool uses_rect(const ActorData& e){return e.rect.enabled&&(!e.canvas.enabled||(e.parent>=0&&!e.canvas.screen_overlay));}
    static bool arranges(const ActorData& e){return e.layout_container.enabled&&e.layout_container.kind!=LayoutKind::None;}
    static bool overlay_canvas(const ActorData& e){return e.canvas.enabled&&e.canvas.screen_overlay;}
    static uint8_t axis_flags(const ActorData& e,int axis){return e.layout_element.enabled?(axis?e.layout_element.vertical:e.layout_element.horizontal):uint8_t(1);}
    // A container measures and places exactly the children it draws, so an
    // inactive one takes no cell and the list it sits in closes up.
    static bool participates(const ActorData& e){return e.rect.enabled&&e.alive&&e.active&&!overlay_canvas(e);}
    // Widest child of grid column `line` (axis 0), or tallest child of grid row `line` (axis 1).
    Fixed grid_span(ActorData* entities,int index,const int* first,const int* next,int columns,int line,int axis)const{
        Fixed most=0.0;int k=0;
        for(int child=first[index];child>=0;child=next[child]){if(!participates(entities[child]))continue;if((axis?k/columns:k%columns)==line&&measured[child][axis]>most)most=measured[child][axis];++k;}
        return most;
    }
    // Place `want` inside [begin,begin+space]: Fill takes the whole cell, otherwise
    // the measured size sits at the begin edge, centred, or at the end edge.
    static void place(Fixed begin,Fixed space,Fixed want,uint8_t flags,bool centred,Fixed& position,Fixed& size){
        if(flags&1){position=begin;size=space;return;}
        size=want;
        if(flags&8)position=begin+space-want;else if((flags&4)||centred)position=begin+(space-want)/2;else position=begin;
    }
    // Post-order minimum size. A leaf starts from the space its text needs, a
    // container from the children it will arrange; both are then floored below.
    void extent(ActorData* entities,size_t count,int index,const int* first,const int* next,int depth){
        if(depth>32)return;
        auto& e=entities[index];if(!e.alive||!e.active)return;
        for(int child=first[index];child>=0;child=next[child])if(entities[child].rect.enabled)extent(entities,count,child,first,next,depth+1);
        Fixed size[2]={0.0,0.0};
        if(arranges(e)){
            const auto& c=e.layout_container;
            const Fixed pad[2]={c.padding[0]+c.padding[2],c.padding[1]+c.padding[3]};
            int n=0,head=-1;Fixed sum[2]={0.0,0.0},most[2]={0.0,0.0};
            for(int child=first[index];child>=0;child=next[child]){
                if(!participates(entities[child]))continue;
                if(head<0)head=child;
                for(int a=0;a<2;++a){sum[a]+=measured[child][a];if(measured[child][a]>most[a])most[a]=measured[child][a];}
                ++n;
            }
            if(c.kind==LayoutKind::Horizontal||c.kind==LayoutKind::Vertical){
                const int axis=c.kind==LayoutKind::Vertical;
                size[axis]=pad[axis]+sum[axis];if(n>1)size[axis]+=c.spacing[axis]*(n-1);
                size[1-axis]=pad[1-axis]+most[1-axis];
            }else if(c.kind==LayoutKind::Grid){
                const int columns=c.columns?c.columns:1,rows=(n+columns-1)/columns;
                Fixed line[2]={0.0,0.0};
                for(int j=0;j<columns;++j)line[0]+=grid_span(entities,index,first,next,columns,j,0);
                for(int i=0;i<rows;++i)line[1]+=grid_span(entities,index,first,next,columns,i,1);
                size[0]=pad[0]+line[0];if(columns>1)size[0]+=c.spacing[0]*(columns-1);
                size[1]=pad[1]+line[1];if(rows>1)size[1]+=c.spacing[1]*(rows-1);
            }else for(int a=0;a<2;++a)size[a]=pad[a]+(c.kind==LayoutKind::Margin?(head>=0?measured[head][a]:Fixed(0.0)):most[a]);
        }else if(e.text.enabled){
            // One line of the element's own font: the advances it really walks,
            // which for the built-in atlas is the byte count times eight.
            const Font* font=font_for(e.text.font);
            size[1]=Fixed(font?int(font->line_height):16,0);
            if(!e.text.wrap){int32_t run=0;for(size_t n=0;n<511&&e.text.value[n];){const uint32_t c=utf8_scalar(e.text.value,n);if(c!=10)run+=place_glyph(font,c).advance;}size[0]=Fixed(run,0);}
        }
        // One rule for every child of a container, a nested container included: the
        // formula or intrinsic, floored by the node's own rect size and its minimum.
        for(int a=0;a<2;++a){
            Fixed m=size[a];
            if(e.rect.size[a]>m)m=e.rect.size[a];
            if(e.layout_element.enabled&&e.layout_element.minimum[a]>m)m=e.layout_element.minimum[a];
            measured[index][a]=m.raw()<0?Fixed(0.0):m;
        }
    }
    // Top-down placement. `assigned` is the rect a parent container decided for
    // this node; without one the node resolves its own anchors exactly as before.
    void arrange(ActorData* entities,size_t count,int index,Rect parent,const int* first,const int* next,int depth,const Rect* assigned,const Affine2& inherited){
        if(depth>32)return;
        auto& e=entities[index];if(!e.alive||!e.active)return;
        const bool overlay=e.canvas.enabled&&e.canvas.screen_overlay;
        Rect r=overlay?Rect{0.0,0.0,Fixed(width,0),Fixed(height,0)}:parent;
        if(uses_rect(e)){if(layouts++>=budget.layouts){++stats.dropped;return;}r=assigned?*assigned:resolve(parent,e.rect);}
        rects[index]=r;
        // Layout is axis-aligned; the rotation rides alongside it so a child of a
        // rotated panel turns with the panel without its rect changing.
        Affine2 m=overlay?Affine2{}:inherited;
        if(uses_rect(e)&&e.rect.rotation.raw())
            m=compose(inherited,rotation_about(r.x+r.w*e.rect.pivot[0],r.y+r.h*e.rect.pivot[1],e.rect.rotation));
        transforms[index]=m;
        if(!arranges(e)){for(int child=first[index];child>=0;child=next[child])if(entities[child].rect.enabled)arrange(entities,count,child,r,first,next,depth+1,nullptr,m);return;}
        const auto& c=e.layout_container;
        const Fixed origin[2]={r.x+c.padding[0],r.y+c.padding[3]};
        const Fixed inner[2]={r.w-c.padding[0]-c.padding[2],r.h-c.padding[1]-c.padding[3]};
        const Fixed top=origin[1]+inner[1];
        int n=0,expanders=0;Fixed mins[2]={0.0,0.0},stretch=0.0;
        const int axis=c.kind==LayoutKind::Vertical;
        for(int child=first[index];child>=0;child=next[child]){
            if(!participates(entities[child]))continue;
            for(int a=0;a<2;++a)mins[a]+=measured[child][a];
            if(axis_flags(entities[child],axis)&2){++expanders;stretch+=entities[child].layout_element.stretch;}
            ++n;
        }
        if(c.kind==LayoutKind::Horizontal||c.kind==LayoutKind::Vertical){
            Fixed leftover=inner[axis]-mins[axis];if(n>1)leftover-=c.spacing[axis]*(n-1);
            if(leftover.raw()<0)leftover=0.0;
            Fixed given=0.0,cursor=axis?top:origin[0];int seen=0;
            for(int child=first[index];child>=0;child=next[child]){
                if(overlay_canvas(entities[child])){arrange(entities,count,child,r,first,next,depth+1,nullptr,m);continue;}
                if(!participates(entities[child]))continue;
                Fixed cell=measured[child][axis];
                if(axis_flags(entities[child],axis)&2){
                    // The last expander absorbs the division remainder, so the cells fill the box exactly.
                    Fixed share=leftover-given;
                    if(++seen<expanders)share=stretch.raw()?leftover*entities[child].layout_element.stretch/stretch:leftover/expanders;
                    given+=share;cell+=share;
                }
                // +Y is up, so a Vertical box walks down from the padded top edge.
                const Fixed begin=axis?cursor-cell:cursor;
                cursor=axis?begin-c.spacing[1]:cursor+cell+c.spacing[0];
                Fixed position[2],size[2];
                place(begin,cell,measured[child][axis],axis_flags(entities[child],axis),false,position[axis],size[axis]);
                place(origin[1-axis],inner[1-axis],measured[child][1-axis],axis_flags(entities[child],1-axis),false,position[1-axis],size[1-axis]);
                const Rect box{position[0],position[1],size[0],size[1]};
                arrange(entities,count,child,r,first,next,depth+1,&box,m);
            }
            return;
        }
        if(c.kind==LayoutKind::Grid){
            const int columns=c.columns?c.columns:1,rows=(n+columns-1)/columns;
            Fixed span[2]={0.0,0.0};
            for(int j=0;j<columns;++j)span[0]+=grid_span(entities,index,first,next,columns,j,0);
            for(int i=0;i<rows;++i)span[1]+=grid_span(entities,index,first,next,columns,i,1);
            Fixed extra[2]={inner[0]-span[0],inner[1]-span[1]};
            if(columns>1)extra[0]-=c.spacing[0]*(columns-1);
            if(rows>1)extra[1]-=c.spacing[1]*(rows-1);
            if(extra[0].raw()<0)extra[0]=0.0;extra[0]=extra[0]/columns;
            if(extra[1].raw()<0)extra[1]=0.0;if(rows)extra[1]=extra[1]/rows;
            Fixed x=origin[0],y=top,height_of_row=0.0;int k=0;
            for(int child=first[index];child>=0;child=next[child]){
                if(overlay_canvas(entities[child])){arrange(entities,count,child,r,first,next,depth+1,nullptr,m);continue;}
                if(!participates(entities[child]))continue;
                if(k%columns==0){x=origin[0];if(k)y-=c.spacing[1];height_of_row=grid_span(entities,index,first,next,columns,k/columns,1)+extra[1];y-=height_of_row;}
                const Fixed width_of_column=grid_span(entities,index,first,next,columns,k%columns,0)+extra[0];
                const Fixed cell[2]={width_of_column,height_of_row},begin[2]={x,y};
                Fixed position[2],size[2];
                for(int a=0;a<2;++a)place(begin[a],cell[a],measured[child][a],axis_flags(entities[child],a),false,position[a],size[a]);
                const Rect box{position[0],position[1],size[0],size[1]};
                arrange(entities,count,child,r,first,next,depth+1,&box,m);
                x+=width_of_column+c.spacing[0];++k;
            }
            return;
        }
        // Margin and Center hand every participating child the padded rect; only Center aligns it.
        const bool centred=c.kind==LayoutKind::Center;
        for(int child=first[index];child>=0;child=next[child]){
            if(overlay_canvas(entities[child])){arrange(entities,count,child,r,first,next,depth+1,nullptr,m);continue;}
            if(!participates(entities[child]))continue;
            Fixed position[2],size[2];
            for(int a=0;a<2;++a)place(origin[a],inner[a],measured[child][a],axis_flags(entities[child],a),centred,position[a],size[a]);
            const Rect box{position[0],position[1],size[0],size[1]};
            arrange(entities,count,child,r,first,next,depth+1,&box,m);
        }
    }
    // Same traversal and the same layout-budget pruning as arrange, reading the
    // rects it decided. The budget counter is local so the drop count is unchanged.
    void emit(ActorData* entities,size_t count,int index,const int* first,const int* next,int depth,unsigned& counted){
        if(depth>32)return;
        auto& e=entities[index];if(!e.alive||!e.active)return;
        if(uses_rect(e)&&counted++>=budget.layouts)return;
        const int previous_focus=focused;
        if(e.canvas.enabled)focused=e.canvas.focused;
        const Rect r=rects[index];
        owner=index;
        current=transforms[index];rotated=!identity(current);
        // Tint keeps the existing image/fill contract; an opted-in outline adds
        // up to four ordinary primitives and leaves text/markup colours intact.
        const bool lit=e.focusable.enabled&&focused==index;
        const bool outlined=lit&&e.focusable.outline&&(e.image.enabled||e.progress.enabled||e.text.enabled||e.focusable.kind==FocusKind::Toggle);
        for(int k=0;k<3;++k)tint[k]=lit?e.focusable.highlight[k]:uint8_t(255);
        constexpr bool cached=requires {sink.begin_node(owner,r,current,tint,outlined,stats);sink.end_node(owner,stats);};
        bool reused=false;
        if constexpr(cached)reused=sink.begin_node(owner,r,current,tint,outlined,stats);
        if(!reused){
        if(e.image.enabled)picture(r,e.image);
        if(e.progress.enabled){
            fill(r,e.progress.background);auto part=r;auto value=e.progress.value;if(value.raw()<0)value=0.0;if(value.raw()>4096)value=1.0;part.w*=value;fill(part,e.progress.color);
            if(e.focusable.enabled&&e.progress.step.raw()>0)handle(r,value);
        }
        if(outlined)focus_outline(r);
        if(e.text.enabled)label(r,e.text);
        if(e.focusable.enabled&&e.focusable.kind==FocusKind::Toggle)check_box(r,e.focusable.checked);
        if constexpr(cached)sink.end_node(owner,stats);
        }
        for(int child=first[index];child>=0;child=next[child])if(entities[child].rect.enabled)emit(entities,count,child,first,next,depth+1,counted);
        focused=previous_focus;
    }
public:
    HudStats stats;
    Compiler(Sink& sink,int width,int height,Budget budget):sink(sink),width(width),height(height),budget(budget){}
    // Measure and arrange without drawing: `rects` holds every laid-out node and
    // zeroes elsewhere, and `affines` the accumulated rotation about each pivot.
    void layout(ActorData* entities,size_t count,int* first,int* next,Fixed (*sizes)[2],Rect* boxes,Affine2* affines,bool measure_intrinsics=true){
        stats={};layouts=0;measured=sizes;rects=boxes;transforms=affines;
        bool needs_measure=measure_intrinsics;
        for(size_t i=0;i<count;++i){first[i]=next[i]=-1;sizes[i][0]=sizes[i][1]=0.0;boxes[i]=Rect{};affines[i]=Affine2{};needs_measure|=arranges(entities[i]);}
        for(size_t i=count;i-->0;){int p=entities[i].parent;if(p>=0&&size_t(p)<count){next[i]=first[p];first[p]=int(i);}}
        const Rect screen{0.0,0.0,Fixed(width*4096,Fixed::RAW),Fixed(height*4096,Fixed::RAW)};
        // Anchored rects do not read intrinsic sizes. Text measurement is only
        // needed for a container, or when layout() exposes sizes to its caller.
        if(needs_measure)for(size_t i=0;i<count;++i)if(root_of_layout(entities[i]))extent(entities,count,int(i),first,next,0);
        for(size_t i=0;i<count;++i)if(root_of_layout(entities[i]))arrange(entities,count,int(i),screen,first,next,0,nullptr,Affine2{});
    }
    void draw(ActorData* entities,size_t count,int* first,int* next,Fixed (*sizes)[2],Rect* boxes,Affine2* affines,bool measure_intrinsics=false){
        layout(entities,count,first,next,sizes,boxes,affines,measure_intrinsics);
        // Optional native diagnostics observe the boundary without replacing
        // either pass or adding a callback to ordinary draw sinks.
        if constexpr(requires {sink.layout_finished();})sink.layout_finished();
        unsigned counted=0;
        for(size_t i=0;i<count;++i)if(root_of_layout(entities[i])){focused=-1;emit(entities,count,int(i),first,next,0,counted);}
    }
};
}
