# Gameplay API parity coverage

Generated from `coverage.json` by `tools/gameplay_api_parity.py`; do not edit by hand.

Baseline commit: `4490d2c5d8d1c3e214c362f34382c2d8ec0bf796`. Frozen semantic candidate rows: **3310**.

A classified row is not necessarily implemented. The strict checker fails until every public gameplay row has executable support, tests, examples, and cost evidence on all five surfaces.

## Dispositions

| Disposition | Rows |
| --- | ---: |
| `compatibility_alias` | 44 |
| `hardware_backend` | 6 |
| `internal_implementation` | 2796 |
| `public_gameplay` | 464 |

## Public gameplay gaps

| Surface | Missing rows |
| --- | ---: |
| `cpp` | 0 |
| `blueprint` | 0 |
| `lua_native_cpp` | 0 |
| `lua_vm_bytecode` | 0 |
| `lua_vm_source` | 0 |

## Categories

| Category | Rows |
| --- | ---: |
| `audio_music` | 70 |
| `cameras_scenes` | 37 |
| `collision_3d` | 74 |
| `components_hierarchy` | 14 |
| `gameplay_2d` | 100 |
| `implementation_support` | 979 |
| `input_time` | 61 |
| `language_adapter` | 135 |
| `materials_visuals` | 50 |
| `math_values` | 6 |
| `memory_card` | 70 |
| `objects_actors` | 528 |
| `particles_effects` | 186 |
| `resources_diagnostics` | 58 |
| `runtime_facade` | 546 |
| `skeletal_animation` | 24 |
| `sprites_palettes` | 76 |
| `static_editable_meshes` | 71 |
| `timelines_sequences` | 72 |
| `ui_text` | 34 |
| `utilities_events` | 119 |

## Public gameplay rows

| Capability | Module | C++ | BP | Lua AOT | Lua bytecode | Lua source | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `epok::Actor::active` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::begin_play` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::component_count` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::component_id` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::destroy` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::end_play` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::level_id` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::logical_parent` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::on_disable` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::on_enable` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::on_frame` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::root_id` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::set_active` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::set_wants_tick` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::tick` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Actor::wants_tick` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ActorComponent::begin_play` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ActorComponent::end_play` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ActorComponent::on_disable` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ActorComponent::on_enable` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ActorComponent::on_frame` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ActorComponent::owner_id` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ActorComponent::tick` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ActorComponent::trigger_event` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::clip` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::is_playing` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::pitch` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::play` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::play_on_start` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::priority` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::set_clip` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::set_pitch` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::set_play_on_start` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::set_priority` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::set_volume` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::stop` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::AudioComponent::volume` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BlobShadowComponent::configure` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BlobShadowComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BlobShadowComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Camera3DComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Camera3DComponent::field_of_view` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Camera3DComponent::make_active` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Camera3DComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Camera3DComponent::set_field_of_view` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CanvasComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CanvasComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::layer` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::mask` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::set_center` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::set_half_extents` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::set_layer` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::set_mask` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::set_trigger` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Collider3DComponent::trigger` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionLibrary::ground` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionLibrary::move` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionLibrary::overlap_box` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionLibrary::raycast_segment` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::burst` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::play` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::stop` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusLibrary::add` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusLibrary::clear` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusLibrary::layout` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusLibrary::move` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusLibrary::navigate` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusLibrary::snapshot` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ImageComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ImageComponent::set_color` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ImageComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ImageComponent::set_region` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ImageComponent::set_texture` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputLibrary::analog` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputLibrary::axis` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputLibrary::connected` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputLibrary::frame_pressed` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputLibrary::frame_released` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputLibrary::held` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputLibrary::pressed` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputLibrary::released` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Light3DComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Light3DComponent::intensity` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Light3DComponent::set_color` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Light3DComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Light3DComponent::set_intensity` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Light3DComponent::set_mode` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Light3DComponent::set_range` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Light3DComponent::set_type` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MathLibrary::add` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MathLibrary::clamp` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MathLibrary::lerp` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MathLibrary::scale` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MathLibrary::smoothstep` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MathLibrary::vector3` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::clear_staged_payload` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::file` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::list` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::loaded_word` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::payload` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::probe` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::read` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::set_staged_word` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::snapshot` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::staged_word` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::write` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardLibrary::write_staged` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::bone_count` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::clip_count` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::geometry_state` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::lighting_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::material_state` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::pause_animation` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::play_clip` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::playback_state` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::quad_count` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::request_geometry` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::resume_animation` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::sample_bone` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::sample_geometry_vertex` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::sample_vertex` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::sample_vertices` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::set_lighting_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::set_material_blend` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::set_material_color` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::set_material_depth_bias` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::set_material_texture` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::set_material_unlit` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::set_uv_scroll` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::stop_animation` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::streamed` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Mesh3DComponent::vertex_count` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimatorComponent::configure` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimatorComponent::reset` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimatorComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimatorComponent::state` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterComponent::burst` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterComponent::play` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterComponent::set_lifetime` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterComponent::set_max_particles` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterComponent::set_rate` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterComponent::state` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterComponent::stop` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::burst_effect` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::effect_sequence` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::effect_state` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::pause_effect` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::pause_sequence` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::play_effect` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::play_sequence` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::resume_effect` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::resume_sequence` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::sequence_state` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::stop_effect` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PlaybackLibrary::stop_sequence` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ProgressBarComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ProgressBarComponent::set_colors` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ProgressBarComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ProgressBarComponent::set_value` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ProgressBarComponent::value` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::RectTransformComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::RectTransformComponent::set_anchors` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::RectTransformComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::RectTransformComponent::set_pivot` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::RectTransformComponent::set_position` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::RectTransformComponent::set_size` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceLibrary::clear_skeletal_queries` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceLibrary::skeletal_queries` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceLibrary::snapshot` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent2D::attach_to` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent2D::position_x` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent2D::position_y` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent2D::rotation` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent2D::set_position` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent2D::set_rotation` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent2D::set_scale` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent3D::attach_to` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent3D::local_transform` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent3D::set_local_position` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent3D::set_local_rotation` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent3D::set_local_scale` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent3D::set_local_transform` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent3D::teleport` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneComponent3D::world_affine` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::active_camera_actor` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::fog` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::project` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::request` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::request_with_transition` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::set_camera` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::set_fog` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::snapshot` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneLibrary::transition_snapshot` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::pause_animation` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::play_clip` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::playback_state` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::poll_event` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::resume_animation` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::set_color` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::set_flip` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::set_size` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::set_texture` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Sprite3DComponent::take_completion` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::clear_text` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::set_color` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::set_enabled` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::set_number` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::set_text_word` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::set_unsigned` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::set_wrap` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TextComponent::text_word` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TimeLibrary::paused` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TimeLibrary::set_paused` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TimeLibrary::snapshot` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::ease` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::event_queue_clear` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::event_queue_emit` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::event_queue_poll` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::tween_advance` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::tween_cancel` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::tween_schedule` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::tween_start` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityLibrary::tween_value` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityVectorLibrary::vector_tween_advance` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityVectorLibrary::vector_tween_cancel` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityVectorLibrary::vector_tween_schedule` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityVectorLibrary::vector_tween_start` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::UtilityVectorLibrary::vector_tween_value` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::World2DLibrary::screen_to_world` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::World2DLibrary::world_to_screen` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BoneSample::basis_x` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BoneSample::basis_y` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BoneSample::basis_z` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BoneSample::error` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BoneSample::parent` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BoneSample::position` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BoneSample::sampled_frame` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::BoneSample::success` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CardFileSample::blocks` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CardFileSample::name_hash` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CardFileSample::valid` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionHitSample::actor` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionHitSample::fraction` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionHitSample::hit` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionHitSample::normal` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionHitSample::point` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::CollisionHitSample::started_inside` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::color` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::enabled` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::opacity` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::playing` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::position` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::rate` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::size` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EffectLayer::velocity` | `effect-types` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EventQueueMutation::accepted` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EventQueueMutation::state` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EventQueuePoll::event` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EventQueuePoll::state` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::EventQueuePoll::valid` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusSnapshot::count` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusSnapshot::current` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FocusSnapshot::valid` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FogSettings::blue` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FogSettings::enabled` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FogSettings::end_distance` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FogSettings::green` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FogSettings::red` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::FogSettings::start_distance` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayAabb::maximum` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayAabb::minimum` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayCamera2D::position` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayCamera2D::rotation` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayCamera2D::viewport_height` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayCamera2D::viewport_width` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayCamera2D::viewport_x` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayCamera2D::viewport_y` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayCamera2D::zoom` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventQueue4::count` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventQueue4::dropped` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventQueue4::item0` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventQueue4::item1` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventQueue4::item2` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventQueue4::item3` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventSample::kind` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventSample::source` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayEventSample::value` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayPlaybackSnapshot::result` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayPlaybackSnapshot::revision` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionOptions::blue` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionOptions::fade_in_ms` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionOptions::fade_out_ms` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionOptions::green` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionOptions::red` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionSnapshot::audio_gain` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionSnapshot::busy` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionSnapshot::opacity` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionSnapshot::phase` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTransitionSnapshot::presented` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::completion_pending` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::cycles_remaining` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::delay` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::duration` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::easing` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::elapsed` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::from` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::loop` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::reversed` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::running` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayTweenState::to` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayVector2::x` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayVector2::y` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayVector3::x` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayVector3::y` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayVector3::z` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayVector3TweenState::from` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayVector3TweenState::timing` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::GameplayVector3TweenState::to` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputAxisSample::analog` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputAxisSample::connected` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::InputAxisSample::value` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::blend` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::blue` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::depth_bias` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::green` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::red` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::texture` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::unlit` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::uv_x` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::uv_y` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MaterialSnapshot::valid` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardSnapshot::completed` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardSnapshot::error` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardSnapshot::file_count` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardSnapshot::last_rejection` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardSnapshot::operation` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardSnapshot::payload_bytes` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardSnapshot::request` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MemoryCardSnapshot::state` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MeshVertexSample::data_state` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MeshVertexSample::error` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MeshVertexSample::position` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MeshVertexSample::success` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MoveSample::actor` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MoveSample::blocked` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MoveSample::displacement` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MoveSample::grounded` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MoveSample::normal` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MoveSample::unresolved_overlap` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::MoveSample::valid` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::count` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::item0` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::item1` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::item2` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::item3` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::item4` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::item5` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::item6` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::item7` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ObjectBatch8::total` | `object-model` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimationState::enabled` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimationState::first` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimationState::last` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimationState::offset` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimationState::reverse` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimationState::speed` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimationState::texture` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::PaletteAnimationState::valid` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterState::continuous` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterState::enabled` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterState::max_particles` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterState::pending` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterState::playing` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ParticleEmitterState::valid` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ProjectedPoint::success` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ProjectedPoint::x` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ProjectedPoint::y` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::active_slots` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::active_texture_bytes` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::alive_slots` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::dropped_primitives` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::frame_scanlines` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::mesh_triangles` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::particle_dropped` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::particle_peak` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::particles` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::resident_texture_bytes` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::scene_banks` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::slot_capacity` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::sprite_triangles` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::ResourceSnapshot::textures` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SavePayload8::word0` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SavePayload8::word1` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SavePayload8::word2` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SavePayload8::word3` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SavePayload8::word4` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SavePayload8::word5` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SavePayload8::word6` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SavePayload8::word7` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneSnapshot::active` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneSnapshot::loading` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneSnapshot::rejected` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneSnapshot::transitions` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SceneSnapshot::waiting` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalPlaybackState::clip` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalPlaybackState::enabled` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalPlaybackState::looping` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalPlaybackState::playing` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalPlaybackState::sampled_frame` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalPlaybackState::ticks` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalPlaybackState::valid` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalQuerySnapshot::bones` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalQuerySnapshot::calls` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalQuerySnapshot::decoded_bytes` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalQuerySnapshot::failures` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SkeletalQuerySnapshot::vertices` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::clip` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::clip_count` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::completed` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::dropped_events` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::enabled` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::frame` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::pending_events` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::playing` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::SpritePlaybackState::valid` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TimeSnapshot::dropped_steps` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TimeSnapshot::fixed_delta` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TimeSnapshot::frame_microseconds` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TimeSnapshot::paused` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TimeSnapshot::simulation_ticks` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TweenAdvanceSample::completed` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TweenAdvanceSample::state` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::TweenAdvanceSample::value` | `utility` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Vector3TweenAdvanceSample::completed` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Vector3TweenAdvanceSample::state` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::Vector3TweenAdvanceSample::value` | `gameplay-api` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexIndexBatch4::count` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexIndexBatch4::index0` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexIndexBatch4::index1` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexIndexBatch4::index2` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexIndexBatch4::index3` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSample::error` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSample::position` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSample::sampled_frame` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSample::success` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSamples4::count` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSamples4::sample0` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSamples4::sample1` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSamples4::sample2` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSamples4::sample3` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::VertexSamples4::total` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::WorldAffineSample::basis_x` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::WorldAffineSample::basis_y` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::WorldAffineSample::basis_z` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::WorldAffineSample::position` | `epok` | yes | yes | yes | yes | yes | `implemented` |
| `epok::WorldAffineSample::success` | `epok` | yes | yes | yes | yes | yes | `implemented` |
