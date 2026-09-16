# Gameplay API parity examples

These two small projects contain no project-authored C++ source.

- `lua-only` uses Gameplay profile 2 to read composite time/resource/playback
  snapshots, call typed mesh/audio/particle receivers, sample the bounded
  skeletal and static-geometry availability paths, inspect scene-transition
  state, interact with another actor, drive focus, pause and
  resume from the rendered-frame callback, retain tween/event-queue records,
  and stage a Memory Card payload.
- `blueprint-only` contains a rendered-frame override that pauses/resumes time
  and stages save data through catalog-operation nodes. It uses operation UUIDs
  from the engine declarations rather than legacy builtin nodes.

Switch `lua_execution` among `native_cpp`, `vm_bytecode` and `vm_source` without
changing `GameplayDemo.lua`. Open either `.epokproject` in the editor and use
Build/Play normally. The examples intentionally avoid C++ probes; the independent
native oracle lives under `tests/runtime/`.

The projects are also inputs to
`tests/integration/verify_gameplay_api_parity.py`.
