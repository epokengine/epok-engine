# Dialogue widget classes

Dialogue widgets are ordinary `UIActor` classes derived from `DialoguePresenter`.
Their child controls, component properties and behavior belong to that class.
Conversation assets hold the lines, choices, roles, conditions and actions.
See [reusable widget classes](hud.md#reusable-widget-classes) for the common
authoring and inheritance model.

`UIActor` is the general widget class: a HUD, menu or other UI can own child
controls and its own C++, Blueprint or Lua behavior without using dialogue.
`DialoguePresenter` specializes that class with dialogue bindings and events.
A project subclass owns its appearance and can override that behavior. The four
starters below provide editable initial controls; they are not fixed runtime
visual styles or a requirement for creating a custom widget.

## Create and edit

1. In Project, choose **Create > Dialogue Widgets**, then Classic Box, Portrait
   Conversation, Subtitles or Speech Bubble and C++, Blueprint or Lua.
2. Edit the hierarchy and controls in the normal UI class designer. Change their
   RectTransforms, anchors, fonts, colors, images and Focusable properties through
   the component inspectors. **Behavior** opens the same class's source or graph.
3. Place the class in the level. On an object's `DialogueInteractionComponent`,
   assign a conversation and reference the placed widget as its presenter.
4. Bind the interaction through the ordinary interaction helpers and use Play to
   test the conversation and project callbacks.

The CLI creates the same class:

```text
epok-editor --project PATH --new-dialogue-widget MyDialogue --gameplay cpp --presentation classic_box
```

Providers are `cpp`, `blueprint`, `lua`. Presentations are `classic_box`,
`portrait_conversation`, `subtitles`, `speech_bubble`. Existing classes are never
replaced by this command. A source class retains its C++/Lua declaration and an
`.epokclass` composition companion; Blueprint stores the composition in `.epokbp`.

## Extend the dialogue base

The base exposes typed component references for the panel, optional choice popup (`choice_panel`), body, speaker name,
portrait, four choice rows, an optional question (`prompt`) and an optional
continuation label (`continuation`). Bind them to controls in the class composition.
`can_present`, `present(DialogueSnapshot)`, `dialogue_opened(DialogueSnapshot)` and
`dialogue_closed(DialogueSnapshot)` are overridable events in all three providers.
Call the parent `present` implementation to retain standard binding while adding
your own behavior, or implement presentation using the shared session API.

Opening supplies the new session's identity before its first line. Closing runs
once after the base hides its bound controls and releases input capture, including
cancellation and invalidation. Its snapshot retains participant identities and
the terminal result; resolve object references before using them because an object
may have been destroyed. A derived widget can animate its close: the base does
not repeatedly hide controls after it has released the session.

The standard binder updates content, visibility, portrait texture and Canvas
focus. A bound prompt displays the question while revealing its pages and while
selecting an option. Without it, the body displays question pages before selection.
The continuation label is enabled while waiting for confirmation and retains its
authored text, font and appearance. Control geometry, font, wrapping, color,
alignment and panel material remain authored values. Choices use each row's
ordinary Focusable component and Canvas selection. The starters enable its
editable focus outline. The optional choice popup is enabled during selection
and hidden when the session closes.

The widget's structured `default_timing` property contains `reveal_ms` (30)
and `auto_advance_ms` (0, manual). These fields provide timing when the source has no Style asset. Subtitles start with instant
text. An assigned Dialogue Style asset replaces those defaults and supplies
open, advance and close sounds. Create one with **Create > Data & Localization >
Dialogue Timing** and edit it in the ordinary asset Inspector. Visual properties
belong to the widget's controls.

The base uses 13 exposed properties, including two timing fields in one
`DialogueTiming` value. Derived classes can add three project properties within
the common 16-property hierarchy budget; richer project settings can use
supported configuration values or composed components.

Style version 2 stores timing and sounds only. Version 1 remains readable without
rewriting the file or requiring its obsolete font/panel references. Explicitly
saving it preserves the entire original package under
`.epok/migrations/dialogue-style/<asset-id>/<source-hash>.epokasset`, then writes
version 2. The backup is outside the asset catalog and retains the old appearance
data for reference; migration does not apply it to an unrelated widget.

Build/Play paginates localized text against the bound control areas and fonts.
Names and options must fit their own controls; longer body and prompt text can
span pages using their respective control sizes. An unbound optional name does
not impose a font or glyph requirement on invisible content.
Different widgets can use different page layouts for the same conversation.
Starter controls use anchors for the supported 320×240 and 640×480 viewports.

The Classic Box, Portrait Conversation and Subtitles starters use 12 actors and
29 components, including four Focusable components. Speech Bubble uses 13 actors
and 31 components, adding a normal content container and a ScreenAnchorComponent.
They remain subject to the level's 64-actor limit and
the normal HUD budget. Remove optional controls or author a smaller composition
when the level needs more capacity.

The generic class-designer preview shows the resolved actor/component/Text-control
counts, occupied Text-slot bytes, referenced texture/font banks and actual frame
glyphs/drops. Hover the storage line for per-actor component usage, UTF-8 buffer
capacity and the configured HUD budgets. Conversation-only portraits are included
in its resource bank. Slot occupancy describes capacity consumed by the class;
Build's memory report measures the complete linked RAM allocation.

## Preview and current limits

The generic designer's **Preview state** selects authored defaults, a dialogue
line sample, a question sample, a choice sample or a closed sample. Samples affect rendering only;
dragging and saving retain the authored class values. They do not execute project
callbacks. Screenshot automation can select a sample with
`--open-ui-class MyDialogue --ui-preview line|prompt|choices|closed`.

Speech Bubble binds its optional `anchor` to a ScreenAnchorComponent on the widget.
The component moves its content container using the current speaker's world
position, or the interaction source when no speaker role is assigned. Child
controls retain their local geometry. The existing 2D camera or 3D projection
supplies the screen position; no additional UI hierarchy is created.

World offsets, pixel offsets and screen margins are normal component properties.
An off-screen, behind-camera or missing target uses a bottom-center fallback.
Clearing the target or disabling the component freezes the last container position,
so a derived widget can animate its close. The container must be a direct child
of the widget Canvas, fit within the viewport and margin, and have no layout
container on its Canvas parent. Invalid enabled content makes the presenter
unavailable and releases its session capture. Native Play exercises projection;
physically nested anchored canvases must enable **Screen overlay**, as the Speech
Bubble starter does, to keep their world projection in viewport coordinates.
The UI class designer previews the authored layout. The older isolated HUD
simulation reports that world anchors require Native Play.

Classic Box and Portrait Conversation use separate framed dialogue and choice
panels. Subtitles use a low black bar with centered text. Speech Bubble uses a
rounded image with a tail; its choice popup stays on the root Canvas while the
bubble content follows the speaker. Starter images are ordinary project-owned
Texture assets in `assets/GameHelpers/UI`. Their PNG sources and packages are
installed once and reused, preserving subsequent project edits. Samples use
placeholder content; choose Conversation for actual localized pages.

New helper textures record the package identity `epok.game_helpers.ui` and
version 1 in their normal asset metadata. Resource installation and composition
validation share a transaction: failure removes only files that operation
created and whose contents still match its writes. Existing resources and
files edited after creation are retained. Successful scene undo/redo reuses
the project-owned assets.

Choose **Conversation** in Preview state to use a project conversation. Select its
timing asset, language, viewport, speaker and start node. Give each condition an
explicit False or True preview value. **Confirm**, **Up**, **Down**, **Cancel** and
**+100 ms** send synthetic input to the native session. **Preview events** records
node entries and actions. It does not apply project actions or execute the class's
own C++, Blueprint or Lua overrides. Explicit Play exercises that code, sounds and
world anchoring. The conversation preview uses the authored layout for bubbles.

The native dialogue base binds the actual localized pages, reveal count, choices,
name, portrait and focus. Pagination uses the same control areas and imported
fonts as cooking. Preview inputs do not change the class or its controls and are
not saved with it. Changing conversation, geometry, language, timing or condition
values restarts the preview. **Restart** also resets an untouched initial state.

Screenshot automation can select the same preview, using actual asset/node UUIDs:

```text
epok-editor --project PATH --open-ui-class MyDialogue --ui-preview conversation --ui-conversation CONVERSATION_UUID --ui-preview-language 1 --ui-preview-condition CONDITION_UUID=true --ui-preview-start NODE_UUID --ui-preview-input confirm,confirm,down
```

`--ui-preview-speaker PARTICIPANT_UUID` starts at that speaker's first line or
choice; `--ui-preview-timing STYLE_UUID` selects timing. Input accepts `confirm`,
`up`, `down`, `cancel` and `step:MILLISECONDS`, separated by commas. A confirm or
navigation command includes its release, so repeated commands are separate presses.

Reusable widget
classes expand as nested child compositions through the common class hierarchy.
Pagination uses the
control size/font authored at cook time; runtime size/font changes do not reflow
a session.
