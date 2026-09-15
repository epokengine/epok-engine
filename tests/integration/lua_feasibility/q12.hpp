#pragma once
#include <stdint.h>

// Self-contained mirror of Epok's Blueprint numeric contract.
//
// Mirrored file: runtime/blueprint_runtime.hpp, namespace epok::bp
//   (saturate, iadd, isub, imul, idiv, imod, ineg,
//    uadd, usub, umul, udiv, umod,
//    add, sub, mul, div, neg, from_int, to_int)
// Q12 Fixed is a raw int32 where 4096 == 1.0. Arithmetic saturates to the
// int32 range, division by zero yields 0, and division truncates toward zero.
//
// This header deliberately does NOT include any Epok runtime header: the
// feasibility harness must build without the engine. If
// runtime/blueprint_runtime.hpp ever changes these semantics, this file must
// be updated by hand and the harness re-run.
namespace q12 {

inline constexpr int32_t ONE = 4096;

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

// Fixed is the raw Q12 int32 itself; Epok wraps it in epok::Fixed, but the
// arithmetic contract below is identical.
constexpr int32_t add(int32_t a, int32_t b) { return iadd(a, b); }
constexpr int32_t sub(int32_t a, int32_t b) { return isub(a, b); }
constexpr int32_t mul(int32_t a, int32_t b) { return saturate(int64_t(a) * b / 4096); }
constexpr int32_t div(int32_t a, int32_t b) { return b ? saturate(int64_t(a) * 4096 / b) : 0; }
constexpr int32_t neg(int32_t a) { return ineg(a); }
constexpr int32_t from_int(int32_t value) { return saturate(int64_t(value) * 4096); }
constexpr int32_t to_int(int32_t value) { return value / 4096; }

struct Vector3 {
    int32_t x = 0, y = 0, z = 0;
};

}  // namespace q12
