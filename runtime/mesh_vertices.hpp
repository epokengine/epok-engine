#pragma once
#include <stddef.h>
#include <stdint.h>

namespace epok {
struct MeshVertex {
    int32_t values[3];
    int32_t operator[](size_t axis) const { return values[axis]; }
};

// Non-owning, lossless Q12 storage. Small chunks keep the original six-byte
// vertices; large authored faces may use twelve-byte vertices instead of
// creating extra polygons just to satisfy an int16 coordinate restriction.
// A streamed view has exactly the same pin lifetime as its backing page.
struct MeshVertices {
    const void* data = nullptr;
    uint8_t stride = 6;
    constexpr MeshVertices() = default;
    constexpr MeshVertices(decltype(nullptr)) {}
    constexpr MeshVertices(const int16_t (*p)[3]) : data(p) {}
    constexpr MeshVertices(const int32_t (*p)[3]) : data(p), stride(12) {}
    constexpr MeshVertices(const void* p, uint8_t bytes) : data(p), stride(bytes) {}
    explicit constexpr operator bool() const { return data != nullptr; }
    constexpr bool valid_format() const { return stride == 6 || stride == 12; }
    constexpr bool wide() const { return stride == 12; }
    constexpr bool operator==(const MeshVertices& other) const {
        return data == other.data && stride == other.stride;
    }
    MeshVertices bind(const void* p) const { return {p, stride}; }
    MeshVertices operator+(size_t first) const {
        return bind(static_cast<const uint8_t*>(data) + first * stride);
    }
    auto narrow_data() const -> const int16_t (*)[3] {
        return static_cast<const int16_t (*)[3]>(data);
    }
    auto wide_data() const -> const int32_t (*)[3] {
        return static_cast<const int32_t (*)[3]>(data);
    }
    MeshVertex operator[](size_t index) const {
        if (wide()) {
            const auto& p = wide_data()[index];
            return {{p[0], p[1], p[2]}};
        }
        const auto& p = narrow_data()[index];
        return {{p[0], p[1], p[2]}};
    }
};
}
