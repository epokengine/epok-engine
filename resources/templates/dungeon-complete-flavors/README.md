# Dungeon Complete gameplay ports

These project-owned classes are being migrated subsystem by subsystem. They are
not yet complete Blueprint/Lua template offerings. Project creation continues to
require C++ until combat, encounters, inventory, economy and terminals also have
their selected-provider implementations and full acceptance.

`DungeonRunFlow` is the first policy port. Its C++ implementation remains in the
bundled template. The Blueprint graph and Lua script here implement the same
intro, defeat/restart and victory rules independently. The common
`DungeonRunFlowBase.hpp` defines typed data and dispatch only; it contains no
transition decisions. Every flavor retains the same class and member identities.

The Blueprint uses normal branches, calls, arithmetic and split record pins.
`tools/generate-dungeon-run-flow.py` reproduces the initial graph. A generated
project edits its own `.epokbp` through the ordinary Blueprint editor.

`tests/integration/verify_dungeon_run_flow.py` runs each policy on PSX and compares
6,000 exact Q12/state words. The Lua version additionally runs bytecode and source
execution. The port fixtures exclude the C++ policy from Blueprint/Lua projects.
