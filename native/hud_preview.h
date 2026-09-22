#pragma once
#include <stdint.h>

// Versioned wire/FFI records contain fixed-width values, never pointers or C++
// object layouts. Coordinates are Q12 at the boundary and pixels in commands.

// Native preview protocol. Bump on any change to the frame header or command
// record; both sides check it (the editor in src/hud_native.rs, the child in
// native/hud_runner.cpp) so a stale cached executable is rejected rather than
// misread. Version 2 added the version and capability words to the header.
// Version 3 widened EpokHudNode with the layout element and container records.
// Version 4 added image tiling, the focus record and the rect's rotation, and
// widened EpokHudCommand for the rotated quad kinds.
#define EPOK_HUD_PREVIEW_MAGIC 0x31445548u   // "HUD1"
#define EPOK_HUD_PREVIEW_PROTOCOL_VERSION 4u
// Capabilities the child reports in every frame header. The editor shows a
// diagnostic instead of pretending an absent capability worked.
//   SIMULATE  the session runs gameplay: BeginPlay/start, update, frame_update.
//             Clear in the edit phase, which only calls editor_preview hooks.
//   BLUEPRINT Blueprint classes execute. Never set: the native preview compiles
//             C++ controllers only, so Blueprint logic is not simulated here.
#define EPOK_HUD_PREVIEW_CAP_SIMULATE 1u
#define EPOK_HUD_PREVIEW_CAP_BLUEPRINT 2u
// rect: anchor min/max, pivot, position and size as five Q12 vec2s, then the
// rotation in Q12 degrees.
// layout_element: horizontal flags, vertical flags, minimum x/y and stretch in Q12.
// layout_container: kind, spacing x/y, padding left/top/right/bottom, columns.
// focusable: enabled, four neighbour actor indices (left, right, up, down,
// -1 for none), tab order, highlight r/g/b.
// canvas_focused: the focused actor index of this node's canvas, or -1.
// flags bit 256 enables the element, 512 the container, 1024 the focus record;
// bits 1..128 are taken.
struct EpokHudNode {
    int32_t parent,flags,rect[11],texture,tiling,image_color[3],region[4],borders[4],text_color[3],progress[7],layout_element[5],layout_container[8],focusable[9],canvas_focused;
    char text[512];
};
// Command kinds: 0 rectangle, 1 image, 2 glyph, 3 quad, 4 textured quad,
// 5 rotated glyph. The first three fill v[3..6] with the screen box and
// v[7..10] with the source box; the rotated three fill v[3..10] with four
// screen corners and v[14..21] with their four source corners. Every kind puts
// its colour in v[11..13].
struct EpokHudCommand { int32_t v[24]; };
extern "C" {
uint32_t epok_hud_compile(const EpokHudNode*,uint32_t,const int32_t*,uint32_t,int32_t,int32_t,const uint32_t*,EpokHudCommand*,uint32_t,uint32_t*);
// Measure and arrange only: writes count*12 Q12 ints per node — the
// axis-aligned rect (x,y,w,h) then its four transformed corners — zeroed for a
// node that is not laid out, and returns the number of nodes written.
uint32_t epok_hud_layout(const EpokHudNode*,uint32_t,int32_t,int32_t,int32_t*);
void epok_hud_resolve(const int32_t*,const int32_t*,int32_t*);
}
