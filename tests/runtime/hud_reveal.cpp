// Run the complete existing HUD contract with reveal enabled, then compare every
// partial draw with a prefix of the full UTF-8/markup draw at identical positions.
#define EPOK_TEXT_REVEAL 1
#define main full_hud_contract
#include "hud_layout.cpp"
#undef main
static_assert(sizeof(epok::Text)==528,"Reveal must fit the existing Text padding");
int main(){
    full_hud_contract();
    for(auto alignment:{TextAlign::Left,TextAlign::Center,TextAlign::Right}) {
        canvas_with(1,64,64);
        auto& text=objects[1].text;text.enabled=true;text.align=alignment;
        text.set_text("\xc3\x81[color=#ff0000]BC[/color]\nD");
        assert(run(2,true).glyphs==4);const auto full=sink.commands;
        for(uint16_t count=0;count<=4;++count) {
            text.visible_glyphs=count;
            const auto stats=run(2,true);
            assert(stats.glyphs==count&&stats.dropped==0&&sink.commands.size()==count);
            for(unsigned i=0;i<count;++i)assert(sink.commands[i]==full[i]);
        }
        text.visible_glyphs=0xffffu;
        assert(run(2,true).glyphs==4&&sink.commands==full);
        CachedRecorder cached;
        const uint16_t limits[]={0,1,3,2,4,0xffffu,0,4};
        for(unsigned frame=0;frame<48;++frame){
            text.visible_glyphs=limits[frame/6];
            const auto expected=run(2,true);const auto commands=sink.commands;
            cached.parity=frame&1;cached.cache.rebuild(cached.parity);cached.commands.clear();
            hud_core::Compiler compiler(cached,screen_width,screen_height,{64,256,64,1024,128});
            compiler.draw(objects,2,first,next,measured,rects,transforms);
            assert(cached.commands==commands&&compiler.stats.glyphs==expected.glyphs&&compiler.stats.dropped==0);
        }
        assert(cached.hits>10);
    }
    std::puts("Text reveal: complete layout, UTF-8 scalar boundaries, color markup and alignment stay identical; no additional Text storage.");
}
