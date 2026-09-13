# C++ scripting and exports

## Creating and inheriting classes

Use **New C++ Script** in the Project panel or **Add Component**. Choose an asset name, a folder relative to `assets/scripts`, and a searchable parent class. The hierarchy shows eligible classes and inherited properties/event signatures. Folder components use portable identifiers such as `Enemies/Bosses`.

**Create** writes an annotated `.hpp` and `.cpp`; **Create and Attach** also binds the concrete class to the selected entity. Clang validates the transaction; failure removes only newly created files and empty directories. Abstract bases are selectable, but a still-abstract child cannot attach until its pure virtual events are implemented. Generated classes are not `final`. Children retain parent behavior; only a direct Behaviour child gets the empty `update` required by Behaviour's pure virtual contract.

One Behaviour is supported per entity. **Edit > Undo Component Attachment / Redo Component Attachment** restores bindings and values, rejecting intervening scene edits. Undoing attachment does not delete the source asset.

## Semantic reflection

Build both host binaries with `cargo build --locked --bins` and run dependency setup for pinned libclang 18.1.1. Keep `epok-header-tool` beside the editor. It parses real PsyQo headers using MIPS1/o32/little-endian C++20 settings and the configured MIPS compiler's include directories; it is never linked into the game. Metadata under `.epok/reflection` is invalidated by the extractor, configuration, compiler, and transitively included files. Extraction errors preserve the last cache but prevent stale metadata from building.

```cpp
#pragma once
#include "epok.hpp"

class EPOK_CLASS(Blueprintable, Id="a1df4e7f-9f49-4e74-a1c6-4d703047215b") Enemy
    : public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere) epok::Fixed health = 100.0;
    EPOK_PROPERTY(EditAnywhere) bool aggressive = true;
    EPOK_FUNCTION(BlueprintCallable) void damage(epok::Fixed amount) { health -= amount; }
    EPOK_FUNCTION(BlueprintEvent) virtual void defeated() {}
    void update(epok::Transform& transform, epok::Fixed dt) override {
        transform.rotation[1] += dt;
    }
};
```

Place the class annotation **after `class`**. Supported fields are bool, signed/unsigned 32-bit integers, Q12 Fixed, enums up to 32-bit storage, and Fixed arrays of length two or three. Properties must be public instance fields with declarative constant defaults. Constructors must be implicit/defaulted: arbitrary constructor effects cannot be inspected. Multiple/virtual/private inheritance, raw pointers, dynamic containers, nested reflected classes, and reflected templates are rejected. Concrete classes must be default-constructible and assignable for bank reset.

Function metadata includes parameter types/directions, returns, visibility, virtual/abstract/final state, and overridden member IDs. This milestone does not include graph authoring or a general dynamic invocation system.

## Values and compatibility

The Inspector and picker consume one registry. Editing a field stores an explicit instance override. **Reset to Inherited** removes it and uses the current class default. Legacy serialized values remain explicit even when equal to a default. Removed fields retain their values and produce a build diagnostic. **Discard orphan override** is an explicit user action, not an automatic migration.

Generated classes receive a UUID. Unspecified member IDs use Clang USRs, stable across relocation but not renaming. Use `Id="<UUID>"` on properties/functions before first use when rename-stable identity is required. Bindings save class/member IDs and versioned provider/backend identifiers alongside names. Changed member identities require explicit migration instead of reinterpreting old values. Scene version 1 loads into version 2 in memory and is written only on Save.

Legacy `.hpp`, `.cpp`, and `.epokscript` assets remain readable through a compatibility provider. Their exposed fields are Fixed; they are not inheritance bases until annotated. New classes do not need a separate metadata document.

```yaml
name: Spinner
properties:
  - name: speed
    default: 90.0
```

Unavailable providers/backends remain serialized and diagnose their missing capability. Lua creation, a VM, and Lua dependencies are not enabled.

## Runtime semantics

Positions, rotations and scales are local to the parent. Rotations are in degrees and numeric values use Q12 fixed point on PSX.

Properties are assigned before `start`. Measured elapsed time drives fixed 60 Hz `update` steps, with at most eight catch-up steps per rendered frame; Q12 deltas alternate 68/69 raw units to preserve elapsed time. `frame_update` runs once per rendered frame, including while simulation is paused. Use `time.set_paused` and the input API for pause menus; see [input and collision](input-collision.md).

A script on an Empty entity can animate a hierarchy. Scripts must preserve valid transforms and parent relationships. Zero or negative scales, cycles and manually edited runtime parent indices are unsupported.

Activation, destruction, safe handles, scene transitions, tweens and event helpers are documented in [runtime services](runtime-services.md). Additional APIs are documented under [sprites and particles](sprites-particles.md), [cameras](camera-resources.md), [HUD](hud.md) and [Memory Card](memory-card.md).

`entity()` returns a Behaviour's owning entity. `get<T>()` returns null for a disabled component, `add<T>()` enables it and `remove<T>()` disables it. Transform is always available. `find_entity(name)` returns the first match. See [HUD](hud.md) and [Lighting](lighting.md) for component APIs.

## Standalone export

Use the editor's File menu to export a C++ project. Export writes a new timestamped folder under `exports/`, containing runtime sources, scripts, scene data, a Makefile, instructions and license notices.

The exported project rebuilds with Make, the pinned Nugget SDK and a MIPS toolchain, without the editor or extractor. On Windows, use `build.ps1 -Make <make.exe> -Nugget <SDK> -ToolchainBin <MIPS bin>`. Unicode export directories use an ASCII Windows short alias; if unavailable on that volume, move the complete export to an ASCII directory. SDK/tool installations still require ASCII paths without spaces. See the [runtime build instructions](../runtime/README.md). Generated scene data is a snapshot; continue authoring original scenes and scripts in the game project.

Only the Epok runtime is covered by the included Epok MIT license. User-authored game scripts and assets retain their owners' chosen licenses. Third-party runtime notices accompany the export.

## Visual Blueprints

For visual class assets, graph authoring, entity templates, references, timelines,
and instrumented node debugging, see [Blueprints](blueprints.md).
