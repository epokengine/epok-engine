// Host entry points for the PSX video decoder. The editor imports and previews
// videos through runtime/fmv_bitstream.hpp, the same code the console runs.
#include "fmv_bitstream.hpp"

extern "C" uint32_t epok_fmv_decode_frame(const uint8_t *frame, uint32_t bytes, uint32_t width,
                                          uint32_t height, uint32_t *out, uint32_t capacity,
                                          uint32_t tv_levels, uint32_t *words) {
  const auto result =
      epok::fmv::decode_frame(frame, bytes, width, height, out, capacity, tv_levels != 0);
  *words = result.words;
  return uint32_t(result.status);
}

// The MDEC tables the player uploads: quant tables (luma then chroma, zigzag
// order) and the IDCT scale table.
extern "C" void epok_fmv_tables(uint32_t tv_levels, uint8_t *quant, int16_t *idct) {
  const auto &tables = tv_levels ? epok::fmv::tv_quant : epok::fmv::standard_quant;
  for (int i = 0; i < 64; ++i) {
    quant[i] = tables.luma[i];
    quant[64 + i] = tables.chroma[i];
    idct[i] = epok::fmv::mdec_idct[i];
  }
}
