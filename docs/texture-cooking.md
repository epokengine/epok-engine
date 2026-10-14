# PSX texture cooking

Source PNG packages, imported RGBA previews and authored UVs are unchanged. PSX
builds derive indexed GPU data in `texture_cook.rs`:

- Retain the existing indexed 8-bit colors. Use 4 bits only when every source
  index fits in a nibble and all palette entries after index 15 are zero. No
  resampling, color quantization, texture deletion or transparent/STP-bit changes
  are introduced by this cook.
- Pad each upload row to an even number of 16-bit words. Upload 16 or 256 palette
  entries according to the cooked format, aligned to a 16-word CLUT boundary.
- Pack actual pixel rectangles and CLUT rows offline, deterministically. Several
  MaxRects orderings are attempted; an unfittable bank fails explicitly.
- Keep pixels inside a 256x256-texel page using the correct 4/8-bit word density.
  Generated mesh, streaming, baked-draw and skeletal UVs include both page-local
  origins. Dynamic geometry, sprites and HUD use the same runtime convention.
- Place authored HUD fonts first, as 4bpp atlases with a sixteen-entry CLUT,
  from the build's font set alone. Fonts are global, so every scene bank packs
  its textures around the same font placement.
- Reserve every VRAM column the configured display's framebuffers use
  (x below the display width, at every row), and the loading-image/font
  corner at `(960,384,64,128)`. A 640-wide display leaves 376,832 bytes before
  fragmentation; a 320-wide display leaves 704,512 bytes and a 256-wide display
  770,048 bytes. Authored fonts are always placed beyond column 640, so their
  placement does not depend on the resolution. Every generated bank
  `static_assert`s that no texture or CLUT lies inside the display columns.
  This is the engine's conservative layout, not a count-based texture limit.

The generated `Texture` descriptor appends `bits`, `stream_page` and
`stream_offset`; old nine-field initializers default to resident 8-bit pixels.
Source `texture::Data` is still the host's 8-bit representation. Memory reports
use cooked word widths and palette sizes.

The report places progressive framebuffers at Y=0 and Y=256 (240 rows each),
matching PsyQo's flip/scissor/clear operations. All 512 rows before the display width
are excluded by the allocator: gaps at 240..255 and 496..511 are reserved, not
available texture space. Interlaced mode has one 480-row surface and a 32-row
reserved tail. These non-overlapping rectangles account for the entire reserve.
Reports generated before this correction undercounted it by 40,960 bytes and
drew the second progressive framebuffer at Y=240; the actual allocator/runtime
already used the safe layout, so this correction does not move any pixels.

With Geometry Streaming enabled, pixel words are appended to checksummed
64 KiB pages in `GEOMETRY.BIN`. Geometry page IDs and visibility sets do not
change. A texture never crosses a page boundary. Bank activation borrows the
existing page pool, waits for each GPU upload to finish, then releases its CPU
page. Pixels remain in VRAM; only CLUT data stays resident in CPU RAM for palette
animation and reset. No texture uploads are added to the ordinary draw loop.
Without streaming, pixels keep their resident executable representation.

The entire descriptor bank is validated before changing VRAM. A failed read or
checksum never publishes a partially loaded bank or advances gameplay. START
retries only once outstanding CD/host callbacks have relinquished their buffers.
Already verified pages can be reused. Loading a different bank reloads its
textures using the same pool; this is bank loading, not camera-based texture
eviction within a bank. All textures of an active bank must still fit VRAM.

Palette animation caches only controlled textures, keyed by descriptor rather
than a physical palette row. Multiple CLUTs on the same row are independent.
Unchanged animations do not upload; removing/disabling a controller restores the
source palette once. Scene initialization clears the animation cache after the
new bank's textures are loaded. Static textures are not reuploaded each frame.

Ordered rendering also uses a four-byte-per-texture packet-address table,
constant-initialized alongside each generated bank. It stores CLUT address,
base TPAGE and presence, never palette colors. The renderer selects the table
from the successfully published bank once per world pass and combines its
words with blend/dither/display mode when the material changes. Failed bank
uploads still publish nothing; custom descriptor banks retain the original
lookup path. Empty slots and invalid texture indices remain untextured. Neither
the Texture descriptor nor the streamed face/archive ABI changes.

`tests/runtime/verify_spatial.py texture_packet` checks constant initialization
and 2,048,000 packet combinations against the actual PsyQo CLUT/TPAGE types,
including 4/8-bit images, resident/streamed sources, all blend modes,
dithering/interlace, page boundaries and maximum legal CLUT addresses. This
checks address words, not rasterization or CPU performance.

## Verification

### Native indexed TIM sources

The same Texture asset/import/reimport/snapshot workflow also accepts `.tim`
sources. This intentionally strict subset supports one uncompressed 4-bit or
8-bit image, exactly one 16/256-entry CLUT, and dimensions no larger than
256x256. Multiple CLUTs, direct-color TIMs, malformed lengths and trailing data
fail explicitly. Original VRAM coordinates are not placement requests: the
normal offline packer relocates the image safely.

Unlike PNG quantization, TIM decoding preserves every index and all 16 bits of
every CLUT entry. Zero **color words**, not zero indices, are transparent;
`0x8000` remains visible black. STP-clear nonzero colors remain opaque even when
the primitive enables blending. No PSX runtime decoding or per-frame texture
conversion is added. Existing PNG assets retain their previous alpha threshold,
palette quantization and STP-on behavior.

Scene GPU uploads encode coverage/STP in alpha (0/128/255). Blending draws with
an STP-clear color use adjacent opaque-only and blended-only fragment masks;
ordinary PNG draws keep one pass. Palette cycling carries STP with the color.
This reproduces per-texel blend **selection**, not the complete PSX rasterizer:
the Windows preview still uses its existing linear-color blending and depth
buffer, not PSX RGB5 arithmetic and ordering-table rasterization. Thumbnails keep
ordinary visible/cutout RGBA rather than displaying an artificial alpha of 128.

The TIM layout follows the [documented file format](https://psx-spx.consoledev.net/ps1/cdr/cdromfileformats/graphics/#tim-format).
`cargo test tim_` covers bit/index preservation, full 256-color palettes, invalid
inputs, portable snapshots and discovery. `cargo test native_stp --
--include-ignored` additionally reads actual offscreen GPU pixels in all five
material modes; it requires a graphics adapter but not an unlocked desktop.

### Cook and runtime uploads

`cargo test texture` covers exact index/palette conversion including odd widths,
hundreds of small crops, full-height atlases, page/VRAM reservations, deterministic
placement and genuine overflow. `tests/runtime/verify_spatial.py palette_renderer`
compiles the production palette renderer against a recording GPU transport and
checks arbitrary CLUT positions, 4/8-bit upload widths, conflicts and restoration.
The `texture_upload` runtime test uses the production streaming backend and page
pool, verifying leases through upload completion, eviction/reload, malformed
descriptors, read/checksum/timeout failures and safe retry. The Rust archive test
checks exact payloads, deterministic packing and unchanged geometry page bytes.

For a real resolved map, set `EPOK_TEXTURE_PROJECT` and `EPOK_TEXTURE_SCENE` and run
`cargo test real_scene_cook_roundtrips_every_texel_and_fits_vram -- --ignored --nocapture`.
It simulates all uploads into one VRAM array and samples every texel through its
page-relative coordinates, checking both index and palette-word equality without
changing the map. This does not replace emulator VRAM readback or visual tests.
