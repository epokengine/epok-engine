//! PSX full-motion video. psxavenc encodes a clip as a `.STR` stream of
//! 2336-byte Mode 2 sectors that interleave BS video frames with XA-ADPCM
//! audio. This module validates those streams with the rules the console
//! player applies, decodes frames through the same C++ decoder
//! (`runtime/fmv_bitstream.hpp`) and models the MDEC for editor previews.
use serde::{Deserialize, Serialize};

/// XA subheader plus Mode 2 payload, as psxavenc and mkpsxiso exchange it.
pub const SECTOR: usize = 2336;
/// Video bytes per sector after the 32-byte STR header.
pub const CHUNK: usize = 2016;
const HEADER: usize = 32;
/// Payload of the data sector the cook appends after the last frame.
pub const END_MARKER: &[u8; 8] = b"EPOKEND1";
/// Double-speed CD: sectors per second the player streams.
pub const SECTORS_PER_SECOND: u32 = 150;
/// MDEC words the decoder needs beyond a frame's own: room for one whole block
/// (`capacity_margin` in runtime/fmv_bitstream.hpp).
pub const CAPACITY_MARGIN: u32 = 33;

const SUBMODE_AUDIO: u8 = 0x04;
const SUBMODE_DATA: u8 = 0x08;

unsafe extern "C" {
    fn epok_fmv_decode_frame(
        frame: *const u8,
        bytes: u32,
        width: u32,
        height: u32,
        out: *mut u32,
        capacity: u32,
        tv_levels: u32,
        words: *mut u32,
    ) -> u32;
    fn epok_fmv_tables(tv_levels: u32, quant: *mut u8, idct: *mut i16);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct XaFormat {
    pub rate: u32,
    pub channels: u8,
}

/// What the player and the Inspector need to know about a validated stream.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StreamInfo {
    pub width: u32,
    pub height: u32,
    pub frames: u32,
    pub sectors: u32,
    pub codec: u16,
    pub audio: Option<XaFormat>,
    pub xa_file: u8,
    pub xa_channel: u8,
    /// Largest frame, in sectors: the console's assembly buffer size.
    pub max_frame_sectors: u32,
    /// Largest decoded frame in MDEC words, including the decode command.
    pub max_mdec_words: u32,
}
impl StreamInfo {
    pub fn seconds(&self) -> f32 {
        self.sectors as f32 / SECTORS_PER_SECOND as f32
    }
    pub fn frame_bytes(&self) -> u32 {
        self.max_frame_sectors * CHUNK as u32
    }
    /// RAM the player allocates while this clip plays: two assembly buffers,
    /// the MDEC codes and two 16-pixel columns of decoded pixels.
    pub fn player_bytes(&self) -> u32 {
        2 * self.frame_bytes() + 4 * self.mdec_capacity() + 2 * 16 * 240 * 2
    }
    /// The decoder's buffer for this clip, in words.
    pub fn mdec_capacity(&self) -> u32 {
        self.max_mdec_words + CAPACITY_MARGIN
    }
    /// Milliseconds the console needs for the clip's largest frame: the CPU
    /// bitstream pass grows with the MDEC codes, the MDEC pass and upload with
    /// the macroblocks. Measured in PCSX-Redux with tests/integration/verify_fmv.py.
    pub fn decode_estimate_ms(&self) -> f32 {
        let macroblocks = (self.width / 16) * (self.height / 16);
        6.0 + 0.0042 * self.max_mdec_words as f32 + 0.041 * macroblocks as f32
    }
}

#[derive(Debug)]
pub struct Frame {
    pub number: u32,
    /// First sector of the frame within the stream.
    pub sector: u32,
    pub data: Vec<u8>,
}
#[derive(Debug)]
pub struct Stream {
    pub info: StreamInfo,
    pub frames: Vec<Frame>,
}

fn u16_at(bytes: &[u8], at: usize) -> u32 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]]) as u32
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn xa_format(coding: u8) -> Result<XaFormat, String> {
    if coding & 0x30 != 0 {
        return Err("Video audio must be 4-bit XA-ADPCM".into());
    }
    Ok(XaFormat {
        rate: if coding & 0x0C == 0 { 37800 } else { 18900 },
        channels: if coding & 0x03 == 1 { 2 } else { 1 },
    })
}

/// Splits a stream into frames. Chunks must arrive in order and frames must be
/// numbered from 1 without gaps, exactly as the console expects them.
pub fn parse(bytes: &[u8]) -> Result<Stream, String> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(SECTOR) {
        return Err("Video stream must contain whole 2336-byte sectors".into());
    }
    let mut info = StreamInfo::default();
    let mut frames: Vec<Frame> = vec![];
    let mut assembling: Option<(Frame, u32, u32, u32)> = None; // frame, next chunk, chunks, bytes
    let mut address: Option<(u8, u8)> = None;
    let mut ended = false;
    for (index, sector) in bytes.chunks_exact(SECTOR).enumerate() {
        let index = index as u32;
        if ended {
            return Err(format!("Sector {index} follows the end marker"));
        }
        let (sub, data) = sector.split_at(8);
        if sub[..4] != sub[4..] {
            return Err(format!("Sector {index} has an inconsistent XA subheader"));
        }
        let (file, channel, submode, coding) = (sub[0], sub[1], sub[2], sub[3]);
        if submode & SUBMODE_DATA != 0 && data.starts_with(END_MARKER) {
            ended = true;
            info.sectors = index;
            continue;
        }
        if address.is_some_and(|a| a != (file, channel)) {
            return Err(format!("Sector {index} uses another XA file/channel"));
        }
        address = Some((file, channel));
        if submode & SUBMODE_AUDIO != 0 {
            let format = xa_format(coding)?;
            if info.audio.is_some_and(|a| a != format) {
                return Err(format!("Sector {index} changes the audio format"));
            }
            info.audio = Some(format);
            continue;
        }
        if submode & SUBMODE_DATA == 0 || u16_at(data, 0) != 0x0160 || u16_at(data, 2) != 0x8001 {
            return Err(format!("Sector {index} is neither STR video nor XA audio"));
        }
        let (chunk, chunks, number) = (u16_at(data, 4), u16_at(data, 6), u32_at(data, 8));
        let (used, width, height) = (u32_at(data, 12), u16_at(data, 16), u16_at(data, 18));
        if chunks == 0 || chunk >= chunks {
            return Err(format!(
                "Sector {index} has an invalid chunk {chunk}/{chunks}"
            ));
        }
        if width == 0 || height == 0 || width % 16 != 0 || height % 16 != 0 {
            return Err(format!(
                "Frame {number} is {width}x{height}; use multiples of 16"
            ));
        }
        if info.width == 0 {
            (info.width, info.height) = (width, height);
        } else if (info.width, info.height) != (width, height) {
            return Err(format!("Frame {number} changes the video size"));
        }
        if chunk == 0 {
            if assembling.is_some() {
                return Err(format!(
                    "Frame {number} starts before the previous one ended"
                ));
            }
            let expected = frames.len() as u32 + 1;
            if number != expected {
                return Err(format!(
                    "Frame {number} found where frame {expected} was expected"
                ));
            }
            if used < 8 || used as usize > chunks as usize * CHUNK {
                return Err(format!(
                    "Frame {number} declares {used} bytes in {chunks} sectors"
                ));
            }
            assembling = Some((
                Frame {
                    number,
                    sector: index,
                    data: Vec::with_capacity(chunks as usize * CHUNK),
                },
                0,
                chunks,
                used,
            ));
        }
        let Some((frame, next, count, declared)) = assembling.as_mut() else {
            return Err(format!(
                "Sector {index} continues a frame that never started"
            ));
        };
        if frame.number != number || *next != chunk || *count != chunks || *declared != used {
            return Err(format!("Frame {number} has missing or reordered sectors"));
        }
        frame.data.extend_from_slice(&data[HEADER..HEADER + CHUNK]);
        *next += 1;
        if *next == *count {
            let (mut frame, _, chunks, used) = assembling.take().unwrap();
            frame.data.truncate(used as usize);
            info.max_frame_sectors = info.max_frame_sectors.max(chunks);
            frames.push(frame);
        }
    }
    if assembling.is_some() {
        return Err("The last frame is incomplete".into());
    }
    if frames.is_empty() {
        return Err("The stream contains no video frames".into());
    }
    if !ended {
        info.sectors = (bytes.len() / SECTOR) as u32;
    }
    let (file, channel) = address.unwrap_or_default();
    info.xa_file = file;
    info.xa_channel = channel;
    info.frames = frames.len() as u32;
    info.codec = u16_at(&frames[0].data, 6) as u16;
    Ok(Stream { info, frames })
}

/// Appends the end marker the player stops at. Data sectors reach the CPU
/// whatever the XA filter selects, so the marker uses the stream's address.
pub fn finish(mut stream: Vec<u8>, info: &StreamInfo) -> Vec<u8> {
    let mut end = [0_u8; SECTOR];
    let sub = [info.xa_file, info.xa_channel, 0x89, 0];
    end[..4].copy_from_slice(&sub);
    end[4..8].copy_from_slice(&sub);
    end[8..16].copy_from_slice(END_MARKER);
    stream.extend_from_slice(&end);
    stream
}

/// The frame as the halfwords the decoder reads (it requires 2-byte alignment).
fn halfwords(frame: &[u8]) -> Vec<u16> {
    frame
        .chunks(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair.get(1).copied().unwrap_or(0)]))
        .collect()
}

/// Decodes one frame to MDEC words (decode command first), exactly as the
/// console's player does.
pub fn decode(frame: &[u8], width: u32, height: u32, tv_levels: bool) -> Result<Vec<u32>, String> {
    let macroblocks = (width / 16) * (height / 16);
    let capacity = 1 + (macroblocks * 6 * 65).div_ceil(64) * 32;
    let mut out = vec![0_u32; capacity as usize];
    let mut words = 0;
    let data = halfwords(frame);
    // SAFETY: `data` holds `frame.len()` bytes and `out` `capacity` words; the
    // decoder never writes past `capacity`.
    let status = unsafe {
        epok_fmv_decode_frame(
            data.as_ptr().cast(),
            frame.len() as u32,
            width,
            height,
            out.as_mut_ptr(),
            capacity,
            tv_levels as u32,
            &mut words,
        )
    };
    let reason = match status {
        0 => {
            out.truncate(words as usize);
            return Ok(out);
        }
        1 => "invalid BS header",
        2 => "unsupported BS version (expected 2 or 3)",
        3 => "invalid frame size",
        4 => "invalid code",
        5 => "more than 64 coefficients in a block",
        6 => "data ends before the last macroblock",
        _ => "decoded data exceeds the frame buffer",
    };
    Err(format!("Video frame: {reason}"))
}

impl Stream {
    /// Decodes every frame once: proves the stream plays and sizes the
    /// console's MDEC buffer.
    pub fn validate(&mut self, tv_levels: bool) -> Result<(), String> {
        let (width, height) = (self.info.width, self.info.height);
        let mut largest = 0;
        for frame in &self.frames {
            let words = decode(&frame.data, width, height, tv_levels)
                .map_err(|e| format!("Frame {}: {e}", frame.number))?;
            largest = largest.max(words.len() as u32);
        }
        self.info.max_mdec_words = largest;
        Ok(())
    }
}

/// A decoded picture: RGBA8 with the console's 15-bit colour precision.
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    /// Luma samples per level, before colour conversion.
    pub histogram: [u32; 256],
}

/// Reference model of the MDEC's run-length expansion, dequantisation, IDCT
/// and YCbCr to RGB conversion, for previews only.
pub struct Mdec {
    luma: [u8; 64],
    chroma: [u8; 64],
    cosines: [[f32; 8]; 8],
}
impl Mdec {
    pub fn new(tv_levels: bool) -> Self {
        let mut quant = [0_u8; 128];
        let mut idct = [0_i16; 64];
        // SAFETY: both buffers have the sizes the native function fills.
        unsafe { epok_fmv_tables(tv_levels as u32, quant.as_mut_ptr(), idct.as_mut_ptr()) };
        let mut cosines = [[0_f32; 8]; 8];
        for (frequency, row) in cosines.iter_mut().enumerate() {
            for (n, value) in row.iter_mut().enumerate() {
                // The console's scale table, back in [-1, 1] and including C(k)/2.
                *value = idct[frequency * 8 + n] as f32 / 32768.0 / 2.0;
            }
        }
        Self {
            luma: quant[..64].try_into().unwrap(),
            chroma: quant[64..].try_into().unwrap(),
            cosines,
        }
    }
    fn block(&self, codes: &mut impl Iterator<Item = u16>, quant: &[u8; 64]) -> Option<[f32; 64]> {
        let first = codes.find(|&code| code != 0xFE00)?;
        let scale = (first >> 10) as i32;
        let mut coefficients = [0_f32; 64];
        let extend = |code: u16| ((code as i32 & 0x3FF) << 22) >> 22;
        coefficients[0] = (extend(first) * quant[0] as i32).clamp(-1024, 1023) as f32;
        let mut k = 0;
        loop {
            let code = codes.next()?;
            k += (code >> 10) as usize + 1;
            if k > 63 {
                break;
            }
            let value = if scale == 0 {
                extend(code) * 2
            } else {
                (extend(code) * quant[k] as i32 * scale + 4) >> 3
            };
            let position = if scale == 0 { k } else { ZIGZAG[k] };
            coefficients[position] = value.clamp(-1024, 1023) as f32;
        }
        // Separable IDCT: rows, then columns.
        let mut rows = [0_f32; 64];
        for y in 0..8 {
            for x in 0..8 {
                rows[y * 8 + x] = (0..8)
                    .map(|u| coefficients[y * 8 + u] * self.cosines[u][x])
                    .sum();
            }
        }
        let mut pixels = [0_f32; 64];
        for y in 0..8 {
            for x in 0..8 {
                pixels[y * 8 + x] = (0..8).map(|v| rows[v * 8 + x] * self.cosines[v][y]).sum();
            }
        }
        Some(pixels)
    }
    /// Expands one decoded frame (`decode` output) into a picture.
    pub fn picture(&self, words: &[u32], width: u32, height: u32) -> Result<Picture, String> {
        let mut codes = words
            .iter()
            .skip(1)
            .flat_map(|word| [*word as u16, (*word >> 16) as u16]);
        let mut rgba = vec![255_u8; (width * height * 4) as usize];
        let mut histogram = [0_u32; 256];
        let missing = || "Video frame ends before its last macroblock".to_string();
        for mx in 0..width / 16 {
            for my in 0..height / 16 {
                let cr = self.block(&mut codes, &self.chroma).ok_or_else(missing)?;
                let cb = self.block(&mut codes, &self.chroma).ok_or_else(missing)?;
                let mut y_blocks = [[0_f32; 64]; 4];
                for block in &mut y_blocks {
                    *block = self.block(&mut codes, &self.luma).ok_or_else(missing)?;
                }
                for y in 0..16 {
                    for x in 0..16 {
                        let chroma = (y / 2) * 8 + x / 2;
                        let (r, b) = (cr[chroma], cb[chroma]);
                        let sample = y_blocks[(y / 8) * 2 + x / 8][(y % 8) * 8 + x % 8];
                        let level = (sample.round().clamp(-128., 127.) + 128.) as u8;
                        histogram[level as usize] += 1;
                        let channel = |value: f32| {
                            let full = (value.round().clamp(-128., 127.) + 128.) as u8;
                            // Five bits per channel, replicated like the display does.
                            (full & 0xF8) | (full >> 5)
                        };
                        let at =
                            ((my as usize * 16 + y) * width as usize + mx as usize * 16 + x) * 4;
                        rgba[at] = channel(sample + 1.402 * r);
                        rgba[at + 1] = channel(sample - 0.3437 * b - 0.7143 * r);
                        rgba[at + 2] = channel(sample + 1.772 * b);
                    }
                }
            }
        }
        Ok(Picture {
            width,
            height,
            rgba,
            histogram,
        })
    }
}
/// The stream's XA-ADPCM audio as interleaved 16-bit PCM, for previews: the
/// CD controller decodes the same sound groups on the console.
pub fn audio_pcm(bytes: &[u8]) -> Option<(Vec<i16>, XaFormat)> {
    const POSITIVE: [i32; 4] = [0, 60, 115, 98];
    const NEGATIVE: [i32; 4] = [0, 0, -52, -55];
    let mut format = None;
    let mut samples = Vec::new();
    let mut history = [[0_i32; 2]; 2];
    for sector in bytes.chunks_exact(SECTOR) {
        if sector[2] & SUBMODE_AUDIO == 0 {
            continue;
        }
        let xa = xa_format(sector[3]).ok()?;
        format = Some(xa);
        let stereo = xa.channels == 2;
        // 18 sound groups of 128 bytes; each holds eight 28-sample units.
        for group in sector[8..8 + 18 * 128].chunks_exact(128) {
            let mut units = [[0_i16; 28]; 8];
            for (unit, decoded) in units.iter_mut().enumerate() {
                let parameter = group[4 + unit];
                let shift = match parameter & 0x0F {
                    shift @ 0..=12 => shift as u32,
                    _ => 9,
                };
                let filter = ((parameter >> 4) & 3) as usize;
                let channel = if stereo { unit & 1 } else { 0 };
                let [old, older] = &mut history[channel];
                for (word, sample) in decoded.iter_mut().enumerate() {
                    let byte = group[16 + word * 4 + unit / 2];
                    let nibble = if unit & 1 == 1 {
                        byte >> 4
                    } else {
                        byte & 0x0F
                    };
                    let raw = ((nibble as i32) << 28 >> 28 << 12) >> shift;
                    let value = (raw
                        + ((*old * POSITIVE[filter] + *older * NEGATIVE[filter] + 32) >> 6))
                        .clamp(-32768, 32767);
                    *older = *old;
                    *old = value;
                    *sample = value as i16;
                }
            }
            if stereo {
                for pair in units.chunks_exact(2) {
                    for (left, right) in pair[0].iter().zip(&pair[1]) {
                        samples.extend_from_slice(&[*left, *right]);
                    }
                }
            } else {
                for unit in &units {
                    samples.extend_from_slice(unit);
                }
            }
        }
    }
    Some((samples, format?))
}
// Raster position of each zigzag index (runtime/fmv_bitstream.hpp `zigzag_raster`).
const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

#[cfg(test)]
mod tests {
    use super::*;
    const V3: &[u8] = include_bytes!("../tests/fixtures/fmv-pattern-v3.str");
    const V2: &[u8] = include_bytes!("../tests/fixtures/fmv-pattern-v2.str");

    fn pixel(picture: &Picture, x: u32, y: u32) -> [u8; 3] {
        let at = ((y * picture.width + x) * 4) as usize;
        picture.rgba[at..at + 3].try_into().unwrap()
    }
    fn luma(picture: &Picture) -> (usize, usize) {
        let levels = picture
            .histogram
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0);
        let levels = levels.map(|(level, _)| level).collect::<Vec<_>>();
        (levels[0], *levels.last().unwrap())
    }
    fn near(actual: [u8; 3], expected: [u8; 3]) -> bool {
        actual
            .iter()
            .zip(expected)
            .all(|(a, e)| (*a as i32 - e as i32).abs() <= 24)
    }

    #[test]
    fn fixture_streams_decode_to_the_generated_pattern() {
        // psxavenc resamples the 30 fps source and repeats its second frame.
        for (bytes, codec, squares, audio) in [
            (
                V3,
                3,
                &[4, 16, 16, 28][..],
                Some(XaFormat {
                    rate: 37800,
                    channels: 2,
                }),
            ),
            (V2, 2, &[4, 16, 16][..], None),
        ] {
            let mut stream = parse(bytes).unwrap();
            assert_eq!(
                (stream.info.width, stream.info.height, stream.info.codec),
                (64, 48, codec)
            );
            assert_eq!(
                (stream.info.frames as usize, stream.info.audio),
                (squares.len(), audio)
            );
            stream.validate(false).unwrap();
            assert!(stream.info.max_mdec_words > 32 && stream.info.max_frame_sectors >= 1);
            let mdec = Mdec::new(false);
            for frame in &stream.frames {
                let words = decode(&frame.data, 64, 48, false).unwrap();
                assert_eq!(words[0] >> 16, 0x3800);
                assert_eq!(words[0] & 0xFFFF, words.len() as u32 - 1);
                assert_eq!((words.len() - 1) % 32, 0);
                // The decoder emits exactly the word count the encoder declared.
                assert_eq!(
                    u16::from_le_bytes([frame.data[0], frame.data[1]]) as usize,
                    words.len() - 1
                );
                // The console sizes its buffer as the largest frame plus the margin.
                let mut exact = vec![0_u32; words.len() + CAPACITY_MARGIN as usize];
                let mut written = 0;
                let data = halfwords(&frame.data);
                let status = unsafe {
                    epok_fmv_decode_frame(
                        data.as_ptr().cast(),
                        frame.data.len() as u32,
                        64,
                        48,
                        exact.as_mut_ptr(),
                        exact.len() as u32,
                        0,
                        &mut written,
                    )
                };
                assert_eq!((status, written as usize), (0, words.len()));
                assert_eq!(&exact[..words.len()], &words[..]);
                let picture = mdec.picture(&words, 64, 48).unwrap();
                assert!(near(pixel(&picture, 12, 8), [255, 0, 0]));
                assert!(near(pixel(&picture, 52, 8), [0, 255, 0]));
                assert!(near(pixel(&picture, 12, 40), [0, 0, 255]));
                assert!(near(pixel(&picture, 52, 40), [255, 255, 255]));
                let square = squares[frame.number as usize - 1];
                assert!(near(pixel(&picture, square + 4, 24), [0, 0, 0]));
                assert!(!near(pixel(&picture, square + 12, 24), [0, 0, 0]));
                let (dark, bright) = luma(&picture);
                assert!(dark < 24 && bright > 230);
            }
        }
    }

    #[test]
    fn tv_levels_expand_limited_range_to_full_range() {
        let stream = parse(V3).unwrap();
        let frame = &stream.frames[0];
        let full = Mdec::new(false)
            .picture(&decode(&frame.data, 64, 48, false).unwrap(), 64, 48)
            .unwrap();
        let expanded = Mdec::new(true)
            .picture(&decode(&frame.data, 64, 48, true).unwrap(), 64, 48)
            .unwrap();
        // Full-range white stays clipped white and black stays black; mid
        // tones move away from grey.
        assert!(near(pixel(&expanded, 52, 40), [255, 255, 255]));
        assert!(near(pixel(&expanded, 8, 24), [0, 0, 0]));
        assert!(luma(&expanded).1 >= luma(&full).1);
    }

    #[test]
    fn corrupted_streams_are_rejected_with_their_sector() {
        assert!(parse(&V3[..SECTOR * 3 + 1]).is_err());
        let mut truncated = V2.to_vec();
        truncated.truncate(truncated.len() - SECTOR);
        assert!(parse(&truncated).unwrap_err().contains("incomplete"));
        let mut reordered = V2.to_vec();
        let (a, b) = reordered.split_at_mut(SECTOR);
        a.swap_with_slice(&mut b[..SECTOR]);
        assert!(parse(&reordered).is_err());
        let mut damaged = V2.to_vec();
        damaged[8 + HEADER + 3] ^= 0xFF; // BS magic of frame 1
        let mut stream = parse(&damaged).unwrap();
        assert!(stream.validate(false).is_err());
    }

    #[test]
    fn end_marker_closes_the_stream() {
        let stream = parse(V3).unwrap();
        let finished = finish(V3.to_vec(), &stream.info);
        let parsed = parse(&finished).unwrap();
        assert_eq!(parsed.info.frames, stream.info.frames);
        assert_eq!(parsed.info.sectors as usize, V3.len() / SECTOR);
        let mut extra = finished.clone();
        extra.extend_from_slice(&V3[..SECTOR]);
        assert!(parse(&extra).unwrap_err().contains("end marker"));
    }

    #[test]
    fn xa_audio_decodes_to_the_fixture_tone() {
        let (pcm, format) = audio_pcm(V3).unwrap();
        assert_eq!(
            format,
            XaFormat {
                rate: 37800,
                channels: 2
            }
        );
        // Three audio sectors of 2016 stereo frames each.
        assert_eq!(pcm.len(), 3 * 2016 * 2);
        // A 440 Hz left channel crosses zero about 880 times a second.
        let left: Vec<i16> = pcm.iter().step_by(2).copied().collect();
        let crossings = left.windows(2).filter(|w| (w[0] < 0) != (w[1] < 0)).count();
        let seconds = left.len() as f32 / 37800.;
        let rate = crossings as f32 / seconds;
        assert!((700. ..1100.).contains(&rate), "{rate} crossings/s");
        let peak = left.iter().map(|s| s.unsigned_abs()).max().unwrap();
        assert!((4000..12000).contains(&peak), "peak {peak}");
        assert!(audio_pcm(V2).is_none());
    }

    #[test]
    fn shared_tables_match_the_console_defaults() {
        let mut quant = [0_u8; 128];
        let mut idct = [0_i16; 64];
        unsafe { epok_fmv_tables(0, quant.as_mut_ptr(), idct.as_mut_ptr()) };
        assert_eq!(&quant[..8], &[2, 16, 16, 19, 16, 19, 22, 22]);
        assert_eq!(quant[..64], quant[64..]);
        for (k, row) in idct.chunks(8).enumerate() {
            for (n, value) in row.iter().enumerate() {
                let scale = if k == 0 {
                    std::f64::consts::FRAC_1_SQRT_2
                } else {
                    1.0
                };
                let expected = (32768.0
                    * scale
                    * ((2 * n + 1) as f64 * k as f64 * std::f64::consts::PI / 16.0).cos())
                .floor();
                assert_eq!(*value as f64, expected);
            }
        }
        unsafe { epok_fmv_tables(1, quant.as_mut_ptr(), idct.as_mut_ptr()) };
        assert_eq!(quant[0], 2);
        assert_eq!(quant[1], 19); // 16 * 255 / 219
        assert_eq!(quant[65], 18); // 16 * 255 / 224
    }
}
