#pragma once
#include <stdint.h>
namespace epok {
// Snapshot published after each rendered frame. Pixel coverage counts submitted
// sprite triangle area before transparent texel rejection; it estimates fill cost.
struct ResourceUsage {
    uint32_t alive_slots=0,active_slots=0,slot_capacity=0,scene_banks=0;
    uint32_t textures=0,vram_texture_words=0,vram_palette_words=0;
    uint32_t resident_texture_bytes=0,active_texture_bytes=0;
    uint32_t particles=0,particle_peak=0,particle_dropped=0;
    uint32_t mesh_triangles=0,sprite_triangles=0,dropped_primitives=0;
    uint32_t sprite_estimated_pixels=0,sprite_overdraw_per_mille=0,frame_scanlines=0;
};
inline ResourceUsage resource_usage;
}
