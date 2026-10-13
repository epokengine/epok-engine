// Host contract tests for the World2D components and service (World2D initiative,
// phases 1 and 2): sprite frame clips, Camera2D following, CharacterBody2D move_and_slide
// with floor/wall/one-way contacts, trigger delivery to overlap_2d, registration lifetime,
// the stable draw-item sort, tile map cells and runtime edits, tile collision, rays and
// overlaps against tiles, tile drawing, parallax and canvas layers, paths and followers,
// lines, polygons, particles, on-screen notifiers, remote transforms, 2D audio, cut-out
// skeletons with IK and navigation. Compiles the real runtime headers; only the audio
// service and the cooked class table are provided here, as a generated build would.
#include <cassert>
#include <cmath>
#include <cstdio>
// A navigation scratch large enough for the test grids, as a cooked build would size it.
#define EPOK_WORLD2D_NAV_CELLS 256
#define EPOK_WORLD2D_NAV_OPEN 512
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
    {TileMapLayerComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<TileMapLayerComponent>, &object_destruct, sizeof(TileMapLayerComponent), alignof(TileMapLayerComponent), &ObjectPool<TileMapLayerComponent, 4>::acquire, &ObjectPool<TileMapLayerComponent, 4>::release},
    {Parallax2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<Parallax2DComponent>, &object_destruct, sizeof(Parallax2DComponent), alignof(Parallax2DComponent), &ObjectPool<Parallax2DComponent, 4>::acquire, &ObjectPool<Parallax2DComponent, 4>::release},
    {CanvasLayer2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<CanvasLayer2DComponent>, &object_destruct, sizeof(CanvasLayer2DComponent), alignof(CanvasLayer2DComponent), &ObjectPool<CanvasLayer2DComponent, 4>::acquire, &ObjectPool<CanvasLayer2DComponent, 4>::release},
    {CanvasModulate2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<CanvasModulate2DComponent>, &object_destruct, sizeof(CanvasModulate2DComponent), alignof(CanvasModulate2DComponent), &ObjectPool<CanvasModulate2DComponent, 4>::acquire, &ObjectPool<CanvasModulate2DComponent, 4>::release},
    {Path2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<Path2DComponent>, &object_destruct, sizeof(Path2DComponent), alignof(Path2DComponent), &ObjectPool<Path2DComponent, 4>::acquire, &ObjectPool<Path2DComponent, 4>::release},
    {PathFollow2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<PathFollow2DComponent>, &object_destruct, sizeof(PathFollow2DComponent), alignof(PathFollow2DComponent), &ObjectPool<PathFollow2DComponent, 4>::acquire, &ObjectPool<PathFollow2DComponent, 4>::release},
    {Line2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<Line2DComponent>, &object_destruct, sizeof(Line2DComponent), alignof(Line2DComponent), &ObjectPool<Line2DComponent, 4>::acquire, &ObjectPool<Line2DComponent, 4>::release},
    {Polygon2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<Polygon2DComponent>, &object_destruct, sizeof(Polygon2DComponent), alignof(Polygon2DComponent), &ObjectPool<Polygon2DComponent, 4>::acquire, &ObjectPool<Polygon2DComponent, 4>::release},
    {Particles2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<Particles2DComponent>, &object_destruct, sizeof(Particles2DComponent), alignof(Particles2DComponent), &ObjectPool<Particles2DComponent, 4>::acquire, &ObjectPool<Particles2DComponent, 4>::release},
    {AudioPlayer2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<AudioPlayer2DComponent>, &object_destruct, sizeof(AudioPlayer2DComponent), alignof(AudioPlayer2DComponent), &ObjectPool<AudioPlayer2DComponent, 4>::acquire, &ObjectPool<AudioPlayer2DComponent, 4>::release},
    {VisibleOnScreenNotifier2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<VisibleOnScreenNotifier2DComponent>, &object_destruct, sizeof(VisibleOnScreenNotifier2DComponent), alignof(VisibleOnScreenNotifier2DComponent), &ObjectPool<VisibleOnScreenNotifier2DComponent, 4>::acquire, &ObjectPool<VisibleOnScreenNotifier2DComponent, 4>::release},
    {RemoteTransform2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<RemoteTransform2DComponent>, &object_destruct, sizeof(RemoteTransform2DComponent), alignof(RemoteTransform2DComponent), &ObjectPool<RemoteTransform2DComponent, 4>::acquire, &ObjectPool<RemoteTransform2DComponent, 4>::release},
    {Skeleton2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<Skeleton2DComponent>, &object_destruct, sizeof(Skeleton2DComponent), alignof(Skeleton2DComponent), &ObjectPool<Skeleton2DComponent, 4>::acquire, &ObjectPool<Skeleton2DComponent, 4>::release},
    {NavigationRegion2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<NavigationRegion2DComponent>, &object_destruct, sizeof(NavigationRegion2DComponent), alignof(NavigationRegion2DComponent), &ObjectPool<NavigationRegion2DComponent, 4>::acquire, &ObjectPool<NavigationRegion2DComponent, 4>::release},
    {NavigationAgent2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<NavigationAgent2DComponent>, &object_destruct, sizeof(NavigationAgent2DComponent), alignof(NavigationAgent2DComponent), &ObjectPool<NavigationAgent2DComponent, 4>::acquire, &ObjectPool<NavigationAgent2DComponent, 4>::release},
    {RigidBody2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<RigidBody2DComponent>, &object_destruct, sizeof(RigidBody2DComponent), alignof(RigidBody2DComponent), &ObjectPool<RigidBody2DComponent, 4>::acquire, &ObjectPool<RigidBody2DComponent, 4>::release},
    {Joint2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<Joint2DComponent>, &object_destruct, sizeof(Joint2DComponent), alignof(Joint2DComponent), &ObjectPool<Joint2DComponent, 4>::acquire, &ObjectPool<Joint2DComponent, 4>::release},
    {SubViewport2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<SubViewport2DComponent>, &object_destruct, sizeof(SubViewport2DComponent), alignof(SubViewport2DComponent), &ObjectPool<SubViewport2DComponent, 4>::acquire, &ObjectPool<SubViewport2DComponent, 4>::release},
    {SplitScreen2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<SplitScreen2DComponent>, &object_destruct, sizeof(SplitScreen2DComponent), alignof(SplitScreen2DComponent), &ObjectPool<SplitScreen2DComponent, 4>::acquire, &ObjectPool<SplitScreen2DComponent, 4>::release},
    {Listener2D::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::None, world2d_bit, ObjectClassMultiple, &object_construct<Listener2D>, &object_destruct, sizeof(Listener2D), alignof(Listener2D), &ObjectPool<Listener2D, 4>::acquire, &ObjectPool<Listener2D, 4>::release},
    {Level::static_class_id, Object::static_class_id, ObjectFamily::Level, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Level), alignof(Level), nullptr, nullptr},
};
const size_t object_class_count = sizeof(object_classes) / sizeof(object_classes[0]);
}  // namespace epok

namespace {
ObjectRegistryStorage<128> registry_storage;
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
    for (int i = 0; i < 6; ++i) items[i] = world2d::DrawItem{keys[i], uint16_t(i + 1), 128};
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


// ---- tile maps --------------------------------------------------------------------------
// A 16 x 16 tile set: a solid block (atlas 0, 0), a one-way ledge whose shape is the top
// 4 pixels (atlas 1, 0) and an animated, non-colliding flower (atlas 0, 1; 2 frames).
constexpr TileInfo tile_infos[] = {
    {{0, 0}, 0, 0, TileCollision::Full, 0, {0, 0, 0, 0}, 1, 0, {0, 0}, Fixed(0.0)},
    {{1, 0}, 16, 0, TileCollision::Rect, tile_one_way, {0, 0, 16, 4}, 1, 0, {0, 0}, Fixed(0.0)},
    {{0, 1}, 0, 16, TileCollision::None, 0, {0, 0, 0, 0}, 2, 0, {16, 0}, Fixed(0.25)},
};
constexpr int32_t tile_custom[] = {7, 0, 3, 1, 0, 2};  // two layers per tile
constexpr TileSetData tile_set = {16, 16, 3, tile_infos, 2, tile_custom};
// Two chunks side by side: a floor on row 6 across both (x 0..31), a wall at x = 10 on
// rows 3..5, a one-way ledge at x 2..5 on row 3 and a flower at (12, 5).
uint16_t tile_cells[2 * tile_chunk_cells];
constexpr int16_t tile_grid[2] = {0, 1};
void build_tiles() {
    for (auto& cell : tile_cells) cell = 0;
    auto put = [](int x, int y, uint16_t value) { tile_cells[(x >> 4) * tile_chunk_cells + (y & 15) * 16 + (x & 15)] = value; };
    for (int x = 0; x < 32; ++x) put(x, 6, 1);
    for (int y = 3; y <= 5; ++y) put(10, y, 1);
    for (int x = 2; x <= 5; ++x) put(x, 3, 2);
    put(12, 5, 3);
}
TileMapLayerComponent* tile_layer(Actor* actor) {
    build_tiles();
    auto* layer = game_level.add_component<TileMapLayerComponent>(*actor, "Tiles");
    assert(layer);
    layer->tileset = &tile_set;
    layer->texture = 3;
    layer->cells = tile_cells;
    layer->grid = tile_grid;
    layer->grid_size[0] = 2;
    layer->grid_size[1] = 1;
    return layer;
}

// 7. Cells, atlas lookups, custom data, map conversion and runtime edits.
void tile_cells_and_edits() {
    reset();
    auto* map = spawn<Actor2D>("Map", -32, 0);
    auto* layer = tile_layer(map);
    assert(world2d::tile_layers.count == 1 && layer->slot);
    assert(!layer->is_cell_empty(0, 6) && layer->is_cell_empty(0, 5) && layer->is_cell_empty(-1, 6) && layer->is_cell_empty(40, 6));
    assert(layer->get_cell_atlas_x(3, 3) == 1 && layer->get_cell_atlas_y(3, 3) == 0 && layer->get_cell_atlas_x(3, 4) == -1);
    assert(layer->is_cell_solid(10, 4) && layer->is_cell_solid(3, 3) && !layer->is_cell_solid(12, 5));
    assert(layer->get_cell_custom_data(10, 4, 0) == 7 && layer->get_cell_custom_data(12, 5, 1) == 2 && layer->get_cell_custom_data(12, 5, 9) == 0);
    // The grid starts at the owner's position; negative coordinates floor.
    assert(layer->world_to_map_x(q(-32)) == 0 && layer->world_to_map_x(q(-33)) == -1 && layer->world_to_map_y(q(15.9)) == 0);
    assert(real(layer->map_to_world_x(0)) == -24 && real(layer->map_to_world_y(6)) == 104);
    // Edits override cooked cells, and restoring the authored tile frees the entry.
    assert(layer->set_cell(1, 1, 0, 0) && layer->get_cell_atlas_x(1, 1) == 0 && layer->edit_count == 1);
    assert(layer->erase_cell(0, 6) && layer->is_cell_empty(0, 6) && layer->edit_count == 2);
    assert(layer->set_cell(0, 6, 0, 0) && !layer->is_cell_empty(0, 6) && layer->edit_count == 1);
    assert(!layer->set_cell(2, 2, 5, 5));  // not a tile of the set
    assert(layer->set_cell_flipped(-5, -5, 1, 0, true, false, false));
    assert((layer->cell(-5, -5) & tile_flip_h) && layer->get_cell_atlas_x(-5, -5) == 1);
    // The table is bounded and reports a full table instead of overwriting.
    for (int i = 0; i < EPOK_WORLD2D_TILE_EDITS; ++i) layer->set_cell(100 + i, 0, 0, 0);
    assert(world2d::stats.dropped_tile_edits > 0 && world2d::stats.tile_edits == EPOK_WORLD2D_TILE_EDITS);
    // Destroying the layer forgets its edits.
    game_level.destroy_actor(map->id());
    registry_storage.collect_quarantined();
    assert(world2d::tile_layers.count == 0 && world2d::stats.tile_edits == 0);
}

// 8. Bodies land on, walk along and are blocked by tiles; one-way tiles only catch falls.
void tile_collision() {
    reset();
    auto* map = spawn<Actor2D>("Map", 0, 0);
    tile_layer(map);
    auto* hero = spawn<Actor2D>("Hero", 40, 40);
    box(hero, 12, 24);
    auto* body = game_level.add_component<CharacterBody2DComponent>(*hero, "Body");
    const Fixed dt(epok::time.delta_raw, Fixed::RAW);
    auto run = [&](double vx, double vy, int steps) {
        for (int i = 0; i < steps; ++i) {
            body->set_velocity(q(vx), q(vy));
            epok::time.begin_tick();
            body->move_and_slide();
        }
    };
    // The floor's top is y = 96: the body's centre rests 12 pixels above it.
    for (int i = 0; i < 120 && !body->is_on_floor(); ++i) {
        body->set_velocity(q(0), body->velocity_y() + q(600) * dt);
        epok::time.begin_tick();
        body->move_and_slide();
    }
    assert(body->is_on_floor() && real(root(hero).transform.position[1]) == 84);
    assert(body->last_collision_actor() == map->id() && body->last_collision_is_tile() && body->last_tile_y() == 6);
    // Walking right slides along the floor and stops at the wall's left face (x = 160).
    run(120, 10, 120);
    assert(close(real(root(hero).transform.position[0]), 160 - 6, 1e-3) && body->is_on_wall());
    // Walking left crosses tile seams without catching and keeps its floor.
    run(-120, 10, 30);
    assert(real(root(hero).transform.position[0]) < 120 && body->is_on_floor());
    // Jumping from below passes the one-way ledge (top y = 48)...
    root(hero).transform.position[0] = q(56);
    root(hero).transform.position[1] = q(84);
    run(0, -400, 12);
    assert(real(root(hero).transform.position[1]) < 48 - 12);
    // ...and falling back lands on its top edge.
    for (int i = 0; i < 120 && !(body->is_on_floor() && real(root(hero).transform.position[1]) < 60); ++i) run(0, 300, 1);
    assert(body->is_on_floor() && real(root(hero).transform.position[1]) == 36);
    assert(body->last_tile_y() == 3 && body->last_tile_x() >= 2 && body->last_tile_x() <= 5);
    // A disabled tile layer stops colliding.
    world2d::find<TileMapLayerComponent>(map)->set_collision_enabled(false);
    run(0, 300, 4);
    assert(!body->is_on_floor() && real(root(hero).transform.position[1]) > 36);
}

// 9. Rays and overlap queries see tiles, nearest first.
void tile_queries() {
    reset();
    auto* map = spawn<Actor2D>("Map", 0, 0);
    tile_layer(map);
    const Fixed origin[2] = {q(8), q(0)}, down[2] = {q(0), q(200)};
    const SpatialHit2D hit = world2d::service.raycast(origin, down, 0xffffffffu, false);
    assert(hit && close(real(hit.point[1]), 96, 0.01) && world2d::service.actor_at(hit.index) == map->id());
    // A long diagonal ray crosses many cells and still stops at the first solid one.
    const Fixed start[2] = {q(-40), q(-40)}, across[2] = {q(400), q(400)};
    const SpatialHit2D diagonal = world2d::service.raycast(start, across, 0xffffffffu, false);
    assert(diagonal && real(diagonal.point[1]) <= 96.01 && real(diagonal.point[1]) >= 47.99);
    // A ray up through the one-way ledge passes it.
    const Fixed below[2] = {q(40), q(80)}, up[2] = {q(0), q(-80)};
    assert(!world2d::service.raycast(below, up, 0xffffffffu, false));
    ObjectId found[4];
    const Aabb2D on_floor = {{q(20), q(90)}, {q(30), q(100)}}, in_air = {{q(20), q(70)}, {q(30), q(80)}};
    assert(world2d::service.overlap(on_floor, found, 4, 0xffffffffu, false) == 1 && found[0] == map->id());
    assert(world2d::service.overlap(in_air, found, 4, 0xffffffffu, false) == 0);
}

// 10. Tile layers draw as one sorted entry each; tiles cull, animate and flip.
void tile_drawing() {
    reset();
    auto size = [](int index, uint16_t& w, uint16_t& h) { if (index != 3) return false; w = 64; h = 64; return true; };
    auto* map = spawn<Actor2D>("Map", -display_width / 2., -display_height / 2.);
    auto* layer = tile_layer(map);
    world2d::SpriteDraw draws[16];
    world2d::DrawItem items[16], scratch[16];
    size_t count = world2d::collect_sprites(draws, items, scratch, 16, size);
    assert(count == 1 && draws[items[0].index].kind == world2d::DrawKind::Tiles && draws[items[0].index].component == layer);
    // The map's origin is the screen's top-left corner: the floor tiles that fit on screen,
    // 3 wall, 4 ledge and 1 flower tiles, all exact rectangles.
    const unsigned visible = (display_width / 16 < 32 ? display_width / 16 : 32) + 8;
    unsigned tiles = 0, rectangles = 0, flower_u = 0;
    auto visit = [&](const world2d::TileDraw& tile) {
        ++tiles;
        rectangles += tile.rectangle;
        if (tile.v == 16) flower_u = tile.u;
    };
    world2d::each_tile(draws[items[0].index], 64, 64, visit);
    assert(tiles == visible && rectangles == visible && flower_u == 0);
    // Animation time moves the flower to its second frame.
    while (world2d::tile_clock < q(0.3)) step();
    tiles = rectangles = 0;
    world2d::each_tile(draws[items[0].index], 64, 64, visit);
    assert(flower_u == 16);
    // A flipped tile falls back to a quad; scrolling the map off screen culls tiles.
    layer->set_cell_flipped(0, 6, 0, 0, false, true, false);
    tiles = rectangles = 0;
    world2d::each_tile(draws[items[0].index], 64, 64, visit);
    assert(tiles == visible && rectangles == visible - 1);
    root(map).transform.position[0] = q(-1000);
    tiles = 0;
    world2d::each_tile(draws[items[0].index], 64, 64, visit);
    assert(tiles == 0);
    // An invisible layer is not collected.
    layer->set_visible(false);
    assert(world2d::collect_sprites(draws, items, scratch, 16, size) == 0);
}

// 11. Parallax layers scroll against the camera, drift and repeat to fill the screen.
void parallax() {
    reset();
    auto size = [](int index, uint16_t& w, uint16_t& h) { if (index != 3) return false; w = 64; h = 32; return true; };
    auto* hero = spawn<Actor2D>("Hero", 0, 0);
    game_level.add_component<Camera2DComponent>(*hero, "Camera");
    auto* hills = spawn<Actor2D>("Hills", 10, 20);
    auto* parallax = game_level.add_component<Parallax2DComponent>(*hills, "Parallax");
    parallax->set_scroll_scale(q(0.5), q(1));
    step();
    assert(real(root(hills).transform.position[0]) == 10 && real(root(hills).transform.position[1]) == 20);
    // The camera moves 100 pixels right: a half-speed layer follows by 50.
    root(hero).transform.position[0] = q(100);
    step();
    assert(real(root(hills).transform.position[0]) == 60 && real(root(hills).transform.position[1]) == 20);
    // Autoscroll drifts by its speed every second.
    parallax->set_autoscroll(q(30), q(0));
    for (int i = 0; i < 60; ++i) step();
    const double seconds = 60 * real(Fixed(epok::time.delta_raw, Fixed::RAW));
    assert(close(real(root(hills).transform.position[0]), 60 + 30 * seconds, 0.6));
    // A repeating sprite is copied across the screen at its period.
    parallax->set_autoscroll(q(0), q(0));
    parallax->set_repeat_size(q(64), q(0));
    auto* sprite = game_level.add_component<Sprite2DComponent>(*hills, "Sprite");
    sprite->texture = 3;
    world2d::SpriteDraw draws[32];
    world2d::DrawItem items[32], scratch[32];
    const size_t count = world2d::collect_sprites(draws, items, scratch, 32, size);
    assert(count >= size_t(display_width / 64) && count <= size_t(display_width / 64) + 3);
    for (size_t i = 1; i < count; ++i) assert(draws[items[i].index].x[0] - draws[items[i - 1].index].x[0] == 64);
}

// 12. Canvas layers order whole canvases, screen canvases ignore the camera and a
// CanvasModulate2D tints only its own canvas.
void canvas_layers() {
    reset();
    auto size = [](int index, uint16_t& w, uint16_t& h) { if (index != 3) return false; w = 16; h = 16; return true; };
    auto* hero = spawn<Actor2D>("Hero", 500, 0);
    game_level.add_component<Camera2DComponent>(*hero, "Camera");
    auto* world_sprite = spawn<Actor2D>("World", 500, 0);
    auto* hud = spawn<Actor2D>("Hud", 0, 0);
    auto* canvas = game_level.add_component<CanvasLayer2DComponent>(*hud, "Canvas");
    auto* icon = spawn<Actor2D>("Icon", 20, 30);
    root(icon).attach_parent = root(hud).id();
    Sprite2DComponent* a = game_level.add_component<Sprite2DComponent>(*world_sprite, "Sprite");
    Sprite2DComponent* b = game_level.add_component<Sprite2DComponent>(*icon, "Sprite");
    a->texture = b->texture = 3;
    a->set_z_index(100);  // a higher z on the world canvas still draws under canvas 1
    auto* night = spawn<Actor2D>("Night", 0, 0);
    auto* modulate = game_level.add_component<CanvasModulate2DComponent>(*night, "Tint");
    modulate->set_color(128, 128, 255);
    step();
    world2d::SpriteDraw draws[4];
    world2d::DrawItem items[4], scratch[4];
    size_t count = world2d::collect_sprites(draws, items, scratch, 4, size);
    assert(count == 2);
    const auto& under = draws[items[0].index];
    const auto& over = draws[items[1].index];
    // The icon is drawn in screen pixels at its position, centred on (20, 30).
    assert(over.x[0] == 12 && over.y[0] == 22 && over.rectangle);
    assert(under.x[0] == display_width / 2 - 8);
    // The modulate sits on the world canvas: only the world sprite is tinted.
    assert(under.color[0] == 128 && under.color[2] == 255 && over.color[0] == 255);
    // Moving the canvas behind the world reverses the order; hiding it hides the icon.
    canvas->set_layer(-1);
    count = world2d::collect_sprites(draws, items, scratch, 4, size);
    assert(count == 2 && draws[items[0].index].x[0] == 12);
    canvas->set_visible(false);
    assert(world2d::collect_sprites(draws, items, scratch, 4, size) == 1);
}

// 13. Followers walk a path at their speed, wrap, bounce, turn with it and carry riders.
void path_follow() {
    reset();
    static constexpr Fixed points[] = {Fixed(0.0), Fixed(0.0), Fixed(100.0), Fixed(0.0), Fixed(100.0), Fixed(50.0)};
    static constexpr Fixed lengths[] = {Fixed(0.0), Fixed(100.0), Fixed(150.0)};
    auto* track = spawn<Actor2D>("Track", 20, 10);
    auto* path = game_level.add_component<Path2DComponent>(*track, "Path");
    path->points = points;
    path->lengths = lengths;
    path->point_count = 3;
    assert(real(path->get_length()) == 150 && real(path->sample_x(125)) == 120 && real(path->sample_y(125)) == 35);
    auto* cart = spawn<Actor2D>("Cart", 0, 0);
    root(cart).attach_parent = root(track).id();
    auto* follow = game_level.add_component<PathFollow2DComponent>(*cart, "Follow");
    follow->speed = q(50);
    follow->rotates = true;
    const double dt = real(Fixed(epok::time.delta_raw, Fixed::RAW));
    const int second = int(1.0 / dt + 0.5);
    for (int i = 0; i < second; ++i) step();
    // Local to the track: 50 pixels along the first segment, facing right.
    assert(close(real(root(cart).transform.position[0]), 50, 0.5) && close(real(root(cart).transform.position[1]), 0, 0.01));
    assert(close(real(root(cart).transform.rotation), 0, 0.5));
    for (int i = 0; i < second * 3 / 2; ++i) step();
    // 125 pixels: on the vertical segment, facing down; then it wraps to the start.
    assert(close(real(root(cart).transform.position[0]), 100, 0.5) && close(real(root(cart).transform.position[1]), 25, 1.0));
    assert(close(real(root(cart).transform.rotation), 90, 0.5));
    follow->set_progress_ratio(q(0.5));
    assert(close(real(follow->get_progress()), 75, 0.01));
    follow->ping_pong = true;
    follow->set_progress(q(149));
    for (int i = 0; i < 4; ++i) step();
    assert(follow->get_speed() < 0.0 && real(follow->get_progress()) < 150);
    // A platform moving along a path carries the body standing on it.
    reset();
    static constexpr Fixed flat[] = {Fixed(0.0), Fixed(0.0), Fixed(1000.0), Fixed(0.0)};
    static constexpr Fixed flat_lengths[] = {Fixed(0.0), Fixed(1000.0)};
    auto* rail = spawn<Actor2D>("Rail", 0, 100);
    auto* line = game_level.add_component<Path2DComponent>(*rail, "Path");
    line->points = flat;
    line->lengths = flat_lengths;
    line->point_count = 2;
    auto* platform = spawn<Actor2D>("Platform", 0, 0);
    root(platform).attach_parent = root(rail).id();
    box(platform, 64, 16);
    auto* mover = game_level.add_component<PathFollow2DComponent>(*platform, "Follow");
    mover->speed = q(60);
    auto* hero = spawn<Actor2D>("Hero", 0, 70);
    box(hero, 12, 24);
    auto* body = game_level.add_component<CharacterBody2DComponent>(*hero, "Body");
    auto frame = [&] {
        body->set_velocity(q(0), q(300));
        epok::time.begin_tick();
        body->move_and_slide();
        world2d::service.step(Fixed(epok::time.delta_raw, Fixed::RAW), &game_level);
    };
    for (int i = 0; i < 60 && !body->is_on_floor(); ++i) frame();
    assert(body->is_on_floor() && body->last_collision_actor() == platform->id());
    const double offset = real(root(hero).transform.position[0]) - real(root(platform).transform.position[0]);
    const double start = real(root(platform).transform.position[0]);
    for (int i = 0; i < 60; ++i) frame();
    // The rider kept its place on the platform while the platform travelled.
    assert(real(root(platform).transform.position[0]) - start > 30 && body->is_on_floor());
    assert(close(real(root(hero).transform.position[0]) - real(root(platform).transform.position[0]), offset, 0.01));
}

// 14. Lines and polygons expand into on-screen quads and triangles.
void shapes() {
    reset();
    auto* wire = spawn<Actor2D>("Wire", 0, 0);
    auto* line = game_level.add_component<Line2DComponent>(*wire, "Line");
    assert(line->add_point(q(-50), q(0)) && line->add_point(q(0), q(0)) && line->add_point(q(0), q(40)));
    line->set_width(q(4));
    line->set_colors(255, 0, 0, 0, 0, 255);
    auto* shape = spawn<Actor2D>("Shape", 2000, 0);
    auto* polygon = game_level.add_component<Polygon2DComponent>(*shape, "Polygon");
    static constexpr Fixed vertices[] = {Fixed(0.0), Fixed(0.0), Fixed(16.0), Fixed(0.0), Fixed(16.0), Fixed(16.0), Fixed(0.0), Fixed(16.0)};
    static constexpr uint16_t triangles[] = {0, 1, 2, 0, 2, 3};
    polygon->vertices = vertices;
    polygon->vertex_count = 4;
    polygon->triangles = triangles;
    polygon->triangle_count = 2;
    world2d::SpriteDraw draws[8];
    world2d::DrawItem items[8], scratch[8];
    size_t count = world2d::collect_sprites(draws, items, scratch, 8, [](int, uint16_t&, uint16_t&) { return false; });
    assert(count == 2);
    unsigned quads = 0, triangles_drawn = 0;
    const world2d::ShapeQuad* first = nullptr;
    world2d::ShapeQuad copy{};
    for (size_t i = 0; i < count; ++i) {
        world2d::each_line_quad(draws[items[i].index], [&](const world2d::ShapeQuad& quad) {
            if (!quads++) { copy = quad; first = &copy; }
        });
        world2d::each_polygon_triangle(draws[items[i].index], [&](const world2d::ShapeTriangle&) { ++triangles_drawn; });
    }
    // Two segments; the polygon is far off screen.
    assert(quads == 2 && triangles_drawn == 0 && first);
    // The first segment runs left to right, 4 pixels tall, red at its start.
    assert((first->y[2] - first->y[0] == 4 || first->y[0] - first->y[2] == 4) && first->color[0][0] == 255 && first->color[0][2] == 0 && first->color[1][2] > 0);
    root(shape).transform.position[0] = q(0);
    world2d::collect_sprites(draws, items, scratch, 8, [](int, uint16_t&, uint16_t&) { return false; });
    triangles_drawn = 0;
    for (size_t i = 0; i < 2; ++i)
        world2d::each_polygon_triangle(draws[items[i].index], [&](const world2d::ShapeTriangle& t) {
            ++triangles_drawn;
            assert(t.x[0] == display_width / 2 && t.y[0] == display_height / 2);
        });
    assert(triangles_drawn == 2);
}

// 15. Emitters stream or burst, age, report a finished one-shot and share a bounded pool.
void particles() {
    reset();
    auto* fire = spawn<Actor2D>("Fire", 0, 0);
    auto* emitter = game_level.add_component<Particles2DComponent>(*fire, "Particles");
    emitter->amount = 8;
    emitter->lifetime = q(0.5);
    emitter->explosiveness = q(1);
    emitter->one_shot = true;
    emitter->texture = 3;
    emitter->region[2] = 8;
    emitter->region[3] = 8;
    step();
    assert(emitter->get_alive_count() == 8 && !emitter->is_emitting());
    world2d::SpriteDraw draws[4];
    world2d::DrawItem items[4], scratch[4];
    const size_t count = world2d::collect_sprites(draws, items, scratch, 4, [](int, uint16_t&, uint16_t&) { return false; });
    assert(count == 1 && draws[items[0].index].kind == world2d::DrawKind::Particles);
    unsigned drawn = 0;
    world2d::each_particle(draws[items[0].index], 16, 16, [&](const world2d::SpriteDraw& p) {
        ++drawn;
        assert(p.source[2] == 8 && p.texture == 3);
    });
    assert(drawn == 8);
    for (int i = 0; i < 40 && emitter->get_alive_count(); ++i) step();
    step();
    assert(emitter->get_alive_count() == 0 && emitter->take_finished() && !emitter->take_finished());
    // A stream spawns amount / lifetime particles per second and never exceeds amount.
    emitter->one_shot = false;
    emitter->explosiveness = q(0);
    emitter->amount = 200;
    emitter->lifetime = q(4);
    emitter->restart();
    for (int i = 0; i < 300; ++i) step();
    assert(emitter->get_alive_count() <= EPOK_WORLD2D_PARTICLES && world2d::stats.dropped_particles > 0);
    // Destroying the emitter frees its particles.
    game_level.destroy_actor(fire->id());
    registry_storage.collect_quarantined();
    step();
    assert(world2d::stats.particles == 0);
}

// 16. Notifiers report entering and leaving the view and can switch their owner off.
void notifiers() {
    reset();
    auto* hero = spawn<Actor2D>("Hero", 0, 0);
    game_level.add_component<Camera2DComponent>(*hero, "Camera");
    auto* enemy = spawn<Actor2D>("Enemy", 1000, 0);
    auto* notifier = game_level.add_component<VisibleOnScreenNotifier2DComponent>(*enemy, "Notifier");
    notifier->disable_when_hidden = true;
    step();
    assert(!notifier->is_on_screen() && !enemy->active());
    root(hero).transform.position[0] = q(950);
    step();
    assert(notifier->is_on_screen() && notifier->take_screen_entered() && !notifier->take_screen_entered() && enemy->active());
    root(hero).transform.position[0] = q(0);
    step();
    assert(notifier->take_screen_exited() && !enemy->active());
}

// 17. A remote transform pushes its world transform onto another actor.
void remote_transforms() {
    reset();
    auto* hand = spawn<Actor2D>("Hand", 30, 40);
    root(hand).transform.rotation = q(45);
    auto* sword = spawn<Actor2D>("Sword", 0, 0);
    auto* remote = game_level.add_component<RemoteTransform2DComponent>(*hand, "Remote");
    remote->set_target(sword->id());
    remote->update_scale = false;
    step();
    assert(real(root(sword).transform.position[0]) == 30 && real(root(sword).transform.position[1]) == 40);
    assert(real(root(sword).transform.rotation) == 45);
}

// 18. 2D audio fades with distance from the view's centre and pans to its side.
void audio_2d() {
    reset();
    auto* hero = spawn<Actor2D>("Hero", 0, 0);
    game_level.add_component<Camera2DComponent>(*hero, "Camera");
    auto* bell = spawn<Actor2D>("Bell", 80, 0);
    auto* player = game_level.add_component<AudioPlayer2DComponent>(*bell, "Audio");
    player->max_distance = q(320);
    step();
    // A quarter of the way to silence, halfway to the right edge.
    assert(close(real(player->source.volume), 0.75, 0.01) && player->source.pan > 60 && player->source.pan < 66);
    root(bell).transform.position[0] = q(-400);
    step();
    assert(player->source.volume.raw() == 0 && player->source.pan == -127);
}

// 19. Skeletons blend poses (rotations along the shorter arc), loop or finish, and
// two-bone IK puts the tip on the target, bending to the chosen side.
Fixed world_x(Actor* actor) {
    Affine2D m;
    assert(world2d::root_world(root(actor), m));
    return m.t[0];
}
Fixed world_y(Actor* actor) {
    Affine2D m;
    assert(world2d::root_world(root(actor), m));
    return m.t[1];
}
void skeletons() {
    reset();
    auto* body = spawn<Actor2D>("Body", 0, 0);
    auto* arm = spawn<Actor2D>("Arm", 10, 0);
    root(arm).attach_parent = root(body).id();
    auto* skeleton = game_level.add_component<Skeleton2DComponent>(*body, "Skeleton");
    assert(skeleton);
    static const Fixed rest[] = {q(0), q(10), q(0)};
    static const Fixed times[] = {q(0), q(1)};
    static const Fixed poses[] = {q(0), q(10), q(0), q(90), q(10), q(4)};
    static const Fixed wrap_times[] = {q(0), q(1)};
    static const Fixed wrap_poses[] = {q(170), q(0), q(0), q(-170), q(0), q(0)};
    static SkeletonClip2D clips[3];
    clips[0] = SkeletonClip2D{times, poses, 2, true, q(2)};
    clips[1] = SkeletonClip2D{wrap_times, wrap_poses, 2, false, q(1)};
    clips[2] = SkeletonClip2D{times, poses, 2, false, q(1)};
    skeleton->bones[0] = arm->id();
    skeleton->rest = rest;
    skeleton->bone_count = 1;
    skeleton->clips = clips;
    skeleton->clip_count = 3;
    skeleton->play(0);
    skeleton->seek(q(0.5));
    assert(close(real(root(arm).transform.rotation), 45, 0.01) && close(real(root(arm).transform.position[1]), 2, 0.01));
    // Past the last key a loop blends back into the first pose.
    skeleton->seek(q(1.5));
    assert(close(real(root(arm).transform.rotation), 45, 0.01));
    skeleton->seek(q(2.5));
    assert(close(real(skeleton->get_time()), 0.5, 0.001));
    // 170 to -170 crosses 180, not 0.
    skeleton->play(1);
    skeleton->seek(q(0.5));
    assert(close(real(root(arm).transform.rotation) < 0 ? -real(root(arm).transform.rotation) : real(root(arm).transform.rotation), 180, 0.01));
    // A non-looping animation stops on its last pose and reports it once.
    skeleton->play(2);
    for (int i = 0; i < 90; ++i) step();
    assert(!skeleton->is_playing() && skeleton->take_finished() && !skeleton->take_finished());
    assert(close(real(root(arm).transform.rotation), 90, 0.01));
    skeleton->reset_pose();
    assert(real(root(arm).transform.rotation) == 0 && real(root(arm).transform.position[1]) == 0);

    // IK: shoulder, elbow 10 px along, hand 10 px further.
    reset();
    auto* shoulder = spawn<Actor2D>("Shoulder", 0, 0);
    auto* elbow = spawn<Actor2D>("Elbow", 10, 0);
    auto* hand = spawn<Actor2D>("Hand", 10, 0);
    auto* target = spawn<Actor2D>("Target", 10, 10);
    root(elbow).attach_parent = root(shoulder).id();
    root(hand).attach_parent = root(elbow).id();
    auto* rig = game_level.add_component<Skeleton2DComponent>(*shoulder, "Skeleton");
    rig->bones[0] = shoulder->id();
    rig->bones[1] = elbow->id();
    rig->bone_count = 2;
    rig->ik[0] = SkeletonIk2D{0, 1, hand->id(), target->id(), true, true};
    rig->ik_count = 1;
    step();
    assert(close(real(world_x(hand)), 10, 0.6) && close(real(world_y(hand)), 10, 0.6));
    const double bent = real(world_y(elbow)) - 5;  // the elbow leaves the shoulder-target line
    rig->ik[0].bend_positive = false;
    step();
    assert(close(real(world_x(hand)), 10, 0.6) && close(real(world_y(hand)), 10, 0.6));
    assert((real(world_y(elbow)) - 5) * bent < 0);
    // Out of reach: the arm points straight at the target.
    root(target).transform.position[0] = q(100);
    root(target).transform.position[1] = q(0);
    step();
    assert(close(real(world_x(hand)), 20, 0.6) && close(real(world_y(hand)), 0, 0.6));
    rig->set_ik_enabled(0, false);
    root(target).transform.position[1] = q(50);
    step();
    assert(close(real(world_y(hand)), 0, 0.6));
}

// 20. Navigation: grid A* bends around walls without cutting corners and stops next to
// an unreachable target; navmesh paths turn at the inner corner; agents walk to targets.
uint8_t wall_bits[13];
world2d::NavGrid2D wall_grid(int gap_rows) {
    for (auto& b : wall_bits) b = 0;
    // 10 x 10 cells of 16 px; column 4 blocked except the last `gap_rows` rows.
    for (int y = 0; y < 10 - gap_rows; ++y) {
        const int i = y * 10 + 4;
        wall_bits[i >> 3] |= uint8_t(1u << (i & 7));
    }
    world2d::NavGrid2D grid;
    grid.blocked = wall_bits;
    grid.columns = grid.rows = 10;
    grid.cell[0] = grid.cell[1] = q(16);
    return grid;
}
void navigation() {
    reset();
    Fixed out[world2d::nav_path_limit][2];
    bool reached = false;
    auto grid = wall_grid(2);
    const Fixed from[2] = {q(8), q(8)}, to[2] = {q(136), q(8)};
    size_t n = world2d::grid_path(grid, from, to, out, world2d::nav_path_limit, reached);
    assert(reached && n >= 2);
    bool below = false;
    for (size_t i = 0; i < n; ++i) below |= real(out[i][1]) > 128;
    assert(below && real(out[n - 1][0]) == 136 && real(out[n - 1][1]) == 8);
    // No squeezing between two diagonal blocks.
    uint8_t corner_bits[2] = {uint8_t((1u << 1) | (1u << 3)), 0};
    world2d::NavGrid2D corner;
    corner.blocked = corner_bits;
    corner.columns = corner.rows = 3;
    corner.cell[0] = corner.cell[1] = q(16);
    assert(!world2d::grid_line_of_sight(corner, 0, 0, 1, 1));
    // A full wall: the path ends at the closest reachable cell.
    grid = wall_grid(0);
    n = world2d::grid_path(grid, from, to, out, world2d::nav_path_limit, reached);
    assert(!reached && n >= 1 && real(out[n - 1][0]) == 3 * 16 + 8 && real(out[n - 1][1]) == 8);

    // An L-shaped navmesh: across the top, then down the right side.
    static const Fixed vertices[] = {q(0), q(0), q(100), q(0), q(100), q(100), q(70), q(100), q(70), q(30), q(0), q(30)};
    static const uint8_t triangles[] = {0, 1, 4, 0, 4, 5, 1, 2, 4, 2, 3, 4};
    static const int8_t neighbours[] = {-1, 2, 1, 0, -1, -1, -1, 3, 0, -1, -1, 2};
    world2d::NavMesh2D mesh{vertices, triangles, neighbours, 4};
    const Fixed a[2] = {q(10), q(15)}, b[2] = {q(85), q(90)};
    n = world2d::mesh_path(mesh, a, b, out, world2d::nav_path_limit, reached);
    assert(reached && n == 2 && real(out[0][0]) == 70 && real(out[0][1]) == 30 && real(out[1][0]) == 85);
    // Off the mesh: the closest point on its edge, here on the vertical arm.
    const Fixed outside[2] = {q(40), q(80)};
    n = world2d::mesh_path(mesh, a, outside, out, world2d::nav_path_limit, reached);
    assert(!reached && n >= 1 && close(real(out[n - 1][0]), 70, 0.01) && close(real(out[n - 1][1]), 80, 0.01));

    // An agent on the grid walks round the wall to its target and reports it once.
    auto* map = spawn<Actor2D>("Floor", 0, 0);
    auto* region = game_level.add_component<NavigationRegion2DComponent>(*map, "Region");
    region->grid = wall_grid(2);
    assert(region->is_walkable(q(8), q(8)) && !region->is_walkable(q(72), q(8)));
    auto* walker = spawn<Actor2D>("Walker", 8, 8);
    auto* agent = game_level.add_component<NavigationAgent2DComponent>(*walker, "Agent");
    agent->max_speed = q(120);
    agent->set_target_position(q(136), q(8));
    for (int i = 0; i < 600 && !agent->is_navigation_finished(); ++i) step();
    assert(agent->is_navigation_finished() && agent->is_target_reachable() && agent->take_arrived() && !agent->take_arrived());
    assert(close(real(root(walker).transform.position[0]), 136, 4.5) && close(real(root(walker).transform.position[1]), 8, 4.5));
    // Following an actor re-plans as it moves.
    auto* goal = spawn<Actor2D>("Goal", 136, 136);
    agent->set_target_actor(goal->id());
    for (int i = 0; i < 600 && !agent->is_navigation_finished(); ++i) step();
    assert(close(real(root(walker).transform.position[1]), 136, 4.5));
    root(goal).transform.position[0] = q(8);
    for (int i = 0; i < 900; ++i) step();
    assert(close(real(root(walker).transform.position[0]), 8, 4.5));
}

// 21. Rigid bodies fall, bounce, slide to a stop, trade momentum, get pushed by
// characters, roll, sleep, and joints hold them as rods, ropes and springs.
RigidBody2DComponent* rigid(Actor* actor, double size, bool circle = false) {
    auto* shape = box(actor, size, size);
    if (circle) {
        shape->shape.shape = Collider2D::Circle;
        shape->shape.radius = q(size / 2);
    }
    auto* body = game_level.add_component<RigidBody2DComponent>(*actor, "Body");
    assert(body);
    return body;
}
void rigid_bodies() {
    reset();
    auto* floor = spawn<Actor2D>("Floor", 0, 100);
    box(floor, 400, 20);
    auto* crate = spawn<Actor2D>("Crate", 0, 0);
    auto* body = rigid(crate, 16);
    body->bounce = q(0);
    for (int i = 0; i < 120; ++i) step();
    // Resting on the floor's top (90), centred 8 above it, asleep, contact reported once.
    assert(close(real(root(crate).transform.position[1]), 82, 0.5));
    assert(body->is_sleeping() && body->take_contact() && !body->take_contact() && real(body->get_impact()) > 100);
    // A bouncy ball comes back up after its first contact.
    auto* ball = spawn<Actor2D>("Ball", 100, 0);
    auto* bouncy = rigid(ball, 16, true);
    bouncy->bounce = q(0.6);
    bool rose = false;
    for (int i = 0; i < 90; ++i) {
        step();
        rose |= bouncy->velocity[1].raw() < 0;
    }
    assert(rose);
    // Friction brings a sliding body to rest; an impulse wakes it.
    body->apply_impulse(q(120), q(0));
    assert(!body->is_sleeping());
    const double start = real(root(crate).transform.position[0]);
    for (int i = 0; i < 120; ++i) step();
    assert(real(root(crate).transform.position[0]) > start + 4 && body->velocity[0].raw() == 0);

    // Two equal bodies, perfectly elastic: the moving one stops, the other takes its speed.
    reset();
    auto* ground = spawn<Actor2D>("Ground", 0, 100);
    box(ground, 400, 20);
    auto* left = spawn<Actor2D>("Left", -40, 82);
    auto* right = spawn<Actor2D>("Right", 0, 82);
    auto* a = rigid(left, 16);
    auto* b = rigid(right, 16);
    a->bounce = b->bounce = q(1);
    a->friction = b->friction = q(0);
    a->set_velocity(q(120), q(0));
    for (int i = 0; i < 40; ++i) step();
    assert(real(b->velocity[0]) > 100 && close(real(a->velocity[0]), 0, 1));

    // A character walking into a pushable crate moves it.
    reset();
    auto* level_floor = spawn<Actor2D>("Floor", 0, 100);
    box(level_floor, 400, 20);
    auto* hero = spawn<Actor2D>("Hero", -30, 80);
    box(hero, 12, 20);
    auto* walker = game_level.add_component<CharacterBody2DComponent>(*hero, "Body");
    auto* pushed = spawn<Actor2D>("Crate", 0, 82);
    auto* crate_body = rigid(pushed, 16);
    for (int i = 0; i < 90; ++i) {
        epok::time.begin_tick();
        walker->set_velocity(q(60), walker->velocity_y() + q(12));
        walker->move_and_slide();
        world2d::service.step(Fixed(epok::time.delta_raw, Fixed::RAW), &game_level);
    }
    assert(real(root(pushed).transform.position[0]) > 20 && real(root(hero).transform.position[0]) > -10);
    crate_body->pushable = false;
    const double held = real(root(pushed).transform.position[0]);
    for (int i = 0; i < 30; ++i) {
        epok::time.begin_tick();
        walker->set_velocity(q(60), walker->velocity_y() + q(12));
        walker->move_and_slide();
        world2d::service.step(Fixed(epok::time.delta_raw, Fixed::RAW), &game_level);
    }
    // Only the last push's momentum carries it on, a few pixels before friction stops it.
    assert(real(root(pushed).transform.position[0]) < held + 4 && crate_body->velocity[0].raw() == 0);

    // A ball rolling along the floor turns.
    reset();
    auto* lane = spawn<Actor2D>("Lane", 0, 100);
    box(lane, 600, 20);
    auto* wheel = spawn<Actor2D>("Wheel", 0, 82);
    auto* rolling = rigid(wheel, 16, true);
    rolling->friction = q(0);
    rolling->set_velocity(q(50), q(0));
    for (int i = 0; i < 30; ++i) step();
    assert(rolling->get_angular_velocity().raw() > 0 && root(wheel).transform.rotation.raw() != 0);

    // A rod pendulum from the joint node keeps its length and swings down.
    reset();
    auto* pivot = spawn<Actor2D>("Pivot", 0, 0);
    auto* bob = spawn<Actor2D>("Bob", 32, 0);
    auto* weight = rigid(bob, 8);
    weight->can_sleep = false;
    auto* rod = game_level.add_component<Joint2DComponent>(*pivot, "Rod");
    rod->body_b = bob->id();
    rod->length = q(32);
    double lowest = 0;
    for (int i = 0; i < 40; ++i) {
        step();
        const double x = real(root(bob).transform.position[0]), y = real(root(bob).transform.position[1]);
        assert(close(std::sqrt(x * x + y * y), 32, 1.0));
        if (y > lowest) lowest = y;
    }
    assert(lowest > 28);
    // A rope lets the body fall until it is taut.
    rod->kind = JointKind2D::Rope;
    rod->length = q(48);
    root(bob).transform.position[0] = q(0);
    root(bob).transform.position[1] = q(10);
    weight->set_velocity(q(0), q(0));
    step();
    assert(real(root(bob).transform.position[1]) > 10);
    for (int i = 0; i < 60; ++i) step();
    assert(close(real(root(bob).transform.position[1]), 48, 1.0));
    // A spring settles below its length by the weight it carries.
    rod->kind = JointKind2D::Spring;
    rod->length = q(32);
    rod->stiffness = q(60);
    rod->damping = q(8);
    weight->linear_damping = q(1);
    for (int i = 0; i < 600; ++i) step();
    // mg / k = 720 / 60 = 12 px of stretch.
    assert(close(real(root(bob).transform.position[1]), 44, 2.0));
    // A line from the pivot through the bob follows it.
    auto* cord = game_level.add_component<Line2DComponent>(*pivot, "Cord");
    cord->follow[0] = pivot->id();
    cord->follow[1] = bob->id();
    cord->follow_count = 2;
    step();
    assert(cord->point_count == 2 && real(cord->points[0][0]) == 0);
    assert(cord->points[1][0].raw() == root(bob).transform.position[0].raw() && cord->points[1][1].raw() == root(bob).transform.position[1].raw());
    rod->break_joint();
    for (int i = 0; i < 20; ++i) step();
    assert(real(root(bob).transform.position[1]) > 50);
}

// 22. Render passes: a pass's camera and canvas filter decide what is collected, a sprite
// can show a SubViewport2D's picture, and a split screen halves the display.
void viewports() {
    reset();
    auto size = [](int index, uint16_t& w, uint16_t& h) { if (index != 3) return false; w = 16; h = 16; return true; };
    auto* hero = spawn<Actor2D>("Hero", 0, 0);
    game_level.add_component<Camera2DComponent>(*hero, "Camera");
    auto* rival = spawn<Actor2D>("Rival", 600, 0);
    auto* second = game_level.add_component<Camera2DComponent>(*rival, "Camera");
    second->current = false;
    auto* crate = spawn<Actor2D>("Crate", 0, 0);
    game_level.add_component<Sprite2DComponent>(*crate, "Sprite")->texture = 3;
    auto* hud = spawn<Actor2D>("Hud", 0, 0);
    game_level.add_component<CanvasLayer2DComponent>(*hud, "Canvas");
    auto* icon = spawn<Actor2D>("Icon", 20, 30);
    root(icon).attach_parent = root(hud).id();
    game_level.add_component<Sprite2DComponent>(*icon, "Sprite")->texture = 3;
    step();
    world2d::SpriteDraw draws[4];
    world2d::DrawItem items[4], scratch[4];
    assert(world2d::collect_sprites(draws, items, scratch, 4, size) == 2);
    // Through the rival's camera the crate is far off screen; the screen canvas stays.
    Camera2D far = second->view();
    world2d::render_pass.camera = &far;
    assert(world2d::collect_sprites(draws, items, scratch, 4, size) == 1);
    // World canvases only, then canvas 1 only, then canvas 1 hidden.
    world2d::render_pass = world2d::RenderPass{};
    world2d::render_pass.screen = false;
    assert(world2d::collect_sprites(draws, items, scratch, 4, size) == 1);
    world2d::render_pass = world2d::RenderPass{};
    world2d::render_pass.only_layer = 1;
    assert(world2d::collect_sprites(draws, items, scratch, 4, size) == 1 && draws[items[0].index].x[0] == 12);
    world2d::render_pass = world2d::RenderPass{};
    world2d::render_pass.hide(1);
    assert(world2d::collect_sprites(draws, items, scratch, 4, size) == 1 && draws[items[0].index].x[0] != 12);
    world2d::render_pass = world2d::RenderPass{};
    // A sprite showing a SubViewport2D's picture: its size, no texture lookup.
    auto* monitor = spawn<Actor2D>("Monitor", 0, 0);
    auto* picture = game_level.add_component<SubViewport2DComponent>(*monitor, "Viewport");
    picture->width = 64;
    picture->height = 32;
    auto* screen = spawn<Actor2D>("Screen", 0, 0);
    auto* shows = game_level.add_component<Sprite2DComponent>(*screen, "Sprite");
    shows->set_viewport(monitor->id());
    world2d::render_pass.screen = false;
    const size_t count = world2d::collect_sprites(draws, items, scratch, 4, size);
    bool found = false;
    for (size_t i = 0; i < count; ++i) {
        const auto& d = draws[items[i].index];
        if (d.texture == -2) {
            found = d.component == picture && d.source[2] == 64 && d.source[3] == 32 && d.x[1] - d.x[0] == 64;
        }
    }
    assert(found);
    world2d::render_pass = world2d::RenderPass{};
    Camera2D view;
    assert(picture->view(view) && view.viewport[2] == 64 && view.viewport[3] == 32);
    // The two halves of a split screen and the views centred on them.
    auto* split_node = spawn<Actor2D>("Split", 0, 0);
    auto* split = game_level.add_component<SplitScreen2DComponent>(*split_node, "Split");
    split->cameras[0] = hero->id();
    split->cameras[1] = rival->id();
    assert(world2d::active_split() == split);
    int16_t left[4], right[4];
    split->half(0, left);
    split->half(1, right);
    assert(left[0] == 0 && left[2] + split->divider + right[2] == display_width && right[0] == left[2] + split->divider);
    Camera2D through;
    assert(split->view(1, through) && through.viewport[0] == right[0] && real(through.position[0]) == 600);
    split->set_side_by_side(false);
    split->half(1, right);
    assert(right[0] == 0 && right[2] == display_width && right[1] > 0);
    split->cameras[1] = ObjectId{};
    assert(!world2d::active_split());
}

void size_report() {
    std::printf("World2D components (host, %zu-bit pointers): Sprite2D %zu  Camera2D %zu  Collider2D %zu  CharacterBody2D %zu  TileMapLayer %zu  Parallax2D %zu  Skeleton2D %zu\n",
                sizeof(void*) * 8, sizeof(Sprite2DComponent), sizeof(Camera2DComponent), sizeof(Collider2DComponent),
                sizeof(CharacterBody2DComponent), sizeof(TileMapLayerComponent), sizeof(Parallax2DComponent), sizeof(Skeleton2DComponent));
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
    tile_cells_and_edits();
    tile_collision();
    tile_queries();
    tile_drawing();
    parallax();
    canvas_layers();
    path_follow();
    shapes();
    particles();
    notifiers();
    remote_transforms();
    audio_2d();
    skeletons();
    navigation();
    rigid_bodies();
    viewports();
    size_report();
    std::puts("World2D sprite clips, camera, character body, triggers, draw sort, tile map, parallax, canvas layer, path, shape, particle, notifier, remote transform, 2D audio, skeleton, navigation, rigid body and viewport tests passed.");
    return 0;
}
