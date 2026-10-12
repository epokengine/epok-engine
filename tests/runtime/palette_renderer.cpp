#include <cassert>
#include <cstdio>
#include "palette-renderer/palette.hpp"
using namespace epok;
struct Object {PaletteAnimator palette_animator;};
int main(){
    alignas(4) uint16_t pixels[2]={},small[16]={},large[256]={};
    for(unsigned i=1;i<16;++i)small[i]=uint16_t(0x8000+i);
    for(unsigned i=1;i<256;++i)large[i]=uint16_t(0x4000+i);
    // Far beyond the old 32-texture limit, with CLUTs on the same arbitrary row.
    texture_assets[195]={16,1,650,270,4,640,73,pixels,small,4};
    texture_assets[195].pixels=nullptr;texture_assets[195].stream_page=3; // pixels no longer CPU-resident
    texture_assets[196]={16,1,660,270,8,656,73,pixels,large,8};
    Object objects[197];PaletteRenderer renderer;psyqo::GPU gpu;
    renderer.upload(gpu,objects,197);
    assert(gpu.uploads.empty()&&gpu.waits==0); // static palettes uploaded by initialization only
    for(unsigned i=195;i<197;++i){
        active_slots[i]=true;
        auto& a=objects[i].palette_animator;a.enabled=true;a.texture=int16_t(i);a.first=1;a.last=4;a.offset=1;
    }
    renderer.upload(gpu,objects,197);
    assert(gpu.waits==1&&gpu.uploads.size()==2&&palette_stats.bytes==544);
    assert(gpu.uploads[0].x==640&&gpu.uploads[0].y==73&&gpu.uploads[0].colors.size()==16);
    assert(gpu.uploads[1].x==656&&gpu.uploads[1].colors.size()==256);
    assert(gpu.uploads[0].colors[0]==0&&gpu.uploads[0].colors[1]==small[2]);
    assert(gpu.uploads[1].colors[1]==large[2]);
    renderer.upload(gpu,objects,197);assert(palette_stats.uploads==0&&gpu.waits==1);
    objects[196].palette_animator.texture=195;
    renderer.upload(gpu,objects,197);
    assert(palette_stats.conflicts==1&&palette_stats.uploads==1);
    assert(gpu.uploads.back().x==656&&gpu.uploads.back().colors[1]==large[1]); // restore removed owner
    objects[195].palette_animator.reverse=true;
    objects[196].palette_animator.enabled=false;
    renderer.upload(gpu,objects,197);assert(gpu.uploads.back().colors[1]==small[4]);
    objects[195].palette_animator.last=31;objects[195].palette_animator.offset=16;
    objects[195].palette_animator.reverse=false;
    renderer.upload(gpu,objects,197);assert(gpu.uploads.back().colors[1]==0);
    active_slots[195]=false;
    renderer.upload(gpu,objects,197);assert(gpu.uploads.back().colors[1]==small[1]);
    renderer.upload(gpu,objects,197);assert(palette_stats.uploads==0);
    active_slots[195]=true;renderer.upload(gpu,objects,197);renderer.clear();
    active_slots[195]=false;renderer.upload(gpu,objects,197);assert(palette_stats.uploads==0);
    // Shared runtime UV calculation must account for both word formats and local origins.
    assert(texture_packed_uv(texture_assets[195],0,0)==uint16_t(40|(14<<8)));
    assert(texture_packed_uv(texture_assets[196],4096,4096)==uint16_t(55|(14<<8)));
    std::puts("Packed CLUT positions, 4/8-bit uploads, sparse animation cache and UV offsets passed.");
}
