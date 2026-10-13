#include <cassert>
#include <cstdio>
#include <vector>
#include "streaming-backend/streaming.hpp"
#include "texture_upload.hpp"
using namespace epok;
static void reset(){
    stream_pool={};streaming_stats={};texture_load_stats={};
    stream_lookup_started=stream_lookup_pending=stream_ready=stream_failed=stream_read_pending=false;
    music_active=music_requested=music_lookup=music_ready=music_boot_failed=music_booting=music_data_owner=false;
    fake_cd::pending={};fake_cd::idle=fake_cd::read_ok=true;
    fake_cd::hung=fake_cd::corrupt=false;fake_cd::now=fake_cd::reads=0;
}
int main(){
    alignas(4) uint16_t palette4[16]={},palette8[256]={},resident[8]={};
    for(unsigned i=0;i<16;++i)palette4[i]=uint16_t(i+40);
    for(unsigned i=0;i<256;++i)palette8[i]=uint16_t(i+100);
    for(auto& word:resident)word=0x2323;
    const Texture bank[]={
        {8,2,640,0,2,640,300,nullptr,palette4,4,0,65528}, // exact page end
        {8,2,704,0,4,768,301,nullptr,palette8,8,1,16},
        {8,2,768,0,4,768,302,resident,palette8,8},{}
    };
    psyqo::GPU gpu;std::vector<uint16_t> vram(1024*512);
    uint32_t pinned=UINT32_MAX,uploads=0;bool pixels_uploaded=false,fail_second=false;
    auto acquire=[&](uint32_t page){
        assert(pinned==UINT32_MAX);if(fail_second&&page==1)fake_cd::read_ok=false;
        const auto* data=streaming_acquire(page,gpu);
        if(data){pinned=page;pixels_uploaded=false;}return data;
    };
    auto release=[&](uint32_t page){
        assert(page==pinned&&pixels_uploaded);streaming_release(page);
        assert(!stream_pool.unpin(page));pinned=UINT32_MAX;
    };
    auto upload=[&](const uint16_t* data,uint16_t x,uint16_t y,uint16_t width,uint16_t height){
        if(y==0&&x!=768){assert(pinned==(x==640?0u:1u));pixels_uploaded=true;}
        else assert(pinned==UINT32_MAX); // palette upload never retains a borrowed page
        for(unsigned row=0;row<height;++row)for(unsigned col=0;col<width;++col)
            vram[(y+row)*1024+x+col]=data[row*width+col];
        ++uploads;
    };
    auto load=[&](){return upload_texture_bank(bank,4,stream_page_count,stream_page_bytes,acquire,release,upload);};
    reset();assert(load()&&uploads==6&&fake_cd::reads==2);
    assert(vram[640]==0&&vram[704]==0x0101&&vram[768]==0x2323);
    assert(vram[300*1024+643]==palette4[3]);
    assert(vram[301*1024+771]==palette8[3]);
    // Geometry can replace every upload buffer; VRAM and CPU palettes survive.
    for(uint32_t page:{2u,3u}){assert(streaming_acquire(page,gpu));streaming_release(page);}
    assert(!stream_pool.ready(0)&&!stream_pool.ready(1)&&vram[704]==0x0101);
    vram[300*1024+643]=999;assert(load()); // next bank activation restores palette/pixels
    assert(vram[300*1024+643]==palette4[3]);
    const auto count=uploads;
    auto invalid=bank[1];invalid.stream_offset=65532;
    assert(!upload_texture_bank(&invalid,1,4,65536,acquire,release,upload));
    assert(texture_load_stats.error==TextureLoadError::Descriptor&&uploads==count);
    invalid=bank[1];invalid.pixels=resident;
    assert(!upload_texture_bank(&invalid,1,4,65536,acquire,release,upload));
    invalid=bank[1];invalid.stream_page=4;
    assert(!upload_texture_bank(&invalid,1,4,65536,acquire,release,upload));
    invalid=bank[1];invalid.stream_offset=2;
    assert(!upload_texture_bank(&invalid,1,4,65536,acquire,release,upload));
    reset();fail_second=true;assert(!load()&&stream_failed&&pinned==UINT32_MAX);
    assert(texture_load_stats.error==TextureLoadError::Read&&!stream_pool.unpin(0));
    assert(streaming_retry());fail_second=false;fake_cd::read_ok=true;assert(load());
    reset();fake_cd::corrupt=true;assert(!load()&&stream_failed&&pinned==UINT32_MAX);
    assert(streaming_retry());fake_cd::corrupt=false;assert(load());
    reset();fake_cd::hung=true;assert(!load()&&stream_failed&&streaming_stats.timeouts==1);
    assert(!streaming_retry()); // in-flight lookup still owns the callback
    fake_cd::hung=false;gpu.pumpCallbacks();streaming_tick();assert(streaming_retry());assert(load());
    stream_failed=true;music_active=true;assert(!streaming_retry());music_active=false;
    music_boot_failed=music_booting=true;assert(streaming_retry());assert(!music_boot_failed&&!music_booting);
    assert(pinned==UINT32_MAX);
    std::puts("Texture upload: real page backend, 4/8-bit VRAM, leases, eviction, reload, malformed banks, read/checksum/timeout recovery passed.");
}
