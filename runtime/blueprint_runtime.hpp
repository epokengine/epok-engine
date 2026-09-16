#pragma once
#include "timeline.hpp"
#define EPOK_INCLUDE_FROM_BLUEPRINT_RUNTIME 1
#include "epok.hpp"
#undef EPOK_INCLUDE_FROM_BLUEPRINT_RUNTIME
#include "object_model.hpp"

// Allocation-free support for generated Blueprint code. All state below belongs
// to an Actor or ActorComponent instance unless the generated game explicitly shares it.
namespace epok::bp {

// Blueprint arithmetic saturates, divides by zero to zero, and rounds division
// toward zero. Intermediates are wide enough for every signed 32-bit operand;
// neither signed overflow nor negative shifts are part of the language contract.
constexpr int32_t saturate(int64_t value) {
    return value > INT32_MAX ? INT32_MAX : value < INT32_MIN ? INT32_MIN : int32_t(value);
}
constexpr int32_t iadd(int32_t a, int32_t b) { return saturate(int64_t(a) + b); }
constexpr int32_t isub(int32_t a, int32_t b) { return saturate(int64_t(a) - b); }
constexpr int32_t imul(int32_t a, int32_t b) { return saturate(int64_t(a) * b); }
constexpr int32_t idiv(int32_t a, int32_t b) { return b ? saturate(int64_t(a) / b) : 0; }
constexpr int32_t imod(int32_t a, int32_t b) { return b ? int32_t(int64_t(a) % b) : 0; }
constexpr int32_t ineg(int32_t a) { return saturate(-int64_t(a)); }
constexpr uint32_t uadd(uint32_t a, uint32_t b) { return UINT32_MAX - a < b ? UINT32_MAX : a + b; }
constexpr uint32_t usub(uint32_t a, uint32_t b) { return a < b ? 0 : a - b; }
constexpr uint32_t umul(uint32_t a, uint32_t b) {
    const uint64_t value = uint64_t(a) * b;
    return value > UINT32_MAX ? UINT32_MAX : uint32_t(value);
}
constexpr uint32_t udiv(uint32_t a, uint32_t b) { return b ? a / b : 0; }
constexpr uint32_t umod(uint32_t a, uint32_t b) { return b ? a % b : 0; }
inline Fixed add(Fixed a, Fixed b) { return Fixed(iadd(a.raw(), b.raw()), Fixed::RAW); }
inline Fixed sub(Fixed a, Fixed b) { return Fixed(isub(a.raw(), b.raw()), Fixed::RAW); }
inline Fixed mul(Fixed a, Fixed b) { return Fixed(saturate(int64_t(a.raw()) * b.raw() / 4096), Fixed::RAW); }
inline Fixed div(Fixed a, Fixed b) {
    return Fixed(b.raw() ? saturate(int64_t(a.raw()) * 4096 / b.raw()) : 0, Fixed::RAW);
}
inline Fixed neg(Fixed a) { return Fixed(ineg(a.raw()), Fixed::RAW); }
inline Fixed from_int(int32_t value) { return Fixed(saturate(int64_t(value) * 4096), Fixed::RAW); }
inline int32_t to_int(Fixed value) { return value.raw() / 4096; }
inline Fixed interpolate(Fixed a, Fixed b, Fixed alpha) {
    const int32_t t = alpha.raw() < 0 ? 0 : alpha.raw() > 4096 ? 4096 : alpha.raw();
    return Fixed(saturate(int64_t(a.raw()) + (int64_t(b.raw()) - a.raw()) * t / 4096), Fixed::RAW);
}
template<size_t Size> struct Vector {
    static_assert(Size > 0 && Size <= 4);
    Fixed values[Size] = {};
    Fixed& operator[](size_t index) { return values[index]; }
    const Fixed& operator[](size_t index) const { return values[index]; }
};
template<size_t Size> inline Vector<Size> add(const Vector<Size>& a, const Vector<Size>& b) {
    Vector<Size> result; for (size_t i = 0; i < Size; ++i) result[i] = add(a[i], b[i]); return result;
}
template<size_t Size> inline Vector<Size> sub(const Vector<Size>& a, const Vector<Size>& b) {
    Vector<Size> result; for (size_t i = 0; i < Size; ++i) result[i] = sub(a[i], b[i]); return result;
}
template<size_t Size> inline Vector<Size> mul(const Vector<Size>& a, const Vector<Size>& b) {
    Vector<Size> result; for (size_t i = 0; i < Size; ++i) result[i] = mul(a[i], b[i]); return result;
}
template<size_t Size> inline Vector<Size> div(const Vector<Size>& a, const Vector<Size>& b) {
    Vector<Size> result; for (size_t i = 0; i < Size; ++i) result[i] = div(a[i], b[i]); return result;
}
template<size_t Size> inline Vector<Size> mul(const Vector<Size>& a, Fixed b) {
    Vector<Size> result; for (size_t i = 0; i < Size; ++i) result[i] = mul(a[i], b); return result;
}
template<size_t Size> inline Vector<Size> div(const Vector<Size>& a, Fixed b) {
    Vector<Size> result; for (size_t i = 0; i < Size; ++i) result[i] = div(a[i], b); return result;
}
constexpr bool same_owner(ObjectId a, ObjectId b) {
    return a == b;
}
constexpr bool same_owner(DataHandle a, DataHandle b) {
    return a.index == b.index && a.generation == b.generation;
}
inline void increment(uint32_t& value) { if (value != UINT32_MAX) ++value; }

struct Continuation {
    uint32_t node = 0;
    ObjectId owner;
    uint32_t scene_generation = 0;
};

// Generated functions store their live variables in their own bounded frame
// slots, addressed by `node` (a compiler-assigned continuation/frame identifier).
// Scheduling never invokes code. Even zero Delay resumes on a later advance.
// Pause and inactive ancestors freeze time; invalid handles and scene changes
// cancel before polling. Poll again after each callback to revalidate lifetime.
template<size_t Capacity = 8> class Continuations {
    static_assert(Capacity > 0 && Capacity <= 256);
    struct Slot {
        Continuation continuation;
        int32_t remaining = 0;
        bool used = false, ready = false, advanced = false;
    };
    Slot slots[Capacity] = {};
    bool dispatch_paused = false;
public:
    uint32_t dropped = 0, cancelled = 0;
    bool delay(uint32_t node, Fixed seconds, ObjectId owner, uint32_t scene_generation = 0) {
        if (seconds.raw() < 0 || !owner.get()) return false;
        for (auto& slot : slots) if (!slot.used) {
            slot = {{node, owner, scene_generation}, seconds.raw(), true, false, false};
            return true;
        }
        increment(dropped);
        return false;
    }
    // External producers only mark an existing frame ready. They never invoke
    // callbacks or allocate another listener/continuation. Negative remaining
    // time is internal state and cannot be authored through Delay.
    bool wait_external(uint32_t node, ObjectId owner, uint32_t scene_generation = 0) {
        if (!owner.get()) return false;
        for (auto& slot : slots) if (!slot.used) {
            slot = {{node, owner, scene_generation}, -1, true, false, false};
            return true;
        }
        increment(dropped); return false;
    }
    bool waiting(uint32_t node) const {
        for (const auto& slot : slots) if (slot.used && !slot.ready && slot.remaining < 0 && slot.continuation.node == node) return true;
        return false;
    }
    bool signal(uint32_t node, uint32_t scene_generation = 0) {
        for (auto& slot : slots) if (slot.used && slot.remaining < 0 && slot.continuation.node == node) {
            if (!slot.continuation.owner.get() || slot.continuation.scene_generation != scene_generation) {
                slot.used = false; increment(cancelled); return false;
            }
            // Retain an observed result while inactive or paused, but dispatch
            // only after a later advance and a fresh owner/lifecycle check.
            slot.ready = true; return true;
        }
        return false;
    }
    void advance(Fixed dt, uint32_t scene_generation = 0, bool paused = false) {
        dispatch_paused = paused;
        for (auto& slot : slots) if (slot.used) {
            auto* owner = slot.continuation.owner.get();
            if (!owner || slot.continuation.scene_generation != scene_generation) {
                slot.used = false;
                increment(cancelled);
                continue;
            }
            if (dt.raw() >= 0) slot.advanced = true;
            if (paused || !is_active(owner) || dt.raw() < 0 || slot.ready || slot.remaining < 0) continue;
            if (dt.raw() >= slot.remaining) { slot.remaining = 0; slot.ready = true; }
            else slot.remaining -= dt.raw();
        }
    }
    bool poll(Continuation& result) {
        if (dispatch_paused) return false;
        for (auto& slot : slots) if (slot.used && slot.ready && slot.advanced) {
            auto* owner = slot.continuation.owner.get();
            if (!owner) { slot.used = false; increment(cancelled); continue; }
            if (!is_active(owner)) continue;
            result = slot.continuation;
            slot.used = false;
            return true;
        }
        return false;
    }
    void cancel_owner(ObjectId owner) {
        for (auto& slot : slots) if (slot.used && same_owner(slot.continuation.owner, owner)) {
            slot.used = false; increment(cancelled);
        }
    }
    void cancel(uint32_t node) {
        for (auto& slot : slots) if (slot.used && slot.continuation.node == node) {
            slot.used = false; increment(cancelled);
        }
    }
    void reset() {
        for (auto& slot : slots) slot = {};
        dropped = cancelled = 0;
        dispatch_paused = false;
    }
    void cancel_all() { for (auto& slot : slots) if (slot.used) { slot.used = false; increment(cancelled); } }
    void clear() { cancel_all(); }
    size_t size() const { size_t result = 0; for (const auto& slot : slots) if (slot.used) ++result; return result; }
};

struct TimelineKey { Fixed time = 0.0, value = 0.0; };
struct TimelineSample {
    Fixed value = 0.0;
    bool updated = false, completed = false;
    uint32_t loops = 0;
};
// One scalar track per object. Generated code applies samples to typed targets
// and dispatches Update/Finished pins; the track never retains a field pointer.
// Large dt is O(key capacity), not O(number of crossed loops). Completion is
// emitted exactly once; a looping track exposes the crossed-loop count instead.
template<size_t Capacity = 16> class Timeline {
    static_assert(Capacity >= 2 && Capacity <= 256);
    epok::timeline::Key keys[Capacity] = {};
    size_t count = 0;
    int32_t elapsed = 0;
    ObjectId owner;
    uint32_t generation = 0;
    bool running = false, looping = false;
public:
    bool configure(const TimelineKey* data, size_t length) {
        if (!data || length < 2 || length > Capacity || data[0].time.raw() != 0) return false;
        for (size_t i = 1; i < length; ++i) if (data[i].time.raw() <= data[i - 1].time.raw()) return false;
        for (size_t i = 0; i < length; ++i) keys[i] = {data[i].time.raw(),data[i].value.raw()};
        count = length;
        running = false;
        elapsed = 0;
        return true;
    }
    bool play(ObjectId source, uint32_t scene_generation = 0, bool loop = false) {
        if (count < 2 || !source.get()) return false;
        owner = source; generation = scene_generation; elapsed = 0; looping = loop; running = true;
        return true;
    }
    Fixed value() const {
        return Fixed(epok::timeline::sample({keys,uint16_t(count)},elapsed),Fixed::RAW);
    }
    TimelineSample advance(Fixed dt, uint32_t scene_generation = 0, bool paused = false) {
        TimelineSample result{value()};
        if (!running) return result;
        auto* source = owner.get();
        if (!source || scene_generation != generation) { running = false; return result; }
        if (paused || !is_active(source) || dt.raw() <= 0) return result;
        const int64_t next = int64_t(elapsed) + dt.raw();
        const int32_t duration = keys[count - 1].tick;
        if (looping) { result.loops = uint32_t(next / duration); elapsed = int32_t(next % duration); }
        else if (next >= duration) { elapsed = duration; running = false; result.completed = true; }
        else elapsed = int32_t(next);
        result.updated = true;
        result.value = value();
        return result;
    }
    void cancel() { running = false; }
    void reset() { running = false; elapsed = 0; owner = {}; generation = 0; }
    bool playing() const { return running; }
};

enum class TraceKind : uint8_t { Enter, Value, Suspend, Resume, Error, Breakpoint };
struct Trace {
    uint32_t class_id = 0, node_id = 0;
    ObjectId owner;
    TraceKind kind = TraceKind::Enter;
    int32_t value = 0;
};
template<size_t Capacity = 128> class TraceRing {
    static_assert(Capacity > 0);
    Trace entries[Capacity] = {};
    size_t first = 0, count = 0;
public:
    uint32_t dropped = 0;
    bool push(Trace entry) {
        if (count == Capacity) { increment(dropped); return false; }
        entries[(first + count) % Capacity] = entry; ++count; return true;
    }
    bool poll(Trace& entry) {
        if (!count) return false;
        entry = entries[first]; first = (first + 1) % Capacity; --count; return true;
    }
    void clear() { first = count = 0; dropped = 0; }
    size_t size() const { return count; }
};

struct Breakpoint {
    uint32_t class_id = 0, node_id = 0;
    ObjectId owner;
    bool instance_only = false;
};
// Cooperative execution gate, not a busy wait or platform trap. A generated
// resumable function must preserve its program counter when checkpoint is false.
// Synchronous native calls remain atomic; this API does not unwind their stack.
template<size_t Capacity = 32> class Debugger {
    Breakpoint points[Capacity] = {};
    size_t count = 0;
    Trace stopped;
    bool halted = false, stepping = false, skip_stopped = false, has_stop = false;
    bool is_stopped(uint32_t class_id, uint32_t node_id, ObjectId owner) const {
        return has_stop && stopped.class_id == class_id && stopped.node_id == node_id && same_owner(stopped.owner, owner);
    }
public:
    uint32_t dropped = 0;
    bool add(Breakpoint point) {
        for (size_t i = 0; i < count; ++i) if (points[i].class_id == point.class_id && points[i].node_id == point.node_id &&
            points[i].instance_only == point.instance_only && (!point.instance_only || same_owner(points[i].owner, point.owner))) return true;
        if (count == Capacity) { increment(dropped); return false; }
        points[count++] = point; return true;
    }
    void clear_breakpoints() { count = 0; }
    bool checkpoint(uint32_t class_id, uint32_t node_id, ObjectId owner) {
        if (halted) return false;
        bool skip = skip_stopped && is_stopped(class_id, node_id, owner);
        skip_stopped = false;
        if (!skip && !stepping) for (size_t i = 0; i < count; ++i) {
            const auto& point = points[i];
            if (point.class_id == class_id && point.node_id == node_id && (!point.instance_only || same_owner(point.owner, owner))) {
                stopped = {class_id, node_id, owner, TraceKind::Breakpoint, 0};
                has_stop = halted = true;
                return false;
            }
        }
        if (stepping) {
            stopped = {class_id, node_id, owner, TraceKind::Enter, 0};
            has_stop = halted = true;
            stepping = false;
        }
        return true;
    }
    void pause() { halted = true; stepping = false; skip_stopped = false; }
    void resume() { halted = false; stepping = false; skip_stopped = has_stop; }
    void step() { halted = false; stepping = true; skip_stopped = has_stop; }
    bool paused() const { return halted; }
    const Trace* location() const { return has_stop ? &stopped : nullptr; }
    void reset() { halted = stepping = skip_stopped = has_stop = false; dropped = 0; }
};

// The cook defines EPOK_BLUEPRINT_TRACE only for instrumented builds. Release
// builds carry neither the ring nor trace call work. IDs are compact cook-time
// IDs, not runtime hashes or C++ RTTI; the host retains their asset/node mapping.
#if defined(EPOK_BLUEPRINT_TRACE) && EPOK_BLUEPRINT_TRACE
inline TraceRing<128> traces;
inline void trace(uint32_t class_id, uint32_t node_id, ObjectId owner,
                  TraceKind kind = TraceKind::Enter, int32_t value = 0) {
    traces.push({class_id, node_id, owner, kind, value});
}
#else
inline void trace(uint32_t, uint32_t, ObjectId, TraceKind = TraceKind::Enter, int32_t = 0) {}
#endif
}
