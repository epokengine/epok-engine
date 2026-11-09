#define EPOK_TEXT_REVEAL 1
#include "../../runtime/hud_core.hpp"
#include <cassert>
#include <cstdio>
using namespace epok;
static uint32_t key(const ActorData& value){return hud_core::render_node_key(value,123,456);}
int main(){
 ActorData text;text.rect.enabled=true;text.text.enabled=true;text.text.set_text("ABCD");
 text.text.visible_glyphs=0;const auto hidden=key(text);
 text.text.visible_glyphs=1;assert(key(text)!=hidden);
 const auto one=key(text);text.text.visible_glyphs=4;assert(key(text)!=one);
 const auto four=key(text);text.text.visible_glyphs=0;assert(key(text)==hidden&&key(text)!=four);
 text.text.visible_glyphs=4;
 const auto disabled=key(text);
 text.layout_element.minimum[0]=200.0;text.layout_element.stretch=9.0;
 text.layout_container.padding[0]=100.0;text.layout_container.columns=8;
 text.focusable.highlight[0]=42;text.focusable.neighbors[0]=12;
 assert(key(text)==disabled);
 text.layout_element.enabled=true;assert(key(text)!=disabled);
 text.layout_element.enabled=false;text.layout_container.enabled=true;assert(key(text)!=disabled);
 text.layout_container.enabled=false;text.focusable.enabled=true;assert(key(text)!=disabled);
 text.focusable.enabled=false;text.text.set_text("ABCE");assert(key(text)!=disabled);
 text.text.enabled=false;const auto no_text=key(text);text.text.visible_glyphs=1;assert(key(text)==no_text);
 text.rect.enabled=false;assert(key(text)==0);
 text.rect.enabled=true;text.active=false;const auto inactive=key(text);text.text.set_text("INACTIVE");assert(key(text)==inactive);
 text.active=true;assert(hud_core::render_node_key(text,123,456)!=hud_core::render_node_key(text,124,456));
 ActorData image;image.rect.enabled=true;image.image.enabled=true;
 const auto appearance=key(image),shape=hud_core::render_node_key(image,123,456,true);
 image.image.color[0]=64;image.image.borders[0]=7;
 assert(key(image)!=appearance&&hud_core::render_node_key(image,123,456,true)==shape);
 image.rect.size[0]=400.0;assert(hud_core::render_node_key(image,123,456,true)!=shape);
 const auto resized=hud_core::render_node_key(image,123,456,true);
 image.text.enabled=true;image.text.set_text("A");image.text.visible_glyphs=0;
 const auto unrevealed=hud_core::render_node_key(image,123,456,true);
 image.text.visible_glyphs=1;assert(hud_core::render_node_key(image,123,456,true)!=unrevealed);
 image.text.enabled=false;assert(hud_core::render_node_key(image,123,456,true)==resized);
 std::puts("HUD packet keys invalidate on reveal and shrink.");
}
