// Host contract tests for the actor service adapters (actor-architecture P9).
// Covers the AudioComponent play_on_start policy, stop on disable/end_play, the
// asynchronous-consumer quarantine that keeps component-owned AudioSource storage out of
// the pool, and the TriggerWatcher pause/trigger forwarding contract.
// Compiles the real runtime/object_model.hpp through epok.hpp; only the audio service and
// the class table are stubbed, exactly as the cooked build would provide them.
#include <cassert>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/epok.hpp"

namespace epok {
// Audio service stub. `plays`/`stops` count per source, so "played exactly once" is a
// property of the individual AudioSource, not of the process.
struct AudioProbe { unsigned plays = 0, stops = 0; bool playing = false; };
AudioProbe audio_probe[8];
const AudioSource* audio_probe_source[8] = {};
AudioProbe& probe_of(const AudioSource* source) {
    for (size_t i = 0; i < 8; ++i) if (audio_probe_source[i] == source) return audio_probe[i];
    for (size_t i = 0; i < 8; ++i) if (!audio_probe_source[i]) { audio_probe_source[i] = source; return audio_probe[i]; }
    assert(false);
    return audio_probe[0];
}
void audio_probe_reset() {
    for (size_t i = 0; i < 8; ++i) { audio_probe[i] = AudioProbe{}; audio_probe_source[i] = nullptr; }
}
void AudioSource::play() { auto& p = probe_of(this); ++p.plays; p.playing = true; }
void AudioSource::stop() { auto& p = probe_of(this); ++p.stops; p.playing = false; }
bool AudioSource::is_playing() const { return probe_of(this).playing; }

// Stand-in for runtime/music.hpp's XA consumer: the CD driver's lookup/stop completions
// retain this address, which is what allocate_actor_data quarantines legacy slots against.
const AudioSource* stub_music_active = nullptr;
bool stub_music_retains(const AudioSource* source) { return source && source == stub_music_active; }
}  // namespace epok
using namespace epok;

namespace {
// ---- probe classes -----------------------------------------------------------------
struct AudioActor : Actor3D {
    static constexpr uint64_t static_class_id = 0x3001;
    uint64_t class_id() const override { return static_class_id; }
};
struct AudioPanel : UIActor {
    static constexpr uint64_t static_class_id = 0x3002;
    uint64_t class_id() const override { return static_class_id; }
};
struct TriggerWatcher : ActorComponent {
    static constexpr uint64_t static_class_id=0x3003;
    unsigned starts = 0, updates = 0, frames = 0, triggers = 0;
    DataHandle last_other;
    TriggerPhase last_phase = TriggerPhase::Exit;
    void begin_play() override { ++starts; }
    void tick(Fixed) override { ++updates; }
    void frame_update(uint32_t) override { ++frames; }
    void on_trigger(DataHandle other, TriggerPhase phase) override {
        ++triggers;
        last_other = other;
        last_phase = phase;
    }
};
}  // namespace

// ---- cooked class table ------------------------------------------------------------
namespace epok {
const ClassDescriptor object_classes[] = {
    {Object::static_class_id, 0, ObjectFamily::Object, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Object), alignof(Object), nullptr, nullptr},
    {Actor::static_class_id, Object::static_class_id, ObjectFamily::Actor, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Actor), alignof(Actor), nullptr, nullptr},
    {Actor3D::static_class_id, Actor::static_class_id, ObjectFamily::Actor, ObjectDomain::World3D, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), nullptr, nullptr, sizeof(Actor3D), alignof(Actor3D), nullptr, nullptr},
    {UIActor::static_class_id, Actor::static_class_id, ObjectFamily::Actor, ObjectDomain::UI, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), nullptr, nullptr, sizeof(UIActor), alignof(UIActor), nullptr, nullptr},
    {ActorComponent::static_class_id, Object::static_class_id, ObjectFamily::Component, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(ActorComponent), alignof(ActorComponent), nullptr, nullptr},
    {SceneComponent3D::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::World3D, object_domain_bit(ObjectDomain::World3D), ObjectClassRoot, &object_construct<SceneComponent3D>, &object_destruct, sizeof(SceneComponent3D), alignof(SceneComponent3D), &ObjectPool<SceneComponent3D, 4>::acquire, &ObjectPool<SceneComponent3D, 4>::release},
    {UIComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::UI, object_domain_bit(ObjectDomain::UI), ObjectClassAbstract, nullptr, nullptr, sizeof(UIComponent), alignof(UIComponent), nullptr, nullptr},
    {RectTransformComponent::static_class_id, UIComponent::static_class_id, ObjectFamily::Component, ObjectDomain::UI, object_domain_bit(ObjectDomain::UI), ObjectClassRoot, &object_construct<RectTransformComponent>, &object_destruct, sizeof(RectTransformComponent), alignof(RectTransformComponent), &ObjectPool<RectTransformComponent, 4>::acquire, &ObjectPool<RectTransformComponent, 4>::release},
    {AudioComponent::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::None,
     uint8_t(object_domain_bit(ObjectDomain::World3D) | object_domain_bit(ObjectDomain::World2D) | object_domain_bit(ObjectDomain::UI)),
     ObjectClassMultiple, &object_construct<AudioComponent>, &object_destruct, sizeof(AudioComponent), alignof(AudioComponent), &ObjectPool<AudioComponent, 4>::acquire, &ObjectPool<AudioComponent, 4>::release},
    {TriggerWatcher::static_class_id, ActorComponent::static_class_id, ObjectFamily::Component, ObjectDomain::None,
     uint8_t(object_domain_bit(ObjectDomain::World3D) | object_domain_bit(ObjectDomain::World2D) | object_domain_bit(ObjectDomain::UI)),
     ObjectClassMultiple, &object_construct<TriggerWatcher>, &object_destruct, sizeof(TriggerWatcher), alignof(TriggerWatcher), &ObjectPool<TriggerWatcher, 2>::acquire, &ObjectPool<TriggerWatcher, 2>::release},
    {Level::static_class_id, Object::static_class_id, ObjectFamily::Level, ObjectDomain::None, 0, ObjectClassAbstract, nullptr, nullptr, sizeof(Level), alignof(Level), nullptr, nullptr},
    {AudioActor::static_class_id, Actor3D::static_class_id, ObjectFamily::Actor, ObjectDomain::World3D, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), &object_construct<AudioActor>, &object_destruct, sizeof(AudioActor), alignof(AudioActor), &ObjectPool<AudioActor, 4>::acquire, &ObjectPool<AudioActor, 4>::release},
    {AudioPanel::static_class_id, UIActor::static_class_id, ObjectFamily::Actor, ObjectDomain::UI, 0, uint8_t(ObjectClassPlaceable | ObjectClassSpawnable), &object_construct<AudioPanel>, &object_destruct, sizeof(AudioPanel), alignof(AudioPanel), &ObjectPool<AudioPanel, 2>::acquire, &ObjectPool<AudioPanel, 2>::release},
};
const size_t object_class_count = sizeof(object_classes) / sizeof(object_classes[0]);
}  // namespace epok

namespace {
const ClassDescriptor& class_of(uint64_t id) {
    const auto* found = find_object_class(id);
    assert(found);
    return *found;
}
ObjectRegistryStorage<24> registry_storage;
Level game_level;

void reset() {
    if (game_level.registry()) {
        game_level.end_play_all(EndPlayReason::LevelUnloaded);
        stub_music_active = nullptr;
        registry_storage.collect_quarantined();
        registry_storage.release(game_level.id());
    }
    game_level = Level{};
    active_object_registry = &registry_storage;
    audio_source_retained = &stub_music_retains;
    stub_music_active = nullptr;
    registry_storage.stats = ObjectStats{};
    assert(game_level.bind(registry_storage));
    audio_probe_reset();
    assert((ObjectPool<AudioComponent, 4>::live() == 0));
    assert((ObjectPool<AudioActor, 4>::live() == 0));
}
ObjectId spawn(uint64_t class_id, const char* name, ObjectId parent = {}) {
    return game_level.spawn_actor(class_of(class_id), name, parent);
}

// 1. A component that views a legacy slot never re-triggers play_on_start: the scene
// bank's bank-load loop and bp::activate_spawn_audio own that start.
void slot_backed_audio_never_double_plays() {
    reset();
    static ActorData slot;
    slot = ActorData{};
    slot.audio.enabled = true;
    slot.audio.play_on_start = true;

    const ObjectId id = spawn(AudioActor::static_class_id, "Speaker");
    auto* actor = registry_storage.resolve<AudioActor>(id);
    assert(actor);
    actor->root.bind_slot(slot);
    auto* audio = game_level.add_component<AudioComponent>(*actor, "Voice");
    assert(audio);
    audio->bind_slot(slot);
    assert(!audio->owns_source() && audio->source == &slot.audio && audio->entity_slot() == &slot);
    // add_component after begin_play runs begin_play on the component immediately.
    assert(probe_of(&slot.audio).plays == 0);

    // The legacy path (lifecycle.hpp's activate_spawn_audio equivalent) starts it once.
    slot.audio.play();
    assert(probe_of(&slot.audio).plays == 1);

    // A fresh batch with the component present from the start also must not add a play.
    const ObjectId second = spawn(AudioActor::static_class_id, "Speaker2");
    auto* other = registry_storage.resolve<AudioActor>(second);
    assert(other);
    auto* shared = game_level.add_component<AudioComponent>(*other, "Shared");
    assert(shared);
    shared->bind_slot(slot);
    assert(probe_of(&slot.audio).plays == 1);

    // Teardown still stops it, exactly once per stop request.
    const unsigned stops = probe_of(&slot.audio).stops;
    assert(game_level.destroy_actor(id));
    assert(probe_of(&slot.audio).stops == stops + 1 && !probe_of(&slot.audio).playing);
    // Slot storage is owned by the scene bank, so the component never holds a slot back.
    assert(registry_storage.stats.quarantined == 0);
}

// 2. Component-owned storage starts exactly once in begin_play, and only when enabled,
// play_on_start is set and the owner is active.
void owned_audio_plays_once() {
    reset();
    ObjectId id = spawn(AudioActor::static_class_id, "Owned");
    auto* actor = registry_storage.resolve<AudioActor>(id);
    auto* audio = game_level.add_component<AudioComponent>(*actor, "Voice");
    assert(audio);
    audio->bind_local();
    assert(audio->owns_source() && audio->source == &audio->local);
    // Defaults: enabled is false, so nothing starts even with play_on_start.
    assert(audio->local.play_on_start && !audio->local.enabled);
    assert(probe_of(&audio->local).plays == 0);

    // A component added after the owner began play begins immediately; enable first.
    reset();
    id = spawn(AudioActor::static_class_id, "Owned");
    actor = registry_storage.resolve<AudioActor>(id);
    audio = game_level.add_component<AudioComponent>(*actor, "Voice");
    assert(audio);
    audio->bind_local();
    audio->local.enabled = true;
    // begin_play already ran with enabled=false; nothing played.
    assert(probe_of(&audio->local).plays == 0);
    audio->begin_play();                       // the batch's begin_play, replayed
    assert(probe_of(&audio->local).plays == 1 && audio->is_playing());
    audio->begin_play();                       // idempotent: it is already playing
    assert(probe_of(&audio->local).plays == 1);

    // play_on_start = false never starts.
    auto* quiet = game_level.add_component<AudioComponent>(*actor, "Quiet");
    assert(quiet);
    quiet->bind_local();
    quiet->local.enabled = true;
    quiet->local.play_on_start = false;
    quiet->begin_play();
    assert(probe_of(&quiet->local).plays == 0);

    // An inactive owner does not start its audio.
    assert(game_level.set_active(id, false));
    auto* muted = game_level.add_component<AudioComponent>(*actor, "Muted");
    assert(muted);
    muted->bind_local();
    muted->local.enabled = true;
    muted->begin_play();
    assert(probe_of(&muted->local).plays == 0);
    // An inactive logical parent counts too.
    assert(game_level.set_active(id, true));
    const ObjectId child = spawn(AudioActor::static_class_id, "Child", id);
    auto* nested = game_level.add_component<AudioComponent>(*registry_storage.resolve<AudioActor>(child), "Nested");
    assert(nested);
    nested->bind_local();
    nested->local.enabled = true;
    assert(game_level.set_active(id, false));
    nested->begin_play();
    assert(probe_of(&nested->local).plays == 0);
}

// 3. Deactivation stops owned audio; end_play stops it again on teardown.
void owned_audio_stops_on_disable_and_end_play() {
    reset();
    const ObjectId id = spawn(AudioActor::static_class_id, "Owned");
    auto* actor = registry_storage.resolve<AudioActor>(id);
    auto* audio = game_level.add_component<AudioComponent>(*actor, "Voice");
    assert(audio);
    audio->bind_local();
    audio->local.enabled = true;
    audio->begin_play();
    assert(audio->is_playing());

    // on_disable through the ordinary activation path.
    assert(game_level.set_active(id, false));
    assert(!audio->is_playing() && probe_of(&audio->local).stops == 1);
    // Stopping an already stopped source is a no-op request at this level.
    assert(game_level.set_active(id, true));
    assert(!audio->is_playing());
    audio->play();
    assert(audio->is_playing());

    // end_play: teardown disables and then ends, so a playing source is stopped.
    const unsigned stops = probe_of(&audio->local).stops;
    assert(game_level.destroy_actor(id, EndPlayReason::LevelUnloaded));
    assert(probe_of(&audio->local).stops == stops + 1);
}

// 4. Quarantine: while a stub music_active points at component-owned storage the slot and
// its pool entry are not reused; clearing it releases them on the next collection.
void owned_audio_quarantine_blocks_reuse() {
    reset();
    const ObjectId id = spawn(AudioActor::static_class_id, "Music");
    auto* actor = registry_storage.resolve<AudioActor>(id);
    auto* audio = game_level.add_component<AudioComponent>(*actor, "Stream");
    assert(audio);
    audio->bind_local();
    audio->local.enabled = true;
    assert(audio->releasable());

    // The XA consumer retains the address across its asynchronous completion.
    stub_music_active = &audio->local;
    assert(!audio->releasable());
    const ObjectId component = audio->id();
    const size_t pooled = ObjectPool<AudioComponent, 4>::live();
    assert(pooled == 1);

    assert(game_level.destroy_actor(id));
    // The handle is dead immediately either way.
    assert(!registry_storage.get(component));
    // ... but the storage is still the consumer's, so it did not return to the pool.
    assert((ObjectPool<AudioComponent, 4>::live() == 1));
    assert(registry_storage.stats.quarantined == 1);
    registry_storage.collect_quarantined();
    assert((ObjectPool<AudioComponent, 4>::live() == 1));

    // The consumer releases it; the next collection returns the storage.
    stub_music_active = nullptr;
    registry_storage.collect_quarantined();
    assert((ObjectPool<AudioComponent, 4>::live() == 0));
    assert(registry_storage.stats.quarantined == 0);

    // With no hook installed nothing is ever quarantined.
    reset();
    audio_source_retained = nullptr;
    const ObjectId plain = spawn(AudioActor::static_class_id, "Plain");
    auto* owner = registry_storage.resolve<AudioActor>(plain);
    auto* voice = game_level.add_component<AudioComponent>(*owner, "Voice");
    assert(voice);
    voice->bind_local();
    assert(voice->releasable());
    assert(game_level.destroy_actor(plain));
    assert((ObjectPool<AudioComponent, 4>::live() == 0));
    audio_source_retained = &stub_music_retains;
}

// 5. Audio is accepted by every actor domain, including UI, and stays Multiple.
void audio_is_domain_agnostic() {
    reset();
    const ObjectId panel = spawn(AudioPanel::static_class_id, "Panel");
    auto* ui = registry_storage.resolve<AudioPanel>(panel);
    assert(ui);
    auto* click = game_level.add_component<AudioComponent>(*ui, "Click");
    auto* hover = game_level.add_component<AudioComponent>(*ui, "Hover");
    assert(click && hover && click != hover);
    click->bind_local();
    hover->bind_local();
    click->local.enabled = true;
    click->begin_play();
    hover->begin_play();
    assert(probe_of(&click->local).plays == 1 && probe_of(&hover->local).plays == 0);
}

// 6. TriggerWatcher: frame_update keeps running while the simulation clock is
// paused (the legacy contract), and update() does not.
void component_frame_update_runs_while_paused() {
    reset();
    static ActorData slot;
    slot = ActorData{};
    const ObjectId id = spawn(AudioActor::static_class_id, "Script");
    auto* actor = registry_storage.resolve<AudioActor>(id);
    assert(actor);
    actor->root.bind_slot(slot);
    auto* adapter = game_level.add_component<TriggerWatcher>(*actor, "Logic");
    assert(adapter);auto& watcher=*adapter;

    // Paused: main.cpp keeps calling Level::frame_update and stops calling Level::tick.
    for (unsigned frame = 0; frame < 3; ++frame) game_level.frame_update(16666);
    assert(watcher.frames == 3 && watcher.updates == 0);
    // start() still ran before the first frame_update, exactly once.
    assert(watcher.starts == 1);
    // Resumed: the fixed steps arrive as ticks.
    game_level.tick(0.25);
    assert(watcher.updates == 1 && watcher.frames == 3);
    // An inactive actor receives neither.
    assert(game_level.set_active(id, false));
    game_level.frame_update(16666);
    game_level.tick(0.25);
    assert(watcher.frames == 3 && watcher.updates == 1);
}

// 7. dispatch_trigger fans a collision event out to the owner's components; the Level
// itself never generates one.
void component_trigger_delivery() {
    reset();
    static ActorData slot;
    slot = ActorData{};
    const ObjectId id = spawn(AudioActor::static_class_id, "Trigger");
    auto* actor = registry_storage.resolve<AudioActor>(id);
    assert(actor);
    actor->root.bind_slot(slot);
    auto* adapter = game_level.add_component<TriggerWatcher>(*actor, "Logic");
    assert(adapter);auto& watcher=*adapter;

    const DataHandle other{7, 3};
    // Root + adapter both receive the call; only the adapter forwards it.
    assert(dispatch_trigger(game_level, id, other, TriggerPhase::Enter) == 2);
    assert(watcher.triggers == 1 && watcher.last_phase == TriggerPhase::Enter);
    assert(watcher.last_other.index == 7 && watcher.last_other.generation == 3);
    assert(dispatch_trigger(game_level, id, other, TriggerPhase::Exit) == 2);
    assert(watcher.triggers == 2 && watcher.last_phase == TriggerPhase::Exit);

    // Inactive, unknown and destroyed actors receive nothing.
    assert(game_level.set_active(id, false));
    assert(dispatch_trigger(game_level, id, other, TriggerPhase::Stay) == 0);
    assert(watcher.triggers == 2);
    assert(game_level.set_active(id, true));
    assert(game_level.destroy_actor(id));
    assert(dispatch_trigger(game_level, id, other, TriggerPhase::Stay) == 0);
    assert(dispatch_trigger(game_level, ObjectId{}, other, TriggerPhase::Stay) == 0);
}
}  // namespace

int main() {
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    slot_backed_audio_never_double_plays();
    owned_audio_plays_once();
    owned_audio_stops_on_disable_and_end_play();
    owned_audio_quarantine_blocks_reuse();
    audio_is_domain_agnostic();
    component_frame_update_runs_while_paused();
    component_trigger_delivery();
    reset();
    std::puts("Actor service adapters: audio start/stop/release policy, quarantine, pause and trigger forwarding tests passed.");
    return 0;
}
