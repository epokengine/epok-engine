#pragma once
#include <cassert>
#include <cstdint>
#include <cstddef>
#include <functional>
#include <utility>
#include <cstring>
namespace fake_cd {
inline std::function<void()> pending;
inline bool idle = true, read_ok = true, hung = false, corrupt = false;
inline uint32_t now = 0, reads = 0, pauses = 0, idle_checks = 0;
inline void pump() {
  now += 1000;
  if (!hung && pending) {
    auto fn = std::move(pending);
    pending = {};
    idle = true;
    fn();
  }
}
}
namespace psyqo {
struct GPU {
  uint32_t now() const { return fake_cd::now; }
  void pumpCallbacks() { fake_cd::pump(); }
};
struct ISO9660Parser {
  struct DirEntry { enum { INVALID, FILE }; int type = INVALID; uint32_t LBA = 100;
#ifdef EPOK_TEST_STREAMING_ARCHIVE_FITS
    uint32_t size = 2 * 65536;
#else
    uint32_t size = 4 * 65536;
#endif
  };
  void getDirentry(const char *, DirEntry *entry, std::function<void(bool)> callback) {
    assert(fake_cd::idle && !fake_cd::pending);
    fake_cd::idle = false;
    fake_cd::pending = [entry, callback] { entry->type = DirEntry::FILE; callback(true); };
  }
};
}
namespace epok {
#ifdef EPOK_TEST_STREAMING_DISABLED
inline constexpr uint32_t stream_page_count = 0;
inline constexpr size_t stream_pool_pages = 0;
#elif defined(EPOK_TEST_STREAMING_ARCHIVE_FITS)
inline constexpr uint32_t stream_page_count = 2;
inline constexpr size_t stream_pool_pages = 2;
#else
inline constexpr uint32_t stream_page_count = 4;
inline constexpr size_t stream_pool_pages = 2;
#endif
inline constexpr const char *stream_archive_path = "GEOMETRY.BIN;1";
// Independently generated FNV1a32 for pages filled with bytes 0, 1, 2 and 3.
inline constexpr uint32_t stream_page_hashes[] = {1582341573u, 2715917765u, 2786893253u, 3818495429u};
struct Material {
  uint8_t color[3]; bool unlit; int32_t texture, blend; int16_t depth_bias; int32_t uv_scroll[2];
};
struct MeshQuad {
  uint16_t indices[4]; uint8_t face; int16_t normal[3]; Material material;
  uint32_t color_offset; int16_t uv[4][2]; bool packed_uv; uint16_t uvw[4];
};
struct MeshGeometry {
  mutable const int16_t (*vertices)[3] = nullptr;
  size_t vertex_count = 1;
  mutable const MeshQuad *quads = nullptr;
  size_t quad_count = 1;
  uint32_t stream_page = UINT32_MAX;
  uint16_t stream_vertex_offset = 0, stream_quad_offset = 8;
  const MeshGeometry *next = nullptr;
};
enum class CoordinateSpace:uint32_t { Model=0, World=1 };
enum class MeshDataState:uint32_t { Unavailable=0, Pending=1, Ready=2, Failed=3 };
enum class MeshVertexError:uint32_t {
  None=0,MissingGeometry=1,InvalidVertex=2,Pending=3,
  StreamFailed=4,InvalidCoordinateSpace=5,WorldUnavailable=6
};
struct Fixed {
  enum Raw { RAW };
  int32_t value=0;
  Fixed()=default;
  Fixed(int32_t input,Raw):value(input){}
  int32_t raw()const{return value;}
};
struct MeshVertexSample {
  bool success=false;
  MeshVertexError error=MeshVertexError::MissingGeometry;
  MeshDataState data_state=MeshDataState::Unavailable;
  Fixed position[3]{};
};
struct ActorData { const MeshGeometry* geometry=nullptr; };
inline bool skeletal_world_point(const ActorData&,const Fixed* model,Fixed* world) {
  if(!model||!world)return false;
  for(int axis=0;axis<3;++axis)world[axis]=Fixed(model[axis].raw()+4096,Fixed::RAW);
  return true;
}
inline bool music_active = false, music_requested = false, music_lookup = false,
            music_ready = false, music_boot_failed = false, music_data_owner = false;
struct Drive {
  bool isIdle() { ++fake_cd::idle_checks; return fake_cd::idle; }
  void readSectors(uint32_t sector, uint32_t sectors, void *buffer, std::function<void(bool)> callback) {
    assert(fake_cd::idle && !fake_cd::pending && !music_active && !music_lookup && music_data_owner);
    assert(sectors == 32);
    ++fake_cd::reads;
    fake_cd::idle = false;
    fake_cd::pending = [sector, buffer, callback] {
      if (fake_cd::read_ok) std::memset(buffer, int((sector - 100) / 32), 65536);
      if (fake_cd::corrupt) static_cast<uint8_t *>(buffer)[1234] ^= 1;
      callback(fake_cd::read_ok);
    };
  }
};
inline Drive music_drive;
inline psyqo::ISO9660Parser music_files;
inline void music_prepare(bool) {}
inline void music_tick() {
  music_ready = true;
  if (music_data_owner && music_active) { music_active = false; ++fake_cd::pauses; }
  else if (!music_data_owner && music_requested) music_active = true;
}
}
