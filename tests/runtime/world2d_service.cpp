// Host contract tests for the World2D components and service (World2D initiative,
// phases 1 and 2): sprite frame clips, Camera2D following, CharacterBody2D move_and_slide
// with floor/wall/one-way contacts, trigger delivery to overlap_2d, registration lifetime,
// the stable draw-item sort, tile map cells and runtime edits, tile collision, rays and
// overlaps against tiles, tile drawing, parallax and canvas layers. Compiles the real runtime headers; only the audio service and
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
    {TileMapLayerComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, ObjectClassMultiple, &object_construct<TileMapLayerComponent>, &object_destruct, sizeof(TileMapLayerComponent), alignof(TileMapLayerComponent), &ObjectPool<TileMapLayerComponent, 4>::acquire, &ObjectPool<TileMapLayerComponent, 4>::release},
    {Parallax2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<Parallax2DComponent>, &object_destruct, sizeof(Parallax2DComponent), alignof(Parallax2DComponent), &ObjectPool<Parallax2DComponent, 4>::acquire, &ObjectPool<Parallax2DComponent, 4>::release},
    {CanvasLayer2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<CanvasLayer2DComponent>, &object_destruct, sizeof(CanvasLayer2DComponent), alignof(CanvasLayer2DComponent), &ObjectPool<CanvasLayer2DComponent, 4>::acquire, &ObjectPool<CanvasLayer2DComponent, 4>::release},
    {CanvasModulate2DComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World2D, world2d_bit, 0, &object_construct<CanvasModulate2DComponent>, &object_destruct, sizeof(CanvasModulate2DComponent), alignof(CanvasModulate2DComponent), &ObjectPool<CanvasModulate2DComponent, 4>::acquire, &ObjectPool<CanvasModulate2DComponent, 4>::release},
    {Listener2D::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::None, world2d_bit, ObjectClassMultiple, &object_construct<Listener2D>, &object_destruct, sizeof(Listener2D), alignof(Listener2D), &ObjectPool<Listener2D, 4>::acquire, &ObjectPool<Listener2D, 4>::release},
    {Level::static_class_id, Object::static_class_id, ObjectFamily::Level, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Level), alignof(Level), nullptr, nullptr},
};
const size_t object_class_count = sizeof(object_classes) / sizeof(object_classes[0]);
}  // namespace epok

namespace {
ObjectRegistryStorage<96> registry_storage;
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
    assert(count == 1 && draws[items[0].index].tiles == layer);
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

void size_report() {
    std::printf("World2D components (host, %zu-bit pointers): Sprite2D %zu  Camera2D %zu  Collider2D %zu  CharacterBody2D %zu  TileMapLayer %zu  Parallax2D %zu\n",
                sizeof(void*) * 8, sizeof(Sprite2DComponent), sizeof(Camera2DComponent), sizeof(Collider2DComponent),
                sizeof(CharacterBody2DComponent), sizeof(TileMapLayerComponent), sizeof(Parallax2DComponent));
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
    size_report();
    std::puts("World2D sprite clips, camera, character body, triggers, draw sort, tile map, parallax and canvas layer tests passed.");
    return 0;
}
