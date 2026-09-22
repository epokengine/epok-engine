# C++ API reference

This reference is generated from the exact C++ headers shipped with Epok. It is the symbol-by-symbol companion to the workflow guides: use those guides to learn a system, then use this section while writing code.

The [web API explorer](https://epokengine.github.io/docs/api/) presents the same snapshot like a scripting reference: browse 539 classes, structs, enums and aliases, 2274 callable overloads and 2515 public fields, constants and enum values. Every item has its own permanent page and usage snippet. The Markdown modules below remain the compact, header-oriented version for offline reading and repository reviews.

## Choose the right layer

| Layer | Start here | Public callables | Best for |
| --- | --- | ---: | --- |
| Epok runtime | [Browse Epok modules](epok.md) | 1330 | Normal game code, engine components and bounded runtime services |
| PsyQo | [Browse PsyQo modules](psyqo.md) | 944 | Lower-level GPU, GTE, SPU, CD-ROM, pad, task and kernel control |

Prefer Epok when both layers solve the same problem. It preserves the editor/runtime contract and its resource accounting. Reach for PsyQo when you need hardware control Epok does not expose. That extra freedom is useful, but it also makes synchronization, packet lifetime and memory budgets your responsibility.

## Version contract

- Epok declarations come from the runtime headers in this documentation snapshot.
- PsyQo declarations come from the pinned Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`.
- EASTL, the C standard library, OpenBIOS internals and third-party implementation helpers are outside this API reference.
- “Public callable” means a free function, constructor, destructor, conversion, function template or public method declared in the documented namespaces. Private/protected members and compiler-generated lambda call operators are excluded.

## Fast lookup

The web documentation indexes qualified names, declarations and descriptions. Search for a full name such as `epok::raycast`, a method such as `GPU::sendPrimitive`, or a concept such as “memory card callback”. Each module also begins with a compact callable index.

## Reading the warnings

PSX APIs are intentionally explicit. A pointer may refer to DMA-visible memory, a callback may complete on a later frame, a fixed-capacity container can fill, and a successful host preview is not proof of real-console timing. The warning block on every entry calls out these ownership, timing and capacity risks.
