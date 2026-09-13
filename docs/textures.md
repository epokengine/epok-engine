# PNG textures and materials

Copy a PNG into the project's `assets/` directory, then import it in **Imports and Assets**. Texture packages retain the PNG snapshot and stable UUID, so moving a project or losing the external PNG does not break an exported game. Reimport can use either the linked source or the stored snapshot. The CLI equivalent is `--project <folder> --import-texture assets/art.png --asset assets/art.epokasset`.

PNG images must be 1–256 pixels on each axis. Import converts RGB to PSX RGB555 and uses an 8-bit palette with one transparent entry and up to 255 visible colors. Alpha below 128 becomes transparent. Opaque black remains visible. Images with more colors use deterministic nearest-color quantization; the inspector and Scene preview show the converted colors. Sampling is nearest-neighbor.

Select the Texture in a mesh's material inspector or a Blockout material override. Blockout UVs are interpolated through mesh subdivision and clipping, exported in Q12, and mapped to the texture's texel centers. Textured mesh UVs must be normalized to 0–1. Split or unwrap repeating surfaces into atlas regions before export. Material color and baked/realtime vertex lighting modulate the texture. Cutout is the default; Average, Add, Subtract and AddQuarter use the four native PSX semitransparency operations. Positive depth bias moves a polygon farther back in the ordering table, in quarter-world-unit buckets.

Sprites, particle sprites and HUD Images reference the same Texture UUID. Atlas regions use pixel x/y/width/height; zero width and height select the remaining full image. Regions are validated against the imported image dimensions. Scene preview shares texture sampling and blend modes across geometry, sprites and particles; Game runs the actual PSX renderer.

VRAM allocation is deterministic per scene bank. It reserves the entire framebuffer region x=0–639, all font pixels x=960–1023/y=448–511, and a palette bank x=640–895/y=480–511. Textures occupy three 128-word columns beginning at x=640, with separate 256-line page bands. Images never cross a texture page. Row upload sizes are padded to the GPU DMA alignment. Each active bank has at most 32 palettes; image dimensions can exhaust the pixel area earlier. A bank transition uploads the next bank's texture layout and releases the previous layout, while immutable source pixels are shared in executable memory. Export validates each bank independently and fails before compilation if a bank's textures do not fit, if a Texture is missing, or if an atlas/UV reference is invalid. Smaller images and reuse of atlas regions reduce occupancy.

The [loading overlay](play.md) additionally reserves x=960–1023/y=384–447 for a
resident RGB555 image. Scene texture layouts keep that area free across bank changes.

Current scene-material limits: 8-bit indexed textures only, binary source alpha, static palettes, and resident shared texture data. There is no mipmapping, automatic atlas repacking, texture streaming, or arbitrary repeat UV addressing. The PSX ordering table still sorts whole triangles, so intersecting translucent surfaces may need subdivision and deliberate depth bias.
