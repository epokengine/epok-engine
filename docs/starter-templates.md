# Starter classes and authoring language

Choose C++, Blueprint or Lua when creating Basic or Sample. The selected language
owns the generated project class. All three authoring systems remain available
for additional classes in the same project.

| Template | Project class | Initial scene |
| --- | --- | --- |
| Basic | `GameRoot : Actor3D` | Camera and an inert GameRoot actor; empty world |
| Sample | `Spinner : ActorComponent` | Camera, two cubes and ground; Spinner on one cube |

Basic's GameRoot has the ordinary inherited 3D root component. Add lifecycle
events, properties, components or derived classes through the normal source and
class tools. Its initial class has no authored callbacks or active helpers.

All six templates offer C++, Blueprint and Lua AOT authoring. Dungeon Complete
places shared data bindings and writes its game policies and ordinary UIActor
widgets in the selected language. Its Blueprint compositions live in their class
assets; C++/Lua use ordinary companion class compositions. Lua UUIDs and template
roles preserve the same authored scene identities across providers. Generated
C++ is a build artifact; edit the Lua rules and widget source directly.

Sample's Spinner turns its owner around Y. Its `speed` property defaults to 45
degrees per second and is editable on a component instance. A derived Spinner
inherits the behavior and speed property, so it can be attached to another actor
and configured independently. Component owner/domain validation still applies.

## Generated files

| Language | Basic | Sample |
| --- | --- | --- |
| C++ | `assets/scripts/GameRoot.hpp`, `GameRoot.cpp` | `assets/scripts/Spinner.hpp`, `Spinner.cpp` |
| Blueprint | `assets/Blueprints/GameRoot.epokbp` | `assets/Blueprints/Spinner.epokbp` |
| Lua | `assets/scripts/GameRoot.lua` | `assets/scripts/Spinner.lua` |

Lua class UUIDs are recorded in `ProjectSettings/LuaClasses.epoksettings`.
Compiler outputs and staging data belong to `.epok`. Select Lua Native C++, VM
bytecode or VM source through the normal project settings; the authored Lua class
stays editable in each mode.

The Hub describes the selected class and file location. CLI creation reports the
same description:

```text
epok-editor --create-project PATH --template basic --gameplay lua
epok-editor --create-project PATH --template sample --gameplay blueprint
```

## Semantic template roles

`ProjectSettings/Template.epoksettings` records the template origin, chosen flavor
and semantic class roles. Basic records `game_root`; Sample records `spinner`.
Third Person and Dungeon record `player`, `camera`, `controller`, `locomotion`
and `camera_input` for their shared character assembly.
Dungeon also records `key`, `door` and `key_state`. Regeneration resolves these class UUIDs,
so compatible project replacements and renamed classes keep their bindings.
Older installation records adopt their recorded gameplay choice on first use;
partial or invalid key/door roles report an error without replacing the file.
Platformer 2D records `player`, `controller`, `locomotion`, `pickup` and `camera`;
its camera role references a component class on the attached camera actor.
Each role stores an ordinary `ClassReference` with its stable class UUID and a
readable name. The common class model resolves the UUID, including after a class
rename. Actor/component bindings in scene documents retain their own identities.

The installation file contains references to project classes; class declarations,
defaults and behavior remain in their normal source or Blueprint assets. Opening
a project validates the installation document and preserves its bytes. Older
projects without this file continue to open. Unknown versions or malformed
references report an error while preserving the authored file for repair.

Basic and Sample enable additional gameplay mechanisms when the project uses
their classes or APIs. Place reusable characters, cameras, interaction or
[world gameplay helpers](world-gameplay-helpers.md) through the common catalog.
Create dialogue presentation as an ordinary editable
[dialogue widget class](dialogue-widgets.md).
