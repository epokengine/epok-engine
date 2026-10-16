#pragma once
#include <stdint.h>

// The runtime builds for size; this decoder is the player's hottest loop.
#if defined(__GNUC__) && !defined(__clang__)
#pragma GCC push_options
#pragma GCC optimize("O2")
#endif

// BS v2/v3 video frames (the payload of .STR video sectors) to MDEC run-length
// codes. The PSX player and the editor preview compile this same header, so the
// Inspector shows exactly what the console decodes.
//
// A frame is an 8-byte header followed by 16-bit little-endian words read most
// significant bit first. Macroblocks are stored column by column; each holds
// Cr, Cb and four Y blocks. A block is one DC value followed by MPEG-1 intra AC
// codes (escape: 6-bit run + 10-bit level) and the "10" end-of-block code. v2
// stores DC as a raw signed 10-bit value; v3 uses MPEG-1 DC size codes with
// deltas in steps of four, predicted separately for Cr, Cb and Y.
namespace epok::fmv {
inline constexpr uint16_t bs_magic = 0x3800, mdec_eob = 0xFE00;
// MDEC "decode macroblocks" command, 15-bit unsigned output, word count in the
// low 16 bits.
inline constexpr uint32_t mdec_decode_15bit = 0x38000000;
// Coefficients, DC and EOB of one block can never exceed this many codes.
inline constexpr uint32_t max_block_codes = 65;
// Room for one more block than a frame uses: `decode_frame` checks for a
// whole block before decoding it, so its buffer is the frame's words plus this.
inline constexpr uint32_t capacity_margin = (max_block_codes + 1) / 2;
static_assert(capacity_margin == 33, "Mirrored by fmv::CAPACITY_MARGIN in src/fmv.rs");

// MDEC "set scale table": floor(32768 * c(k) * cos((2n + 1) * k * pi / 16)),
// c(0) = 1/sqrt(2), c(k) = 1, row k = frequency.
inline constexpr int16_t mdec_idct[64] = {
    23170, 23170, 23170, 23170, 23170, 23170, 23170, 23170,
    32138, 27245, 18204, 6392, -6393, -18205, -27246, -32139,
    30273, 12539, -12540, -30274, -30274, -12540, 12539, 30273,
    27245, -6393, -32139, -18205, 18204, 32138, 6392, -27246,
    23170, -23171, -23171, 23170, 23170, -23171, -23171, 23170,
    18204, -32139, 6392, 27245, -27246, -6393, 32138, -18205,
    12539, -30274, 30273, -12540, -12540, 30273, -30274, 12539,
    6392, -18205, 27245, -32139, 32138, -27246, 18204, -6393,
};
// Raster position of each zigzag index.
inline constexpr uint8_t zigzag_raster[64] = {
    0,  1,  8,  16, 9,  2,  3,  10, 17, 24, 32, 25, 18, 11, 4,  5,  12, 19, 26, 33, 40, 48,
    41, 34, 27, 20, 13, 6,  7,  14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23,
    30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
};
// MPEG-1 default intra matrix in raster order, with the DC step the BS
// encoders assume (2).
inline constexpr uint8_t intra_matrix[64] = {
    2,  16, 19, 22, 26, 27, 29, 34, 16, 16, 22, 24, 27, 29, 34, 37, 19, 22, 26, 27, 29, 34,
    34, 38, 22, 22, 26, 27, 29, 34, 37, 40, 22, 26, 27, 29, 32, 35, 40, 48, 26, 27, 29, 32,
    35, 40, 48, 58, 26, 27, 29, 34, 38, 46, 56, 69, 27, 29, 35, 38, 46, 56, 69, 83,
};

// "TV levels" clips were encoded from limited-range video (luma 16..235,
// chroma 16..240). The player expands them to the full range for free: AC
// steps grow in the MDEC quant tables and the decoder maps each DC value.
struct QuantTables {
  uint8_t luma[64], chroma[64]; // zigzag order, as MDEC command 2 expects
};
constexpr QuantTables quant_tables(bool tv_levels) {
  QuantTables tables{};
  for (uint32_t k = 0; k < 64; ++k) {
    const uint32_t step = intra_matrix[zigzag_raster[k]];
    const bool scaled = tv_levels && k;
    tables.luma[k] = uint8_t(scaled ? (step * 510 + 219) / 438 : step);
    tables.chroma[k] = uint8_t(scaled ? (step * 510 + 224) / 448 : step);
  }
  return tables;
}
inline constexpr QuantTables standard_quant = quant_tables(false), tv_quant = quant_tables(true);
constexpr int32_t divide_rounded(int32_t value, int32_t divisor) {
  return value >= 0 ? (value + divisor / 2) / divisor : -((divisor / 2 - value) / divisor);
}
// DC levels are a quarter of the block's mean pixel offset from 128.
constexpr int32_t tv_dc(int32_t dc, bool luma) {
  const int32_t value = luma ? divide_rounded(dc * 255 + 2112, 219) : divide_rounded(dc * 255, 224);
  return value < -512 ? -512 : value > 511 ? 511 : value;
}
static_assert(tv_dc(-448, true) == -512 && tv_dc(428, true) == 508 && tv_dc(0, false) == 0,
              "Limited black/white map to full black/white");

enum class BsStatus : uint8_t { OK, HEADER, VERSION, SIZE, CODE, BLOCK, OVERRUN, CAPACITY };

struct AcTables {
  // Codes of 2..8 bits (first three bits not all zero), indexed by nine bits
  // so the sign comes with them: the MDEC code (16 bits) | bits consumed << 16
  // | coefficients advanced << 20. Advancing 0 marks the end-of-block code.
  uint32_t fast[512];
  // Entry: level (6 bits) | run (5 bits) << 6 | code length (5 bits) << 11,
  // sign bit not included. Level 0 marks the escape code (length 6).
  uint16_t middle[128]; // 0000001.. to 0001....: codes of 6, 7 and 10 bits
  uint16_t low[512];    // seven leading zeros: codes of 12..16 bits
};

namespace detail {
struct AcCode {
  uint8_t length, code, run, level;
};
// MPEG-1 DCT coefficient codes without their trailing sign bit. Codes of 10
// bits and longer are "0..01" followed by four (10 bits) or five bits.
inline constexpr AcCode ac_codes[] = {
    {2, 0b11, 0, 1},       {3, 0b011, 1, 1},      {4, 0b0100, 0, 2},     {4, 0b0101, 2, 1},
    {5, 0b00101, 0, 3},    {5, 0b00110, 4, 1},    {5, 0b00111, 3, 1},    {6, 0b000100, 7, 1},
    {6, 0b000101, 6, 1},   {6, 0b000110, 1, 2},   {6, 0b000111, 5, 1},   {7, 0b0000100, 2, 2},
    {7, 0b0000101, 9, 1},  {7, 0b0000110, 0, 4},  {7, 0b0000111, 8, 1},  {8, 0b00100000, 13, 1},
    {8, 0b00100001, 0, 6}, {8, 0b00100010, 12, 1}, {8, 0b00100011, 11, 1}, {8, 0b00100100, 3, 2},
    {8, 0b00100101, 1, 3}, {8, 0b00100110, 0, 5}, {8, 0b00100111, 10, 1},
    {10, 0x8, 16, 1},  {10, 0x9, 5, 2},   {10, 0xA, 0, 7},   {10, 0xB, 2, 3},
    {10, 0xC, 1, 4},   {10, 0xD, 15, 1},  {10, 0xE, 14, 1},  {10, 0xF, 4, 2},
    {12, 0x10, 0, 11}, {12, 0x11, 8, 2},  {12, 0x12, 4, 3},  {12, 0x13, 0, 10},
    {12, 0x14, 2, 4},  {12, 0x15, 7, 2},  {12, 0x16, 21, 1}, {12, 0x17, 20, 1},
    {12, 0x18, 0, 9},  {12, 0x19, 19, 1}, {12, 0x1A, 18, 1}, {12, 0x1B, 1, 5},
    {12, 0x1C, 3, 3},  {12, 0x1D, 0, 8},  {12, 0x1E, 6, 2},  {12, 0x1F, 17, 1},
    {13, 0x10, 10, 2}, {13, 0x11, 9, 2},  {13, 0x12, 5, 3},  {13, 0x13, 3, 4},
    {13, 0x14, 2, 5},  {13, 0x15, 1, 7},  {13, 0x16, 1, 6},  {13, 0x17, 0, 15},
    {13, 0x18, 0, 14}, {13, 0x19, 0, 13}, {13, 0x1A, 0, 12}, {13, 0x1B, 26, 1},
    {13, 0x1C, 25, 1}, {13, 0x1D, 24, 1}, {13, 0x1E, 23, 1}, {13, 0x1F, 22, 1},
    {14, 0x10, 0, 31}, {14, 0x11, 0, 30}, {14, 0x12, 0, 29}, {14, 0x13, 0, 28},
    {14, 0x14, 0, 27}, {14, 0x15, 0, 26}, {14, 0x16, 0, 25}, {14, 0x17, 0, 24},
    {14, 0x18, 0, 23}, {14, 0x19, 0, 22}, {14, 0x1A, 0, 21}, {14, 0x1B, 0, 20},
    {14, 0x1C, 0, 19}, {14, 0x1D, 0, 18}, {14, 0x1E, 0, 17}, {14, 0x1F, 0, 16},
    {15, 0x10, 0, 40}, {15, 0x11, 0, 39}, {15, 0x12, 0, 38}, {15, 0x13, 0, 37},
    {15, 0x14, 0, 36}, {15, 0x15, 0, 35}, {15, 0x16, 0, 34}, {15, 0x17, 0, 33},
    {15, 0x18, 0, 32}, {15, 0x19, 1, 14}, {15, 0x1A, 1, 13}, {15, 0x1B, 1, 12},
    {15, 0x1C, 1, 11}, {15, 0x1D, 1, 10}, {15, 0x1E, 1, 9},  {15, 0x1F, 1, 8},
    {16, 0x10, 1, 18}, {16, 0x11, 1, 17}, {16, 0x12, 1, 16}, {16, 0x13, 1, 15},
    {16, 0x14, 6, 3},  {16, 0x15, 16, 2}, {16, 0x16, 15, 2}, {16, 0x17, 14, 2},
    {16, 0x18, 13, 2}, {16, 0x19, 12, 2}, {16, 0x1A, 11, 2}, {16, 0x1B, 31, 1},
    {16, 0x1C, 30, 1}, {16, 0x1D, 29, 1}, {16, 0x1E, 28, 1}, {16, 0x1F, 27, 1},
};
constexpr uint16_t ac_entry(uint32_t length, uint32_t run, uint32_t level) {
  return uint16_t(level | run << 6 | length << 11);
}
// Fills every table slot whose leading bits are `code`. Returns false when a
// slot is already taken, which would mean the code list is not prefix-free.
constexpr bool ac_fill(uint16_t *table, uint32_t index_bits, uint32_t code, uint32_t length,
                       uint16_t entry) {
  const uint32_t first = code << (index_bits - length), count = 1u << (index_bits - length);
  for (uint32_t i = first; i < first + count; ++i) {
    if (table[i])
      return false;
    table[i] = entry;
  }
  return true;
}
constexpr bool fast_fill(uint32_t *table, uint32_t code, uint32_t length, uint32_t entry) {
  const uint32_t first = code << (9 - length), count = 1u << (9 - length);
  for (uint32_t i = first; i < first + count; ++i) {
    if (table[i])
      return false;
    table[i] = entry;
  }
  return true;
}
struct AcBuild {
  AcTables tables;
  bool valid;
};
constexpr AcBuild ac_build() {
  AcBuild build{};
  build.valid = fast_fill(build.tables.fast, 0b10, 2, 2u << 16) &&
                ac_fill(build.tables.middle, 10, 0b000001, 6, ac_entry(6, 0, 0));
  for (const auto &c : ac_codes) {
    const uint16_t entry = ac_entry(c.length, c.run, c.level);
    const uint32_t leading = c.length < 3 ? c.code : c.code >> (c.length - 3);
    if (c.length <= 8 && leading) {
      for (uint32_t negative = 0; negative < 2; ++negative) {
        const uint32_t level = (negative ? 0u - c.level : uint32_t(c.level)) & 0x3FF;
        const uint32_t code = uint32_t(c.run) << 10 | level;
        build.valid = build.valid && fast_fill(build.tables.fast, uint32_t(c.code) << 1 | negative,
                                               c.length + 1u,
                                               code | (c.length + 1u) << 16 | (c.run + 1u) << 20);
      }
    } else if (c.length <= 10)
      // Indexed by the first ten bits; the three leading zeros stay implicit.
      build.valid = build.valid && ac_fill(build.tables.middle, 10, c.code, c.length, entry);
    else
      build.valid = build.valid && ac_fill(build.tables.low, 16, c.code, c.length, entry);
  }
  return build;
}
inline constexpr AcBuild ac_built = ac_build();
static_assert(ac_built.valid, "MPEG-1 AC code list must be prefix-free");

static_assert(__BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__, "BS words are read as native halfwords");
// Reads 16-bit little-endian words most significant bit first. At least 17
// bits stay buffered, so one read of up to 16 bits never needs a refill. Words
// past the end read as zero; only consuming them counts as an overrun. The
// data must be 2-byte aligned.
struct BitReader {
  const uint16_t *next, *end;
  uint32_t window = 0;
  int32_t available = 0, phantom = 0;
  BitReader(const uint8_t *data, uint32_t bytes)
      : next(reinterpret_cast<const uint16_t *>(data)),
        end(reinterpret_cast<const uint16_t *>(data) + bytes / 2) {
    refill();
    refill();
  }
  bool overrun() const { return available < phantom; }
  void refill() {
    if (available > 16)
      return;
    uint32_t word = 0;
    if (next < end) {
      word = *next++;
    } else {
      phantom += 16;
    }
    window |= word << (16 - available);
    available += 16;
  }
  uint32_t peek16() const { return window >> 16; }
  void skip(uint32_t bits) {
    window <<= bits;
    available -= int32_t(bits);
    refill();
  }
  uint32_t take(uint32_t bits) {
    const uint32_t value = window >> (32 - bits);
    skip(bits);
    return value;
  }
};
constexpr int32_t extend(uint32_t value, uint32_t bits) {
  return int32_t(value << (32 - bits)) >> (32 - bits);
}
// MPEG-1 DC size: the number of differential bits that follow.
inline int32_t dc_size(BitReader &bits, bool luma) {
  const uint32_t w = bits.peek16();
  if (luma) {
    if (w < 0x8000) {
      bits.skip(2);
      return w < 0x4000 ? 1 : 2;
    }
    if (w < 0xA000) {
      bits.skip(3);
      return 0;
    }
    if (w < 0xC000) {
      bits.skip(3);
      return 3;
    }
    if (w < 0xE000) {
      bits.skip(3);
      return 4;
    }
    for (uint32_t ones = 3; ones < 7; ++ones)
      if (!(w & (0x8000u >> ones))) {
        bits.skip(ones + 1);
        return int32_t(ones + 2);
      }
    return -1;
  }
  if (w < 0xC000) {
    bits.skip(2);
    return int32_t(w >> 14);
  }
  for (uint32_t ones = 2; ones < 8; ++ones)
    if (!(w & (0x8000u >> ones))) {
      bits.skip(ones + 1);
      return int32_t(ones + 1);
    }
  return -1;
}
} // namespace detail

struct BsHeader {
  uint16_t words, magic, quant, version;
};
inline BsHeader read_header(const uint8_t *frame) {
  auto u16 = [frame](int i) { return uint16_t(frame[i] | frame[i + 1] << 8); };
  return {u16(0), u16(2), u16(4), u16(6)};
}

struct BsResult {
  BsStatus status;
  // MDEC words written to `out`, including the leading decode command.
  uint32_t words;
};

// Decodes one frame into `out`: the MDEC decode command followed by the
// run-length codes, padded with end-of-block codes to whole 32-word DMA blocks.
// `capacity` counts 32-bit words and must be at least 1 + the padded codes.
// `tv_levels` expands limited-range DC values; pair it with `tv_quant`.
inline BsResult decode_frame(const uint8_t *frame, uint32_t bytes, uint32_t width, uint32_t height,
                             uint32_t *out, uint32_t capacity, bool tv_levels = false) {
  if (bytes < 8 || !width || !height || width % 16 || height % 16 || capacity < 33 ||
      reinterpret_cast<uintptr_t>(frame) & 1)
    return {BsStatus::SIZE, 0};
  const BsHeader header = read_header(frame);
  if (header.magic != bs_magic || header.quant > 63)
    return {BsStatus::HEADER, 0};
  if (header.version != 2 && header.version != 3)
    return {BsStatus::VERSION, 0};
  const bool v3 = header.version == 3;
  const uint32_t quant = uint32_t(header.quant) << 10;
  const auto &tables = detail::ac_built.tables;
  detail::BitReader bits(frame + 8, bytes - 8);
  auto *codes = reinterpret_cast<uint16_t *>(out + 1);
  const uint32_t limit = (capacity - 1) * 2;
  uint32_t count = 0;
  int32_t predictor[3] = {0, 0, 0};
  const uint32_t macroblocks = (width / 16) * (height / 16);
  for (uint32_t mb = 0; mb < macroblocks; ++mb)
    for (uint32_t block = 0; block < 6; ++block) {
      if (count + max_block_codes > limit)
        return {BsStatus::CAPACITY, 0};
      int32_t dc;
      const uint32_t channel = block < 2 ? block : 2;
      if (v3) {
        const int32_t size = detail::dc_size(bits, channel == 2);
        if (size < 0)
          return {BsStatus::CODE, 0};
        int32_t delta = 0;
        if (size) {
          const uint32_t raw = bits.take(uint32_t(size));
          delta = raw >> (size - 1) ? int32_t(raw) : int32_t(raw) - ((1 << size) - 1);
        }
        // Wrap like the console decoder so either v3 DC flavour round-trips.
        dc = detail::extend(uint32_t(predictor[channel] + delta * 4) & 0x3FF, 10);
        predictor[channel] = dc;
      } else {
        dc = detail::extend(bits.take(10), 10);
      }
      if (tv_levels)
        dc = tv_dc(dc, channel == 2);
      codes[count++] = uint16_t(quant | (uint32_t(dc) & 0x3FF));
      uint32_t index = 0;
      for (;;) {
        const uint32_t w = bits.peek16();
        if (w >= 0x2000) {
          // Nearly every code: one lookup gives the MDEC code, sign included.
          const uint32_t fast = tables.fast[w >> 7];
          bits.skip((fast >> 16) & 0xF);
          const uint32_t advance = fast >> 20;
          if (!advance) {
            codes[count++] = mdec_eob;
            break;
          }
          index += advance;
          if (index > 63)
            return {BsStatus::BLOCK, 0};
          codes[count++] = uint16_t(fast);
          continue;
        }
        const uint32_t entry = w >= 0x0200 ? tables.middle[w >> 6] : tables.low[w];
        if (!entry)
          return {BsStatus::CODE, 0};
        const uint32_t length = entry >> 11, level = entry & 0x3F;
        bits.skip(length);
        uint32_t code;
        if (level) {
          const uint32_t run = (entry >> 6) & 0x1F;
          index += run + 1;
          code = run << 10 | (bits.take(1) ? (0u - level) & 0x3FF : level);
        } else {
          code = bits.take(16);
          index += (code >> 10) + 1;
        }
        if (index > 63)
          return {BsStatus::BLOCK, 0};
        codes[count++] = uint16_t(code);
      }
      if (bits.overrun())
        return {BsStatus::OVERRUN, 0};
    }
  while (count % 64)
    codes[count++] = mdec_eob;
  out[0] = mdec_decode_15bit | count / 2;
  return {BsStatus::OK, 1 + count / 2};
}
} // namespace epok::fmv

#if defined(__GNUC__) && !defined(__clang__)
#pragma GCC pop_options
#endif
