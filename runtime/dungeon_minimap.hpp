#pragma once
#include <stddef.h>
#include <stdint.h>
#include "dungeon_map.hpp"
#include "frame_clear.hpp"
#include "psyqo/fragments.hh"
#include "psyqo/gpu.hh"
#include "psyqo/primitives/control.hh"
#include "psyqo/primitives/rectangles.hh"

namespace epok::dungeon {
// Draws the dungeon minimap over the world and under the HUD: one flat rectangle per
// shown map rectangle and icon, in fixed double-buffered packets. `Capacity` is the
// most map rectangles any map of the project cooks; 0 compiles the minimap out.
template<size_t Capacity,bool Interlaced> class MinimapRenderer {
    static constexpr size_t icon_capacity=40;
    static constexpr size_t packet_capacity=Capacity?Capacity+icon_capacity+1:1;
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> pages[2];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::Rectangle> packets[2][packet_capacity];
    psyqo::GPU* gpu=nullptr;
    size_t used=0;
    unsigned parity=0;
    int clip[4]={0,0,0,0};
    void fill(int x0,int y0,int x1,int y1,const uint8_t* color,bool translucent=false) {
        if(x0<clip[0])x0=clip[0];
        if(y0<clip[1])y0=clip[1];
        if(x1>clip[2])x1=clip[2];
        if(y1>clip[3])y1=clip[3];
        if(x1<=x0||y1<=y0||used>=packet_capacity)return;
        auto& f=packets[parity][used++];
        f.primitive.position={{.x=int16_t(x0),.y=int16_t(y0)}};
        f.primitive.size={{.w=int16_t(x1-x0),.h=int16_t(y1-y0)}};
        f.primitive.setColor(psyqo::Color{{.r=color[0],.g=color[1],.b=color[2]}});
        if(translucent)f.primitive.setSemiTrans();
        else f.primitive.setOpaque();
        gpu->chain(f);
    }
    // 0 hidden, 1 seen from an explored room, 2 explored.
    static int shown(uint16_t i) {
        if(minimap.flags&minimap_everything||rect_explored(i))return 2;
        const auto& r=minimap.rects[i];
        return r.room<0&&(room_explored(r.near[0])||room_explored(r.near[1]))?1:0;
    }
    static bool shown_at(const int32_t* point) {
        int32_t cx,cy;
        if(!map_cell(point,cx,cy))return false;
        const int32_t i=rect_at(cx,cy);
        return i>=0&&shown(uint16_t(i))>0;
    }

public:
    // `player` is the player's world point (raw Q12), or null when there is none;
    // `frame` counts frames for the player's blink. Exploring happens whether or not
    // the map shows.
    template<class Active>
    void draw(psyqo::GPU& output,int screen_width,int screen_height,const int32_t* player,uint32_t frame,Active&& active) {
        if constexpr(Capacity==0) {
            (void)output;(void)screen_width;(void)screen_height;(void)player;(void)frame;(void)active;
        } else {
            const auto& m=minimap;
            if(!m.count||!m.rects)return;
            int32_t pcx=0,pcy=0;
            const bool on_map=player&&map_cell(player,pcx,pcy);
            if(player)explore_at(player);
            if(!(m.flags&minimap_visible))return;
            gpu=&output;parity=output.getParity();used=0;
            const bool full=m.flags&minimap_full;
            int bx,by,bw,bh,cell;
            if(full) {
                bx=16;by=16;bw=screen_width-32;bh=screen_height-32;
            } else {
                bw=m.width;bh=m.height;
                bx=m.flags&minimap_right?screen_width-m.x-bw:m.x;
                by=m.flags&minimap_bottom?screen_height-m.y-bh:m.y;
            }
            const bool follow=!full&&(m.flags&minimap_follow);
            if(follow)cell=m.cell?m.cell:1;
            else {
                cell=bw/m.columns<bh/m.rows?bw/m.columns:bh/m.rows;
                if(cell<1)cell=1;
            }
            int ox,oy;
            if(follow&&on_map) {
                ox=bx+bw/2-int(pcx)*cell-cell/2;
                oy=by+bh/2-int(pcy)*cell-cell/2;
            } else {
                ox=bx+(bw-int(m.columns)*cell)/2;
                oy=by+(bh-int(m.rows)*cell)/2;
            }
            clip[0]=bx;clip[1]=by;clip[2]=bx+bw;clip[3]=by+bh;
            // The backdrop halves what is behind it.
            auto& page=pages[parity];
            page.primitive.attr.set(static_cast<psyqo::Prim::TPageAttr::SemiTrans>(0)).setDithering(false);
            configure_display_field<Interlaced>(page.primitive.attr);
            output.chain(page);
            fill(bx,by,bx+bw,by+bh,m.colors[color_background],true);
            for(uint16_t i=0;i<m.count;++i) {
                const int state=shown(i);
                if(!state)continue;
                const auto& r=m.rects[i];
                const uint8_t* color=state==1?m.colors[color_seen]:r.room>=0?m.colors[color_room]:m.colors[color_corridor];
                fill(ox+r.x*cell,oy+r.y*cell,ox+(r.x+r.w)*cell,oy+(r.y+r.h)*cell,color);
            }
            const int size=cell<2?2:cell;
            // An icon at a world point; one that must stay in view sits on the box's edge.
            auto icon=[&](const int32_t* point,const uint8_t* color,bool pinned) {
                int32_t cx,cy;
                if(!map_cell(point,cx,cy))return;
                int x=ox+int(cx)*cell+cell/2-size/2,y=oy+int(cy)*cell+cell/2-size/2;
                if(pinned&&follow) {
                    if(x<bx)x=bx;
                    if(y<by)y=by;
                    if(x>bx+bw-size)x=bx+bw-size;
                    if(y>by+bh-size)y=by+bh-size;
                }
                fill(x,y,x+size,y+size,color);
            };
            const bool everything=m.flags&minimap_everything;
            if(map.goal>=0&&(everything||room_explored(map.goal)))icon(map.rooms[map.goal].center,m.colors[color_goal],true);
            for(uint16_t i=0;i<map.item_count;++i) {
                const auto& item=map.items[i];
                if(!item.lock)continue;
                const int16_t actor=m.item_actors?m.item_actors[i]:-1;
                if(actor>=0&&!active(size_t(actor)))continue;
                if(everything||room_explored(item.room))icon(item.position,m.colors[color_key],true);
            }
            for(uint16_t i=0;i<map.lock_count;++i) {
                const auto& lock=map.locks[i];
                const int16_t actor=m.lock_actors?m.lock_actors[i]:-1;
                if(actor>=0&&!active(size_t(actor)))continue;
                if(everything||room_explored(lock.room)||shown_at(lock.position))icon(lock.position,m.colors[color_lock],false);
            }
            if(on_map) {
                // A little larger than the icons, and blinking, so the room under it shows.
                const int mark=cell+1<3?3:cell+1;
                const int x=ox+int(pcx)*cell+cell/2-mark/2,y=oy+int(pcy)*cell+cell/2-mark/2;
                if(m.flags&minimap_full||(frame>>4&3)!=0)fill(x,y,x+mark,y+mark,m.colors[color_player]);
            }
        }
    }
};
}
