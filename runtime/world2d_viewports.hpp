#pragma once
// World2D viewports (World2D initiative, phase 4): SubViewport2D and SplitScreen2D.
//
// A SubViewport2D draws the world through a camera (or the active one) into its own
// rectangle of VRAM every `interval` frames, before the main view; a Sprite2D whose
// `viewport` names it shows that picture as its texture: minimaps, security monitors,
// picture-in-picture. It may draw only one canvas layer and hide that layer from every
// other view, so a Sprite2D can show the layer as one picture: a group faded as a whole
// (the role of a canvas group). The picture is 15-bit colour, placed by the texture
// packer in a page of its own.
//
// A SplitScreen2D draws the world twice, through two cameras, side by side or one above
// the other, with a divider line; screen canvases (HUD layers) are drawn once on top.
//
// The renderer and the PC runner set `render_pass` while they collect each view; the
// draw collection reads its camera, canvas filter and culling rectangle.
//
// MIPS constraints apply: C++20 without RTTI, exceptions, heap or STL containers.
#include "world2d_physics.hpp"

namespace epok {
class SubViewport2DComponent;
class SplitScreen2DComponent;

namespace world2d {
inline List<SubViewport2DComponent> subviewports;
inline List<SplitScreen2DComponent> split_screens;
inline constexpr int16_t every_layer = 0x7fff;

// What the view being collected draws.
struct RenderPass {
    const Camera2D* camera = nullptr;  // null: the active camera
    bool world = true, screen = true;  // world canvases, screen canvases
    int16_t only_layer = every_layer;
    int8_t hidden[4] = {};
    uint8_t hidden_count = 0;
    int16_t clip[4] = {0, 0, int16_t(display_width), int16_t(display_height)};
    bool accepts(const Canvas& canvas) const {
        if (canvas.screen ? !screen : !world) return false;
        if (only_layer != every_layer) return canvas.layer == only_layer;
        for (uint8_t i = 0; i < hidden_count; ++i)
            if (hidden[i] == canvas.layer) return false;
        return true;
    }
    void hide(int8_t layer) {
        if (hidden_count < 4) hidden[hidden_count++] = layer;
    }
};
inline RenderPass render_pass;
}  // namespace world2d

// ---- SubViewport2D -------------------------------------------------------------------------
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=viewport2d, Id="f5dc7806-fd18-40aa-9984-3b475e48df5d") SubViewport2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("f5dc7806-fd18-40aa-9984-3b475e48df5d");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::subviewports.add(this); }
    ~SubViewport2DComponent() override { world2d::subviewports.remove(this); }
    world2d::Link<SubViewport2DComponent> link2d;

    // Authored: the camera (an actor with a Camera2D; none: the active camera), the
    // picture's size and background, how often it redraws, and its canvas filter.
    ObjectId camera;
    uint16_t width = 128, height = 96;
    uint8_t background[3] = {0, 0, 0};
    uint8_t interval = 1;
    int16_t layer = world2d::every_layer;
    bool hide_layer = false, enabled = true;
    // Cooked VRAM placement of the picture (a 64-halfword-aligned column, inside one
    // 256-line page row).
    uint16_t target_x = 0, target_y = 0;
    // Runtime.
    uint8_t countdown = 0;
    uint16_t native_index = 0;

    EPOK_FUNCTION(BlueprintCallable, Id="28e5d077-d0a0-47cc-80f7-82d914b59453") void set_enabled(bool value) { enabled = value; }
    EPOK_FUNCTION(BlueprintPure, Id="9baa1eec-88fa-4da0-85b5-a034a01d2901") bool is_enabled() const { return enabled; }
    EPOK_FUNCTION(BlueprintCallable, Id="c7515e49-f02f-4db8-99be-ae2ee8ddb78a") void set_camera(ObjectId actor) { camera = actor; }
    EPOK_FUNCTION(BlueprintPure, Id="6bb93ca8-6d7b-4864-b287-0c621d9a57ce") ObjectId get_camera() const { return camera; }
    // Redraw every `frames` frames (1: every frame); a still picture costs nothing between.
    EPOK_FUNCTION(BlueprintCallable, Id="c1efb22a-a765-40a0-96c1-cfb4d76ebe41") void set_interval(uint32_t frames) {
        interval = uint8_t(frames < 1 ? 1 : frames > 255 ? 255 : frames);
    }
    // The view this picture is drawn through: the camera's, recentred on the picture.
    bool view(Camera2D& out) const;
};

// ---- SplitScreen2D -------------------------------------------------------------------------
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=viewport2d, Id="54813814-1419-4a34-b8f1-6183acb48376") SplitScreen2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("54813814-1419-4a34-b8f1-6183acb48376");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::split_screens.add(this); }
    ~SplitScreen2DComponent() override { world2d::split_screens.remove(this); }
    world2d::Link<SplitScreen2DComponent> link2d;

    ObjectId cameras[2];
    bool side_by_side = true, enabled = true;
    uint8_t divider = 2;
    uint8_t divider_color[3] = {0, 0, 0};

    EPOK_FUNCTION(BlueprintCallable, Id="ab3362cc-9976-433c-87f4-320bd700a33d") void set_enabled(bool value) { enabled = value; }
    EPOK_FUNCTION(BlueprintPure, Id="a88870fb-7818-467e-a31c-d23bdd7e03f5") bool is_enabled() const { return enabled; }
    EPOK_FUNCTION(BlueprintCallable, Id="02c4b77a-09a1-4792-aca4-9f6aeaa9d0ab") void set_side_by_side(bool value) { side_by_side = value; }
    EPOK_FUNCTION(BlueprintPure, Id="0f8259fd-02dd-44ac-8002-d09dda0131d0") bool is_side_by_side() const { return side_by_side; }
    // Screen rectangle (x, y, w, h) of half `index`, either side of the divider.
    void half(int index, int16_t* rect) const {
        const int16_t w = int16_t(display_width), h = int16_t(display_height), gap = int16_t(divider);
        if (side_by_side) {
            const int16_t left = int16_t((w - gap) / 2);
            rect[0] = index ? int16_t(left + gap) : int16_t(0);
            rect[1] = 0;
            rect[2] = index ? int16_t(w - left - gap) : left;
            rect[3] = h;
        } else {
            const int16_t top = int16_t((h - gap) / 2);
            rect[0] = 0;
            rect[1] = index ? int16_t(top + gap) : int16_t(0);
            rect[2] = w;
            rect[3] = index ? int16_t(h - top - gap) : top;
        }
    }
    // The view through camera `index`, its projection centred on its half.
    bool view(int index, Camera2D& out) const;
};

namespace world2d {
inline Camera2DComponent* camera_on(ObjectId actor) {
    auto* registry = active_object_registry;
    auto* owner = registry && actor.valid() ? registry->resolve<Actor>(actor) : nullptr;
    return owner ? find<Camera2DComponent>(owner) : nullptr;
}
inline SubViewport2DComponent* subviewport_on(ObjectId actor) {
    auto* registry = active_object_registry;
    auto* owner = registry && actor.valid() ? registry->resolve<Actor>(actor) : nullptr;
    return owner ? find<SubViewport2DComponent>(owner) : nullptr;
}
// The active split screen, when both its cameras exist.
inline SplitScreen2DComponent* active_split() {
    SplitScreen2DComponent* found = nullptr;
    split_screens.each([&](SplitScreen2DComponent& split) {
        if (!found && split.enabled && is_active(&split) && camera_on(split.cameras[0]) && camera_on(split.cameras[1])) found = &split;
    });
    return found;
}
}  // namespace world2d

inline bool SplitScreen2DComponent::view(int index, Camera2D& out) const {
    Camera2DComponent* camera = world2d::camera_on(cameras[index ? 1 : 0]);
    if (!camera) return false;
    out = camera->view();
    half(index, out.viewport);
    return true;
}
}  // namespace epok
