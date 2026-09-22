#pragma once
#include <stdint.h>

// Versioned wire/FFI records contain fixed-width values, never pointers or C++
// object layouts. Coordinates are Q12 at the boundary and pixels in commands.

// Native preview protocol. Bump on any change to the frame header or command
// record; both sides check it (the editor in src/hud_native.rs, the child in
// native/hud_runner.cpp) so a stale cached executable is rejected rather than
// misread. Version 2 added the version and capability words to the header.
// Version 3 widened EpokHudNode with the layout element and container records.
#define EPOK_HUD_PREVIEW_MAGIC 0x31445548u   // "HUD1"
#define EPOK_HUD_PREVIEW_PROTOCOL_VERSION 3u
// Capabilities the child reports in every frame header. The editor shows a
// diagnostic instead of pretending an absent capability worked.
//   SIMULATE  the session runs gameplay: BeginPlay/start, update, frame_update.
//             Clear in the edit phase, which only calls editor_preview hooks.
//   BLUEPRINT Blueprint classes execute. Never set: the native preview compiles
//             C++ controllers only, so Blueprint logic is not simulated here.
#define EPOK_HUD_PREVIEW_CAP_SIMULATE 1u
#define EPOK_HUD_PREVIEW_CAP_BLUEPRINT 2u
// layout_element: horizontal flags, vertical flags, minimum x/y and stretch in Q12.
// layout_container: kind, spacing x/y, padding left/top/right/bottom, columns.
// flags bit 256 enables the element, bit 512 the container; bits 1..128 are taken.
struct EpokHudNode {
    int32_t parent,flags,rect[10],texture,image_color[3],region[4],borders[4],text_color[3],progress[7],layout_element[5],layout_container[8];
    char text[512];
};
struct EpokHudCommand { int32_t v[16]; };
extern "C" {
uint32_t epok_hud_compile(const EpokHudNode*,uint32_t,const int32_t*,uint32_t,int32_t,int32_t,const uint32_t*,EpokHudCommand*,uint32_t,uint32_t*);
// Measure and arrange only: writes count*4 Q12 ints (x,y,w,h), zeroed for a node
// that is not laid out, and returns the number of nodes written.
uint32_t epok_hud_layout(const EpokHudNode*,uint32_t,int32_t,int32_t,int32_t*);
void epok_hud_resolve(const int32_t*,const int32_t*,int32_t*);
}
