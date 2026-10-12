// Cross-compiled by tools/test_gte_transfer_codegen.py; no PSX hardware stub.
#include "gte_geometry.hpp"

extern "C" __attribute__((noinline)) void load_triple(const epok::GteProjectionInput* inputs) {
    epok::begin_projection_triple(inputs);
}
extern "C" __attribute__((noinline)) bool store_triple(uint32_t* screen,int32_t* depth) {
    return epok::finish_projection_triple(screen,depth);
}
extern "C" __attribute__((noinline)) bool project_single(
    const epok::GteProjectionInput& input,uint32_t& screen,int32_t& depth) {
    return epok::project_packed_vertex(input,screen,depth);
}
extern "C" __attribute__((noinline)) bool load_direct(
    const int32_t* world,const int32_t* offset) {
    bool fits=epok::load_projection_input<0>(world,offset);
    fits&=epok::load_projection_input<1>(world+3,offset);
    fits&=epok::load_projection_input<2>(world+6,offset);
    if(fits)epok::begin_loaded_projection_triple();
    return fits;
}
