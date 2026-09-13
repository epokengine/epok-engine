#include "streaming-host/streaming.hpp"
#include <cstdio>
using namespace epok;
void reset(){
    stream_pool={};streaming_stats={};stream_host_file=-1;
    stream_lookup_started=stream_ready=stream_failed=stream_read_pending=false;
    music_data_owner=false;fake_host::size=4*65536;fake_host::reads=fake_host::position=0;
    fake_host::missing=fake_host::short_read=fake_host::corrupt=fake_host::seek_error=false;
}
int main(){
    psyqo::GPU gpu;reset();streaming_prepare();
    const auto* first=streaming_acquire(1,gpu);assert(first&&first[0]==1&&fake_host::reads==1);
    assert(!streaming_prefetch(2)&&fake_host::reads==1);
    const auto* second=streaming_acquire(2,gpu);assert(second&&second[0]==2);
    assert(!streaming_acquire(3,gpu)&&!stream_failed); // Both slots pinned.
    streaming_release(1);streaming_release(2);
    assert(streaming_acquire(3,gpu));streaming_release(3);
    assert(streaming_acquire(1,gpu));streaming_release(1); // Evicted page reloaded.
    assert(fake_host::reads==4&&streaming_stats.bytes==4*65536&&fake_cd::reads==0);
    reset();fake_host::missing=true;assert(!streaming_acquire(0,gpu)&&stream_failed&&fake_host::reads==0);
    reset();--fake_host::size;assert(!streaming_acquire(0,gpu)&&stream_failed&&fake_host::reads==0);
    reset();fake_host::short_read=true;assert(!streaming_acquire(0,gpu)&&stream_failed&&stream_pool.resident_count()==0);
    reset();fake_host::corrupt=true;assert(!streaming_acquire(0,gpu)&&stream_failed&&stream_pool.resident_count()==0);
    reset();streaming_lookup();fake_host::seek_error=true;assert(!streaming_acquire(0,gpu)&&stream_failed&&fake_host::reads==0);
    std::puts("PCDrv streaming: reads, pins, eviction, missing/truncated files, checksum and seek errors passed.");
}
