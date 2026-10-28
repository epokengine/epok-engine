#define EPOK_HAS_SEQUENCES 1
#include <cassert>
#include <cstdint>
#include <cstring>
#include <cstdio>
namespace epok {
inline unsigned pause_depth=0,guarded_calls=0;
struct SequenceStoragePause {
  SequenceStoragePause(){++pause_depth;}
  ~SequenceStoragePause(){--pause_depth;}
};
inline constexpr unsigned scene_bank_count=1,scene_file_count=1;
struct Bank {int file;};
inline constexpr Bank scene_banks[]={{0}};
struct Record {uint32_t bytes,hash;};
inline Record scene_file_records[]={{32768,0}};
inline constexpr const char* scene_file_names[]={"SCENE00.BIN"};
extern "C" {
uint8_t __epok_scene_region_start[32768]{};
uint8_t __epok_scene_region_end[1]{};
}
}
#include "scene-files-host/scene_files.hpp"
using namespace epok;
static bool missing=false,bad_size=false,short_read=false,corrupt=false;
static unsigned reads=0,closes=0;
static void guarded(){assert(pause_depth>0);++guarded_calls;}
int PCinit(){guarded();return 0;}
int PCopen(const char* path,int flags,int){
  guarded();assert(std::strcmp(path,"SCENE00.BIN")==0&&flags==0);return missing?-1:7;
}
int PCclose(int fd){guarded();assert(fd==7);++closes;return 0;}
int PClseek(int fd,int offset,int mode){
  guarded();assert(fd==7&&offset==0);return mode==PCDRV_SEEK_END?(bad_size?32767:32768):0;
}
int PCread(int fd,void* buffer,int bytes){
  guarded();assert(fd==7&&bytes==16384);++reads;std::memset(buffer,0,bytes);
  if(corrupt)static_cast<uint8_t*>(buffer)[0]=1;
  return short_read?bytes-1:bytes;
}
static void reset(){
  assert(pause_depth==0);scene_file_stats={};scene_file_load={};
  scene_file_stats.state=SceneFileState::Waiting;scene_file_stats.loading=0;scene_file_stats.total=32768;
  missing=bad_size=short_read=corrupt=false;reads=closes=guarded_calls=0;
  std::memset(__epok_scene_region_start,0,32768);
  scene_file_records[0].hash=streaming_hash_slice(2166136261u,__epok_scene_region_start,32768);
}
int main(){
  reset();scene_file_service(1000);assert(reads==1&&pause_depth==0&&scene_file_stats.loaded==16384);
  scene_file_service(2000);assert(reads==2&&closes==1&&pause_depth==0);
  assert(scene_file_stats.state==SceneFileState::Ready&&scene_file_stats.loads==1&&guarded_calls>=7);
  reset();missing=true;scene_file_service(1000);
  assert(scene_file_stats.error==SceneFileError::Missing&&reads==0&&closes==0&&pause_depth==0);
  reset();bad_size=true;scene_file_service(1000);
  assert(scene_file_stats.error==SceneFileError::Size&&reads==0&&closes==1&&pause_depth==0);
  reset();short_read=true;scene_file_service(1000);
  assert(scene_file_stats.error==SceneFileError::Read&&reads==1&&closes==1&&pause_depth==0);
  reset();corrupt=true;scene_file_service(1000);scene_file_service(2000);
  assert(scene_file_stats.error==SceneFileError::Checksum&&reads==2&&closes==1&&pause_depth==0);
  // A failure can close an existing handle outside the normal service call.
  reset();scene_file_load.host=7;scene_file_fail(SceneFileError::Read);
  assert(closes==1&&pause_depth==0);
  std::puts("External scene PCDrv: MIDI storage guards, chunked reads and failure cleanup passed.");
}
