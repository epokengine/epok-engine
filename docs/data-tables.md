# Asset DataTables

Create a **Data Table** from the Content Browser's **Add > Data & Localization** menu. It is a portable `.epokasset` with its own UUID. Double-click it to edit its rows. Each row has a case-sensitive string key, an asset kind and a persistent asset UUID. Moving an asset does not change its reference.

The magnifying-glass button beside each reference finds that asset in the Project browser: it opens its folder, clears search/type filters, selects it and scrolls it into view. Missing references disable the button.

Choose **AudioClip** as the row type for a sound-effects table, or **Asset** for mixed kinds. **Add all matching assets** adds the selected kind without duplicating asset references. Keys may be renamed; game code must then use the new key. Save validates duplicate keys, missing/conflicting UUIDs and type mismatches. Save refuses external edits until the table is reopened or reverted. Undo/Redo applies to the table; closing an unfinished edit retains it.

Runtime reference kinds are Texture, AudioClip, MusicSequence, Video, PocketStationApp and DataTable. Every row in a reachable table is packaged, including nested tables, even if no actor directly uses the asset. Replacing a row invalidates the generated scene header and the game must be rebuilt. Table edits do not hot-swap a running game. Missing runtime keys and failed checked casts return an empty reference.

## Lua (Gameplay v2)

Declare and assign a table property in the Inspector:

```lua
SoundPlayer.effects = epok.AssetRef("DataTable")

function SoundPlayer:begin_play()
    local clip = epok.find_asset(self.effects, "GustaffShot", "AudioClip")
    epok.set_audio_clip(self.ref, clip)
    epok.play_audio(self.ref)
end
```

The target needs an AudioSource. For a mixed table, retrieve with kind `"Asset"` and use `epok.cast_asset(asset, "AudioClip")`. Keys and kind names at Lua call sites must be literal strings: they specialize the same operation as a Blueprint node. Asset identities use two 32-bit words across the VM boundary; they are never converted to a Lua number.

## Blueprints

Add an `AssetRef<DataTable>` variable and assign the table. Use **Data Table > Find sound effect** and enter its **Row key** in the node Inspector. Connect the returned asset to **Audio > Set clip**, then **Audio > Play**. **Find asset** returns a generic asset; **Cast to sound effect** checks its kind. The result kind can be changed in the node Inspector.

## C++

Expose an Inspector property with zero as the native default; assign the table UUID through the Inspector or scene document:

```cpp
EPOK_PROPERTY(EditAnywhere, AssetKind=DataTable,
              Id="4c7d9dd3-68a9-4bd5-930e-872df2918c65")
uint64_t sound_effects = 0;

auto clip = epok::assets::find(sound_effects, "GustaffShot",
                               epok::assets::Kind::AudioClip);
int index = epok::assets::audio_clip(clip);
if (index >= 0) {
    source.stop();
    source.clip = index;
    source.play();
}
```

`epok::assets::find(table, key)` returns a generic reference from the table; `epok::assets::cast(reference, kind)` checks a returned table reference. `epok.hpp` includes these APIs. An unknown reference maps to audio index `-1`, never the first sound.

## Command line

```text
epok-editor --project Game --create-data-table assets/DataTables/SoundEffects.epokasset --row-type AudioClip --all-matching-assets
epok-editor --project Game --validate-data-tables
epok-editor --project Game --open-data-table assets/DataTables/SoundEffects.epokasset
```
