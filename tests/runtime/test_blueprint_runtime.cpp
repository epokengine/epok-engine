#include <array>
#include <cassert>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/blueprint_runtime.hpp"

// The real public ActorData, DataHandle and PsyQo Q12 types are compiled above.
// Only the scene lookup service is supplied here, as in the utility host suite.
#include "actor_scene_fixture.hpp"
using namespace epok;
using namespace epok::bp;
static Fixed raw(int32_t value) { return Fixed(value, Fixed::RAW); }
static void arithmetic() {
    static_assert(iadd(INT32_MAX, 1) == INT32_MAX);
    static_assert(isub(INT32_MIN, 1) == INT32_MIN);
    static_assert(imul(INT32_MIN, -1) == INT32_MAX);
    static_assert(idiv(INT32_MIN, -1) == INT32_MAX);
    static_assert(idiv(10, 0) == 0 && imod(INT32_MIN, -1) == 0 && imod(10, 0) == 0);
    static_assert(ineg(INT32_MIN) == INT32_MAX);
    static_assert(uadd(UINT32_MAX, 1) == UINT32_MAX && usub(0, 1) == 0);
    static_assert(umul(UINT32_MAX, UINT32_MAX) == UINT32_MAX && udiv(1, 0) == 0 && umod(1, 0) == 0);
    assert(add(raw(INT32_MAX), raw(1)).raw() == INT32_MAX);
    assert(sub(raw(INT32_MIN), raw(1)).raw() == INT32_MIN);
    assert(mul(raw(INT32_MAX), raw(INT32_MAX)).raw() == INT32_MAX);
    assert(mul(raw(INT32_MIN), raw(INT32_MAX)).raw() == INT32_MIN);
    assert(mul(raw(-3), raw(2048)).raw() == -1);
    assert(bp::div(raw(-3), raw(8192)).raw() == -1);
    assert(bp::div(raw(INT32_MIN), raw(-1)).raw() == INT32_MAX);
    assert(bp::div(raw(1), raw(0)).raw() == 0);
    assert(neg(raw(INT32_MIN)).raw() == INT32_MAX);
    assert(from_int(INT32_MAX).raw() == INT32_MAX && from_int(INT32_MIN).raw() == INT32_MIN);
    assert(to_int(raw(-8191)) == -1);
    assert(interpolate(raw(INT32_MIN), raw(INT32_MAX), 0.5).raw() == -1);
    assert(interpolate(raw(INT32_MIN), raw(INT32_MAX), 1.0).raw() == INT32_MAX);
    const Vector<3> vector{{1.0, -2.0, 3.0}}, scale{{2.0, 0.0, -0.5}};
    assert(add(vector, vector)[1] == Fixed(-4.0));
    assert(sub(vector, vector)[2].raw() == 0);
    assert(mul(vector, scale)[2] == Fixed(-1.5));
    assert(bp::div(vector, scale)[1].raw() == 0);
    assert(bp::div(vector, Fixed(2.0))[0] == Fixed(0.5));
    assert(mul(vector, Fixed(2.0))[0] == Fixed(2.0));
    uint32_t random = 12345;
    for (unsigned i = 0; i < 50000; ++i) {
        random = random * 1664525u + 1013904223u;
        const int32_t a = int32_t(random & 0x7fffffffu) * (random & 0x80000000u ? -1 : 1);
        random = random * 1664525u + 1013904223u;
        const int32_t b = int32_t(random & 0x7fffffffu) * (random & 0x80000000u ? -1 : 1);
        assert(add(raw(a), raw(b)).raw() == saturate(int64_t(a) + b));
        assert(mul(raw(a), raw(b)).raw() == saturate(int64_t(a) * b / 4096));
        assert(bp::div(raw(a), raw(b)).raw() == (b ? saturate(int64_t(a) * 4096 / b) : 0));
    }
}
static void continuations() {
    test_reset_scene();
    const ObjectId a=test_owner(0),b=test_owner(1);
    Continuations<2> first, second;
    Continuation result;
    assert(!first.delay(1, -1.0, a) && !first.delay(1, 1.0, {}));
    assert(first.delay(10, 0.5, a, 7) && second.delay(20, 1.0, b, 7));
    first.advance(0.25, 7); second.advance(0.25, 7);
    assert(!first.poll(result) && !second.poll(result));
    first.advance(1.0, 7, true);
    test_set_active(0,false); first.advance(1.0, 7);
    test_set_active(0,true); first.advance(-1.0, 7);
    assert(!first.poll(result));
    first.advance(0.25, 7);
    assert(first.poll(result) && result.node == 10 && same_owner(result.owner, a) && result.scene_generation == 7);
    assert(!first.poll(result) && !second.poll(result));
    assert(first.delay(1, 0.0, a) && !first.poll(result));
    first.advance(0.0);
    first.advance(0.0, 0, true); assert(!first.poll(result));
    first.advance(0.0);
    assert(first.poll(result) && result.node == 1);
    assert(first.delay(1, 1.0, a) && first.delay(2, 1.0, a));
    assert(!first.delay(3, 1.0, a) && first.dropped == 1);
    first.cancel(1); assert(first.size() == 1 && first.cancelled == 1);
    first.cancel_owner(a); assert(first.size() == 0 && first.cancelled == 2);
    assert(first.delay(5, 0.0, a)); first.advance(0.0);
    test_invalidate(0); // Destruction between advance and poll is checked.
    assert(!first.poll(result) && first.cancelled == 3);
    second.advance(2.0, 8, true); // Scene invalidation runs even while paused.
    assert(second.size() == 0 && second.cancelled == 1);
    for (unsigned i = 0; i < 100; ++i) {
        assert(first.delay(i, 1.0, b)); first.clear(); assert(!first.size());
    }
    first.reset(); assert(!first.dropped && !first.cancelled);
    assert(first.wait_external(100,b,9));
    first.advance(10.0,9);assert(first.waiting(100)&&!first.poll(result));
    test_set_active(1,false);
    assert(first.signal(100,9)&&!first.waiting(100)&&!first.poll(result));
    first.advance(0.0,9,true);test_set_active(1,true);assert(!first.poll(result));
    first.advance(0.0,9);assert(first.poll(result)&&result.node==100&&!first.poll(result));
    // A synchronously signalled fresh wait cannot resume in the dispatch that
    // created it, preventing ready/rearm cycles from blocking the frame.
    assert(first.wait_external(101,b,9)&&first.signal(101,9)&&!first.poll(result));
    first.advance(0.0,9);assert(first.poll(result)&&result.node==101);
    assert(first.wait_external(102,b,9)&&first.delay(103,1.0,b,9));
    assert(!first.wait_external(104,b,9)&&first.dropped==1);
    assert(!first.signal(102,10)&&first.cancelled==1&&first.size()==1);
    first.clear();assert(!first.size());
    assert(first.wait_external(105,b,9)&&first.signal(105,9));
    test_invalidate(1);first.advance(0.0,9);assert(!first.poll(result));
}
static void timelines() {
    test_reset_scene();
    const ObjectId owner=test_owner(0);
    Timeline<3> first, second;
    TimelineKey keys[] = {{0.0, -10.0}, {0.5, 0.0}, {1.0, 10.0}};
    assert(!first.configure(nullptr, 3) && !first.configure(keys, 4));
    assert(first.configure(keys, 3) && second.configure(keys, 3));
    keys[1].value = 9.0; // Timelines retain owned immutable key data.
    assert(first.play(owner, 3) && second.play(owner, 3));
    auto sample = first.advance(0.25, 3);
    assert(sample.updated && !sample.completed && sample.value == Fixed(-5.0));
    assert(second.value() == Fixed(-10.0));
    assert(!first.advance(3.0, 3, true).updated);
    test_set_active(0,false); assert(!first.advance(3.0, 3).updated);
    test_set_active(0,true);
    sample = first.advance(3.0, 3);
    assert(sample.updated && sample.completed && sample.value == Fixed(10.0) && !first.playing());
    assert(!first.advance(3.0, 3).completed);
    assert(first.play(owner, 3, true)); sample = first.advance(2.25, 3);
    assert(sample.loops == 2 && !sample.completed && sample.value == Fixed(-5.0));
    first.advance(0.5, 4); assert(!first.playing());
    test_invalidate(0); assert(!second.advance(0.5, 3).updated && !second.playing());
    TimelineKey extremes[] = {{0.0, raw(INT32_MIN)}, {raw(INT32_MAX), raw(INT32_MAX)}};
    assert(first.configure(extremes, 2));
    test_restore(0);assert(first.play(test_owner(0)));
    assert(first.advance(raw(INT32_MAX)).value.raw() == INT32_MAX);
    extremes[1].time = 0.0; assert(!first.configure(extremes, 2));
    first.reset(); assert(!first.playing() && first.value().raw() == INT32_MIN);
}
static void traces_and_debugger() {
    test_reset_scene();
    const ObjectId a=test_owner(0),b=test_owner(1);
    TraceRing<2> ring;
    Trace entry;
    assert(ring.push({1, 2, a}) && ring.push({1, 3, b}));
    assert(!ring.push({1, 4, a}) && ring.dropped == 1);
    assert(ring.poll(entry) && entry.node_id == 2 && same_owner(entry.owner, a));
    assert(ring.push({1, 4, a}));
    assert(ring.poll(entry) && entry.node_id == 3);
    assert(ring.poll(entry) && entry.node_id == 4 && !ring.poll(entry));
    ring.dropped = UINT32_MAX; ring.push({}); ring.push({}); ring.push({}); assert(ring.dropped == UINT32_MAX);
    ring.clear(); assert(!ring.size() && !ring.dropped);
    Debugger<1> debugger;
    assert(debugger.add({1, 2, a, true}) && debugger.add({1, 2, a, true}));
    assert(!debugger.add({1, 3}) && debugger.dropped == 1);
    assert(debugger.checkpoint(1, 2, b));
    assert(!debugger.checkpoint(1, 2, a) && debugger.paused());
    assert(debugger.location()->node_id == 2 && !debugger.checkpoint(1, 3, a));
    debugger.step(); assert(debugger.checkpoint(1, 2, a) && debugger.paused());
    assert(!debugger.checkpoint(1, 3, a));
    debugger.resume(); assert(debugger.checkpoint(1, 3, a));
    assert(!debugger.checkpoint(1, 2, a));
    debugger.resume(); assert(debugger.checkpoint(1, 2, a)); // Resume skips the stopped node once.
    assert(!debugger.checkpoint(1, 2, a)); // Next loop iteration breaks again.
    debugger.clear_breakpoints(); debugger.reset(); assert(debugger.checkpoint(1, 2, a));
    trace(11, 22, a);
#if defined(EPOK_BLUEPRINT_TRACE) && EPOK_BLUEPRINT_TRACE
    assert(traces.poll(entry) && entry.class_id == 11 && entry.node_id == 22);
#endif
}
int main() {
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR); _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    arithmetic(); continuations(); timelines(); traces_and_debugger();
    std::puts("Blueprint Q12/int arithmetic, isolated continuations, timelines, cancellation, traces and stepping passed.");
}
