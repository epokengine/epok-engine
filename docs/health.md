# Health values

`HealthState` is an ordinary configuration value with `maximum` and `current`
unsigned 32-bit fields. Actors, components and encounter records can own it.
`HealthLibrary` supplies bounded arithmetic without a pool, allocation or tick.
It is available through the common reflected catalog in all three languages.

```cpp
epok::HealthState health{100,100};
const auto change=epok::HealthLibrary::damage(health,20);
health=change.state;
if(change.depleted){ /* project death rule */ }
```

The equivalent Lua operations are `epok.combat.make_state`, `damage`, `heal`,
`valid` and `is_depleted`. Blueprint exposes them in the Combat category, with
ordinary record pins that can be split into fields.

`make_state(maximum, initial)` clamps the initial value to the maximum. Damage
and healing take signed 32-bit amounts: zero and negative amounts do nothing.
Positive damage saturates at zero; healing saturates at the maximum without
overflow, including a maximum of 4,294,967,295. A zero maximum is a valid empty
state. A state whose current value exceeds its maximum is invalid.

Both operations return a `HealthChange` containing the resulting `state`, the
actual unsigned `applied` amount, transition flags and a `HealthResult`:

| Result | Meaning |
| --- | --- |
| Unchanged | Valid input; no health change |
| Applied | A positive amount was applied |
| InvalidState | Current exceeds maximum; the original state is returned unchanged |

`depleted` is true only when damage changes a positive current value to zero.
`revived` is true only when healing changes zero to a positive value. Repeated
damage at zero does not repeat the depletion transition. `is_depleted` tests a
valid empty state; it returns false for an invalid state.

The caller decides whether a hit is eligible, applies invulnerability and armor,
selects the amount, assigns the returned value, and triggers animation, sound,
rewards or death. The library cannot resolve or damage another object. It leaves
the original value unchanged, so callers must deliberately store `change.state`.

The native state occupies eight bytes and a change sixteen bytes. Their script
wire representations use two and six words respectively, below the existing
32-word bound. A project that does not call the operations owns no health storage.
The P8 acceptance fixture runs separately authored C++, Blueprint and Lua policies
on PSX, including Lua Native, VM bytecode and VM source. All five executions match
the same independent 180-word oracle; the three provider exports rebuild
byte-identically. The C++ complete game uses these values for player, guard and
boss health. Unused Basic retains its prior RAM and executable sizes. See the
[P8 plan](../knowledge/initiatives/gameplay-helpers/plan.md#p8b-generic-mechanisms)
for the remaining complete-game performance and memory gates.
