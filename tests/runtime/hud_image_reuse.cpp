// Reuse the common compiler's full reference recorder and scene fixture. Only
// the image cache cases below execute from this separate test binary.
#define main epok_hud_layout_reference_entry
#include "hud_layout.cpp"
#undef main

namespace {
using ImageCache=hud_core::ImageReuseCache<8>;
static_assert(sizeof(ImageCache)+sizeof(void*)+sizeof(size_t)<=1024);
struct ImageRecorder:Recorder {
    ImageCache cache;size_t entity_count=0;unsigned parity=0;std::vector<Draw> saved[2];
    uint32_t keys[capacity]={};unsigned last_patch_count=0,last_patch_owner=0;
    void update_keys(){for(size_t i=0;i<entity_count;++i)keys[i]=hud_core::render_node_key(objects[i],123,456);}
    bool begin_node(int owner,const Rect& rect,const Affine2& transform,const uint8_t*,bool,HudStats&){
        cache.capture(parity,size_t(owner),objects,entity_count,rect,transform,uint32_t(rectangle_count()),123,456);return false;
    }
    size_t rectangle_count()const{size_t count=0;for(const auto& command:commands)count+=command.kind==0;return count;}
    void end_node(int owner,const HudStats&){cache.end(parity,size_t(owner),uint32_t(rectangle_count()));}
    void full(unsigned p,size_t count){
        parity=p;entity_count=count;commands.clear();cache.begin(p);update_keys();
        hud_core::Compiler compiler(*this,screen_width,screen_height,{64,256,64,1024,128});
        compiler.draw(objects,count,first,next,measured,rects,transforms);
        cache.seal(p,objects,count,keys,123,456);saved[p]=commands;
    }
    bool patch(unsigned p){
        update_keys();ImageCache::Patch patches[8];unsigned count=0;
        std::memset(patches,0xa5,sizeof(patches));
        if(!cache.prepare(p,objects,entity_count,keys,123,456,screen_width,screen_height,patches,count))return false;
        // The new POD array is poisoned, not zeroed. A successful prepare
        // may not construct/write any slot outside its reported count.
        for(unsigned n=count;n<8;++n)for(size_t byte=0;byte<sizeof(patches[n]);++byte)
            assert(reinterpret_cast<const unsigned char*>(&patches[n])[byte]==0xa5);
        last_patch_count=count;last_patch_owner=count?patches[0].owner:0;
        for(unsigned n=0;n<count;++n)for(unsigned k=0;k<patches[n].rectangles.count;++k){
            size_t rectangle=0;bool found=false;
            for(auto& command:saved[p])if(command.kind==0){if(rectangle++==patches[n].first+k){
                assert(command.owner==int(patches[n].owner));const auto& next=patches[n].rectangles.values[k];
                command.x0=next.x0;command.y0=next.y0;command.x1=next.x1;command.y1=next.y1;
                for(unsigned c=0;c<3;++c)command.color[c]=next.color[c];found=true;break;
            }}assert(found);
        }
        for(unsigned n=0;n<count;++n)cache.applied(p,patches[n],keys[patches[n].owner]);
        return true;
    }
};
void border(size_t owner){
    objects[owner].image.enabled=true;objects[owner].image.texture=-1;
    for(unsigned axis=0;axis<2;++axis){objects[owner].rect.anchor_min[axis]=objects[owner].rect.anchor_max[axis]=objects[owner].rect.pivot[axis]=0.0;}
    objects[owner].rect.position[0]=10.25;objects[owner].rect.position[1]=20.5;
    for(auto& edge:objects[owner].image.borders)edge=3;
}
void golden_edges(){
    Image image;image.color[0]=20;image.color[1]=30;image.color[2]=40;
    image.borders[0]=3;image.borders[1]=4;image.borders[2]=5;image.borders[3]=6;
    hud_core::ImageRectangles output;output.build({10.0,20.0,100.0,60.0},image,320,240);
    const int expected[4][4]={{10,160,110,164},{10,214,110,220},{10,164,13,214},{105,164,110,214}};
    assert(output.count==4);for(unsigned n=0;n<4;++n){const auto& r=output.values[n];
        assert(r.x0==expected[n][0]&&r.y0==expected[n][1]&&r.x1==expected[n][2]&&r.y1==expected[n][3]);
        assert(r.color[0]==20&&r.color[1]==30&&r.color[2]==40);
    }
}
void parity_and_reference(){
    canvas_with(3,100,60);border(1);border(2);
    objects[2].rect.position[0]=-0.5;objects[2].rect.position[1]=220.25;
    objects[3].text.enabled=true;objects[3].text.set_text("UNCHANGED FOCUS");objects[3].focusable.enabled=true;
    ImageRecorder image;image.full(0,4);image.full(1,4);
    unsigned patched=0,fallback=0;
    for(unsigned frame=2;frame<240;++frame){
        const unsigned parity=frame&1;
        for(unsigned owner=1;owner<=2;++owner){for(auto& edge:objects[owner].image.borders)edge=uint16_t(frame%6+1);objects[owner].image.color[0]=uint8_t(frame);}
        if(frame%13==0)objects[3].text.color[0]=uint8_t(frame);
        if(frame%17==0)objects[0].canvas.focused=objects[0].canvas.focused==3?-1:3;
        if(frame%29==0)objects[1].rect.position[0]+=1.0;
        const auto untouched=image.saved[1-parity];
        if(image.patch(parity))++patched;else{++fallback;image.full(parity,4);}
        run(4,true);assert(image.saved[parity]==sink.commands);assert(image.saved[1-parity]==untouched);
    }
    assert(patched>150&&fallback>20);
    // A fill/frame topology change, visibility, rotation, bank and text changes
    // must each take the full compiler path before their packets can be reused.
    for(auto& edge:objects[1].image.borders)edge=0;assert(!image.patch(0));image.full(0,4);
    objects[1].image.color[1]=17;assert(image.patch(0));run(4,true);assert(image.saved[0]==sink.commands);
    objects[1].active=false;assert(!image.patch(0));image.full(0,4);
    objects[1].active=true;assert(!image.patch(0));image.full(0,4);
    objects[1].rect.rotation=17.0;assert(!image.patch(0));image.full(0,4);
    objects[1].image.color[2]=9;assert(!image.patch(0));
    objects[1].rect.rotation=0.0;image.full(0,4);
    objects[1].image.texture=0;assert(!image.patch(0));
    image.cache.invalidate();assert(!image.patch(1));
}
void capacity_and_nonleaf(){
    canvas_with(10,20,20);for(size_t n=1;n<=10;++n)border(n);
    ImageRecorder image;image.full(0,11);
    objects[9].image.color[0]=1;assert(!image.patch(0));
    canvas_with(2,100,60);border(1);objects[2].parent=1;
    image.full(0,3);objects[1].image.color[0]=1;assert(!image.patch(0));
}
void unchanged_images_and_order(){
    canvas_with(3,100,60);border(1);border(2);
    // DFS emits owner2 before owner1: root children2/3, with1 under3.
    objects[1].parent=3;objects[3].parent=0;
    ImageRecorder image;image.full(0,4);image.full(1,4);
    assert(image.patch(0)&&image.last_patch_count==0);
    objects[1].image.color[0]=23;
    assert(image.patch(0)&&image.last_patch_count==1&&image.last_patch_owner==1);
    run(4,true);assert(image.saved[0]==sink.commands);
    assert(image.patch(0)&&image.last_patch_count==0);
    // Other parity has not committed the first update yet.
    assert(image.patch(1)&&image.last_patch_count==1&&image.last_patch_owner==1);
    objects[2].image.color[1]=45;
    assert(image.patch(0)&&image.last_patch_count==1&&image.last_patch_owner==2);
    run(4,true);assert(image.saved[0]==sink.commands);
    // A changed first image plus invalid topology in the second must not
    // commit any full key or write any packet from a failed prepare.
    objects[1].image.color[0]=87;for(auto& edge:objects[2].image.borders)edge=0;
    const auto previous=image.saved[0];assert(!image.patch(0));assert(image.saved[0]==previous);
    for(auto& edge:objects[2].image.borders)edge=3;
    assert(image.patch(0)&&image.last_patch_count==1&&image.last_patch_owner==1);
    run(4,true);assert(image.saved[0]==sink.commands);
    objects[3].rect.position[0]+=1.0;assert(!image.patch(0));
    image.full(0,4);objects[1].image.color[0]=55;assert(image.patch(0));
    // Appearance A -> B -> A must be committed independently on each parity.
    objects[1].image.color[0]=87;assert(image.patch(0)&&image.last_patch_count==1);
    assert(image.patch(0)&&image.last_patch_count==0);
    run(4,true);assert(image.saved[0]==sink.commands);
}

void all_patch_slots_and_atomic_failure(){
    canvas_with(8,100,60);for(size_t n=1;n<=8;++n)border(n);
    ImageRecorder image;image.full(0,9);image.full(1,9);
    for(size_t n=1;n<=8;++n)objects[n].image.color[0]=uint8_t(80+n);
    assert(image.patch(0)&&image.last_patch_count==8);run(9,true);assert(image.saved[0]==sink.commands);
    const auto saved=image.saved[0];objects[1].image.color[0]=99;
    for(auto& edge:objects[8].image.borders)edge=0;
    assert(!image.patch(0));assert(image.saved[0]==saved);
    for(auto& edge:objects[8].image.borders)edge=3;
    assert(image.patch(0)&&image.last_patch_count==1);run(9,true);assert(image.saved[0]==sink.commands);
    assert(image.patch(1)&&image.last_patch_count==8);run(9,true);assert(image.saved[1]==sink.commands);
}

}
int main(){golden_edges();parity_and_reference();capacity_and_nonleaf();unchanged_images_and_order();all_patch_slots_and_atomic_failure();std::puts("HUD appearance reuse: golden edges,238reference frames/parity, topology/focus/layout/bank/capacity fallbacks passed.");}
