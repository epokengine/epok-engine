# Gameplay actions and control capture

Gameplay actions route controller input through temporary contexts. Use them for a player, a menu or a conversation that needs control over shared buttons. C++, Blueprint and Lua use the same Gameplay Input function library and the same bounded native service.

The current Third Person, Platformer 2D and Dungeon Complete controllers use routed button and axis adapters. Their existing movement and animation algorithms remain unchanged.

## Actions and bindings

| Actions | Default source |
| --- | --- |
| MoveX and MoveY | Left stick, with D-pad override |
| LookX and LookY | Right stick |
| Jump, Interact and Confirm | Cross |
| Cancel | Circle |
| Pause | Start |
| Attack | Square |
| Aim | L1 |
| CycleTarget | R1 |
| Inventory | Triangle |
| Sprint | L3 |
| Custom0 through Custom7 | Unbound |

`sample(action, port)` returns value, held, pressed, released and connected. Axis values are Q12 in -1..1; button values are zero or one. `frame_sample` reads rendered-frame edges. Plain samples do not consume input; `take_pressed` and `take_frame_pressed` consume a press so another action sharing its source cannot take it again.

`bind_button` and `bind_axis` configure actions per logical pad. Invalid arguments return false. Scene reset restores the default gameplay bindings; host keyboard/mouse/controller profiles remain governed by [Project settings](settings.md).

## Capturing controls

`capture(owner, actions, priority, port)` returns an InputCapture token. Use `mask`, `movement_actions`, `look_actions` or `all_actions` to choose the captured actions. The owner must be a live Actor or ActorComponent in a loaded level. There are sixteen capture slots; failure returns an invalid token.

Higher priority wins. Equal priorities select the newest live capture, and releasing it restores the lower context. Ports are independent. Captures also protect shared physical sources: capturing Confirm on Cross blocks the base Jump and Interact actions on Cross.

```cpp
using Controls = epok::GameplayInputLibrary;
auto token = Controls::capture(id(), Controls::all_actions(), 100, 0);
// In the owning object's frame callback:
if (Controls::context_take_pressed(token, epok::GameplayAction::Confirm, true)) {
    Controls::release(token);
}
```

`context_sample` and `context_take_pressed` require a live token and an action included in that capture. `valid` checks a token. `release` affects only that token. Owner disable/destruction, inherited activation, level unload and scene reset expire captures automatically. Opening and closing a capture discard affected edges so the same press cannot open a menu and then jump or immediately confirm it.

Capturing movement gates commands while gravity and collision continue to run. Capturing Look is independent, and capturing controls does not pause time.

## Blueprint and Lua

Blueprint's Gameplay Input category exposes the same operations and InputCapture/GameplayActionSample records. Connect execution through capture, consume and release operations. Snapshot a sample before a later mutation if the earlier value is needed.

Lua uses `epok.gameplay_input`. To recover a capture in another callback, use `owner_capture(owner, exact_action_mask, port)`; it returns the newest matching live token. This avoids acquiring a new slot every frame or declaring a record-valued Lua property.

```lua
---@override
function Menu:on_frame(frame_microseconds)
    local actions = epok.gameplay_input.all_actions()
    local token = epok.gameplay_input.owner_capture(self.ref, actions, 0)
    -- GameplayAction.Confirm has the stored enum value 6.
    if epok.gameplay_input.context_take_pressed(token, 6, true) then
        epok.gameplay_input.release(token)
    end
end
```

The menu acquires its capture when it opens. Frame callbacks can handle paused UI; simulation callbacks use tick samples. Consuming an edge in one domain removes it from the other domain and from pending ticks, including across frames with no simulation step.

Raw `epok::input` and the Input library retain their existing independent semantics. Capture guarantees apply to routed input. The runtime selects the `gameplay-control` capability from reflected calls or native references to GameplayInputLibrary; opaque native code can declare `EPOK_NATIVE_REQUIREMENTS("gameplay-control")`. Unused projects reserve no capture service or polling hooks.
