# Epok API: Object Model

> **Header:** `"object_model.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/object_model.hpp)

This module covers the object model module. It documents 297 public callables declared directly in this header.

## Declared types

`epok::Actor`, `epok::Actor2D`, `epok::Actor3D`, `epok::ActorCallbacks`, `epok::ActorComponent`, `epok::ActorSpawnRequest`, `epok::AudioComponent`, `epok::BlobShadowComponent`, `epok::Camera3DComponent`, `epok::CanvasComponent`, `epok::ClassDescriptor`, `epok::Collider3DComponent`, `epok::ComponentCallbacks`, `epok::EndPlayReason`, `epok::ImageComponent`, `epok::Level`, `epok::Level::ActorPrepareFn`, `epok::LevelPendingOp`, `epok::Light3DComponent`, `epok::Mesh3DComponent`, `epok::NativeComponentDefault`, `epok::Object`, `epok::ObjectBatch8`, `epok::ObjectClassFlags`, `epok::ObjectDispatchScope`, `epok::ObjectDomain`, `epok::ObjectFamily`, `epok::ObjectId`, `epok::ObjectPool`, `epok::ObjectRegistry`, `epok::ObjectRegistryStorage`, `epok::ObjectSlot`, `epok::ObjectState`, `epok::ObjectStats`, `epok::PaletteAnimatorComponent`, `epok::ParticleEffectComponent`, `epok::ParticleEmitterComponent`, `epok::ProgressBarComponent`, `epok::RectTransformComponent`, `epok::SceneComponent2D`, `epok::SceneComponent3D`, `epok::SceneScriptActor`, `epok::Sprite3DComponent`, `epok::TextComponent`, `epok::TimelineComponent`, `epok::Transform2D`, `epok::UIActor`, `epok::UIComponent`, `epok::World`

## Callable index

- [`epok::Actor2D::class_id`](#epok-actor2d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor2D::default_root`](#epok-actor2d-default-root-1) — Performs `default root` as part of the object model module.
- [`epok::Actor3D::class_id`](#epok-actor3d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor3D::default_root`](#epok-actor3d-default-root-1) — Performs `default root` as part of the object model module.
- [`epok::Actor::active`](#epok-actor-active-1) — Self flag only; Level::actor_active() folds the logical parent chain.
- [`epok::Actor::begin_play`](#epok-actor-begin-play-1) — Begins play as part of the object model module.
- [`epok::Actor::bind_data`](#epok-actor-bind-data-1) — Performs `bind data` as part of the object model module.
- [`epok::Actor::blueprint_observe`](#epok-actor-blueprint-observe-1) — Performs `blueprint observe` as part of the object model module.
- [`epok::Actor::class_id`](#epok-actor-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor::component_count`](#epok-actor-component-count-1) — Performs `component count` as part of the object model module.
- [`epok::Actor::component_id`](#epok-actor-component-id-1) — Performs `component id` as part of the object model module.
- [`epok::Actor::data`](#epok-actor-data-1) — Legacy adapter: the canonical slot behind the root scene component, or nullptr.
- [`epok::Actor::default_root`](#epok-actor-default-root-1) — Runtime hook, not reflected: the declarative root component embedded in the actor.
- [`epok::Actor::destroy`](#epok-actor-destroy-1) — Destroys destroy as part of the object model module.
- [`epok::Actor::end_play`](#epok-actor-end-play-1) — Ends play as part of the object model module.
- [`epok::Actor::frame_update`](#epok-actor-frame-update-1) — Performs `frame update` as part of the object model module.
- [`epok::Actor::level_id`](#epok-actor-level-id-1) — Reflected identity/hierarchy readers.
- [`epok::Actor::logical_parent`](#epok-actor-logical-parent-1) — Performs `logical parent` as part of the object model module.
- [`epok::Actor::name`](#epok-actor-name-1) — Performs `name` as part of the object model module.
- [`epok::Actor::on_disable`](#epok-actor-on-disable-1) — Performs `on disable` as part of the object model module.
- [`epok::Actor::on_enable`](#epok-actor-on-enable-1) — Performs `on enable` as part of the object model module.
- [`epok::Actor::on_frame`](#epok-actor-on-frame-1) — Performs `on frame` as part of the object model module.
- [`epok::Actor::root_id`](#epok-actor-root-id-1) — Performs `root id` as part of the object model module.
- [`epok::Actor::set_active`](#epok-actor-set-active-1) — Folded activation with the on_enable/on_disable fan-out, and deferred destruction inside a dispatch scope: the same Level paths the legacy Blueprint nodes take.
- [`epok::Actor::set_name`](#epok-actor-set-name-1) — Sets name as part of the object model module.
- [`epok::Actor::set_wants_tick`](#epok-actor-set-wants-tick-1) — Sets wants tick as part of the object model module.
- [`epok::Actor::tick`](#epok-actor-tick-1) — Performs `tick` as part of the object model module.
- [`epok::Actor::wants_tick`](#epok-actor-wants-tick-1) — Performs `wants tick` as part of the object model module.
- [`epok::ActorComponent::attach_slot`](#epok-actorcomponent-attach-slot-1) — Performs `attach slot` as part of the object model module.
- [`epok::ActorComponent::begin_play`](#epok-actorcomponent-begin-play-1) — Begins play as part of the object model module.
- [`epok::ActorComponent::blueprint_observe`](#epok-actorcomponent-blueprint-observe-1) — Performs `blueprint observe` as part of the object model module.
- [`epok::ActorComponent::class_id`](#epok-actorcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ActorComponent::end_play`](#epok-actorcomponent-end-play-1) — Ends play as part of the object model module.
- [`epok::ActorComponent::frame_update`](#epok-actorcomponent-frame-update-1) — Performs `frame update` as part of the object model module.
- [`epok::ActorComponent::get_owner`](#epok-actorcomponent-get-owner-1) — Returns owner as part of the object model module.
- [`epok::ActorComponent::name`](#epok-actorcomponent-name-1) — Performs `name` as part of the object model module.
- [`epok::ActorComponent::on_disable`](#epok-actorcomponent-on-disable-1) — Performs `on disable` as part of the object model module.
- [`epok::ActorComponent::on_enable`](#epok-actorcomponent-on-enable-1) — Performs `on enable` as part of the object model module.
- [`epok::ActorComponent::on_frame`](#epok-actorcomponent-on-frame-1) — Performs `on frame` as part of the object model module.
- [`epok::ActorComponent::on_trigger`](#epok-actorcomponent-on-trigger-1) — Performs `on trigger` as part of the object model module.
- [`epok::ActorComponent::owner_id`](#epok-actorcomponent-owner-id-1) — Performs `owner id` as part of the object model module.
- [`epok::ActorComponent::releasable`](#epok-actorcomponent-releasable-1) — False while a service still holds this component's storage; the registry then keeps the (already dead) slot quarantined instead of returning it to the pool.
- [`epok::ActorComponent::set_name`](#epok-actorcomponent-set-name-1) — Sets name as part of the object model module.
- [`epok::ActorComponent::tick`](#epok-actorcomponent-tick-1) — Performs `tick` as part of the object model module.
- [`epok::ActorComponent::timeline_sync`](#epok-actorcomponent-timeline-sync-1) — Runtime hooks, not reflected.
- [`epok::ActorComponent::trigger_event`](#epok-actorcomponent-trigger-event-1) — Forward-only collision hook.
- [`epok::attach_component`](#epok-attach-component-1) — Spatial attachment between components of the same domain.
- [`epok::AudioComponent::begin_play`](#epok-audiocomponent-begin-play-1) — play_on_start policy.
- [`epok::AudioComponent::bind_local`](#epok-audiocomponent-bind-local-1) — Performs `bind local` as part of the object model module.
- [`epok::AudioComponent::bind_slot`](#epok-audiocomponent-bind-slot-1) — Bind to the legacy slot's AudioSource, or to component-owned storage.
- [`epok::AudioComponent::class_id`](#epok-audiocomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::AudioComponent::clip`](#epok-audiocomponent-clip-1) — Performs `clip` as part of the object model module.
- [`epok::AudioComponent::enabled`](#epok-audiocomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::AudioComponent::end_play`](#epok-audiocomponent-end-play-1) — Ends play as part of the object model module.
- [`epok::AudioComponent::entity_slot`](#epok-audiocomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::AudioComponent::is_playing`](#epok-audiocomponent-is-playing-1) — Reports whether playing as part of the object model module.
- [`epok::AudioComponent::on_disable`](#epok-audiocomponent-on-disable-1) — Deactivation and teardown stop both kinds: lifecycle.hpp::set_active stops a legacy slot's audio through the slot table, and stopping an already stopped source is a no-op, so the two paths are idempotent rather than conflicting.
- [`epok::AudioComponent::owns_source`](#epok-audiocomponent-owns-source-1) — True only for component-owned storage.
- [`epok::AudioComponent::pitch`](#epok-audiocomponent-pitch-1) — Performs `pitch` as part of the object model module.
- [`epok::AudioComponent::play`](#epok-audiocomponent-play-1) — Starts play as part of the object model module.
- [`epok::AudioComponent::play_on_start`](#epok-audiocomponent-play-on-start-1) — Starts on start as part of the object model module.
- [`epok::AudioComponent::priority`](#epok-audiocomponent-priority-1) — Performs `priority` as part of the object model module.
- [`epok::AudioComponent::releasable`](#epok-audiocomponent-releasable-1) — Component-owned storage stays quarantined while an asynchronous consumer (the XA music service) still points at it, mirroring allocate_actor_data's legacy-slot rule.
- [`epok::AudioComponent::set_clip`](#epok-audiocomponent-set-clip-1) — Sets clip as part of the object model module.
- [`epok::AudioComponent::set_enabled`](#epok-audiocomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::AudioComponent::set_pitch`](#epok-audiocomponent-set-pitch-1) — Sets pitch as part of the object model module.
- [`epok::AudioComponent::set_play_on_start`](#epok-audiocomponent-set-play-on-start-1) — Sets play on start as part of the object model module.
- [`epok::AudioComponent::set_priority`](#epok-audiocomponent-set-priority-1) — Sets priority as part of the object model module.
- [`epok::AudioComponent::set_volume`](#epok-audiocomponent-set-volume-1) — Sets volume as part of the object model module.
- [`epok::AudioComponent::stop`](#epok-audiocomponent-stop-1) — Stops stop as part of the object model module.
- [`epok::AudioComponent::volume`](#epok-audiocomponent-volume-1) — Performs `volume` as part of the object model module.
- [`epok::BlobShadowComponent::class_id`](#epok-blobshadowcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::BlobShadowComponent::configure`](#epok-blobshadowcomponent-configure-1) — Performs `configure` as part of the object model module.
- [`epok::BlobShadowComponent::enabled`](#epok-blobshadowcomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::BlobShadowComponent::entity_slot`](#epok-blobshadowcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::BlobShadowComponent::set_enabled`](#epok-blobshadowcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::Camera3DComponent::class_id`](#epok-camera3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Camera3DComponent::enabled`](#epok-camera3dcomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::Camera3DComponent::entity_slot`](#epok-camera3dcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::Camera3DComponent::field_of_view`](#epok-camera3dcomponent-field-of-view-1) — Performs `field of view` as part of the object model module.
- [`epok::Camera3DComponent::make_active`](#epok-camera3dcomponent-make-active-1) — Performs `make active` as part of the object model module.
- [`epok::Camera3DComponent::set_enabled`](#epok-camera3dcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::Camera3DComponent::set_field_of_view`](#epok-camera3dcomponent-set-field-of-view-1) — Sets field of view as part of the object model module.
- [`epok::CanvasComponent::class_id`](#epok-canvascomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::CanvasComponent::enabled`](#epok-canvascomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::CanvasComponent::entity_slot`](#epok-canvascomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::CanvasComponent::set_enabled`](#epok-canvascomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::Collider3DComponent::class_id`](#epok-collider3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Collider3DComponent::enabled`](#epok-collider3dcomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::Collider3DComponent::entity_slot`](#epok-collider3dcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::Collider3DComponent::layer`](#epok-collider3dcomponent-layer-1) — Performs `layer` as part of the object model module.
- [`epok::Collider3DComponent::mask`](#epok-collider3dcomponent-mask-1) — Performs `mask` as part of the object model module.
- [`epok::Collider3DComponent::set_center`](#epok-collider3dcomponent-set-center-1) — Sets center as part of the object model module.
- [`epok::Collider3DComponent::set_enabled`](#epok-collider3dcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::Collider3DComponent::set_half_extents`](#epok-collider3dcomponent-set-half-extents-1) — Sets half extents as part of the object model module.
- [`epok::Collider3DComponent::set_layer`](#epok-collider3dcomponent-set-layer-1) — Sets layer as part of the object model module.
- [`epok::Collider3DComponent::set_mask`](#epok-collider3dcomponent-set-mask-1) — Sets mask as part of the object model module.
- [`epok::Collider3DComponent::set_trigger`](#epok-collider3dcomponent-set-trigger-1) — Sets trigger as part of the object model module.
- [`epok::Collider3DComponent::trigger`](#epok-collider3dcomponent-trigger-1) — Performs `trigger` as part of the object model module.
- [`epok::detail::compact_class_id`](#epok-detail-compact-class-id-1) — Single-block SHA-256: enough for a 36 character UUID (message + padding <= 64 bytes).
- [`epok::detail::sha256_rotr`](#epok-detail-sha256-rotr-1) — Performs `sha256 rotr` as part of the object model module.
- [`epok::dispatch_trigger`](#epok-dispatch-trigger-1) — Collision is owned by the spatial services, not by the Level: they resolve the legacy slot to its actor and call this, which fans the event out to the owner's components in registration order.
- [`epok::find_object_class`](#epok-find-object-class-1) — Finds object class as part of the object model module.
- [`epok::ImageComponent::class_id`](#epok-imagecomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ImageComponent::enabled`](#epok-imagecomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::ImageComponent::entity_slot`](#epok-imagecomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::ImageComponent::set_color`](#epok-imagecomponent-set-color-1) — Sets color as part of the object model module.
- [`epok::ImageComponent::set_enabled`](#epok-imagecomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::ImageComponent::set_region`](#epok-imagecomponent-set-region-1) — Sets region as part of the object model module.
- [`epok::ImageComponent::set_texture`](#epok-imagecomponent-set-texture-1) — Sets texture as part of the object model module.
- [`epok::is_active`](#epok-is-active-1) — Reports whether active as part of the object model module.
- [`epok::Level::actor_active`](#epok-level-actor-active-1) — ---- activation ----------------------------------------------------------------
- [`epok::Level::actor_at`](#epok-level-actor-at-1) — Performs `actor at` as part of the object model module.
- [`epok::Level::actor_count`](#epok-level-actor-count-1) — Performs `actor count` as part of the object model module.
- [`epok::Level::add_component`](#epok-level-add-component-1) — ---- components ----------------------------------------------------------------
- [`epok::Level::bind`](#epok-level-bind-1) — Binds the slot table and registers the Level itself.
- [`epok::Level::class_id`](#epok-level-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Level::create_scene_script`](#epok-level-create-scene-script-1) — The scene script is created by the loader; it is never part of the actor table.
- [`epok::Level::destroy_actor`](#epok-level-destroy-actor-1) — ---- destruction --------------------------------------------------------------- A request made inside a callback marks the actor immediately (it receives no further events) and runs the teardown when the current batch finishes.
- [`epok::Level::end_play_all`](#epok-level-end-play-all-1) — Exit order: scene script first while the actors are still alive, then every actor.
- [`epok::Level::frame_update`](#epok-level-frame-update-1) — Once per rendered frame, even while paused (legacy Behaviour parity).
- [`epok::Level::get_component`](#epok-level-get-component-1) — Returns component as part of the object model module.
- [`epok::Level::get_components`](#epok-level-get-components-1) — Returns components as part of the object model module.
- [`epok::Level::registry`](#epok-level-registry-1) — Performs `registry` as part of the object model module.
- [`epok::Level::remove_component`](#epok-level-remove-component-1) — Removes component as part of the object model module.
- [`epok::Level::scene_script`](#epok-level-scene-script-1) — Performs `scene script` as part of the object model module.
- [`epok::Level::set_active`](#epok-level-set-active-1) — Sets active as part of the object model module.
- [`epok::Level::set_logical_parent`](#epok-level-set-logical-parent-1) — Sets logical parent as part of the object model module.
- [`epok::Level::spawn_actor`](#epok-level-spawn-actor-1) — Performs `spawn actor` as part of the object model module.
- [`epok::Level::spawn_batch`](#epok-level-spawn-batch-1) — All or nothing.
- [`epok::Level::stats`](#epok-level-stats-1) — Performs `stats` as part of the object model module.
- [`epok::Level::tick`](#epok-level-tick-1) — ---- ticking -------------------------------------------------------------------
- [`epok::Light3DComponent::class_id`](#epok-light3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Light3DComponent::enabled`](#epok-light3dcomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::Light3DComponent::entity_slot`](#epok-light3dcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::Light3DComponent::intensity`](#epok-light3dcomponent-intensity-1) — Performs `intensity` as part of the object model module.
- [`epok::Light3DComponent::set_color`](#epok-light3dcomponent-set-color-1) — Sets color as part of the object model module.
- [`epok::Light3DComponent::set_enabled`](#epok-light3dcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::Light3DComponent::set_intensity`](#epok-light3dcomponent-set-intensity-1) — Sets intensity as part of the object model module.
- [`epok::Light3DComponent::set_mode`](#epok-light3dcomponent-set-mode-1) — Sets mode as part of the object model module.
- [`epok::Light3DComponent::set_range`](#epok-light3dcomponent-set-range-1) — Sets range as part of the object model module.
- [`epok::Light3DComponent::set_type`](#epok-light3dcomponent-set-type-1) — Sets type as part of the object model module.
- [`epok::Mesh3DComponent::bone_count`](#epok-mesh3dcomponent-bone-count-1) — Performs `bone count` as part of the object model module.
- [`epok::Mesh3DComponent::class_id`](#epok-mesh3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Mesh3DComponent::clip_count`](#epok-mesh3dcomponent-clip-count-1) — Performs `clip count` as part of the object model module.
- [`epok::Mesh3DComponent::entity_slot`](#epok-mesh3dcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::Mesh3DComponent::geometry_state`](#epok-mesh3dcomponent-geometry-state-1) — Performs `geometry state` as part of the object model module.
- [`epok::Mesh3DComponent::lighting_enabled`](#epok-mesh3dcomponent-lighting-enabled-1) — Performs `lighting enabled` as part of the object model module.
- [`epok::Mesh3DComponent::material_state`](#epok-mesh3dcomponent-material-state-1) — Performs `material state` as part of the object model module.
- [`epok::Mesh3DComponent::pause_animation`](#epok-mesh3dcomponent-pause-animation-1) — Pauses animation as part of the object model module.
- [`epok::Mesh3DComponent::play_clip`](#epok-mesh3dcomponent-play-clip-1) — Starts clip as part of the object model module.
- [`epok::Mesh3DComponent::playback_state`](#epok-mesh3dcomponent-playback-state-1) — Starts back state as part of the object model module.
- [`epok::Mesh3DComponent::quad_count`](#epok-mesh3dcomponent-quad-count-1) — Performs `quad count` as part of the object model module.
- [`epok::Mesh3DComponent::request_geometry`](#epok-mesh3dcomponent-request-geometry-1) — Requests geometry as part of the object model module.
- [`epok::Mesh3DComponent::resume_animation`](#epok-mesh3dcomponent-resume-animation-1) — Resumes animation as part of the object model module.
- [`epok::Mesh3DComponent::sample_bone`](#epok-mesh3dcomponent-sample-bone-1) — Performs `sample bone` as part of the object model module.
- [`epok::Mesh3DComponent::sample_geometry_vertex`](#epok-mesh3dcomponent-sample-geometry-vertex-1) — Performs `sample geometry vertex` as part of the object model module.
- [`epok::Mesh3DComponent::sample_vertex`](#epok-mesh3dcomponent-sample-vertex-1) — Performs `sample vertex` as part of the object model module.
- [`epok::Mesh3DComponent::sample_vertices`](#epok-mesh3dcomponent-sample-vertices-1) — Performs `sample vertices` as part of the object model module.
- [`epok::Mesh3DComponent::set_lighting_enabled`](#epok-mesh3dcomponent-set-lighting-enabled-1) — Sets lighting enabled as part of the object model module.
- [`epok::Mesh3DComponent::set_material_blend`](#epok-mesh3dcomponent-set-material-blend-1) — Sets material blend as part of the object model module.
- [`epok::Mesh3DComponent::set_material_color`](#epok-mesh3dcomponent-set-material-color-1) — Sets material color as part of the object model module.
- [`epok::Mesh3DComponent::set_material_depth_bias`](#epok-mesh3dcomponent-set-material-depth-bias-1) — Sets material depth bias as part of the object model module.
- [`epok::Mesh3DComponent::set_material_texture`](#epok-mesh3dcomponent-set-material-texture-1) — Sets material texture as part of the object model module.
- [`epok::Mesh3DComponent::set_material_unlit`](#epok-mesh3dcomponent-set-material-unlit-1) — Sets material unlit as part of the object model module.
- [`epok::Mesh3DComponent::set_uv_scroll`](#epok-mesh3dcomponent-set-uv-scroll-1) — Sets uv scroll as part of the object model module.
- [`epok::Mesh3DComponent::stop_animation`](#epok-mesh3dcomponent-stop-animation-1) — Stops animation as part of the object model module.
- [`epok::Mesh3DComponent::streamed`](#epok-mesh3dcomponent-streamed-1) — Performs `streamed` as part of the object model module.
- [`epok::Mesh3DComponent::vertex_count`](#epok-mesh3dcomponent-vertex-count-1) — Performs `vertex count` as part of the object model module.
- [`epok::Object::class_id`](#epok-object-class-id-1) — Runtime-owned identity hooks; not an authoring API.
- [`epok::Object::id`](#epok-object-id-1) — Performs `id` as part of the object model module.
- [`epok::Object::is_a`](#epok-object-is-a-1) — Reports whether a as part of the object model module.
- [`epok::Object::state`](#epok-object-state-1) — Performs `state` as part of the object model module.
- [`epok::Object::timeline_sync`](#epok-object-timeline-sync-1) — Performs `timeline sync` as part of the object model module.
- [`epok::Object::~Object`](#epok-object-object-1) — Releases the resources owned by `epok::Object`.
- [`epok::object_class_is_a`](#epok-object-class-is-a-1) — Ancestry walk bounded by the table size; the cook rejects cycles before emitting.
- [`epok::object_construct`](#epok-object-construct-1) — Placement-new into caller storage; destroy runs the virtual destructor of the concrete type, so a base pointer never slices.
- [`epok::object_destruct`](#epok-object-destruct-1) — Performs `object destruct` as part of the object model module.
- [`epok::object_domain_bit`](#epok-object-domain-bit-1) — Owner domain masks for components.
- [`epok::object_storage_quarantined`](#epok-object-storage-quarantined-1) — Defined after AudioComponent; `false` for every object that owns no retained storage.
- [`epok::ObjectDispatchScope::ObjectDispatchScope`](#epok-objectdispatchscope-objectdispatchscope-1) — Constructs `epok::ObjectDispatchScope` for the object model module.
- [`epok::ObjectDispatchScope::ObjectDispatchScope`](#epok-objectdispatchscope-objectdispatchscope-2) — Constructs `epok::ObjectDispatchScope` for the object model module.
- [`epok::ObjectDispatchScope::operator=`](#epok-objectdispatchscope-operator-1) — Performs `operator =` as part of the object model module.
- [`epok::ObjectDispatchScope::~ObjectDispatchScope`](#epok-objectdispatchscope-objectdispatchscope-3) — Releases the resources owned by `epok::ObjectDispatchScope`.
- [`epok::ObjectId::get`](#epok-objectid-get-1) — Returns get as part of the object model module.
- [`epok::ObjectId::operator!=`](#epok-objectid-operator-1) — Performs `operator !=` as part of the object model module.
- [`epok::ObjectId::operator==`](#epok-objectid-operator-2) — Performs `operator ==` as part of the object model module.
- [`epok::ObjectId::valid`](#epok-objectid-valid-1) — Performs `valid` as part of the object model module.
- [`epok::ObjectPool::acquire`](#epok-objectpool-acquire-1) — Performs `acquire` as part of the object model module.
- [`epok::ObjectPool::live`](#epok-objectpool-live-1) — Performs `live` as part of the object model module.
- [`epok::ObjectPool::release`](#epok-objectpool-release-1) — Performs `release` as part of the object model module.
- [`epok::ObjectRegistry::acquire`](#epok-objectregistry-acquire-1) — Pool-backed construction of a concrete class.
- [`epok::ObjectRegistry::adopt`](#epok-objectregistry-adopt-1) — Adopt storage owned by somebody else (an actor's embedded default component, the Level itself).
- [`epok::ObjectRegistry::class_of`](#epok-objectregistry-class-of-1) — Performs `class of` as part of the object model module.
- [`epok::ObjectRegistry::collect_quarantined`](#epok-objectregistry-collect-quarantined-1) — Retry the quarantined slots.
- [`epok::ObjectRegistry::finish_release`](#epok-objectregistry-finish-release-1) — Called when the last dispatch scope unwinds: no callback can still be on the stack of a destroyed object, so its typed storage returns to the pool.
- [`epok::ObjectRegistry::get`](#epok-objectregistry-get-1) — Returns get as part of the object model module.
- [`epok::ObjectRegistry::live`](#epok-objectregistry-live-1) — Performs `live` as part of the object model module.
- [`epok::ObjectRegistry::next_generation`](#epok-objectregistry-next-generation-1) — Performs `next generation` as part of the object model module.
- [`epok::ObjectRegistry::ObjectRegistry`](#epok-objectregistry-objectregistry-1) — Constructs `epok::ObjectRegistry` for the object model module.
- [`epok::ObjectRegistry::ObjectRegistry`](#epok-objectregistry-objectregistry-2) — Constructs `epok::ObjectRegistry` for the object model module.
- [`epok::ObjectRegistry::operator=`](#epok-objectregistry-operator-1) — Performs `operator =` as part of the object model module.
- [`epok::ObjectRegistry::release`](#epok-objectregistry-release-1) — Invalidates the handle immediately; storage is quarantined while callbacks run.
- [`epok::ObjectRegistry::resolve`](#epok-objectregistry-resolve-1) — Generation and class checked; the runtime class must derive from T's static class.
- [`epok::ObjectRegistry::resolve`](#epok-objectregistry-resolve-2) — Performs `resolve` as part of the object model module.
- [`epok::ObjectRegistry::slot`](#epok-objectregistry-slot-1) — Performs `slot` as part of the object model module.
- [`epok::ObjectRegistry::slot`](#epok-objectregistry-slot-2) — Performs `slot` as part of the object model module.
- [`epok::ObjectRegistryStorage::ObjectRegistryStorage<Capacity>`](#epok-objectregistrystorage-objectregistrystorage-capacity-1) — Constructs `epok::ObjectRegistryStorage` for the object model module.
- [`epok::PaletteAnimatorComponent::class_id`](#epok-paletteanimatorcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::PaletteAnimatorComponent::configure`](#epok-paletteanimatorcomponent-configure-1) — Performs `configure` as part of the object model module.
- [`epok::PaletteAnimatorComponent::entity_slot`](#epok-paletteanimatorcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::PaletteAnimatorComponent::reset`](#epok-paletteanimatorcomponent-reset-1) — Resets reset as part of the object model module.
- [`epok::PaletteAnimatorComponent::set_enabled`](#epok-paletteanimatorcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::PaletteAnimatorComponent::state`](#epok-paletteanimatorcomponent-state-1) — Performs `state` as part of the object model module.
- [`epok::ParticleEffectComponent::class_id`](#epok-particleeffectcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ParticleEmitterComponent::burst`](#epok-particleemittercomponent-burst-1) — Performs `burst` as part of the object model module.
- [`epok::ParticleEmitterComponent::class_id`](#epok-particleemittercomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ParticleEmitterComponent::entity_slot`](#epok-particleemittercomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::ParticleEmitterComponent::play`](#epok-particleemittercomponent-play-1) — Starts play as part of the object model module.
- [`epok::ParticleEmitterComponent::set_enabled`](#epok-particleemittercomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::ParticleEmitterComponent::set_lifetime`](#epok-particleemittercomponent-set-lifetime-1) — Sets lifetime as part of the object model module.
- [`epok::ParticleEmitterComponent::set_max_particles`](#epok-particleemittercomponent-set-max-particles-1) — Sets max particles as part of the object model module.
- [`epok::ParticleEmitterComponent::set_rate`](#epok-particleemittercomponent-set-rate-1) — Sets rate as part of the object model module.
- [`epok::ParticleEmitterComponent::state`](#epok-particleemittercomponent-state-1) — Performs `state` as part of the object model module.
- [`epok::ParticleEmitterComponent::stop`](#epok-particleemittercomponent-stop-1) — Stops stop as part of the object model module.
- [`epok::ProgressBarComponent::class_id`](#epok-progressbarcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ProgressBarComponent::enabled`](#epok-progressbarcomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::ProgressBarComponent::entity_slot`](#epok-progressbarcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::ProgressBarComponent::set_colors`](#epok-progressbarcomponent-set-colors-1) — Sets colors as part of the object model module.
- [`epok::ProgressBarComponent::set_enabled`](#epok-progressbarcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::ProgressBarComponent::set_value`](#epok-progressbarcomponent-set-value-1) — Sets value as part of the object model module.
- [`epok::ProgressBarComponent::value`](#epok-progressbarcomponent-value-1) — Performs `value` as part of the object model module.
- [`epok::RectTransformComponent::attach_slot`](#epok-recttransformcomponent-attach-slot-1) — Performs `attach slot` as part of the object model module.
- [`epok::RectTransformComponent::bind_local`](#epok-recttransformcomponent-bind-local-1) — Performs `bind local` as part of the object model module.
- [`epok::RectTransformComponent::bind_slot`](#epok-recttransformcomponent-bind-slot-1) — Performs `bind slot` as part of the object model module.
- [`epok::RectTransformComponent::class_id`](#epok-recttransformcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::RectTransformComponent::enabled`](#epok-recttransformcomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::RectTransformComponent::entity_slot`](#epok-recttransformcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::RectTransformComponent::set_anchors`](#epok-recttransformcomponent-set-anchors-1) — Sets anchors as part of the object model module.
- [`epok::RectTransformComponent::set_enabled`](#epok-recttransformcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::RectTransformComponent::set_pivot`](#epok-recttransformcomponent-set-pivot-1) — Sets pivot as part of the object model module.
- [`epok::RectTransformComponent::set_position`](#epok-recttransformcomponent-set-position-1) — Sets position as part of the object model module.
- [`epok::RectTransformComponent::set_size`](#epok-recttransformcomponent-set-size-1) — Sets size as part of the object model module.
- [`epok::SceneComponent2D::attach_slot`](#epok-scenecomponent2d-attach-slot-1) — Performs `attach slot` as part of the object model module.
- [`epok::SceneComponent2D::attach_to`](#epok-scenecomponent2d-attach-to-1) — Performs `attach to` as part of the object model module.
- [`epok::SceneComponent2D::class_id`](#epok-scenecomponent2d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::SceneComponent2D::position_x`](#epok-scenecomponent2d-position-x-1) — Performs `position x` as part of the object model module.
- [`epok::SceneComponent2D::position_y`](#epok-scenecomponent2d-position-y-1) — Performs `position y` as part of the object model module.
- [`epok::SceneComponent2D::rotation`](#epok-scenecomponent2d-rotation-1) — Performs `rotation` as part of the object model module.
- [`epok::SceneComponent2D::set_position`](#epok-scenecomponent2d-set-position-1) — Sets position as part of the object model module.
- [`epok::SceneComponent2D::set_rotation`](#epok-scenecomponent2d-set-rotation-1) — Sets rotation as part of the object model module.
- [`epok::SceneComponent2D::set_scale`](#epok-scenecomponent2d-set-scale-1) — Sets scale as part of the object model module.
- [`epok::SceneComponent3D::attach_slot`](#epok-scenecomponent3d-attach-slot-1) — Performs `attach slot` as part of the object model module.
- [`epok::SceneComponent3D::attach_to`](#epok-scenecomponent3d-attach-to-1) — Performs `attach to` as part of the object model module.
- [`epok::SceneComponent3D::bind_local`](#epok-scenecomponent3d-bind-local-1) — No legacy slot: the component owns the transform.
- [`epok::SceneComponent3D::bind_slot`](#epok-scenecomponent3d-bind-slot-1) — Canonical storage: the legacy entity slot owns the transform and everything that already reads it (collision, rendering, motion interpolation) keeps working.
- [`epok::SceneComponent3D::class_id`](#epok-scenecomponent3d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::SceneComponent3D::entity_slot`](#epok-scenecomponent3d-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::SceneComponent3D::local_transform`](#epok-scenecomponent3d-local-transform-1) — Performs `local transform` as part of the object model module.
- [`epok::SceneComponent3D::set_local_position`](#epok-scenecomponent3d-set-local-position-1) — Sets local position as part of the object model module.
- [`epok::SceneComponent3D::set_local_rotation`](#epok-scenecomponent3d-set-local-rotation-1) — Sets local rotation as part of the object model module.
- [`epok::SceneComponent3D::set_local_scale`](#epok-scenecomponent3d-set-local-scale-1) — Sets local scale as part of the object model module.
- [`epok::SceneComponent3D::set_local_transform`](#epok-scenecomponent3d-set-local-transform-1) — Sets local transform as part of the object model module.
- [`epok::SceneComponent3D::teleport`](#epok-scenecomponent3d-teleport-1) — Performs `teleport` as part of the object model module.
- [`epok::SceneComponent3D::world_affine`](#epok-scenecomponent3d-world-affine-1) — Performs `world affine` as part of the object model module.
- [`epok::SceneScriptActor::class_id`](#epok-scenescriptactor-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Sprite3DComponent::class_id`](#epok-sprite3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Sprite3DComponent::enabled`](#epok-sprite3dcomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::Sprite3DComponent::entity_slot`](#epok-sprite3dcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::Sprite3DComponent::pause_animation`](#epok-sprite3dcomponent-pause-animation-1) — Pauses animation as part of the object model module.
- [`epok::Sprite3DComponent::play_clip`](#epok-sprite3dcomponent-play-clip-1) — Starts clip as part of the object model module.
- [`epok::Sprite3DComponent::playback_state`](#epok-sprite3dcomponent-playback-state-1) — Starts back state as part of the object model module.
- [`epok::Sprite3DComponent::poll_event`](#epok-sprite3dcomponent-poll-event-1) — Polls event as part of the object model module.
- [`epok::Sprite3DComponent::resume_animation`](#epok-sprite3dcomponent-resume-animation-1) — Resumes animation as part of the object model module.
- [`epok::Sprite3DComponent::set_color`](#epok-sprite3dcomponent-set-color-1) — Sets color as part of the object model module.
- [`epok::Sprite3DComponent::set_enabled`](#epok-sprite3dcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::Sprite3DComponent::set_flip`](#epok-sprite3dcomponent-set-flip-1) — Sets flip as part of the object model module.
- [`epok::Sprite3DComponent::set_size`](#epok-sprite3dcomponent-set-size-1) — Sets size as part of the object model module.
- [`epok::Sprite3DComponent::set_texture`](#epok-sprite3dcomponent-set-texture-1) — Sets texture as part of the object model module.
- [`epok::Sprite3DComponent::take_completion`](#epok-sprite3dcomponent-take-completion-1) — Performs `take completion` as part of the object model module.
- [`epok::TextComponent::class_id`](#epok-textcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::TextComponent::clear_text`](#epok-textcomponent-clear-text-1) — Clears text as part of the object model module.
- [`epok::TextComponent::enabled`](#epok-textcomponent-enabled-1) — Performs `enabled` as part of the object model module.
- [`epok::TextComponent::entity_slot`](#epok-textcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::TextComponent::set_color`](#epok-textcomponent-set-color-1) — Sets color as part of the object model module.
- [`epok::TextComponent::set_enabled`](#epok-textcomponent-set-enabled-1) — Sets enabled as part of the object model module.
- [`epok::TextComponent::set_number`](#epok-textcomponent-set-number-1) — Sets number as part of the object model module.
- [`epok::TextComponent::set_text_word`](#epok-textcomponent-set-text-word-1) — Sets text word as part of the object model module.
- [`epok::TextComponent::set_unsigned`](#epok-textcomponent-set-unsigned-1) — Sets unsigned as part of the object model module.
- [`epok::TextComponent::set_wrap`](#epok-textcomponent-set-wrap-1) — Sets wrap as part of the object model module.
- [`epok::TextComponent::text_word`](#epok-textcomponent-text-word-1) — Performs `text word` as part of the object model module.
- [`epok::TimelineComponent::class_id`](#epok-timelinecomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::UIActor::class_id`](#epok-uiactor-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::UIActor::default_root`](#epok-uiactor-default-root-1) — Performs `default root` as part of the object model module.
- [`epok::UIComponent::class_id`](#epok-uicomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::World::bind`](#epok-world-bind-1) — Performs `bind` as part of the object model module.
- [`epok::World::class_id`](#epok-world-class-id-1) — Performs `class id` as part of the object model module.

<a id="epok-actor2d-class-id-1"></a>

## `epok::Actor2D::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 866](../../../runtime/object_model.hpp#L866)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor2d-default-root-1"></a>

## `epok::Actor2D::default_root`

**Purpose.** Performs `default root` as part of the object model module.

**Exact declaration**

```cpp
ActorComponent* default_root() override
```

- **Declared at:** [line 868](../../../runtime/object_model.hpp#L868)
- **Kind:** `cxx method`

**Returns.** Returns `ActorComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor2D& object = /* obtain a valid instance */;

auto result = object.default_root();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor3d-class-id-1"></a>

## `epok::Actor3D::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 859](../../../runtime/object_model.hpp#L859)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor3d-default-root-1"></a>

## `epok::Actor3D::default_root`

**Purpose.** Performs `default root` as part of the object model module.

**Exact declaration**

```cpp
ActorComponent* default_root() override
```

- **Declared at:** [line 861](../../../runtime/object_model.hpp#L861)
- **Kind:** `cxx method`

**Returns.** Returns `ActorComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor3D& object = /* obtain a valid instance */;

auto result = object.default_root();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-active-1"></a>

## `epok::Actor::active`

**Purpose.** Self flag only; Level::actor_active() folds the logical parent chain.

**Exact declaration**

```cpp
bool active() const
```

- **Declared at:** [line 453](../../../runtime/object_model.hpp#L453)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-actor-begin-play-1"></a>

## `epok::Actor::begin_play`

**Purpose.** Begins play as part of the object model module.

**Exact declaration**

```cpp
virtual void begin_play()
```

- **Declared at:** [line 426](../../../runtime/object_model.hpp#L426)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-bind-data-1"></a>

## `epok::Actor::bind_data`

**Purpose.** Performs `bind data` as part of the object model module.

**Exact declaration**

```cpp
void bind_data(ActorData& value)
```

- **Declared at:** [line 438](../../../runtime/object_model.hpp#L438)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & value

epok::Actor& object = /* obtain a valid instance */;

object.bind_data(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-actor-blueprint-observe-1"></a>

## `epok::Actor::blueprint_observe`

**Purpose.** Performs `blueprint observe` as part of the object model module.

**Exact declaration**

```cpp
virtual void blueprint_observe()
```

- **Declared at:** [line 427](../../../runtime/object_model.hpp#L427)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.blueprint_observe();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-class-id-1"></a>

## `epok::Actor::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 425](../../../runtime/object_model.hpp#L425)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-component-count-1"></a>

## `epok::Actor::component_count`

**Purpose.** Performs `component count` as part of the object model module.

**Exact declaration**

```cpp
size_t component_count() const
```

- **Declared at:** [line 451](../../../runtime/object_model.hpp#L451)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.component_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-component-id-1"></a>

## `epok::Actor::component_id`

**Purpose.** Performs `component id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId component_id(size_t index) const
```

- **Declared at:** [line 450](../../../runtime/object_model.hpp#L450)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::Actor& object = /* obtain a valid instance */;

auto result = object.component_id(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-data-1"></a>

## `epok::Actor::data`

**Purpose.** Legacy adapter: the canonical slot behind the root scene component, or nullptr.

**Exact declaration**

```cpp
ActorData* data() const
```

- **Declared at:** [line 437](../../../runtime/object_model.hpp#L437)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.data();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-default-root-1"></a>

## `epok::Actor::default_root`

**Purpose.** Runtime hook, not reflected: the declarative root component embedded in the actor.

**Exact declaration**

```cpp
virtual ActorComponent* default_root()
```

- **Declared at:** [line 435](../../../runtime/object_model.hpp#L435)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** Returns `ActorComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.default_root();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-destroy-1"></a>

## `epok::Actor::destroy`

**Purpose.** Destroys destroy as part of the object model module.

**Exact declaration**

```cpp
void destroy()
```

- **Declared at:** [line 457](../../../runtime/object_model.hpp#L457)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.destroy();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-end-play-1"></a>

## `epok::Actor::end_play`

**Purpose.** Ends play as part of the object model module.

**Exact declaration**

```cpp
virtual void end_play(EndPlayReason end_play_reason)
```

- **Declared at:** [line 429](../../../runtime/object_model.hpp#L429)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `end_play_reason` | `EndPlayReason` | Input | Value supplied for `end_play_reason`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason end_play_reason

epok::Actor& object = /* obtain a valid instance */;

object.end_play(end_play_reason);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-frame-update-1"></a>

## `epok::Actor::frame_update`

**Purpose.** Performs `frame update` as part of the object model module.

**Exact declaration**

```cpp
virtual void frame_update(uint32_t frame_microseconds)
```

- **Declared at:** [line 433](../../../runtime/object_model.hpp#L433)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame_microseconds` | `uint32_t` | Input | Value supplied for `frame_microseconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t frame_microseconds

epok::Actor& object = /* obtain a valid instance */;

object.frame_update(frame_microseconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-level-id-1"></a>

## `epok::Actor::level_id`

**Purpose.** Reflected identity/hierarchy readers.

**Details.** Every authoring provider (C++, Blueprint, Lua) inherits them from this base instead of bridging a private builtin.

**Exact declaration**

```cpp
ObjectId level_id() const
```

- **Declared at:** [line 447](../../../runtime/object_model.hpp#L447)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** Every authoring provider (C++, Blueprint, Lua) inherits them from this base instead of bridging a private builtin.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.level_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-logical-parent-1"></a>

## `epok::Actor::logical_parent`

**Purpose.** Performs `logical parent` as part of the object model module.

**Exact declaration**

```cpp
ObjectId logical_parent() const
```

- **Declared at:** [line 449](../../../runtime/object_model.hpp#L449)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.logical_parent();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-name-1"></a>

## `epok::Actor::name`

**Purpose.** Performs `name` as part of the object model module.

**Exact declaration**

```cpp
const char* name() const
```

- **Declared at:** [line 439](../../../runtime/object_model.hpp#L439)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.name();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-on-disable-1"></a>

## `epok::Actor::on_disable`

**Purpose.** Performs `on disable` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_disable()
```

- **Declared at:** [line 431](../../../runtime/object_model.hpp#L431)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-on-enable-1"></a>

## `epok::Actor::on_enable`

**Purpose.** Performs `on enable` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_enable()
```

- **Declared at:** [line 430](../../../runtime/object_model.hpp#L430)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.on_enable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-on-frame-1"></a>

## `epok::Actor::on_frame`

**Purpose.** Performs `on frame` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_frame(uint32_t frame_microseconds)
```

- **Declared at:** [line 432](../../../runtime/object_model.hpp#L432)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame_microseconds` | `uint32_t` | Input | Value supplied for `frame_microseconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t frame_microseconds

epok::Actor& object = /* obtain a valid instance */;

object.on_frame(frame_microseconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-root-id-1"></a>

## `epok::Actor::root_id`

**Purpose.** Performs `root id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId root_id() const
```

- **Declared at:** [line 448](../../../runtime/object_model.hpp#L448)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.root_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-set-active-1"></a>

## `epok::Actor::set_active`

**Purpose.** Folded activation with the on_enable/on_disable fan-out, and deferred destruction inside a dispatch scope: the same Level paths the legacy Blueprint nodes take.

**Exact declaration**

```cpp
void set_active(bool active)
```

- **Declared at:** [line 456](../../../runtime/object_model.hpp#L456)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool active

epok::Actor& object = /* obtain a valid instance */;

object.set_active(active);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-set-name-1"></a>

## `epok::Actor::set_name`

**Purpose.** Sets name as part of the object model module.

**Exact declaration**

```cpp
void set_name(const char* value)
```

- **Declared at:** [line 440](../../../runtime/object_model.hpp#L440)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const char *` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const char * value

epok::Actor& object = /* obtain a valid instance */;

object.set_name(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-actor-set-wants-tick-1"></a>

## `epok::Actor::set_wants_tick`

**Purpose.** Sets wants tick as part of the object model module.

**Exact declaration**

```cpp
void set_wants_tick(bool value)
```

- **Declared at:** [line 459](../../../runtime/object_model.hpp#L459)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Actor& object = /* obtain a valid instance */;

object.set_wants_tick(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-tick-1"></a>

## `epok::Actor::tick`

**Purpose.** Performs `tick` as part of the object model module.

**Exact declaration**

```cpp
virtual void tick(Fixed delta_seconds)
```

- **Declared at:** [line 428](../../../runtime/object_model.hpp#L428)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::Actor& object = /* obtain a valid instance */;

object.tick(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-wants-tick-1"></a>

## `epok::Actor::wants_tick`

**Purpose.** Performs `wants tick` as part of the object model module.

**Exact declaration**

```cpp
bool wants_tick() const
```

- **Declared at:** [line 458](../../../runtime/object_model.hpp#L458)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.wants_tick();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-actorcomponent-attach-slot-1"></a>

## `epok::ActorComponent::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
virtual ObjectId* attach_slot()
```

- **Declared at:** [line 492](../../../runtime/object_model.hpp#L492)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** Returns `ObjectId *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.attach_slot();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-begin-play-1"></a>

## `epok::ActorComponent::begin_play`

**Purpose.** Begins play as part of the object model module.

**Exact declaration**

```cpp
virtual void begin_play()
```

- **Declared at:** [line 481](../../../runtime/object_model.hpp#L481)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-blueprint-observe-1"></a>

## `epok::ActorComponent::blueprint_observe`

**Purpose.** Performs `blueprint observe` as part of the object model module.

**Exact declaration**

```cpp
virtual void blueprint_observe()
```

- **Declared at:** [line 490](../../../runtime/object_model.hpp#L490)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

object.blueprint_observe();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-class-id-1"></a>

## `epok::ActorComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 480](../../../runtime/object_model.hpp#L480)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-end-play-1"></a>

## `epok::ActorComponent::end_play`

**Purpose.** Ends play as part of the object model module.

**Exact declaration**

```cpp
virtual void end_play(EndPlayReason end_play_reason)
```

- **Declared at:** [line 483](../../../runtime/object_model.hpp#L483)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `end_play_reason` | `EndPlayReason` | Input | Value supplied for `end_play_reason`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason end_play_reason

epok::ActorComponent& object = /* obtain a valid instance */;

object.end_play(end_play_reason);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-frame-update-1"></a>

## `epok::ActorComponent::frame_update`

**Purpose.** Performs `frame update` as part of the object model module.

**Exact declaration**

```cpp
virtual void frame_update(uint32_t frame_microseconds)
```

- **Declared at:** [line 491](../../../runtime/object_model.hpp#L491)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame_microseconds` | `uint32_t` | Input | Value supplied for `frame_microseconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t frame_microseconds

epok::ActorComponent& object = /* obtain a valid instance */;

object.frame_update(frame_microseconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-get-owner-1"></a>

## `epok::ActorComponent::get_owner`

**Purpose.** Returns owner as part of the object model module.

**Exact declaration**

```cpp
Actor* get_owner()
```

- **Declared at:** [line 502](../../../runtime/object_model.hpp#L502)
- **Kind:** `cxx method`

**Returns.** Returns `Actor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.get_owner();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-name-1"></a>

## `epok::ActorComponent::name`

**Purpose.** Performs `name` as part of the object model module.

**Exact declaration**

```cpp
const char* name() const
```

- **Declared at:** [line 503](../../../runtime/object_model.hpp#L503)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.name();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-on-disable-1"></a>

## `epok::ActorComponent::on_disable`

**Purpose.** Performs `on disable` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_disable()
```

- **Declared at:** [line 485](../../../runtime/object_model.hpp#L485)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-on-enable-1"></a>

## `epok::ActorComponent::on_enable`

**Purpose.** Performs `on enable` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_enable()
```

- **Declared at:** [line 484](../../../runtime/object_model.hpp#L484)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

object.on_enable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-on-frame-1"></a>

## `epok::ActorComponent::on_frame`

**Purpose.** Performs `on frame` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_frame(uint32_t frame_microseconds)
```

- **Declared at:** [line 486](../../../runtime/object_model.hpp#L486)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame_microseconds` | `uint32_t` | Input | Value supplied for `frame_microseconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t frame_microseconds

epok::ActorComponent& object = /* obtain a valid instance */;

object.on_frame(frame_microseconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-on-trigger-1"></a>

## `epok::ActorComponent::on_trigger`

**Purpose.** Performs `on trigger` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_trigger(DataHandle other, TriggerPhase phase)
```

- **Declared at:** [line 497](../../../runtime/object_model.hpp#L497)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `DataHandle` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle other
// TriggerPhase phase

epok::ActorComponent& object = /* obtain a valid instance */;

object.on_trigger(other, phase);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-owner-id-1"></a>

## `epok::ActorComponent::owner_id`

**Purpose.** Performs `owner id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId owner_id() const
```

- **Declared at:** [line 501](../../../runtime/object_model.hpp#L501)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.owner_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-releasable-1"></a>

## `epok::ActorComponent::releasable`

**Purpose.** False while a service still holds this component's storage; the registry then keeps the (already dead) slot quarantined instead of returning it to the pool.

**Exact declaration**

```cpp
virtual bool releasable() const
```

- **Declared at:** [line 500](../../../runtime/object_model.hpp#L500)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.releasable();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-actorcomponent-set-name-1"></a>

## `epok::ActorComponent::set_name`

**Purpose.** Sets name as part of the object model module.

**Exact declaration**

```cpp
void set_name(const char* value)
```

- **Declared at:** [line 504](../../../runtime/object_model.hpp#L504)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const char *` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const char * value

epok::ActorComponent& object = /* obtain a valid instance */;

object.set_name(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-actorcomponent-tick-1"></a>

## `epok::ActorComponent::tick`

**Purpose.** Performs `tick` as part of the object model module.

**Exact declaration**

```cpp
virtual void tick(Fixed delta_seconds)
```

- **Declared at:** [line 482](../../../runtime/object_model.hpp#L482)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::ActorComponent& object = /* obtain a valid instance */;

object.tick(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-timeline-sync-1"></a>

## `epok::ActorComponent::timeline_sync`

**Purpose.** Runtime hooks, not reflected.

**Details.** frame_update runs once per rendered frame even while paused; attach_slot exposes the spatial parent of components that have one.

**Exact declaration**

```cpp
void timeline_sync(uint64_t, bool) override
```

- **Declared at:** [line 489](../../../runtime/object_model.hpp#L489)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint64_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `bool` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** frame_update runs once per rendered frame even while paused; attach_slot exposes the spatial parent of components that have one.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t arg1
// bool arg2

epok::ActorComponent& object = /* obtain a valid instance */;

object.timeline_sync(arg1, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-trigger-event-1"></a>

## `epok::ActorComponent::trigger_event`

**Purpose.** Forward-only collision hook.

**Details.** The Level does not own collision; the collision service calls dispatch_trigger(Level&, ...) which fans the event out to the owner's components. Nothing in the object model generates trigger events.

**Exact declaration**

```cpp
virtual void trigger_event(ObjectId other,TriggerPhase phase)
```

- **Declared at:** [line 496](../../../runtime/object_model.hpp#L496)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `ObjectId` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The Level does not own collision; the collision service calls dispatch_trigger(Level&, ...) which fans the event out to the owner's components. Nothing in the object model generates trigger events.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId other
// TriggerPhase phase

epok::ActorComponent& object = /* obtain a valid instance */;

object.trigger_event(other, phase);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-attach-component-1"></a>

## `epok::attach_component`

**Purpose.** Spatial attachment between components of the same domain.

**Details.** Logical actor parenting never inherits matrices; only this attachment does.

**Exact declaration**

```cpp
inline bool attach_component(ObjectId child,ObjectId parent)
```

- **Declared at:** [line 535](../../../runtime/object_model.hpp#L535)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `ObjectId` | Input | Value supplied for `child`. See the exact type and module contract. |
| `parent` | `ObjectId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Logical actor parenting never inherits matrices; only this attachment does.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId child
// ObjectId parent

auto result = epok::attach_component(child, parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-audiocomponent-begin-play-1"></a>

## `epok::AudioComponent::begin_play`

**Purpose.** play_on_start policy.

**Details.** Exactly one path starts a given AudioSource: * slot-backed: the scene bank's bank-load loop and bp::activate_spawn_audio (runtime/lifecycle.hpp) start it. begin_play here must NOT play, or a migrated entity would be heard twice. * component-owned: no legacy path knows about `local`, so begin_play starts it when play_on_start is set, the source is enabled and the owner is active.

**Exact declaration**

```cpp
void begin_play() override
```

- **Declared at:** [line 641](../../../runtime/object_model.hpp#L641)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Exactly one path starts a given AudioSource: * slot-backed: the scene bank's bank-load loop and bp::activate_spawn_audio (runtime/lifecycle.hpp) start it. begin_play here must NOT play, or a migrated entity would be heard twice. * component-owned: no legacy path knows about `local`, so begin_play starts it when play_on_start is set, the source is enabled and the owner is active.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-bind-local-1"></a>

## `epok::AudioComponent::bind_local`

**Purpose.** Performs `bind local` as part of the object model module.

**Exact declaration**

```cpp
void bind_local()
```

- **Declared at:** [line 628](../../../runtime/object_model.hpp#L628)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.bind_local();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-bind-slot-1"></a>

## `epok::AudioComponent::bind_slot`

**Purpose.** Bind to the legacy slot's AudioSource, or to component-owned storage.

**Exact declaration**

```cpp
void bind_slot(ActorData& value)
```

- **Declared at:** [line 627](../../../runtime/object_model.hpp#L627)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & value

epok::AudioComponent& object = /* obtain a valid instance */;

object.bind_slot(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-audiocomponent-class-id-1"></a>

## `epok::AudioComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 609](../../../runtime/object_model.hpp#L609)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-clip-1"></a>

## `epok::AudioComponent::clip`

**Purpose.** Performs `clip` as part of the object model module.

**Exact declaration**

```cpp
int32_t clip() const
```

- **Declared at:** [line 616](../../../runtime/object_model.hpp#L616)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.clip();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-enabled-1"></a>

## `epok::AudioComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 614](../../../runtime/object_model.hpp#L614)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-end-play-1"></a>

## `epok::AudioComponent::end_play`

**Purpose.** Ends play as part of the object model module.

**Exact declaration**

```cpp
void end_play(EndPlayReason) override
```

- **Declared at:** [line 650](../../../runtime/object_model.hpp#L650)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::AudioComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-entity-slot-1"></a>

## `epok::AudioComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 629](../../../runtime/object_model.hpp#L629)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-is-playing-1"></a>

## `epok::AudioComponent::is_playing`

**Purpose.** Reports whether playing as part of the object model module.

**Exact declaration**

```cpp
bool is_playing() const
```

- **Declared at:** [line 613](../../../runtime/object_model.hpp#L613)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.is_playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-on-disable-1"></a>

## `epok::AudioComponent::on_disable`

**Purpose.** Deactivation and teardown stop both kinds: lifecycle.hpp::set_active stops a legacy slot's audio through the slot table, and stopping an already stopped source is a no-op, so the two paths are idempotent rather than conflicting.

**Exact declaration**

```cpp
void on_disable() override
```

- **Declared at:** [line 649](../../../runtime/object_model.hpp#L649)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-owns-source-1"></a>

## `epok::AudioComponent::owns_source`

**Purpose.** True only for component-owned storage.

**Details.** A slot-backed component is a *view* over a legacy AudioSource that the scene bank and lifecycle.hpp already drive.

**Exact declaration**

```cpp
bool owns_source() const
```

- **Declared at:** [line 632](../../../runtime/object_model.hpp#L632)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** A slot-backed component is a *view* over a legacy AudioSource that the scene bank and lifecycle.hpp already drive.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.owns_source();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-pitch-1"></a>

## `epok::AudioComponent::pitch`

**Purpose.** Performs `pitch` as part of the object model module.

**Exact declaration**

```cpp
Fixed pitch() const
```

- **Declared at:** [line 620](../../../runtime/object_model.hpp#L620)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.pitch();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-play-1"></a>

## `epok::AudioComponent::play`

**Purpose.** Starts play as part of the object model module.

**Exact declaration**

```cpp
void play()
```

- **Declared at:** [line 611](../../../runtime/object_model.hpp#L611)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.play();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-play-on-start-1"></a>

## `epok::AudioComponent::play_on_start`

**Purpose.** Starts on start as part of the object model module.

**Exact declaration**

```cpp
bool play_on_start() const
```

- **Declared at:** [line 624](../../../runtime/object_model.hpp#L624)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.play_on_start();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-priority-1"></a>

## `epok::AudioComponent::priority`

**Purpose.** Performs `priority` as part of the object model module.

**Exact declaration**

```cpp
uint32_t priority() const
```

- **Declared at:** [line 622](../../../runtime/object_model.hpp#L622)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.priority();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-releasable-1"></a>

## `epok::AudioComponent::releasable`

**Purpose.** Component-owned storage stays quarantined while an asynchronous consumer (the XA music service) still points at it, mirroring allocate_actor_data's legacy-slot rule.

**Details.** Slot storage belongs to the scene bank, so this component never holds it back.

**Exact declaration**

```cpp
bool releasable() const override
```

- **Declared at:** [line 654](../../../runtime/object_model.hpp#L654)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Slot storage belongs to the scene bank, so this component never holds it back.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.releasable();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-set-clip-1"></a>

## `epok::AudioComponent::set_clip`

**Purpose.** Sets clip as part of the object model module.

**Exact declaration**

```cpp
void set_clip(int32_t value)
```

- **Declared at:** [line 617](../../../runtime/object_model.hpp#L617)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::AudioComponent& object = /* obtain a valid instance */;

object.set_clip(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-set-enabled-1"></a>

## `epok::AudioComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 615](../../../runtime/object_model.hpp#L615)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::AudioComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-set-pitch-1"></a>

## `epok::AudioComponent::set_pitch`

**Purpose.** Sets pitch as part of the object model module.

**Exact declaration**

```cpp
void set_pitch(Fixed value)
```

- **Declared at:** [line 621](../../../runtime/object_model.hpp#L621)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::AudioComponent& object = /* obtain a valid instance */;

object.set_pitch(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-set-play-on-start-1"></a>

## `epok::AudioComponent::set_play_on_start`

**Purpose.** Sets play on start as part of the object model module.

**Exact declaration**

```cpp
void set_play_on_start(bool value)
```

- **Declared at:** [line 625](../../../runtime/object_model.hpp#L625)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::AudioComponent& object = /* obtain a valid instance */;

object.set_play_on_start(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-set-priority-1"></a>

## `epok::AudioComponent::set_priority`

**Purpose.** Sets priority as part of the object model module.

**Exact declaration**

```cpp
void set_priority(uint32_t value)
```

- **Declared at:** [line 623](../../../runtime/object_model.hpp#L623)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::AudioComponent& object = /* obtain a valid instance */;

object.set_priority(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-set-volume-1"></a>

## `epok::AudioComponent::set_volume`

**Purpose.** Sets volume as part of the object model module.

**Exact declaration**

```cpp
void set_volume(Fixed value)
```

- **Declared at:** [line 619](../../../runtime/object_model.hpp#L619)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::AudioComponent& object = /* obtain a valid instance */;

object.set_volume(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-stop-1"></a>

## `epok::AudioComponent::stop`

**Purpose.** Stops stop as part of the object model module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 612](../../../runtime/object_model.hpp#L612)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-volume-1"></a>

## `epok::AudioComponent::volume`

**Purpose.** Performs `volume` as part of the object model module.

**Exact declaration**

```cpp
Fixed volume() const
```

- **Declared at:** [line 618](../../../runtime/object_model.hpp#L618)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.volume();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-blobshadowcomponent-class-id-1"></a>

## `epok::BlobShadowComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 849](../../../runtime/object_model.hpp#L849)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::BlobShadowComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-blobshadowcomponent-configure-1"></a>

## `epok::BlobShadowComponent::configure`

**Purpose.** Performs `configure` as part of the object model module.

**Exact declaration**

```cpp
void configure(Fixed radius,Fixed strength,Fixed distance)
```

- **Declared at:** [line 853](../../../runtime/object_model.hpp#L853)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `radius` | `Fixed` | Input | Value supplied for `radius`. See the exact type and module contract. |
| `strength` | `Fixed` | Input | Value supplied for `strength`. See the exact type and module contract. |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed radius
// Fixed strength
// Fixed distance

epok::BlobShadowComponent& object = /* obtain a valid instance */;

object.configure(radius, strength, distance);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-blobshadowcomponent-enabled-1"></a>

## `epok::BlobShadowComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 851](../../../runtime/object_model.hpp#L851)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::BlobShadowComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-blobshadowcomponent-entity-slot-1"></a>

## `epok::BlobShadowComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 850](../../../runtime/object_model.hpp#L850)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::BlobShadowComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-blobshadowcomponent-set-enabled-1"></a>

## `epok::BlobShadowComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 852](../../../runtime/object_model.hpp#L852)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::BlobShadowComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera3dcomponent-class-id-1"></a>

## `epok::Camera3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 729](../../../runtime/object_model.hpp#L729)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Camera3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera3dcomponent-enabled-1"></a>

## `epok::Camera3DComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 731](../../../runtime/object_model.hpp#L731)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Camera3DComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-camera3dcomponent-entity-slot-1"></a>

## `epok::Camera3DComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 730](../../../runtime/object_model.hpp#L730)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Camera3DComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera3dcomponent-field-of-view-1"></a>

## `epok::Camera3DComponent::field_of_view`

**Purpose.** Performs `field of view` as part of the object model module.

**Exact declaration**

```cpp
Fixed field_of_view() const
```

- **Declared at:** [line 733](../../../runtime/object_model.hpp#L733)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Camera3DComponent& object = /* obtain a valid instance */;

auto result = object.field_of_view();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera3dcomponent-make-active-1"></a>

## `epok::Camera3DComponent::make_active`

**Purpose.** Performs `make active` as part of the object model module.

**Exact declaration**

```cpp
bool make_active()
```

- **Declared at:** [line 735](../../../runtime/object_model.hpp#L735)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Camera3DComponent& object = /* obtain a valid instance */;

auto result = object.make_active();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-camera3dcomponent-set-enabled-1"></a>

## `epok::Camera3DComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 732](../../../runtime/object_model.hpp#L732)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Camera3DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera3dcomponent-set-field-of-view-1"></a>

## `epok::Camera3DComponent::set_field_of_view`

**Purpose.** Sets field of view as part of the object model module.

**Exact declaration**

```cpp
void set_field_of_view(Fixed value)
```

- **Declared at:** [line 734](../../../runtime/object_model.hpp#L734)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Camera3DComponent& object = /* obtain a valid instance */;

object.set_field_of_view(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvascomponent-class-id-1"></a>

## `epok::CanvasComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 770](../../../runtime/object_model.hpp#L770)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::CanvasComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvascomponent-enabled-1"></a>

## `epok::CanvasComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 772](../../../runtime/object_model.hpp#L772)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::CanvasComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-canvascomponent-entity-slot-1"></a>

## `epok::CanvasComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 771](../../../runtime/object_model.hpp#L771)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::CanvasComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvascomponent-set-enabled-1"></a>

## `epok::CanvasComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 773](../../../runtime/object_model.hpp#L773)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::CanvasComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-class-id-1"></a>

## `epok::Collider3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 754](../../../runtime/object_model.hpp#L754)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Collider3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-enabled-1"></a>

## `epok::Collider3DComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 756](../../../runtime/object_model.hpp#L756)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Collider3DComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-collider3dcomponent-entity-slot-1"></a>

## `epok::Collider3DComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 755](../../../runtime/object_model.hpp#L755)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Collider3DComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-layer-1"></a>

## `epok::Collider3DComponent::layer`

**Purpose.** Performs `layer` as part of the object model module.

**Exact declaration**

```cpp
uint32_t layer() const
```

- **Declared at:** [line 760](../../../runtime/object_model.hpp#L760)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Collider3DComponent& object = /* obtain a valid instance */;

auto result = object.layer();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-mask-1"></a>

## `epok::Collider3DComponent::mask`

**Purpose.** Performs `mask` as part of the object model module.

**Exact declaration**

```cpp
uint32_t mask() const
```

- **Declared at:** [line 762](../../../runtime/object_model.hpp#L762)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Collider3DComponent& object = /* obtain a valid instance */;

auto result = object.mask();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-set-center-1"></a>

## `epok::Collider3DComponent::set_center`

**Purpose.** Sets center as part of the object model module.

**Exact declaration**

```cpp
void set_center(Fixed x,Fixed y,Fixed z)
```

- **Declared at:** [line 764](../../../runtime/object_model.hpp#L764)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `Fixed` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y
// Fixed z

epok::Collider3DComponent& object = /* obtain a valid instance */;

object.set_center(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-set-enabled-1"></a>

## `epok::Collider3DComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 757](../../../runtime/object_model.hpp#L757)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Collider3DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-set-half-extents-1"></a>

## `epok::Collider3DComponent::set_half_extents`

**Purpose.** Sets half extents as part of the object model module.

**Exact declaration**

```cpp
void set_half_extents(Fixed x,Fixed y,Fixed z)
```

- **Declared at:** [line 765](../../../runtime/object_model.hpp#L765)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `Fixed` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y
// Fixed z

epok::Collider3DComponent& object = /* obtain a valid instance */;

object.set_half_extents(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-set-layer-1"></a>

## `epok::Collider3DComponent::set_layer`

**Purpose.** Sets layer as part of the object model module.

**Exact declaration**

```cpp
void set_layer(uint32_t value)
```

- **Declared at:** [line 761](../../../runtime/object_model.hpp#L761)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::Collider3DComponent& object = /* obtain a valid instance */;

object.set_layer(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-set-mask-1"></a>

## `epok::Collider3DComponent::set_mask`

**Purpose.** Sets mask as part of the object model module.

**Exact declaration**

```cpp
void set_mask(uint32_t value)
```

- **Declared at:** [line 763](../../../runtime/object_model.hpp#L763)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::Collider3DComponent& object = /* obtain a valid instance */;

object.set_mask(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-set-trigger-1"></a>

## `epok::Collider3DComponent::set_trigger`

**Purpose.** Sets trigger as part of the object model module.

**Exact declaration**

```cpp
void set_trigger(bool value)
```

- **Declared at:** [line 759](../../../runtime/object_model.hpp#L759)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Collider3DComponent& object = /* obtain a valid instance */;

object.set_trigger(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-trigger-1"></a>

## `epok::Collider3DComponent::trigger`

**Purpose.** Performs `trigger` as part of the object model module.

**Exact declaration**

```cpp
bool trigger() const
```

- **Declared at:** [line 758](../../../runtime/object_model.hpp#L758)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Collider3DComponent& object = /* obtain a valid instance */;

auto result = object.trigger();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-detail-compact-class-id-1"></a>

## `epok::detail::compact_class_id`

**Purpose.** Single-block SHA-256: enough for a 36 character UUID (message + padding <= 64 bytes).

**Exact declaration**

```cpp
constexpr uint64_t compact_class_id(const char* text)
```

- **Declared at:** [line 101](../../../runtime/object_model.hpp#L101)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `text` | `const char *` | Input | Value supplied for `text`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const char * text

auto result = epok::detail::compact_class_id(text);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-sha256-rotr-1"></a>

## `epok::detail::sha256_rotr`

**Purpose.** Performs `sha256 rotr` as part of the object model module.

**Exact declaration**

```cpp
constexpr uint32_t sha256_rotr(uint32_t value, unsigned bits)
```

- **Declared at:** [line 97](../../../runtime/object_model.hpp#L97)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `bits` | `unsigned int` | Input | Value supplied for `bits`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value
// unsigned int bits

auto result = epok::detail::sha256_rotr(value, bits);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dispatch-trigger-1"></a>

## `epok::dispatch_trigger`

**Purpose.** Collision is owned by the spatial services, not by the Level: they resolve the legacy slot to its actor and call this, which fans the event out to the owner's components in registration order.

**Details.** Inactive, unstarted and doomed actors receive nothing.

**Exact declaration**

```cpp
inline size_t dispatch_trigger(Level& level, ObjectId actor, DataHandle other, TriggerPhase phase)
```

- **Declared at:** [line 1471](../../../runtime/object_model.hpp#L1471)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `level` | `Level &` | Input/output; inspect the function contract | Value supplied for `level`. See the exact type and module contract. |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `other` | `DataHandle` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** Inactive, unstarted and doomed actors receive nothing.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Level & level
// ObjectId actor
// DataHandle other
// TriggerPhase phase

auto result = epok::dispatch_trigger(level, actor, other, phase);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-find-object-class-1"></a>

## `epok::find_object_class`

**Purpose.** Finds object class as part of the object model module.

**Exact declaration**

```cpp
inline const ClassDescriptor* find_object_class(uint64_t id)
```

- **Declared at:** [line 167](../../../runtime/object_model.hpp#L167)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `uint64_t` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const ClassDescriptor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t id

auto result = epok::find_object_class(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-imagecomponent-class-id-1"></a>

## `epok::ImageComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 778](../../../runtime/object_model.hpp#L778)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ImageComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-imagecomponent-enabled-1"></a>

## `epok::ImageComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 780](../../../runtime/object_model.hpp#L780)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ImageComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-imagecomponent-entity-slot-1"></a>

## `epok::ImageComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 779](../../../runtime/object_model.hpp#L779)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ImageComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-imagecomponent-set-color-1"></a>

## `epok::ImageComponent::set_color`

**Purpose.** Sets color as part of the object model module.

**Exact declaration**

```cpp
void set_color(uint32_t red,uint32_t green,uint32_t blue)
```

- **Declared at:** [line 783](../../../runtime/object_model.hpp#L783)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::ImageComponent& object = /* obtain a valid instance */;

object.set_color(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-imagecomponent-set-enabled-1"></a>

## `epok::ImageComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 781](../../../runtime/object_model.hpp#L781)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::ImageComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-imagecomponent-set-region-1"></a>

## `epok::ImageComponent::set_region`

**Purpose.** Sets region as part of the object model module.

**Exact declaration**

```cpp
void set_region(uint32_t x,uint32_t y,uint32_t width,uint32_t height)
```

- **Declared at:** [line 784](../../../runtime/object_model.hpp#L784)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `uint32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `uint32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `width` | `uint32_t` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `uint32_t` | Input | Value supplied for `height`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t x
// uint32_t y
// uint32_t width
// uint32_t height

epok::ImageComponent& object = /* obtain a valid instance */;

object.set_region(x, y, width, height);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-imagecomponent-set-texture-1"></a>

## `epok::ImageComponent::set_texture`

**Purpose.** Sets texture as part of the object model module.

**Exact declaration**

```cpp
void set_texture(int32_t value)
```

- **Declared at:** [line 782](../../../runtime/object_model.hpp#L782)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::ImageComponent& object = /* obtain a valid instance */;

object.set_texture(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-is-active-1"></a>

## `epok::is_active`

**Purpose.** Reports whether active as part of the object model module.

**Exact declaration**

```cpp
inline bool is_active(Object* value)
```

- **Declared at:** [line 1490](../../../runtime/object_model.hpp#L1490)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Object *` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object * value

auto result = epok::is_active(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-actor-active-1"></a>

## `epok::Level::actor_active`

**Purpose.** ---- activation ----------------------------------------------------------------

**Exact declaration**

```cpp
bool actor_active(const Actor& actor) const
```

- **Declared at:** [line 1084](../../../runtime/object_model.hpp#L1084)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `const Actor &` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const Actor & actor

epok::Level& object = /* obtain a valid instance */;

auto result = object.actor_active(actor);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-actor-at-1"></a>

## `epok::Level::actor_at`

**Purpose.** Performs `actor at` as part of the object model module.

**Exact declaration**

```cpp
ObjectId actor_at(size_t index) const
```

- **Declared at:** [line 918](../../../runtime/object_model.hpp#L918)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::Level& object = /* obtain a valid instance */;

auto result = object.actor_at(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-actor-count-1"></a>

## `epok::Level::actor_count`

**Purpose.** Performs `actor count` as part of the object model module.

**Exact declaration**

```cpp
size_t actor_count() const
```

- **Declared at:** [line 917](../../../runtime/object_model.hpp#L917)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.actor_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-add-component-1"></a>

## `epok::Level::add_component`

**Purpose.** ---- components ----------------------------------------------------------------

**Exact declaration**

```cpp
template<class T> T* add_component(Actor& owner, const char* name = nullptr)
```

- **Declared at:** [line 1125](../../../runtime/object_model.hpp#L1125)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `Actor &` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// Actor & owner
// const char * name

epok::Level& object = /* obtain a valid instance */;

auto result = object.add_component<T>(owner, name);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-bind-1"></a>

## `epok::Level::bind`

**Purpose.** Binds the slot table and registers the Level itself.

**Details.** Also publishes the registry as the process-wide one used by Actor::entity()/ActorComponent::get_owner().

**Exact declaration**

```cpp
bool bind(ObjectRegistry& value)
```

- **Declared at:** [line 908](../../../runtime/object_model.hpp#L908)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ObjectRegistry &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Also publishes the registry as the process-wide one used by Actor::entity()/ActorComponent::get_owner().

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectRegistry & value

epok::Level& object = /* obtain a valid instance */;

auto result = object.bind(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-class-id-1"></a>

## `epok::Level::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 904](../../../runtime/object_model.hpp#L904)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-create-scene-script-1"></a>

## `epok::Level::create_scene_script`

**Purpose.** The scene script is created by the loader; it is never part of the actor table.

**Exact declaration**

```cpp
ObjectId create_scene_script(const ClassDescriptor& type, const char* name)
```

- **Declared at:** [line 1015](../../../runtime/object_model.hpp#L1015)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ClassDescriptor & type
// const char * name

epok::Level& object = /* obtain a valid instance */;

auto result = object.create_scene_script(type, name);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-destroy-actor-1"></a>

## `epok::Level::destroy_actor`

**Purpose.** ---- destruction --------------------------------------------------------------- A request made inside a callback marks the actor immediately (it receives no further events) and runs the teardown when the current batch finishes.

**Exact declaration**

```cpp
bool destroy_actor(ObjectId actor, EndPlayReason reason = EndPlayReason::Destroyed)
```

- **Declared at:** [line 1032](../../../runtime/object_model.hpp#L1032)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `reason` | `EndPlayReason` | Input | Value supplied for `reason`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor
// EndPlayReason reason

epok::Level& object = /* obtain a valid instance */;

auto result = object.destroy_actor(actor, reason);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-level-end-play-all-1"></a>

## `epok::Level::end_play_all`

**Purpose.** Exit order: scene script first while the actors are still alive, then every actor.

**Exact declaration**

```cpp
void end_play_all(EndPlayReason reason)
```

- **Declared at:** [line 1052](../../../runtime/object_model.hpp#L1052)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `reason` | `EndPlayReason` | Input | Value supplied for `reason`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason reason

epok::Level& object = /* obtain a valid instance */;

object.end_play_all(reason);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-frame-update-1"></a>

## `epok::Level::frame_update`

**Purpose.** Once per rendered frame, even while paused (legacy Behaviour parity).

**Exact declaration**

```cpp
void frame_update(uint32_t elapsed)
```

- **Declared at:** [line 1073](../../../runtime/object_model.hpp#L1073)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t elapsed

epok::Level& object = /* obtain a valid instance */;

object.frame_update(elapsed);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-get-component-1"></a>

## `epok::Level::get_component`

**Purpose.** Returns component as part of the object model module.

**Exact declaration**

```cpp
template<class T> T* get_component(const Actor& owner) const
```

- **Declared at:** [line 1143](../../../runtime/object_model.hpp#L1143)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `const Actor &` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// const Actor & owner

epok::Level& object = /* obtain a valid instance */;

auto result = object.get_component<T>(owner);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-get-components-1"></a>

## `epok::Level::get_components`

**Purpose.** Returns components as part of the object model module.

**Exact declaration**

```cpp
template<class T> size_t get_components(const Actor& owner, T** out, size_t capacity) const
```

- **Declared at:** [line 1150](../../../runtime/object_model.hpp#L1150)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `const Actor &` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `out` | `T **` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// const Actor & owner
// T ** out
// size_t capacity

epok::Level& object = /* obtain a valid instance */;

auto result = object.get_components<T>(owner, out, capacity);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-registry-1"></a>

## `epok::Level::registry`

**Purpose.** Performs `registry` as part of the object model module.

**Exact declaration**

```cpp
ObjectRegistry* registry() const
```

- **Declared at:** [line 915](../../../runtime/object_model.hpp#L915)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectRegistry *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.registry();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-remove-component-1"></a>

## `epok::Level::remove_component`

**Purpose.** Removes component as part of the object model module.

**Exact declaration**

```cpp
bool remove_component(Actor& owner, ObjectId component)
```

- **Declared at:** [line 1161](../../../runtime/object_model.hpp#L1161)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `Actor &` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `component` | `ObjectId` | Input | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Actor & owner
// ObjectId component

epok::Level& object = /* obtain a valid instance */;

auto result = object.remove_component(owner, component);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-scene-script-1"></a>

## `epok::Level::scene_script`

**Purpose.** Performs `scene script` as part of the object model module.

**Exact declaration**

```cpp
ObjectId scene_script() const
```

- **Declared at:** [line 919](../../../runtime/object_model.hpp#L919)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.scene_script();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-set-active-1"></a>

## `epok::Level::set_active`

**Purpose.** Sets active as part of the object model module.

**Exact declaration**

```cpp
bool set_active(ObjectId actor, bool active)
```

- **Declared at:** [line 1093](../../../runtime/object_model.hpp#L1093)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor
// bool active

epok::Level& object = /* obtain a valid instance */;

auto result = object.set_active(actor, active);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-level-set-logical-parent-1"></a>

## `epok::Level::set_logical_parent`

**Purpose.** Sets logical parent as part of the object model module.

**Exact declaration**

```cpp
bool set_logical_parent(ObjectId child,ObjectId parent)
```

- **Declared at:** [line 920](../../../runtime/object_model.hpp#L920)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `ObjectId` | Input | Value supplied for `child`. See the exact type and module contract. |
| `parent` | `ObjectId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId child
// ObjectId parent

epok::Level& object = /* obtain a valid instance */;

auto result = object.set_logical_parent(child, parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-level-spawn-actor-1"></a>

## `epok::Level::spawn_actor`

**Purpose.** Performs `spawn actor` as part of the object model module.

**Exact declaration**

```cpp
virtual ObjectId spawn_actor(const ClassDescriptor& type, const char* name, ObjectId logical_parent = {})
```

- **Declared at:** [line 1005](../../../runtime/object_model.hpp#L1005)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `logical_parent` | `ObjectId` | Input | Value supplied for `logical_parent`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ClassDescriptor & type
// const char * name
// ObjectId logical_parent

epok::Level& object = /* obtain a valid instance */;

auto result = object.spawn_actor(type, name, logical_parent);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-spawn-batch-1"></a>

## `epok::Level::spawn_batch`

**Purpose.** All or nothing.

**Details.** On any failure every reservation of this batch is released, so a half-built actor never keeps orphan components.

**Exact declaration**

```cpp
size_t spawn_batch(const ActorSpawnRequest* requests, size_t count, ObjectId* out, ActorPrepareFn prepare = nullptr)
```

- **Declared at:** [line 948](../../../runtime/object_model.hpp#L948)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `requests` | `const ActorSpawnRequest *` | Input | Value supplied for `requests`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `out` | `ObjectId *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `prepare` | `ActorPrepareFn` | Input | Value supplied for `prepare`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** On any failure every reservation of this batch is released, so a half-built actor never keeps orphan components.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ActorSpawnRequest * requests
// size_t count
// ObjectId * out
// ActorPrepareFn prepare

epok::Level& object = /* obtain a valid instance */;

auto result = object.spawn_batch(requests, count, out, prepare);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-stats-1"></a>

## `epok::Level::stats`

**Purpose.** Performs `stats` as part of the object model module.

**Exact declaration**

```cpp
ObjectStats stats() const
```

- **Declared at:** [line 916](../../../runtime/object_model.hpp#L916)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectStats`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.stats();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-tick-1"></a>

## `epok::Level::tick`

**Purpose.** ---- ticking -------------------------------------------------------------------

**Exact declaration**

```cpp
void tick(Fixed delta)
```

- **Declared at:** [line 1063](../../../runtime/object_model.hpp#L1063)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta` | `Fixed` | Input | Value supplied for `delta`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta

epok::Level& object = /* obtain a valid instance */;

object.tick(delta);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-class-id-1"></a>

## `epok::Light3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 740](../../../runtime/object_model.hpp#L740)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Light3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-enabled-1"></a>

## `epok::Light3DComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 742](../../../runtime/object_model.hpp#L742)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Light3DComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-light3dcomponent-entity-slot-1"></a>

## `epok::Light3DComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 741](../../../runtime/object_model.hpp#L741)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Light3DComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-intensity-1"></a>

## `epok::Light3DComponent::intensity`

**Purpose.** Performs `intensity` as part of the object model module.

**Exact declaration**

```cpp
Fixed intensity() const
```

- **Declared at:** [line 744](../../../runtime/object_model.hpp#L744)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Light3DComponent& object = /* obtain a valid instance */;

auto result = object.intensity();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-set-color-1"></a>

## `epok::Light3DComponent::set_color`

**Purpose.** Sets color as part of the object model module.

**Exact declaration**

```cpp
void set_color(uint32_t red,uint32_t green,uint32_t blue)
```

- **Declared at:** [line 747](../../../runtime/object_model.hpp#L747)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::Light3DComponent& object = /* obtain a valid instance */;

object.set_color(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-set-enabled-1"></a>

## `epok::Light3DComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 743](../../../runtime/object_model.hpp#L743)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Light3DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-set-intensity-1"></a>

## `epok::Light3DComponent::set_intensity`

**Purpose.** Sets intensity as part of the object model module.

**Exact declaration**

```cpp
void set_intensity(Fixed value)
```

- **Declared at:** [line 745](../../../runtime/object_model.hpp#L745)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Light3DComponent& object = /* obtain a valid instance */;

object.set_intensity(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-set-mode-1"></a>

## `epok::Light3DComponent::set_mode`

**Purpose.** Sets mode as part of the object model module.

**Exact declaration**

```cpp
void set_mode(LightMode value)
```

- **Declared at:** [line 749](../../../runtime/object_model.hpp#L749)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `LightMode` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// LightMode value

epok::Light3DComponent& object = /* obtain a valid instance */;

object.set_mode(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-set-range-1"></a>

## `epok::Light3DComponent::set_range`

**Purpose.** Sets range as part of the object model module.

**Exact declaration**

```cpp
void set_range(Fixed value)
```

- **Declared at:** [line 746](../../../runtime/object_model.hpp#L746)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Light3DComponent& object = /* obtain a valid instance */;

object.set_range(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-light3dcomponent-set-type-1"></a>

## `epok::Light3DComponent::set_type`

**Purpose.** Sets type as part of the object model module.

**Exact declaration**

```cpp
void set_type(LightType value)
```

- **Declared at:** [line 748](../../../runtime/object_model.hpp#L748)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `LightType` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// LightType value

epok::Light3DComponent& object = /* obtain a valid instance */;

object.set_type(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-bone-count-1"></a>

## `epok::Mesh3DComponent::bone_count`

**Purpose.** Performs `bone count` as part of the object model module.

**Exact declaration**

```cpp
uint32_t bone_count() const
```

- **Declared at:** [line 671](../../../runtime/object_model.hpp#L671)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.bone_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-class-id-1"></a>

## `epok::Mesh3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 666](../../../runtime/object_model.hpp#L666)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-clip-count-1"></a>

## `epok::Mesh3DComponent::clip_count`

**Purpose.** Performs `clip count` as part of the object model module.

**Exact declaration**

```cpp
uint32_t clip_count() const
```

- **Declared at:** [line 674](../../../runtime/object_model.hpp#L674)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.clip_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-entity-slot-1"></a>

## `epok::Mesh3DComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 667](../../../runtime/object_model.hpp#L667)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-geometry-state-1"></a>

## `epok::Mesh3DComponent::geometry_state`

**Purpose.** Performs `geometry state` as part of the object model module.

**Exact declaration**

```cpp
MeshDataState geometry_state() const
```

- **Declared at:** [line 702](../../../runtime/object_model.hpp#L702)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `MeshDataState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.geometry_state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-lighting-enabled-1"></a>

## `epok::Mesh3DComponent::lighting_enabled`

**Purpose.** Performs `lighting enabled` as part of the object model module.

**Exact declaration**

```cpp
bool lighting_enabled() const
```

- **Declared at:** [line 705](../../../runtime/object_model.hpp#L705)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.lighting_enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-mesh3dcomponent-material-state-1"></a>

## `epok::Mesh3DComponent::material_state`

**Purpose.** Performs `material state` as part of the object model module.

**Exact declaration**

```cpp
MaterialSnapshot material_state() const
```

- **Declared at:** [line 691](../../../runtime/object_model.hpp#L691)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `MaterialSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.material_state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-pause-animation-1"></a>

## `epok::Mesh3DComponent::pause_animation`

**Purpose.** Pauses animation as part of the object model module.

**Exact declaration**

```cpp
void pause_animation()
```

- **Declared at:** [line 680](../../../runtime/object_model.hpp#L680)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.pause_animation();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-play-clip-1"></a>

## `epok::Mesh3DComponent::play_clip`

**Purpose.** Starts clip as part of the object model module.

**Exact declaration**

```cpp
bool play_clip(uint32_t clip,bool looping)
```

- **Declared at:** [line 677](../../../runtime/object_model.hpp#L677)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `clip` | `uint32_t` | Input | Value supplied for `clip`. See the exact type and module contract. |
| `looping` | `bool` | Input | Value supplied for `looping`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t clip
// bool looping

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.play_clip(clip, looping);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-mesh3dcomponent-playback-state-1"></a>

## `epok::Mesh3DComponent::playback_state`

**Purpose.** Starts back state as part of the object model module.

**Exact declaration**

```cpp
SkeletalPlaybackState playback_state() const
```

- **Declared at:** [line 683](../../../runtime/object_model.hpp#L683)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `SkeletalPlaybackState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.playback_state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-quad-count-1"></a>

## `epok::Mesh3DComponent::quad_count`

**Purpose.** Performs `quad count` as part of the object model module.

**Exact declaration**

```cpp
uint32_t quad_count() const
```

- **Declared at:** [line 700](../../../runtime/object_model.hpp#L700)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.quad_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-request-geometry-1"></a>

## `epok::Mesh3DComponent::request_geometry`

**Purpose.** Requests geometry as part of the object model module.

**Exact declaration**

```cpp
bool request_geometry()
```

- **Declared at:** [line 703](../../../runtime/object_model.hpp#L703)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.request_geometry();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-mesh3dcomponent-resume-animation-1"></a>

## `epok::Mesh3DComponent::resume_animation`

**Purpose.** Resumes animation as part of the object model module.

**Exact declaration**

```cpp
void resume_animation()
```

- **Declared at:** [line 681](../../../runtime/object_model.hpp#L681)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.resume_animation();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-sample-bone-1"></a>

## `epok::Mesh3DComponent::sample_bone`

**Purpose.** Performs `sample bone` as part of the object model module.

**Exact declaration**

```cpp
BoneSample sample_bone(uint32_t bone,PoseKind pose,CoordinateSpace space)
```

- **Declared at:** [line 690](../../../runtime/object_model.hpp#L690)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bone` | `uint32_t` | Input | Value supplied for `bone`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `BoneSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t bone
// PoseKind pose
// CoordinateSpace space

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.sample_bone(bone, pose, space);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-sample-geometry-vertex-1"></a>

## `epok::Mesh3DComponent::sample_geometry_vertex`

**Purpose.** Performs `sample geometry vertex` as part of the object model module.

**Exact declaration**

```cpp
MeshVertexSample sample_geometry_vertex(uint32_t vertex,CoordinateSpace space)
```

- **Declared at:** [line 704](../../../runtime/object_model.hpp#L704)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertex` | `uint32_t` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `MeshVertexSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t vertex
// CoordinateSpace space

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.sample_geometry_vertex(vertex, space);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-sample-vertex-1"></a>

## `epok::Mesh3DComponent::sample_vertex`

**Purpose.** Performs `sample vertex` as part of the object model module.

**Exact declaration**

```cpp
VertexSample sample_vertex(uint32_t vertex,PoseKind pose,CoordinateSpace space)
```

- **Declared at:** [line 688](../../../runtime/object_model.hpp#L688)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertex` | `uint32_t` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `VertexSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t vertex
// PoseKind pose
// CoordinateSpace space

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.sample_vertex(vertex, pose, space);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-sample-vertices-1"></a>

## `epok::Mesh3DComponent::sample_vertices`

**Purpose.** Performs `sample vertices` as part of the object model module.

**Exact declaration**

```cpp
VertexSamples4 sample_vertices(VertexIndexBatch4 indices,PoseKind pose,CoordinateSpace space)
```

- **Declared at:** [line 689](../../../runtime/object_model.hpp#L689)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `indices` | `VertexIndexBatch4` | Input | Value supplied for `indices`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `VertexSamples4`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// VertexIndexBatch4 indices
// PoseKind pose
// CoordinateSpace space

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.sample_vertices(indices, pose, space);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-set-lighting-enabled-1"></a>

## `epok::Mesh3DComponent::set_lighting_enabled`

**Purpose.** Sets lighting enabled as part of the object model module.

**Exact declaration**

```cpp
void set_lighting_enabled(bool value)
```

- **Declared at:** [line 706](../../../runtime/object_model.hpp#L706)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.set_lighting_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-set-material-blend-1"></a>

## `epok::Mesh3DComponent::set_material_blend`

**Purpose.** Sets material blend as part of the object model module.

**Exact declaration**

```cpp
void set_material_blend(BlendMode blend)
```

- **Declared at:** [line 697](../../../runtime/object_model.hpp#L697)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `blend` | `BlendMode` | Input | Value supplied for `blend`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// BlendMode blend

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.set_material_blend(blend);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-set-material-color-1"></a>

## `epok::Mesh3DComponent::set_material_color`

**Purpose.** Sets material color as part of the object model module.

**Exact declaration**

```cpp
void set_material_color(uint32_t red,uint32_t green,uint32_t blue)
```

- **Declared at:** [line 694](../../../runtime/object_model.hpp#L694)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.set_material_color(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-set-material-depth-bias-1"></a>

## `epok::Mesh3DComponent::set_material_depth_bias`

**Purpose.** Sets material depth bias as part of the object model module.

**Exact declaration**

```cpp
void set_material_depth_bias(int32_t value)
```

- **Declared at:** [line 698](../../../runtime/object_model.hpp#L698)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.set_material_depth_bias(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-set-material-texture-1"></a>

## `epok::Mesh3DComponent::set_material_texture`

**Purpose.** Sets material texture as part of the object model module.

**Exact declaration**

```cpp
void set_material_texture(int32_t texture)
```

- **Declared at:** [line 695](../../../runtime/object_model.hpp#L695)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `texture` | `int32_t` | Input | Value supplied for `texture`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// int32_t texture

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.set_material_texture(texture);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-set-material-unlit-1"></a>

## `epok::Mesh3DComponent::set_material_unlit`

**Purpose.** Sets material unlit as part of the object model module.

**Exact declaration**

```cpp
void set_material_unlit(bool unlit)
```

- **Declared at:** [line 696](../../../runtime/object_model.hpp#L696)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `unlit` | `bool` | Input | Value supplied for `unlit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool unlit

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.set_material_unlit(unlit);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-set-uv-scroll-1"></a>

## `epok::Mesh3DComponent::set_uv_scroll`

**Purpose.** Sets uv scroll as part of the object model module.

**Exact declaration**

```cpp
void set_uv_scroll(Fixed x,Fixed y)
```

- **Declared at:** [line 699](../../../runtime/object_model.hpp#L699)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.set_uv_scroll(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-stop-animation-1"></a>

## `epok::Mesh3DComponent::stop_animation`

**Purpose.** Stops animation as part of the object model module.

**Exact declaration**

```cpp
void stop_animation()
```

- **Declared at:** [line 682](../../../runtime/object_model.hpp#L682)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

object.stop_animation();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-streamed-1"></a>

## `epok::Mesh3DComponent::streamed`

**Purpose.** Performs `streamed` as part of the object model module.

**Exact declaration**

```cpp
bool streamed() const
```

- **Declared at:** [line 701](../../../runtime/object_model.hpp#L701)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.streamed();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-mesh3dcomponent-vertex-count-1"></a>

## `epok::Mesh3DComponent::vertex_count`

**Purpose.** Performs `vertex count` as part of the object model module.

**Exact declaration**

```cpp
uint32_t vertex_count() const
```

- **Declared at:** [line 668](../../../runtime/object_model.hpp#L668)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.vertex_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-class-id-1"></a>

## `epok::Object::class_id`

**Purpose.** Runtime-owned identity hooks; not an authoring API.

**Exact declaration**

```cpp
virtual uint64_t class_id() const
```

- **Declared at:** [line 189](../../../runtime/object_model.hpp#L189)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Object& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-id-1"></a>

## `epok::Object::id`

**Purpose.** Performs `id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId id() const
```

- **Declared at:** [line 191](../../../runtime/object_model.hpp#L191)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Object& object = /* obtain a valid instance */;

auto result = object.id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-is-a-1"></a>

## `epok::Object::is_a`

**Purpose.** Reports whether a as part of the object model module.

**Exact declaration**

```cpp
bool is_a(uint64_t parent) const
```

- **Declared at:** [line 193](../../../runtime/object_model.hpp#L193)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parent` | `uint64_t` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t parent

epok::Object& object = /* obtain a valid instance */;

auto result = object.is_a(parent);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-object-state-1"></a>

## `epok::Object::state`

**Purpose.** Performs `state` as part of the object model module.

**Exact declaration**

```cpp
ObjectState state() const
```

- **Declared at:** [line 192](../../../runtime/object_model.hpp#L192)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Object& object = /* obtain a valid instance */;

auto result = object.state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-timeline-sync-1"></a>

## `epok::Object::timeline_sync`

**Purpose.** Performs `timeline sync` as part of the object model module.

**Exact declaration**

```cpp
virtual void timeline_sync(uint64_t, bool)
```

- **Declared at:** [line 190](../../../runtime/object_model.hpp#L190)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint64_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `bool` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t arg1
// bool arg2

epok::Object& object = /* obtain a valid instance */;

object.timeline_sync(arg1, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-object-1"></a>

## `epok::Object::~Object`

**Purpose.** Releases the resources owned by `epok::Object`.

**Exact declaration**

```cpp
virtual ~Object() = default
```

- **Declared at:** [line 187](../../../runtime/object_model.hpp#L187)
- **Kind:** `destructor`; qualifiers: `virtual`

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// `epok::Object` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-class-is-a-1"></a>

## `epok::object_class_is_a`

**Purpose.** Ancestry walk bounded by the table size; the cook rejects cycles before emitting.

**Exact declaration**

```cpp
inline bool object_class_is_a(uint64_t child, uint64_t parent)
```

- **Declared at:** [line 173](../../../runtime/object_model.hpp#L173)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `uint64_t` | Input | Value supplied for `child`. See the exact type and module contract. |
| `parent` | `uint64_t` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t child
// uint64_t parent

auto result = epok::object_class_is_a(child, parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-object-construct-1"></a>

## `epok::object_construct`

**Purpose.** Placement-new into caller storage; destroy runs the virtual destructor of the concrete type, so a base pointer never slices.

**Exact declaration**

```cpp
template<class T> Object* object_construct(void* storage)
```

- **Declared at:** [line 204](../../../runtime/object_model.hpp#L204)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `storage` | `void *` | Input/output; inspect the function contract | Value supplied for `storage`. See the exact type and module contract. |

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// void * storage

auto result = epok::object_construct<T>(storage);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-object-destruct-1"></a>

## `epok::object_destruct`

**Purpose.** Performs `object destruct` as part of the object model module.

**Exact declaration**

```cpp
inline void object_destruct(Object* instance)
```

- **Declared at:** [line 205](../../../runtime/object_model.hpp#L205)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Object *` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object * instance

epok::object_destruct(instance);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-object-domain-bit-1"></a>

## `epok::object_domain_bit`

**Purpose.** Owner domain masks for components.

**Details.** None never appears in an owners mask.

**Exact declaration**

```cpp
constexpr uint8_t object_domain_bit(ObjectDomain domain)
```

- **Declared at:** [line 78](../../../runtime/object_model.hpp#L78)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `domain` | `ObjectDomain` | Input | Value supplied for `domain`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** None never appears in an owners mask.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectDomain domain

auto result = epok::object_domain_bit(domain);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-storage-quarantined-1"></a>

## `epok::object_storage_quarantined`

**Purpose.** Defined after AudioComponent; `false` for every object that owns no retained storage.

**Exact declaration**

```cpp
inline bool object_storage_quarantined(Object* instance)
```

- **Declared at:** [line 265](../../../runtime/object_model.hpp#L265)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Object *` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object * instance

auto result = epok::object_storage_quarantined(instance);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectdispatchscope-objectdispatchscope-1"></a>

## `epok::ObjectDispatchScope::ObjectDispatchScope`

**Purpose.** Constructs `epok::ObjectDispatchScope` for the object model module.

**Exact declaration**

```cpp
ObjectDispatchScope(const ObjectDispatchScope&) = delete
```

- **Declared at:** [line 409](../../../runtime/object_model.hpp#L409)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ObjectDispatchScope &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectDispatchScope & arg1

epok::ObjectDispatchScope value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectdispatchscope-objectdispatchscope-2"></a>

## `epok::ObjectDispatchScope::ObjectDispatchScope`

**Purpose.** Constructs `epok::ObjectDispatchScope` for the object model module.

**Exact declaration**

```cpp
explicit ObjectDispatchScope(ObjectRegistry& value) : r
```

- **Declared at:** [line 407](../../../runtime/object_model.hpp#L407)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ObjectRegistry &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectRegistry & value

epok::ObjectDispatchScope value(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectdispatchscope-operator-1"></a>

## `epok::ObjectDispatchScope::operator=`

**Purpose.** Performs `operator =` as part of the object model module.

**Exact declaration**

```cpp
ObjectDispatchScope& operator=(const ObjectDispatchScope&) = delete
```

- **Declared at:** [line 410](../../../runtime/object_model.hpp#L410)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ObjectDispatchScope &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `ObjectDispatchScope &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectDispatchScope & arg1

epok::ObjectDispatchScope& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectdispatchscope-objectdispatchscope-3"></a>

## `epok::ObjectDispatchScope::~ObjectDispatchScope`

**Purpose.** Releases the resources owned by `epok::ObjectDispatchScope`.

**Exact declaration**

```cpp
~ObjectDispatchScope()
```

- **Declared at:** [line 408](../../../runtime/object_model.hpp#L408)
- **Kind:** `destructor`

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// `epok::ObjectDispatchScope` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectid-get-1"></a>

## `epok::ObjectId::get`

**Purpose.** Returns get as part of the object model module.

**Exact declaration**

```cpp
Object* get() const
```

- **Declared at:** [line 45](../../../runtime/object_model.hpp#L45)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectId& object = /* obtain a valid instance */;

auto result = object.get();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectid-operator-1"></a>

## `epok::ObjectId::operator!=`

**Purpose.** Performs `operator !=` as part of the object model module.

**Exact declaration**

```cpp
constexpr bool operator!=(const ObjectId& other) const
```

- **Declared at:** [line 49](../../../runtime/object_model.hpp#L49)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const ObjectId &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectId & other

epok::ObjectId& object = /* obtain a valid instance */;

auto result = object.operator!=(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectid-operator-2"></a>

## `epok::ObjectId::operator==`

**Purpose.** Performs `operator ==` as part of the object model module.

**Exact declaration**

```cpp
constexpr bool operator==(const ObjectId& other) const
```

- **Declared at:** [line 46](../../../runtime/object_model.hpp#L46)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const ObjectId &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectId & other

epok::ObjectId& object = /* obtain a valid instance */;

auto result = object.operator==(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectid-valid-1"></a>

## `epok::ObjectId::valid`

**Purpose.** Performs `valid` as part of the object model module.

**Exact declaration**

```cpp
constexpr bool valid() const
```

- **Declared at:** [line 44](../../../runtime/object_model.hpp#L44)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectId& object = /* obtain a valid instance */;

auto result = object.valid();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-objectpool-acquire-1"></a>

## `epok::ObjectPool::acquire`

**Purpose.** Performs `acquire` as part of the object model module.

**Exact declaration**

```cpp
static Object* acquire()
```

- **Declared at:** [line 216](../../../runtime/object_model.hpp#L216)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

auto result = epok::ObjectPool::acquire();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectpool-live-1"></a>

## `epok::ObjectPool::live`

**Purpose.** Performs `live` as part of the object model module.

**Exact declaration**

```cpp
static size_t live()
```

- **Declared at:** [line 232](../../../runtime/object_model.hpp#L232)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

auto result = epok::ObjectPool::live();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectpool-release-1"></a>

## `epok::ObjectPool::release`

**Purpose.** Performs `release` as part of the object model module.

**Exact declaration**

```cpp
static void release(Object* instance)
```

- **Declared at:** [line 224](../../../runtime/object_model.hpp#L224)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Object *` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object * instance

epok::ObjectPool::release(instance);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-acquire-1"></a>

## `epok::ObjectRegistry::acquire`

**Purpose.** Pool-backed construction of a concrete class.

**Details.** Abstract classes are never created.

**Exact declaration**

```cpp
ObjectId acquire(const ClassDescriptor& type)
```

- **Declared at:** [line 318](../../../runtime/object_model.hpp#L318)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** Abstract classes are never created.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ClassDescriptor & type

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.acquire(type);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-adopt-1"></a>

## `epok::ObjectRegistry::adopt`

**Purpose.** Adopt storage owned by somebody else (an actor's embedded default component, the Level itself).

**Details.** Release never destroys it.

**Exact declaration**

```cpp
ObjectId adopt(Object& instance, const ClassDescriptor& type)
```

- **Declared at:** [line 330](../../../runtime/object_model.hpp#L330)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Object &` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** Release never destroys it.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object & instance
// const ClassDescriptor & type

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.adopt(instance, type);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-class-of-1"></a>

## `epok::ObjectRegistry::class_of`

**Purpose.** Performs `class of` as part of the object model module.

**Exact declaration**

```cpp
const ClassDescriptor* class_of(ObjectId id)
```

- **Declared at:** [line 312](../../../runtime/object_model.hpp#L312)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const ClassDescriptor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.class_of(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-collect-quarantined-1"></a>

## `epok::ObjectRegistry::collect_quarantined`

**Purpose.** Retry the quarantined slots.

**Details.** The scene bank calls it once per frame, next to the point where the legacy path re-checks music_active before reusing a slot.

**Exact declaration**

```cpp
void collect_quarantined()
```

- **Declared at:** [line 374](../../../runtime/object_model.hpp#L374)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The scene bank calls it once per frame, next to the point where the legacy path re-checks music_active before reusing a slot.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistry& object = /* obtain a valid instance */;

object.collect_quarantined();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-finish-release-1"></a>

## `epok::ObjectRegistry::finish_release`

**Purpose.** Called when the last dispatch scope unwinds: no callback can still be on the stack of a destroyed object, so its typed storage returns to the pool.

**Details.** Storage a service still points at stays quarantined; the handle is already dead either way, so the only effect is that the slot and the pool entry are not reused yet. Call again (`collect_quarantined`) once the service releases it.

**Exact declaration**

```cpp
void finish_release()
```

- **Declared at:** [line 355](../../../runtime/object_model.hpp#L355)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Storage a service still points at stays quarantined; the handle is already dead either way, so the only effect is that the slot and the pool entry are not reused yet. Call again (`collect_quarantined`) once the service releases it.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistry& object = /* obtain a valid instance */;

object.finish_release();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-get-1"></a>

## `epok::ObjectRegistry::get`

**Purpose.** Returns get as part of the object model module.

**Exact declaration**

```cpp
Object* get(ObjectId id)
```

- **Declared at:** [line 289](../../../runtime/object_model.hpp#L289)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.get(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-live-1"></a>

## `epok::ObjectRegistry::live`

**Purpose.** Performs `live` as part of the object model module.

**Exact declaration**

```cpp
size_t live() const
```

- **Declared at:** [line 375](../../../runtime/object_model.hpp#L375)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.live();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-next-generation-1"></a>

## `epok::ObjectRegistry::next_generation`

**Purpose.** Performs `next generation` as part of the object model module.

**Exact declaration**

```cpp
static uint16_t next_generation(uint16_t value)
```

- **Declared at:** [line 281](../../../runtime/object_model.hpp#L281)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint16_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t value

auto result = epok::ObjectRegistry::next_generation(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-objectregistry-1"></a>

## `epok::ObjectRegistry::ObjectRegistry`

**Purpose.** Constructs `epok::ObjectRegistry` for the object model module.

**Exact declaration**

```cpp
ObjectRegistry(const ObjectRegistry&) = delete
```

- **Declared at:** [line 278](../../../runtime/object_model.hpp#L278)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ObjectRegistry &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectRegistry & arg1

epok::ObjectRegistry value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-objectregistry-2"></a>

## `epok::ObjectRegistry::ObjectRegistry`

**Purpose.** Constructs `epok::ObjectRegistry` for the object model module.

**Exact declaration**

```cpp
constexpr ObjectRegistry(ObjectSlot* table, uint16_t count) : slots(table), capacity(count)
```

- **Declared at:** [line 277](../../../runtime/object_model.hpp#L277)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `ObjectSlot *` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |
| `count` | `uint16_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectSlot * table
// uint16_t count

epok::ObjectRegistry value(table, count);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-operator-1"></a>

## `epok::ObjectRegistry::operator=`

**Purpose.** Performs `operator =` as part of the object model module.

**Exact declaration**

```cpp
ObjectRegistry& operator=(const ObjectRegistry&) = delete
```

- **Declared at:** [line 279](../../../runtime/object_model.hpp#L279)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ObjectRegistry &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `ObjectRegistry &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectRegistry & arg1

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-release-1"></a>

## `epok::ObjectRegistry::release`

**Purpose.** Invalidates the handle immediately; storage is quarantined while callbacks run.

**Exact declaration**

```cpp
bool release(ObjectId id)
```

- **Declared at:** [line 338](../../../runtime/object_model.hpp#L338)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.release(id);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-objectregistry-resolve-1"></a>

## `epok::ObjectRegistry::resolve`

**Purpose.** Generation and class checked; the runtime class must derive from T's static class.

**Exact declaration**

```cpp
template<class T> T* resolve(ObjectId id)
```

- **Declared at:** [line 294](../../../runtime/object_model.hpp#L294)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.resolve<T>(id);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-objectregistry-resolve-2"></a>

## `epok::ObjectRegistry::resolve`

**Purpose.** Performs `resolve` as part of the object model module.

**Exact declaration**

```cpp
template<class T> const T* resolve(ObjectId id) const
```

- **Declared at:** [line 311](../../../runtime/object_model.hpp#L311)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.resolve<T>(id);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-objectregistry-slot-1"></a>

## `epok::ObjectRegistry::slot`

**Purpose.** Performs `slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectSlot* slot(ObjectId id)
```

- **Declared at:** [line 282](../../../runtime/object_model.hpp#L282)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `ObjectSlot *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.slot(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-slot-2"></a>

## `epok::ObjectRegistry::slot`

**Purpose.** Performs `slot` as part of the object model module.

**Exact declaration**

```cpp
const ObjectSlot* slot(ObjectId id) const
```

- **Declared at:** [line 288](../../../runtime/object_model.hpp#L288)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const ObjectSlot *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.slot(id);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistrystorage-objectregistrystorage-capacity-1"></a>

## `epok::ObjectRegistryStorage::ObjectRegistryStorage<Capacity>`

**Purpose.** Constructs `epok::ObjectRegistryStorage` for the object model module.

**Exact declaration**

```cpp
ObjectRegistryStorage() : O
```

- **Declared at:** [line 402](../../../runtime/object_model.hpp#L402)
- **Kind:** `constructor`

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistryStorage value();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimatorcomponent-class-id-1"></a>

## `epok::PaletteAnimatorComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 839](../../../runtime/object_model.hpp#L839)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::PaletteAnimatorComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimatorcomponent-configure-1"></a>

## `epok::PaletteAnimatorComponent::configure`

**Purpose.** Performs `configure` as part of the object model module.

**Exact declaration**

```cpp
void configure(int32_t texture,uint32_t first,uint32_t last,Fixed speed,bool reverse)
```

- **Declared at:** [line 842](../../../runtime/object_model.hpp#L842)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `texture` | `int32_t` | Input | Value supplied for `texture`. See the exact type and module contract. |
| `first` | `uint32_t` | Input | Value supplied for `first`. See the exact type and module contract. |
| `last` | `uint32_t` | Input | Value supplied for `last`. See the exact type and module contract. |
| `speed` | `Fixed` | Input | Value supplied for `speed`. See the exact type and module contract. |
| `reverse` | `bool` | Input | Value supplied for `reverse`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// int32_t texture
// uint32_t first
// uint32_t last
// Fixed speed
// bool reverse

epok::PaletteAnimatorComponent& object = /* obtain a valid instance */;

object.configure(texture, first, last, speed, reverse);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimatorcomponent-entity-slot-1"></a>

## `epok::PaletteAnimatorComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 840](../../../runtime/object_model.hpp#L840)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::PaletteAnimatorComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimatorcomponent-reset-1"></a>

## `epok::PaletteAnimatorComponent::reset`

**Purpose.** Resets reset as part of the object model module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 844](../../../runtime/object_model.hpp#L844)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::PaletteAnimatorComponent& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimatorcomponent-set-enabled-1"></a>

## `epok::PaletteAnimatorComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool enabled)
```

- **Declared at:** [line 843](../../../runtime/object_model.hpp#L843)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `enabled` | `bool` | Input | Value supplied for `enabled`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool enabled

epok::PaletteAnimatorComponent& object = /* obtain a valid instance */;

object.set_enabled(enabled);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimatorcomponent-state-1"></a>

## `epok::PaletteAnimatorComponent::state`

**Purpose.** Performs `state` as part of the object model module.

**Exact declaration**

```cpp
PaletteAnimationState state() const
```

- **Declared at:** [line 841](../../../runtime/object_model.hpp#L841)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `PaletteAnimationState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::PaletteAnimatorComponent& object = /* obtain a valid instance */;

auto result = object.state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleeffectcomponent-class-id-1"></a>

## `epok::ParticleEffectComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 834](../../../runtime/object_model.hpp#L834)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ParticleEffectComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-burst-1"></a>

## `epok::ParticleEmitterComponent::burst`

**Purpose.** Performs `burst` as part of the object model module.

**Exact declaration**

```cpp
void burst(uint32_t count)
```

- **Declared at:** [line 820](../../../runtime/object_model.hpp#L820)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t count

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

object.burst(count);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-class-id-1"></a>

## `epok::ParticleEmitterComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 815](../../../runtime/object_model.hpp#L815)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-entity-slot-1"></a>

## `epok::ParticleEmitterComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 816](../../../runtime/object_model.hpp#L816)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-play-1"></a>

## `epok::ParticleEmitterComponent::play`

**Purpose.** Starts play as part of the object model module.

**Exact declaration**

```cpp
void play()
```

- **Declared at:** [line 818](../../../runtime/object_model.hpp#L818)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

object.play();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-set-enabled-1"></a>

## `epok::ParticleEmitterComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 821](../../../runtime/object_model.hpp#L821)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-set-lifetime-1"></a>

## `epok::ParticleEmitterComponent::set_lifetime`

**Purpose.** Sets lifetime as part of the object model module.

**Exact declaration**

```cpp
void set_lifetime(Fixed value)
```

- **Declared at:** [line 823](../../../runtime/object_model.hpp#L823)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

object.set_lifetime(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-set-max-particles-1"></a>

## `epok::ParticleEmitterComponent::set_max_particles`

**Purpose.** Sets max particles as part of the object model module.

**Exact declaration**

```cpp
void set_max_particles(uint32_t value)
```

- **Declared at:** [line 824](../../../runtime/object_model.hpp#L824)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

object.set_max_particles(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-set-rate-1"></a>

## `epok::ParticleEmitterComponent::set_rate`

**Purpose.** Sets rate as part of the object model module.

**Exact declaration**

```cpp
void set_rate(Fixed value)
```

- **Declared at:** [line 822](../../../runtime/object_model.hpp#L822)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

object.set_rate(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-state-1"></a>

## `epok::ParticleEmitterComponent::state`

**Purpose.** Performs `state` as part of the object model module.

**Exact declaration**

```cpp
ParticleEmitterState state() const
```

- **Declared at:** [line 817](../../../runtime/object_model.hpp#L817)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ParticleEmitterState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

auto result = object.state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-stop-1"></a>

## `epok::ParticleEmitterComponent::stop`

**Purpose.** Stops stop as part of the object model module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 819](../../../runtime/object_model.hpp#L819)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-progressbarcomponent-class-id-1"></a>

## `epok::ProgressBarComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 804](../../../runtime/object_model.hpp#L804)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ProgressBarComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-progressbarcomponent-enabled-1"></a>

## `epok::ProgressBarComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 806](../../../runtime/object_model.hpp#L806)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ProgressBarComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-progressbarcomponent-entity-slot-1"></a>

## `epok::ProgressBarComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 805](../../../runtime/object_model.hpp#L805)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ProgressBarComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-progressbarcomponent-set-colors-1"></a>

## `epok::ProgressBarComponent::set_colors`

**Purpose.** Sets colors as part of the object model module.

**Exact declaration**

```cpp
void set_colors(uint32_t red,uint32_t green,uint32_t blue,uint32_t background_red,uint32_t background_green,uint32_t background_blue)
```

- **Declared at:** [line 810](../../../runtime/object_model.hpp#L810)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |
| `background_red` | `uint32_t` | Input | Value supplied for `background_red`. See the exact type and module contract. |
| `background_green` | `uint32_t` | Input | Value supplied for `background_green`. See the exact type and module contract. |
| `background_blue` | `uint32_t` | Input | Value supplied for `background_blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue
// uint32_t background_red
// uint32_t background_green
// uint32_t background_blue

epok::ProgressBarComponent& object = /* obtain a valid instance */;

object.set_colors(red, green, blue, background_red, background_green, background_blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-progressbarcomponent-set-enabled-1"></a>

## `epok::ProgressBarComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 807](../../../runtime/object_model.hpp#L807)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::ProgressBarComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-progressbarcomponent-set-value-1"></a>

## `epok::ProgressBarComponent::set_value`

**Purpose.** Sets value as part of the object model module.

**Exact declaration**

```cpp
void set_value(Fixed value)
```

- **Declared at:** [line 809](../../../runtime/object_model.hpp#L809)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::ProgressBarComponent& object = /* obtain a valid instance */;

object.set_value(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-progressbarcomponent-value-1"></a>

## `epok::ProgressBarComponent::value`

**Purpose.** Performs `value` as part of the object model module.

**Exact declaration**

```cpp
Fixed value() const
```

- **Declared at:** [line 808](../../../runtime/object_model.hpp#L808)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ProgressBarComponent& object = /* obtain a valid instance */;

auto result = object.value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-attach-slot-1"></a>

## `epok::RectTransformComponent::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectId* attach_slot() override
```

- **Declared at:** [line 589](../../../runtime/object_model.hpp#L589)
- **Kind:** `cxx method`

**Returns.** Returns `ObjectId *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

auto result = object.attach_slot();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-bind-local-1"></a>

## `epok::RectTransformComponent::bind_local`

**Purpose.** Performs `bind local` as part of the object model module.

**Exact declaration**

```cpp
void bind_local()
```

- **Declared at:** [line 593](../../../runtime/object_model.hpp#L593)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.bind_local();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-bind-slot-1"></a>

## `epok::RectTransformComponent::bind_slot`

**Purpose.** Performs `bind slot` as part of the object model module.

**Exact declaration**

```cpp
void bind_slot(ActorData& value)
```

- **Declared at:** [line 592](../../../runtime/object_model.hpp#L592)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & value

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.bind_slot(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-recttransformcomponent-class-id-1"></a>

## `epok::RectTransformComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 588](../../../runtime/object_model.hpp#L588)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-enabled-1"></a>

## `epok::RectTransformComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 596](../../../runtime/object_model.hpp#L596)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-recttransformcomponent-entity-slot-1"></a>

## `epok::RectTransformComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 594](../../../runtime/object_model.hpp#L594)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-set-anchors-1"></a>

## `epok::RectTransformComponent::set_anchors`

**Purpose.** Sets anchors as part of the object model module.

**Exact declaration**

```cpp
void set_anchors(Fixed min_x,Fixed min_y,Fixed max_x,Fixed max_y)
```

- **Declared at:** [line 600](../../../runtime/object_model.hpp#L600)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `min_x` | `Fixed` | Input | Value supplied for `min_x`. See the exact type and module contract. |
| `min_y` | `Fixed` | Input | Value supplied for `min_y`. See the exact type and module contract. |
| `max_x` | `Fixed` | Input | Value supplied for `max_x`. See the exact type and module contract. |
| `max_y` | `Fixed` | Input | Value supplied for `max_y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed min_x
// Fixed min_y
// Fixed max_x
// Fixed max_y

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.set_anchors(min_x, min_y, max_x, max_y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-set-enabled-1"></a>

## `epok::RectTransformComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 597](../../../runtime/object_model.hpp#L597)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-set-pivot-1"></a>

## `epok::RectTransformComponent::set_pivot`

**Purpose.** Sets pivot as part of the object model module.

**Exact declaration**

```cpp
void set_pivot(Fixed x,Fixed y)
```

- **Declared at:** [line 601](../../../runtime/object_model.hpp#L601)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.set_pivot(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-set-position-1"></a>

## `epok::RectTransformComponent::set_position`

**Purpose.** Sets position as part of the object model module.

**Exact declaration**

```cpp
void set_position(Fixed x,Fixed y)
```

- **Declared at:** [line 598](../../../runtime/object_model.hpp#L598)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.set_position(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-set-size-1"></a>

## `epok::RectTransformComponent::set_size`

**Purpose.** Sets size as part of the object model module.

**Exact declaration**

```cpp
void set_size(Fixed x,Fixed y)
```

- **Declared at:** [line 599](../../../runtime/object_model.hpp#L599)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.set_size(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-attach-slot-1"></a>

## `epok::SceneComponent2D::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectId* attach_slot() override
```

- **Declared at:** [line 567](../../../runtime/object_model.hpp#L567)
- **Kind:** `cxx method`

**Returns.** Returns `ObjectId *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent2D& object = /* obtain a valid instance */;

auto result = object.attach_slot();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-attach-to-1"></a>

## `epok::SceneComponent2D::attach_to`

**Purpose.** Performs `attach to` as part of the object model module.

**Exact declaration**

```cpp
bool attach_to(ObjectId parent)
```

- **Declared at:** [line 578](../../../runtime/object_model.hpp#L578)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parent` | `ObjectId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId parent

epok::SceneComponent2D& object = /* obtain a valid instance */;

auto result = object.attach_to(parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scenecomponent2d-class-id-1"></a>

## `epok::SceneComponent2D::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 566](../../../runtime/object_model.hpp#L566)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-position-x-1"></a>

## `epok::SceneComponent2D::position_x`

**Purpose.** Performs `position x` as part of the object model module.

**Exact declaration**

```cpp
Fixed position_x() const
```

- **Declared at:** [line 575](../../../runtime/object_model.hpp#L575)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent2D& object = /* obtain a valid instance */;

auto result = object.position_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-position-y-1"></a>

## `epok::SceneComponent2D::position_y`

**Purpose.** Performs `position y` as part of the object model module.

**Exact declaration**

```cpp
Fixed position_y() const
```

- **Declared at:** [line 576](../../../runtime/object_model.hpp#L576)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent2D& object = /* obtain a valid instance */;

auto result = object.position_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-rotation-1"></a>

## `epok::SceneComponent2D::rotation`

**Purpose.** Performs `rotation` as part of the object model module.

**Exact declaration**

```cpp
Fixed rotation() const
```

- **Declared at:** [line 577](../../../runtime/object_model.hpp#L577)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent2D& object = /* obtain a valid instance */;

auto result = object.rotation();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-set-position-1"></a>

## `epok::SceneComponent2D::set_position`

**Purpose.** Sets position as part of the object model module.

**Exact declaration**

```cpp
void set_position(Fixed x,Fixed y)
```

- **Declared at:** [line 572](../../../runtime/object_model.hpp#L572)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::SceneComponent2D& object = /* obtain a valid instance */;

object.set_position(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-set-rotation-1"></a>

## `epok::SceneComponent2D::set_rotation`

**Purpose.** Sets rotation as part of the object model module.

**Exact declaration**

```cpp
void set_rotation(Fixed value)
```

- **Declared at:** [line 573](../../../runtime/object_model.hpp#L573)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::SceneComponent2D& object = /* obtain a valid instance */;

object.set_rotation(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-set-scale-1"></a>

## `epok::SceneComponent2D::set_scale`

**Purpose.** Sets scale as part of the object model module.

**Exact declaration**

```cpp
void set_scale(Fixed x,Fixed y)
```

- **Declared at:** [line 574](../../../runtime/object_model.hpp#L574)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::SceneComponent2D& object = /* obtain a valid instance */;

object.set_scale(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-attach-slot-1"></a>

## `epok::SceneComponent3D::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectId* attach_slot() override
```

- **Declared at:** [line 541](../../../runtime/object_model.hpp#L541)
- **Kind:** `cxx method`

**Returns.** Returns `ObjectId *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.attach_slot();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-attach-to-1"></a>

## `epok::SceneComponent3D::attach_to`

**Purpose.** Performs `attach to` as part of the object model module.

**Exact declaration**

```cpp
bool attach_to(ObjectId parent)
```

- **Declared at:** [line 558](../../../runtime/object_model.hpp#L558)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parent` | `ObjectId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId parent

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.attach_to(parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scenecomponent3d-bind-local-1"></a>

## `epok::SceneComponent3D::bind_local`

**Purpose.** No legacy slot: the component owns the transform.

**Exact declaration**

```cpp
void bind_local()
```

- **Declared at:** [line 549](../../../runtime/object_model.hpp#L549)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.bind_local();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-bind-slot-1"></a>

## `epok::SceneComponent3D::bind_slot`

**Purpose.** Canonical storage: the legacy entity slot owns the transform and everything that already reads it (collision, rendering, motion interpolation) keeps working.

**Exact declaration**

```cpp
void bind_slot(ActorData& value)
```

- **Declared at:** [line 547](../../../runtime/object_model.hpp#L547)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & value

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.bind_slot(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenecomponent3d-class-id-1"></a>

## `epok::SceneComponent3D::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 540](../../../runtime/object_model.hpp#L540)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-entity-slot-1"></a>

## `epok::SceneComponent3D::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 550](../../../runtime/object_model.hpp#L550)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-local-transform-1"></a>

## `epok::SceneComponent3D::local_transform`

**Purpose.** Performs `local transform` as part of the object model module.

**Exact declaration**

```cpp
Transform local_transform() const
```

- **Declared at:** [line 552](../../../runtime/object_model.hpp#L552)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Transform`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.local_transform();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-set-local-position-1"></a>

## `epok::SceneComponent3D::set_local_position`

**Purpose.** Sets local position as part of the object model module.

**Exact declaration**

```cpp
void set_local_position(Fixed x,Fixed y,Fixed z)
```

- **Declared at:** [line 554](../../../runtime/object_model.hpp#L554)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `Fixed` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y
// Fixed z

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.set_local_position(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-set-local-rotation-1"></a>

## `epok::SceneComponent3D::set_local_rotation`

**Purpose.** Sets local rotation as part of the object model module.

**Exact declaration**

```cpp
void set_local_rotation(Fixed x,Fixed y,Fixed z)
```

- **Declared at:** [line 555](../../../runtime/object_model.hpp#L555)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `Fixed` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y
// Fixed z

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.set_local_rotation(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-set-local-scale-1"></a>

## `epok::SceneComponent3D::set_local_scale`

**Purpose.** Sets local scale as part of the object model module.

**Exact declaration**

```cpp
void set_local_scale(Fixed x,Fixed y,Fixed z)
```

- **Declared at:** [line 556](../../../runtime/object_model.hpp#L556)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `Fixed` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y
// Fixed z

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.set_local_scale(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-set-local-transform-1"></a>

## `epok::SceneComponent3D::set_local_transform`

**Purpose.** Sets local transform as part of the object model module.

**Exact declaration**

```cpp
void set_local_transform(Transform value)
```

- **Declared at:** [line 553](../../../runtime/object_model.hpp#L553)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Transform` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Transform value

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.set_local_transform(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-teleport-1"></a>

## `epok::SceneComponent3D::teleport`

**Purpose.** Performs `teleport` as part of the object model module.

**Exact declaration**

```cpp
void teleport(Fixed x,Fixed y,Fixed z)
```

- **Declared at:** [line 559](../../../runtime/object_model.hpp#L559)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `Fixed` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y
// Fixed z

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.teleport(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-world-affine-1"></a>

## `epok::SceneComponent3D::world_affine`

**Purpose.** Performs `world affine` as part of the object model module.

**Exact declaration**

```cpp
WorldAffineSample world_affine() const
```

- **Declared at:** [line 557](../../../runtime/object_model.hpp#L557)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `WorldAffineSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.world_affine();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenescriptactor-class-id-1"></a>

## `epok::SceneScriptActor::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 881](../../../runtime/object_model.hpp#L881)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneScriptActor& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-class-id-1"></a>

## `epok::Sprite3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 711](../../../runtime/object_model.hpp#L711)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-enabled-1"></a>

## `epok::Sprite3DComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 719](../../../runtime/object_model.hpp#L719)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sprite3dcomponent-entity-slot-1"></a>

## `epok::Sprite3DComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 712](../../../runtime/object_model.hpp#L712)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-pause-animation-1"></a>

## `epok::Sprite3DComponent::pause_animation`

**Purpose.** Pauses animation as part of the object model module.

**Exact declaration**

```cpp
void pause_animation()
```

- **Declared at:** [line 715](../../../runtime/object_model.hpp#L715)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

object.pause_animation();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-play-clip-1"></a>

## `epok::Sprite3DComponent::play_clip`

**Purpose.** Starts clip as part of the object model module.

**Exact declaration**

```cpp
bool play_clip(uint32_t clip)
```

- **Declared at:** [line 714](../../../runtime/object_model.hpp#L714)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `clip` | `uint32_t` | Input | Value supplied for `clip`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t clip

epok::Sprite3DComponent& object = /* obtain a valid instance */;

auto result = object.play_clip(clip);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sprite3dcomponent-playback-state-1"></a>

## `epok::Sprite3DComponent::playback_state`

**Purpose.** Starts back state as part of the object model module.

**Exact declaration**

```cpp
SpritePlaybackState playback_state() const
```

- **Declared at:** [line 713](../../../runtime/object_model.hpp#L713)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `SpritePlaybackState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

auto result = object.playback_state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-poll-event-1"></a>

## `epok::Sprite3DComponent::poll_event`

**Purpose.** Polls event as part of the object model module.

**Exact declaration**

```cpp
uint32_t poll_event()
```

- **Declared at:** [line 717](../../../runtime/object_model.hpp#L717)
- **Kind:** `cxx method`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

auto result = object.poll_event();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-resume-animation-1"></a>

## `epok::Sprite3DComponent::resume_animation`

**Purpose.** Resumes animation as part of the object model module.

**Exact declaration**

```cpp
void resume_animation()
```

- **Declared at:** [line 716](../../../runtime/object_model.hpp#L716)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

object.resume_animation();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-set-color-1"></a>

## `epok::Sprite3DComponent::set_color`

**Purpose.** Sets color as part of the object model module.

**Exact declaration**

```cpp
void set_color(uint32_t red,uint32_t green,uint32_t blue)
```

- **Declared at:** [line 724](../../../runtime/object_model.hpp#L724)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::Sprite3DComponent& object = /* obtain a valid instance */;

object.set_color(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-set-enabled-1"></a>

## `epok::Sprite3DComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 720](../../../runtime/object_model.hpp#L720)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Sprite3DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-set-flip-1"></a>

## `epok::Sprite3DComponent::set_flip`

**Purpose.** Sets flip as part of the object model module.

**Exact declaration**

```cpp
void set_flip(bool x,bool y)
```

- **Declared at:** [line 723](../../../runtime/object_model.hpp#L723)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `bool` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `bool` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool x
// bool y

epok::Sprite3DComponent& object = /* obtain a valid instance */;

object.set_flip(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-set-size-1"></a>

## `epok::Sprite3DComponent::set_size`

**Purpose.** Sets size as part of the object model module.

**Exact declaration**

```cpp
void set_size(Fixed x,Fixed y)
```

- **Declared at:** [line 722](../../../runtime/object_model.hpp#L722)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::Sprite3DComponent& object = /* obtain a valid instance */;

object.set_size(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-set-texture-1"></a>

## `epok::Sprite3DComponent::set_texture`

**Purpose.** Sets texture as part of the object model module.

**Exact declaration**

```cpp
void set_texture(int32_t value)
```

- **Declared at:** [line 721](../../../runtime/object_model.hpp#L721)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::Sprite3DComponent& object = /* obtain a valid instance */;

object.set_texture(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite3dcomponent-take-completion-1"></a>

## `epok::Sprite3DComponent::take_completion`

**Purpose.** Performs `take completion` as part of the object model module.

**Exact declaration**

```cpp
bool take_completion()
```

- **Declared at:** [line 718](../../../runtime/object_model.hpp#L718)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

auto result = object.take_completion();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-textcomponent-class-id-1"></a>

## `epok::TextComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 789](../../../runtime/object_model.hpp#L789)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::TextComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-clear-text-1"></a>

## `epok::TextComponent::clear_text`

**Purpose.** Clears text as part of the object model module.

**Exact declaration**

```cpp
void clear_text()
```

- **Declared at:** [line 795](../../../runtime/object_model.hpp#L795)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::TextComponent& object = /* obtain a valid instance */;

object.clear_text();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-enabled-1"></a>

## `epok::TextComponent::enabled`

**Purpose.** Performs `enabled` as part of the object model module.

**Exact declaration**

```cpp
bool enabled() const
```

- **Declared at:** [line 791](../../../runtime/object_model.hpp#L791)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::TextComponent& object = /* obtain a valid instance */;

auto result = object.enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-textcomponent-entity-slot-1"></a>

## `epok::TextComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
ActorData* entity_slot() const
```

- **Declared at:** [line 790](../../../runtime/object_model.hpp#L790)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::TextComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-set-color-1"></a>

## `epok::TextComponent::set_color`

**Purpose.** Sets color as part of the object model module.

**Exact declaration**

```cpp
void set_color(uint32_t red,uint32_t green,uint32_t blue)
```

- **Declared at:** [line 798](../../../runtime/object_model.hpp#L798)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::TextComponent& object = /* obtain a valid instance */;

object.set_color(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-set-enabled-1"></a>

## `epok::TextComponent::set_enabled`

**Purpose.** Sets enabled as part of the object model module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 792](../../../runtime/object_model.hpp#L792)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::TextComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-set-number-1"></a>

## `epok::TextComponent::set_number`

**Purpose.** Sets number as part of the object model module.

**Exact declaration**

```cpp
void set_number(int32_t value)
```

- **Declared at:** [line 793](../../../runtime/object_model.hpp#L793)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::TextComponent& object = /* obtain a valid instance */;

object.set_number(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-set-text-word-1"></a>

## `epok::TextComponent::set_text_word`

**Purpose.** Sets text word as part of the object model module.

**Exact declaration**

```cpp
bool set_text_word(uint32_t index,uint32_t packed)
```

- **Declared at:** [line 796](../../../runtime/object_model.hpp#L796)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index
// uint32_t packed

epok::TextComponent& object = /* obtain a valid instance */;

auto result = object.set_text_word(index, packed);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-textcomponent-set-unsigned-1"></a>

## `epok::TextComponent::set_unsigned`

**Purpose.** Sets unsigned as part of the object model module.

**Exact declaration**

```cpp
void set_unsigned(uint32_t value)
```

- **Declared at:** [line 794](../../../runtime/object_model.hpp#L794)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::TextComponent& object = /* obtain a valid instance */;

object.set_unsigned(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-set-wrap-1"></a>

## `epok::TextComponent::set_wrap`

**Purpose.** Sets wrap as part of the object model module.

**Exact declaration**

```cpp
void set_wrap(bool value)
```

- **Declared at:** [line 799](../../../runtime/object_model.hpp#L799)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::TextComponent& object = /* obtain a valid instance */;

object.set_wrap(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-text-word-1"></a>

## `epok::TextComponent::text_word`

**Purpose.** Performs `text word` as part of the object model module.

**Exact declaration**

```cpp
uint32_t text_word(uint32_t index) const
```

- **Declared at:** [line 797](../../../runtime/object_model.hpp#L797)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

epok::TextComponent& object = /* obtain a valid instance */;

auto result = object.text_word(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timelinecomponent-class-id-1"></a>

## `epok::TimelineComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 829](../../../runtime/object_model.hpp#L829)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::TimelineComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-uiactor-class-id-1"></a>

## `epok::UIActor::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 873](../../../runtime/object_model.hpp#L873)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::UIActor& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-uiactor-default-root-1"></a>

## `epok::UIActor::default_root`

**Purpose.** Performs `default root` as part of the object model module.

**Exact declaration**

```cpp
ActorComponent* default_root() override
```

- **Declared at:** [line 875](../../../runtime/object_model.hpp#L875)
- **Kind:** `cxx method`

**Returns.** Returns `ActorComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::UIActor& object = /* obtain a valid instance */;

auto result = object.default_root();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-uicomponent-class-id-1"></a>

## `epok::UIComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 583](../../../runtime/object_model.hpp#L583)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::UIComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world-bind-1"></a>

## `epok::World::bind`

**Purpose.** Performs `bind` as part of the object model module.

**Exact declaration**

```cpp
void bind(Level& value, ObjectRegistry& table)
```

- **Declared at:** [line 1437](../../../runtime/object_model.hpp#L1437)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Level &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |
| `table` | `ObjectRegistry &` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Level & value
// ObjectRegistry & table

epok::World& object = /* obtain a valid instance */;

object.bind(value, table);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world-class-id-1"></a>

## `epok::World::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 1433](../../../runtime/object_model.hpp#L1433)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::World& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
