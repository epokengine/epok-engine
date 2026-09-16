# Gameplay API parity contract

This contract is the normative boundary for the engine-owned gameplay facade.
The C++ annotations in `runtime/gameplay_api.hpp`, `runtime/object_model.hpp` and
the registered values in `runtime/epok.hpp` are the signature authority. The
host registry derives Blueprint nodes, Lua definitions, native adapters and VM
bindings from those declarations.

## Versions and compatibility

- Reflection schema 10 adds value declarations, function libraries and operation
  descriptors. Older manifests remain readable; newly extracted manifests write
  schema 10.
- Epok Lua Gameplay profile 2 adds composite values, service operations, typed
  foreign receivers and full-width handles. Existing projects without an
  explicit profile retain Legacy profile 1 behavior. New projects write profile 2.
- The VM call ABI version follows the Lua profile. ABI 2 moves up to 32 words per
  value through a bounded value arena. Scalar ABI 1 programs keep their direct
  one-word path.
- Existing Blueprint builtin UUIDs remain compatibility aliases. New work uses
  catalog operation UUIDs; loading an old graph does not rewrite the asset.

## Values and calls

`Bool`, `Int32`, `UInt32` and Q12 `Fixed` occupy one wire word. Object identities
occupy one word and are validated against their generation before dispatch.
Asset/class references and timeline/effect handles occupy two words. Vector2,
Vector3 and registered records use the sum of their members and are capped at 32
words. Values are copied snapshots: retaining a collision, playback, resource or
skeletal result never retains renderer/service scratch memory.

Arguments are evaluated once, left to right. Blueprint fan-out materializes an
operation result before projecting members. A foreign instance call first checks
the receiver's generation and reflected class; a null, stale or wrong-class
receiver returns the operation's zero/default result and performs no mutation.
Mutable native references remain unavailable to Lua until an assignable bounded
adapter exists.

## Lifecycle and failure

Lifecycle events run in component-before-owner order where the object model
already defines it. `on_frame` runs once per rendered frame, including while the
simulation is paused; `tick` remains simulation-step based. Trigger callbacks
carry a generational `ObjectId`. Destroyed callback owners are quarantined until
nested dispatch returns.

Synchronous queries report failure in their result record. Asynchronous Memory
Card methods return whether the request was accepted and expose completion/error
through `MemoryCardSnapshot`. Payloads are limited to 4096 bytes. The staged-word
API addresses all 1024 words without heap allocation; reads beyond the completed
payload return zero and writes beyond capacity return false.

Scene transition options expose only bounded fade durations and RGB loading color;
native text/image pointers remain host-authored configuration. Static streamed
geometry uses an eight-page request queue. Request calls never wait for media I/O,
sampling pins a ready cache page only long enough to copy one vertex, and callers
observe unavailable, pending, ready and failed states explicitly.

## Skeletal sampling

Vertex indices always name the portable imported vertex order. `Bind` and
`Current` are sampled in model or world space; world conversion uses the same
authoritative actor transform as rendering. A stopped/no-clip animator samples
bind pose. Returned positions and bone matrices are owned values.

Rigid single-vertex sampling evaluates only the selected bone's ancestor chain.
Rigid batches share posed bones. Baked raw frames address a vertex directly;
compressed frames use an optional 16-vertex seek table and decode at most that
block. Baked bone queries use an optional hierarchy/track sidecar and never
reconstruct bones from vertices. The cooker emits remaps, seek tables and baked
bone sidecars only when the operation/native-use manifest demands them.

## Resource reachability

Blueprint and Lua compilers add each resolved operation's capability requirements
to `Scripts.epokmanifest`. Native translation units are conservatively scanned
for known facade calls and may declare opaque requirements with
`EPOK_NATIVE_REQUIREMENTS("capability", ...)`; the resulting
`native-requirements.json` is hashed as a build input. Missing requirements are a
cook error rather than a target-time fallback.

Function-library declarations do not allocate runtime objects or vtables.
Optional focus state and Memory Card staging use function-local bounded storage,
so an unused inline library is eligible for section garbage collection and does
not add frame polling.

## Deliberate limits

The facade does not invent IK, ragdolls, multi-weight skinning, networking,
triangle-mesh physics or general dynamic Lua. Collision reflects the engine's
existing bounded box/segment facilities. Text chunk operations accept bounded
atlas/ASCII bytes; arbitrary heap strings are not introduced. Physical-console
performance is certified only when a recorded hardware run exists.
