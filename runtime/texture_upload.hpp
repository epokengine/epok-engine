#pragma once
#include <stddef.h>
#include "texture_types.hpp"
namespace epok {
enum class TextureLoadError:uint32_t {None,Descriptor,Read};
struct TextureLoadStats {uint32_t attempts=0,uploads=0,bytes=0,failures=0;TextureLoadError error=TextureLoadError::None;};
inline TextureLoadStats texture_load_stats;

// Acquire returns a pinned, checksum-verified page; upload MUST be blocking.
// The lease ends only after the pixel DMA completes. Nothing retains CPU pixel
// pointers after this function returns. Palettes remain resident for animation.
template<class Acquire,class Release,class Upload>
bool upload_texture_bank(const Texture* bank,size_t count,uint32_t pages,uint32_t page_bytes,
                         Acquire acquire,Release release,Upload upload){
    ++texture_load_stats.attempts;texture_load_stats.error=TextureLoadError::None;
    auto fail=[](TextureLoadError error){++texture_load_stats.failures;texture_load_stats.error=error;return false;};
    if(!bank&&count)return fail(TextureLoadError::Descriptor);
    // Reject a malformed bank before changing any VRAM. Absent global-bank
    // descriptors are valid holes, not zero-size uploads.
    for(size_t i=0;i<count;++i){
        const auto& t=bank[i];if(!texture_present(t)){
            if(t.width||t.height||t.word_width||t.palette)return fail(TextureLoadError::Descriptor);
            continue;
        }
        if(!t.width||t.width>256||!t.height||t.height>256||!t.palette||
           (t.bits!=4&&t.bits!=8)||!t.word_width||(t.word_width&1)||
           t.word_width*unsigned(16/t.bits)<t.width||
           uint32_t(t.x)+t.word_width>1024||uint32_t(t.y)+t.height>512||
           uint32_t(t.clut_x)+texture_colors(t)>1024||t.clut_y>=512||
           (t.clut_x&15)||(t.x%64)*unsigned(16/t.bits)+t.width>256||(t.y%256)+t.height>256)
            return fail(TextureLoadError::Descriptor);
        if(t.stream_page!=UINT32_MAX&&(t.pixels||t.stream_page>=pages||(t.stream_offset&3)||
            uint64_t(t.stream_offset)+uint64_t(t.word_width)*t.height*2>page_bytes))
            return fail(TextureLoadError::Descriptor);
    }
    for(size_t i=0;i<count;++i){
        const auto& t=bank[i];if(!texture_present(t))continue;
        const uint16_t* pixels=t.pixels;
        if(t.stream_page!=UINT32_MAX){
            const auto* data=acquire(t.stream_page);
            if(!data)return fail(TextureLoadError::Read);
            pixels=reinterpret_cast<const uint16_t*>(data+t.stream_offset);
        }
        upload(pixels,t.x,t.y,t.word_width,t.height);
        if(t.stream_page!=UINT32_MAX)release(t.stream_page);
        upload(t.palette,t.clut_x,t.clut_y,texture_colors(t),1);
        texture_load_stats.uploads+=2;
        texture_load_stats.bytes+=2*(uint32_t(t.word_width)*t.height+texture_colors(t));
    }
    return true;
}
}
