#include "timeline.hpp"
#include "psyqo/fixed-point.hh"
#include <cassert>
#include <climits>
#include <cstdio>

int main() {
    using namespace epok::timeline;
    const Key keys[] = {{0, -40960}, {2048, 0}, {4096, 40960}};
    for (int tick = 0; tick <= 4200; tick += 68) {
        const int clamped = tick > 4096 ? 4096 : tick;
        const auto expected = -40960 + int64_t(81920) * clamped / 4096;
        assert(sample({keys, 3}, tick) == expected);
    }
    assert(sample({keys, 3}, -1) == -40960);
    assert(sample({keys, 3}, 2048) == 0);
    const Key extremes[] = {{0, INT32_MIN}, {INT32_MAX, INT32_MAX}};
    assert(sample({extremes, 2}, INT32_MAX / 2) == -2);
    const Key descending[] = {{0, 4096}, {3, -4096}};
    assert(sample({descending, 2}, 1) == 1366); // Signed division toward zero.
    const Key easing[]={{0,0},{4096,4096}};
    const int expected[]={1024,0,640,256,1792};
    const Key unsigned_keys[]={{0,0},{4096,-1}};
    for(int mode=0;mode<5;++mode){
        assert(sample({easing,2,Interpolation(mode)},1024)==expected[mode]);
        assert(sample({easing,2,Interpolation(mode)},4096)==4096);
        uint32_t previous=0;
        for(int tick=0;tick<=4096;++tick){
            const uint32_t value=uint32_t(sample({unsigned_keys,2,Interpolation(mode),true},tick));
            assert(value>=previous);
            previous=value;
        }
        assert(previous==UINT32_MAX);
    }
    const Marker markers[] = {{0, 9}, {68, 2}, {68, 3}, {4096, 4}};
    uint16_t cursor = 0, id = 0;
    assert(poll_marker(markers, 4, cursor, 68, id) && id == 9);
    assert(poll_marker(markers, 4, cursor, 68, id) && id == 2);
    assert(poll_marker(markers, 4, cursor, 68, id) && id == 3);
    assert(!poll_marker(markers, 4, cursor, 68, id));
    assert(poll_marker(markers, 4, cursor, INT32_MAX, id) && id == 4);
    assert(!poll_marker(markers, 4, cursor, INT32_MAX, id));
    cursor = 0;
    assert(poll_marker(markers, 4, cursor, 0, id) && id == 9);
    std::puts("Timeline cooked Q12 curves, endpoints, extremes and ordered marker crossings passed.");
}
