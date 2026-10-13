// Host contract tests for the World2D components and service (World2D initiative,
// phase 1): sprite frame clips, Camera2D following, CharacterBody2D move_and_slide with
// floor/wall/one-way contacts, trigger delivery to overlap_2d, registration lifetime and
// the stable draw-item sort. Compiles the real runtime headers; only the audio service and
// the cooked class table are provided here, as a generated build would.
#include <cassert>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/epok.hpp"
#include "../../runtime/world2d_service.hpp"

namespace epok {
ActorData* DataHandle::get() const { return nullptr; }
void AudioSource::play() {}
void AudioSource::stop() {}
bool AudioSource::is_playing() const { return false; }
}  // namespace epok
using namespace epok;

namespace {
Fixed q(double value) {
    const double scaled = value * 4096.0;
    return Fixed(int32_t(scaled < 0 ? scaled - 0.5 : scaled + 0.5), Fixed::RAW);
}
double real(Fixed value) { return value.raw() / 4096.0; }
bool close(double a, double b, double tolerance) { return (a > b ? a - b : b - a) <= tolerance; }

// An Actor2D that records its overlap_2d events.
struct Probe2D : Actor2D {
    static constexpr uint64_t static_class_id = 0x2D01;
    uint64_t class_id() const override { return static_class_id; }
    unsigned enters = 0, stays = 0, exits = 0;
    ObjectId last_other;
    void overlap_2d(ObjectId other, TriggerPhase phase) override {
        last_other = other;
        if (phase == TriggerPhase::Enter) ++enters;
        if (phase == TriggerPhase::Stay) ++stays;
        if (phase == TriggerPhase::Exit) ++exits;
    }
};
// A component that records trigger_event, the per-component delivery.
struct Listener2D : ActorComponent {
    static constexpr uint64_t static_class_id = 0x2D02;
    uint64_t class_id() const override { return static_class_id; }
    unsigned events = 0;
    void trigger_event(ObjectId, TriggerPhase) override { ++events; }
};
constexpr uint8_t world2d_bit = object_domain_bit(ObjectDomain::World2D);
}  // namespace

namespace epok {
const ClassDescriptor object_classes[] = {
    {Object::static_class_id, 0, ObjectFamily::Object, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Object), alignof(Object), nullptr, nullptr},
    {Actor::static_class_id, Object::static_class_id, ObjectFamily::Actor, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Actor), alignof(Actor), nullptr, nullptr},
    {Actor2D::static_class_id, Actor::static_class_id, ObjectFamily::Actor, ObjectDomain::World2D, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), &object_construct<Actor2D>, &object_destruct, sizeof(Actor2D), alignof(Actor2D), &ObjectPool<Actor2D, 8>::acquire, &ObjectPool<Actor2D, 8>::release},
    {Probe2D::static_class_id, Actor2D::static_class_id, ObjectFamily::Actor, ObjectDomain::World2D, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), &object_construct<Probe2D>, &object_destruct, sizeof(Probe2D), alignof(Probe2D), &ObjectPool<Probe2D, 4>::acquire, &ObjectPool<Probe2D, 4>::release},
    {ActorComponent::static_class_id, Object::static_class_id, ObjectFamily::Component, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(ActorComponent), alignof(ActorComponent), nullptr, nullptr},
    {SceneComponent2D::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassRoot, &object_construct<SceneComponent2D>, &object_destruct, sizeof(SceneComponent2D), alignof(SceneComponent2D), &ObjectPool<SceneComponent2D, 8>::acquire, &ObjectPool<SceneComponent2D, 8>::release},
    {Sprite2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<Sprite2DComponent>, &object_destruct, sizeof(Sprite2DComponent), alignof(Sprite2DComponent), &ObjectPool<Sprite2DComponent, 4>::acquire, &ObjectPool<Sprite2DComponent, 4>::release},
    {Camera2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<Camera2DComponent>, &object_destruct, sizeof(Camera2DComponent), alignof(Camera2DComponent), &ObjectPool<Camera2DComponent, 4>::acquire, &ObjectPool<Camera2DComponent, 4>::release},
    {Collider2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<Collider2DComponent>, &object_destruct, sizeof(Collider2DComponent), alignof(Collider2DComponent), &ObjectPool<Collider2DComponent, 8>::acquire, &ObjectPool<Collider2DComponent, 8>::release},
    {CharacterBody2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<CharacterBody2DComponent>, &object_destruct, sizeof(CharacterBody2DComponent), alignof(CharacterBody2DComponent), &ObjectPool<CharacterBody2DComponent, 4>::acquire, &ObjectPool<CharacterBody2DComponent, 4>::release},
    {Listener2D::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::None, world2d_bit, ObjectClassMultiple, &object_construct<Listener2D>, &object_destruct, sizeof(Listener2D), alignof(Listener2D), &ObjectPool<Listener2D, 4>::acquire, &ObjectPool<Listener2D, 4>::release},
    {Level::static_class_id, Object::static_class_id, ObjectFamily::Level, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Level), alignof(Level), nullptr, nullptr},
};
const size_t object_class_count = sizeof(object_classes) / sizeof(object_classes[0]);
}  // namespace epok

namespace {
ObjectRegistryStorage<64> registry_storage;
Level game_level;
void reset() {
    if (game_level.registry()) {
        game_level.end_play_all(EndPlayReason::LevelUnloaded);
        registry_storage.collect_quarantined();
        registry_storage.release(game_level.id());
    }
    game_level = Level{};
    active_object_registry = &registry_storage;
    registry_storage.stats = ObjectStats{};
    assert(game_level.bind(registry_storage));
    world2d::service.reset();
    world2d::install();
    epok::time = Time{};
}
template<class T> T* spawn(const char* name, double x, double y) {
    const ObjectId id = game_level.spawn_actor(*find_object_class(T::static_class_id), name, {});
    auto* actor = registry_storage.resolve<T>(id);
    assert(actor);
    auto* root = registry_storage.resolve<SceneComponent2D>(actor->root_id());
    assert(root);
    root->transform.position[0] = q(x);
    root->transform.position[1] = q(y);
    return actor;
}
SceneComponent2D& root(Actor* actor) { return *registry_storage.resolve<SceneComponent2D>(actor->root_id()); }
Collider2DComponent* box(Actor* actor, double width, double height) {
    auto* collider = game_level.add_component<Collider2DComponent>(*actor, "Shape");
    assert(collider);
    collider->set_box_size(q(width), q(height));
    return collider;
}
void step() {
    epok::time.begin_tick();
    world2d::service.step(Fixed(epok::time.delta_raw, Fixed::RAW), &game_level);
}

// 1. Clips advance by simulation time, loop or stop, and report events once.
void sprite_clips() {
    reset();
    static constexpr SpriteFrame frames[] = {{{0, 0, 16, 16}, Fixed(0.25), 0}, {{16, 0, 16, 16}, Fixed(0.25), 7}};
    static constexpr SpriteClip clips[] = {{frames, 2, true, "walk"}, {frames, 2, false, "once"}};
    auto* actor = spawn<Actor2D>("Hero", 0, 0);
    auto* sprite = game_level.add_component<Sprite2DComponent>(*actor, "Sprite");
    assert(sprite && world2d::sprites.count == 1);
    sprite->clips = clips;
    sprite->clip_count = 2;
    sprite->play(0u);
    for (int i = 0; i < 14; ++i) step();  // 14 steps of 68/69 raw units < 0.25 s (1024 raw)
    assert(sprite->current_frame() == 0 && sprite->take_frame_event() == 0);
    for (int i = 0; i < 2; ++i) step();
    assert(sprite->current_frame() == 1 && sprite->take_frame_event() == 7 && sprite->take_frame_event() == 0);
    for (int i = 0; i < 15; ++i) step();
    assert(sprite->current_frame() == 0 && sprite->is_playing());  // looped
    assert(sprite->play_named("once") && sprite->current_animation() == 1);
    for (int i = 0; i < 40; ++i) step();
    assert(!sprite->is_playing() && sprite->take_animation_finished() && !sprite->take_animation_finished());
    assert(sprite->current_frame() == 1);
    // speed_scale 0 freezes; frame cells of a plain sheet follow hframes/vframes.
    sprite->clips = nullptr;
    sprite->hframes = 4;
    sprite->vframes = 2;
    sprite->set_frame(6);
    uint16_t rect[4];
    assert(sprite->source_rect(64, 32, rect) && rect[0] == 32 && rect[1] == 16 && rect[2] == 16 && rect[3] == 16);
    // Destroying the component unlinks it.
    actor->destroy();
    registry_storage.collect_quarantined();
    assert(world2d::sprites.count == 0);
}

// 2. The camera follows its owner, respects limits and smooths towards the target.
void camera_follow() {
    reset();
    auto* actor = spawn<Actor2D>("Hero", 100, 50);
    auto* camera = game_level.add_component<Camera2DComponent>(*actor, "Camera");
    assert(camera && world2d::service.active_camera() == camera);
    step();
    assert(real(camera->position[0]) == 100 && real(camera->position[1]) == 50);
    // Limits keep the whole display inside [-200, 200] x [-300, 300]; a view taller than
    // its limits would be is centred between them instead.
    camera->set_limits(q(-200), q(-300), q(200), q(300));
    root(actor).transform.position[0] = q(1000);
    root(actor).transform.position[1] = q(1000);
    step();
    assert(real(camera->position[0]) == 200 - display_width / 2. && real(camera->position[1]) == 300 - display_height / 2.);
    camera->set_limits(q(-200), q(-50), q(200), q(50));
    step();
    assert(real(camera->position[1]) == 0);
    // Smoothing approaches instead of jumping.
    camera->clear_limits();
    camera->smoothing = true;
    camera->smoothing_speed = q(6);
    const double before = real(camera->position[0]);
    step();
    const double after = real(camera->position[0]);
    assert(after > before && after < 1000);
    for (int i = 0; i < 240; ++i) step();
    assert(close(real(camera->position[0]), 1000, 1.));
    // make_current moves the view to another camera.
    auto* other = spawn<Actor2D>("Spectator", -50, 0);
    auto* second = game_level.add_component<Camera2DComponent>(*other, "Camera");
    assert(world2d::service.active_camera() == camera);
    second->make_current();
    assert(world2d::service.active_camera() == second && !camera->is_current());
    // Shake displaces the view but leaves the followed position alone.
    step();
    second->shake(q(4), q(1));
    const Camera2D view = second->view();
    assert(real(second->position[0]) == -50);
    assert(close(real(view.position[0]), -50, 4.01) && close(real(view.position[1]), 0, 4.01));
}

// 3. move_and_slide lands on floors, stops at walls, crosses one-way platforms from
// below and lands on them from above.
void character_body() {
    reset();
    auto* ground = spawn<Actor2D>("Ground", 0, 100);
    box(ground, 512, 32);  // top edge at y = 84
    auto* wall = spawn<Actor2D>("Wall", 100, 60);
    box(wall, 32, 48);  // left face at x = 84
    auto* ledge = spawn<Actor2D>("Ledge", -100, 40);
    auto* platform = box(ledge, 64, 8);  // top edge at y = 36
    platform->shape.one_way = true;
    platform->shape.one_way_margin = q(3);
    auto* hero = spawn<Actor2D>("Hero", 0, 0);
    box(hero, 12, 24);  // feet at y + 12
    auto* body = game_level.add_component<CharacterBody2DComponent>(*hero, "Body");
    assert(body);
    // Falling under gravity lands exactly on the ground and zeroes the fall speed.
    for (int i = 0; i < 120 && !body->is_on_floor(); ++i) {
        body->set_velocity(q(0), body->velocity_y() + q(600) * Fixed(epok::time.delta_raw, Fixed::RAW));
        epok::time.begin_tick();
        body->move_and_slide();
    }
    assert(body->is_on_floor() && real(root(hero).transform.position[1]) == 72 && body->velocity_y().raw() == 0);
    assert(body->last_collision_actor() == ground->id());
    // Walking right stops at the wall face and reports a wall, not a floor.
    for (int i = 0; i < 120; ++i) {
        body->set_velocity(q(120), q(10));
        epok::time.begin_tick();
        body->move_and_slide();
    }
    assert(close(real(root(hero).transform.position[0]), 84 - 6, 1e-3) && body->is_on_wall());
    assert(body->velocity_x().raw() == 0);
    // Jumping up through the one-way ledge from below passes it...
    root(hero).transform.position[0] = q(-100);
    root(hero).transform.position[1] = q(72);
    world2d::service.sync(true);
    body->set_velocity(q(0), q(-400));
    for (int i = 0; i < 12; ++i) {
        epok::time.begin_tick();
        body->move_and_slide();
    }
    assert(real(root(hero).transform.position[1]) < 36 - 12);
    // ...and falling back down lands on its top edge.
    for (int i = 0; i < 120 && !(body->is_on_floor() && real(root(hero).transform.position[1]) < 40); ++i) {
        body->set_velocity(q(0), q(300));
        epok::time.begin_tick();
        body->move_and_slide();
    }
    assert(body->is_on_floor() && real(root(hero).transform.position[1]) == 24);
    // Floating bodies treat every contact as a wall.
    body->floating = true;
    body->set_velocity(q(0), q(300));
    epok::time.begin_tick();
    body->move_and_slide();
    assert(!body->is_on_floor() && body->is_on_wall());
}

// 4. Trigger overlaps reach both actors' overlap_2d and their components' trigger_event.
void triggers() {
    reset();
    auto* coin = spawn<Probe2D>("Coin", 0, 0);
    auto* area = box(coin, 16, 16);
    area->shape.trigger = true;
    auto* listener = game_level.add_component<Listener2D>(*coin, "Listener");
    auto* hero = spawn<Probe2D>("Hero", 100, 0);
    box(hero, 12, 24);
    step();
    assert(coin->enters == 0 && hero->enters == 0);
    root(hero).transform.position[0] = q(4);
    step();
    assert(coin->enters == 1 && hero->enters == 1 && coin->last_other == hero->id() && hero->last_other == coin->id());
    assert(listener->events == 1);
    step();
    assert(coin->stays == 1 && coin->enters == 1);
    root(hero).transform.position[0] = q(100);
    step();
    assert(coin->exits == 1 && hero->exits == 1);
    // A deactivated trigger owner neither collides nor reports.
    root(hero).transform.position[0] = q(0);
    coin->set_active(false);
    step();
    assert(coin->enters == 1 && hero->enters == 1);
}

// 5. The renderer's item sort is stable and orders by key.
void draw_sort() {
    world2d::DrawItem items[6], scratch[6];
    const uint32_t keys[6] = {0x80000005u, 0x7fff0000u, 0x80000005u, 0x80010000u, 0x80000001u, 0x80000005u};
    for (int i = 0; i < 6; ++i) items[i] = world2d::DrawItem{keys[i], uint16_t(i + 1)};
    world2d::sort_draw_items(items, scratch, 6);
    const uint16_t expected[6] = {2, 5, 1, 3, 6, 4};
    for (int i = 0; i < 6; ++i) assert(items[i].index == expected[i]);
}

// 6. Sprite resolution: culling, y-sort order, the exact-rectangle fast path.
void sprite_resolution() {
    reset();
    auto size = [](int index, uint16_t& w, uint16_t& h) { if (index != 3) return false; w = 64; h = 32; return true; };
    auto* back = spawn<Actor2D>("Back", 0, 20);
    auto* front = spawn<Actor2D>("Front", 0, -20);
    auto* far = spawn<Actor2D>("Far", 4000, 0);
    Sprite2DComponent* sprites[3] = {};
    Actor2D* owners[3] = {back, front, far};
    for (int i = 0; i < 3; ++i) {
        sprites[i] = game_level.add_component<Sprite2DComponent>(*owners[i], "Sprite");
        sprites[i]->texture = 3;
        sprites[i]->set_texture_region(0, 0, 16, 16);
        sprites[i]->y_sort = true;
    }
    world2d::SpriteDraw draws[4];
    world2d::DrawItem items[4], scratch[4];
    size_t count = world2d::collect_sprites(draws, items, scratch, 4, size);
    assert(count == 2 && world2d::stats.culled == 1);  // the far sprite is off screen
    // Y-sort: the sprite lower on screen (Back, y = 20) draws last.
    assert(draws[items[0].index].y[0] < draws[items[1].index].y[0]);
    const auto& d = draws[items[1].index];
    assert(d.rectangle && d.x[0] == display_width / 2 - 8 && d.y[0] == display_height / 2 + 12 && d.source[2] == 16);
    // A rotated owner falls back to a quad; a missing texture is culled.
    root(back).transform.rotation = q(30);
    sprites[1]->texture = 9;
    count = world2d::collect_sprites(draws, items, scratch, 4, size);
    assert(count == 1 && !draws[items[0].index].rectangle);
    // A higher z_index wins over y-sort.
    sprites[1]->texture = 3;
    sprites[1]->set_z_index(5);
    count = world2d::collect_sprites(draws, items, scratch, 4, size);
    assert(count == 2 && draws[items[1].index].y[0] < 0 + display_height / 2);
}

void size_report() {
    std::printf("World2D components (host, %zu-bit pointers): Sprite2D %zu  Camera2D %zu  Collider2D %zu  CharacterBody2D %zu\n",
                sizeof(void*) * 8, sizeof(Sprite2DComponent), sizeof(Camera2DComponent), sizeof(Collider2DComponent),
                sizeof(CharacterBody2DComponent));
}
}  // namespace

int main() {
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    sprite_clips();
    camera_follow();
    character_body();
    triggers();
    draw_sort();
    sprite_resolution();
    size_report();
    std::puts("World2D sprite clips, camera, character body, triggers and draw sort tests passed.");
    return 0;
}
