#pragma once
#include "epok.hpp"

namespace epok::hud_core {
// Shared by the PSX packet writer and the native preview. No GPU, heap, or
// floating-point operations: layout, clipping, UVs, text and budgets agree.
struct Rect { Fixed x=0.0,y=0.0,w=0.0,h=0.0; };
struct Budget { unsigned layouts,rectangles,texts,glyphs,rotated; };
inline int pixel(Fixed v) { auto r=v.raw();return int((int64_t(r)+(r>=0?2048:-2048))/4096); }
inline int clamp(int v,int hi) { return v<0?0:v>hi?hi:v; }
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
        int x0=clamp(pixel(r.x),width),x1=clamp(pixel(r.x+r.w),width);
        int y0=clamp(height-pixel(r.y+r.h),height),y1=clamp(height-pixel(r.y),height);
        if(x1<=x0||y1<=y0||!primitive_available())return;
        ++stats.rectangles;sink.rectangle(owner,x0,y0,x1,y1,c);
    }
    void picture(Rect r,const Image& image){
        int texture_width=0,texture_height=0;
        if(!sink.texture_size(image.texture,texture_width,texture_height)){fill(r,image.color);return;}
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
    void label(Rect r,const Text& text){
        if(stats.texts>=budget.texts){++stats.dropped;return;}
        int columns=pixel(r.w)/8,rows=pixel(r.h)/16;if(columns<=0||rows<=0)return;
        ++stats.texts;sink.begin_text();
        int col=0,row=0,left=pixel(r.x),top=height-pixel(r.y+r.h);
        for(size_t n=0;n<511&&text.value[n];++n){
            unsigned c=uint8_t(text.value[n]);
            if(c==10){col=0;++row;continue;}
            if(col>=columns&&text.wrap){col=0;++row;}
            if(row>=rows)break;
            int x=left+col*8,y=top+row*16;++col;
            if(rotated){
                // The screen box a rotated cell lands in is not this one, so only
                // the column cap prunes here and the drawing area clips the rest.
                if(col>columns)continue;
                if(!rotated_room()){++stats.dropped;break;}
                if(c<32||c>142)c='?';
                const Rect cell{Fixed(x*4096,Fixed::RAW),Fixed((height-y-16)*4096,Fixed::RAW),Fixed(8*4096,Fixed::RAW),Fixed(16*4096,Fixed::RAW)};
                int px[4],py[4];corners(cell,px,py);
                const int su[4]={0,8,0,8},sv[4]={0,0,16,16};
                ++stats.rotated;sink.glyph_quad(owner,c,px,py,su,sv,text.color);
                continue;
            }
            if(col>columns||x>=width||y>=height||x+8<=0||y+16<=0)continue;
            if(stats.glyphs>=budget.glyphs){++stats.dropped;break;}
            if(c<32||c>142)c='?';
            int x0=clamp(x,width),y0=clamp(y,height),x1=clamp(x+8,width),y1=clamp(y+16,height);
            ++stats.glyphs;sink.glyph(owner,c,x0,y0,x1,y1,x0-x,y0-y,text.color);
        }
    }
    static bool root_of_layout(const ActorData& e){return (e.canvas.enabled||e.rect.enabled)&&e.parent<0;}
    static bool arranges(const ActorData& e){return e.layout_container.enabled&&e.layout_container.kind!=LayoutKind::None;}
    static uint8_t axis_flags(const ActorData& e,int axis){return e.layout_element.enabled?(axis?e.layout_element.vertical:e.layout_element.horizontal):uint8_t(1);}
    // A container measures and places exactly the children it draws, so an
    // inactive one takes no cell and the list it sits in closes up.
    static bool participates(const ActorData& e){return e.rect.enabled&&e.alive&&e.active;}
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
            size[1]=16.0;if(!e.text.wrap){int32_t length=0;while(length<511&&e.text.value[length])++length;size[0]=Fixed(length*8,0);}
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
        Rect r=parent;
        if(e.rect.enabled&&!e.canvas.enabled){if(layouts++>=budget.layouts){++stats.dropped;return;}r=assigned?*assigned:resolve(parent,e.rect);}
        rects[index]=r;
        // Layout is axis-aligned; the rotation rides alongside it so a child of a
        // rotated panel turns with the panel without its rect changing.
        Affine2 m=inherited;
        if(e.rect.enabled&&!e.canvas.enabled&&e.rect.rotation.raw())
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
        if(e.rect.enabled&&!e.canvas.enabled&&counted++>=budget.layouts)return;
        if(e.canvas.enabled)focused=e.canvas.focused;
        const Rect r=rects[index];
        owner=index;
        current=transforms[index];rotated=!identity(current);
        // The focused element lights up by modulating the colours it already
        // draws with, so focus costs no extra primitive.
        const bool lit=e.focusable.enabled&&focused==index;
        for(int k=0;k<3;++k)tint[k]=lit?e.focusable.highlight[k]:uint8_t(255);
        if(e.image.enabled)picture(r,e.image);
        if(e.progress.enabled){fill(r,e.progress.background);auto part=r;auto value=e.progress.value;if(value.raw()<0)value=0.0;if(value.raw()>4096)value=1.0;part.w*=value;fill(part,e.progress.color);}
        if(e.text.enabled)label(r,e.text);
        for(int child=first[index];child>=0;child=next[child])if(entities[child].rect.enabled)emit(entities,count,child,first,next,depth+1,counted);
    }
public:
    HudStats stats;
    Compiler(Sink& sink,int width,int height,Budget budget):sink(sink),width(width),height(height),budget(budget){}
    // Measure and arrange without drawing: `rects` holds every laid-out node and
    // zeroes elsewhere, and `affines` the accumulated rotation about each pivot.
    void layout(ActorData* entities,size_t count,int* first,int* next,Fixed (*sizes)[2],Rect* boxes,Affine2* affines){
        stats={};layouts=0;measured=sizes;rects=boxes;transforms=affines;
        for(size_t i=0;i<count;++i){first[i]=next[i]=-1;sizes[i][0]=sizes[i][1]=0.0;boxes[i]=Rect{};affines[i]=Affine2{};}
        for(size_t i=count;i-->0;){int p=entities[i].parent;if(p>=0&&size_t(p)<count){next[i]=first[p];first[p]=int(i);}}
        const Rect screen{0.0,0.0,Fixed(width*4096,Fixed::RAW),Fixed(height*4096,Fixed::RAW)};
        for(size_t i=0;i<count;++i)if(root_of_layout(entities[i]))extent(entities,count,int(i),first,next,0);
        for(size_t i=0;i<count;++i)if(root_of_layout(entities[i]))arrange(entities,count,int(i),screen,first,next,0,nullptr,Affine2{});
    }
    void draw(ActorData* entities,size_t count,int* first,int* next,Fixed (*sizes)[2],Rect* boxes,Affine2* affines){
        layout(entities,count,first,next,sizes,boxes,affines);
        unsigned counted=0;
        for(size_t i=0;i<count;++i)if(root_of_layout(entities[i])){focused=-1;emit(entities,count,int(i),first,next,0,counted);}
    }
};
}
