//! Display size of a video source, read from its container headers. The
//! importer letterboxes with it; psxavenc 0.3.1's own aspect fitting encodes
//! the scaled picture with the unscaled size, so Epok always passes an exact
//! size instead.

/// Display width and height of the first video track, when the container
/// stores them where these readers look.
pub fn display_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let iso = bytes.get(4..8).is_some_and(|kind| {
        [b"ftyp", b"moov", b"wide", b"mdat", b"free", b"skip"].contains(&kind.try_into().unwrap())
    });
    let size = if iso {
        iso_media(bytes)
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"AVI ") {
        avi(bytes)
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some((u16le(bytes, 6)?, u16le(bytes, 8)?))
    } else if bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        matroska(bytes)
    } else if bytes.starts_with(b"OggS") {
        theora(bytes)
    } else if bytes.starts_with(&[0, 0, 1]) {
        mpeg(bytes)
    } else {
        None
    }?;
    (size.0 > 0 && size.1 > 0).then_some(size)
}

/// Largest picture inside the `width` x `height` box with the source's
/// proportions, in the 16-pixel steps the MDEC needs.
pub fn fit(source: (u32, u32), width: u32, height: u32) -> (u32, u32) {
    let step = |value: f64, limit: u32| (((value / 16.).round() as u32) * 16).clamp(32, limit);
    let (sw, sh) = (source.0 as f64, source.1 as f64);
    if sw * height as f64 >= sh * width as f64 {
        (width, step(width as f64 * sh / sw, height))
    } else {
        (step(height as f64 * sw / sh, width), height)
    }
}

fn u16le(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?) as u32)
}
fn u32le(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}
fn u32be(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

/// MP4, MOV and M4V: the video track's `tkhd` stores its display size.
fn iso_media(bytes: &[u8]) -> Option<(u32, u32)> {
    fn boxes(data: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
        let mut rest = data;
        std::iter::from_fn(move || {
            let size = u32be(rest, 0)? as u64;
            let kind = rest.get(4..8)?;
            let (header, size) = match size {
                0 => (8, rest.len() as u64),
                1 => (16, u64::from_be_bytes(rest.get(8..16)?.try_into().ok()?)),
                size => (8, size),
            };
            if size < header || size > rest.len() as u64 {
                return None;
            }
            let (item, next) = rest.split_at(size as usize);
            rest = next;
            Some((kind, &item[header as usize..]))
        })
    }
    let child = |data, name: &[u8]| boxes(data).find(|(kind, _)| *kind == name).map(|b| b.1);
    let moov = child(bytes, b"moov")?;
    boxes(moov)
        .filter(|(kind, _)| *kind == b"trak")
        .find_map(|(_, trak)| {
            let handler = child(child(trak, b"mdia")?, b"hdlr")?;
            if handler.get(8..12)? != b"vide" {
                return None;
            }
            let tkhd = child(trak, b"tkhd")?;
            let at = if tkhd.first()? == &1 { 88 } else { 76 };
            Some((u32be(tkhd, at)? >> 16, u32be(tkhd, at + 4)? >> 16))
        })
}

/// AVI: the main header's frame size.
fn avi(bytes: &[u8]) -> Option<(u32, u32)> {
    let at = bytes
        .get(..65536.min(bytes.len()))?
        .windows(4)
        .position(|w| w == b"avih")?
        + 8;
    Some((u32le(bytes, at + 32)?, u32le(bytes, at + 36)?))
}

/// MPEG program/elementary streams: the first sequence header, with MPEG-2's
/// display aspect codes.
fn mpeg(bytes: &[u8]) -> Option<(u32, u32)> {
    let at = bytes.windows(4).position(|w| w == [0, 0, 1, 0xB3])? + 4;
    let header = bytes.get(at..at + 4)?;
    let width = (header[0] as u32) << 4 | (header[1] as u32) >> 4;
    let height = ((header[1] as u32) & 0x0F) << 8 | header[2] as u32;
    Some(match header[3] >> 4 {
        2 => (height * 4 / 3, height),
        3 => (height * 16 / 9, height),
        _ => (width, height),
    })
}

/// Ogg Theora: the identification header's picture size.
fn theora(bytes: &[u8]) -> Option<(u32, u32)> {
    let at = bytes.windows(7).position(|w| w == b"\x80theora")? + 7;
    let u24 = |i: usize| -> Option<u32> {
        let b = bytes.get(at + i..at + i + 3)?;
        Some((b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32)
    };
    Some((u24(7)?, u24(10)?))
}

/// Matroska and WebM: the first video track's display (or pixel) size.
fn matroska(bytes: &[u8]) -> Option<(u32, u32)> {
    // EBML variable-length integer; ids keep their length marker.
    fn vint(data: &[u8], keep_marker: bool) -> Option<(u64, usize)> {
        let first = *data.first()?;
        let length = first.leading_zeros() as usize + 1;
        if length > 8 {
            return None;
        }
        let mut value = if keep_marker {
            first as u64
        } else {
            (first as u64) & (0xFF >> length)
        };
        for byte in data.get(1..length)? {
            value = value << 8 | *byte as u64;
        }
        Some((value, length))
    }
    fn elements(data: &[u8]) -> impl Iterator<Item = (u64, &[u8])> {
        let mut rest = data;
        std::iter::from_fn(move || {
            let (id, a) = vint(rest, true)?;
            let (size, b) = vint(rest.get(a..)?, false)?;
            let body = rest.get(a + b..)?;
            // An unknown size (all ones) runs to the end of the parent.
            let unknown = size == (1 << (7 * b)) - 1;
            let size = if unknown { body.len() } else { size as usize };
            let item = body.get(..size)?;
            rest = &body[size..];
            Some((id, item))
        })
    }
    let unsigned = |data: &[u8]| data.iter().fold(0_u32, |v, b| v << 8 | *b as u32);
    let child = |data, id| elements(data).find(|(item, _)| *item == id).map(|e| e.1);
    let segment = child(bytes, 0x1853_8067)?;
    let tracks = child(segment, 0x1654_AE6B)?;
    elements(tracks)
        .filter(|(id, _)| *id == 0xAE)
        .find_map(|(_, entry)| {
            let video = child(entry, 0xE0)?;
            let pixels = (unsigned(child(video, 0xB0)?), unsigned(child(video, 0xBA)?));
            Some(match (child(video, 0x54B0), child(video, 0x54BA)) {
                (Some(w), Some(h)) => (unsigned(w), unsigned(h)),
                _ => pixels,
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iso_box(kind: &[u8], body: &[u8]) -> Vec<u8> {
        let mut data = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        data.extend_from_slice(kind);
        data.extend_from_slice(body);
        data
    }

    #[test]
    fn container_headers_report_display_size() {
        let mut tkhd = vec![0_u8; 84];
        tkhd[76..80].copy_from_slice(&(1280_u32 << 16).to_be_bytes());
        tkhd[80..84].copy_from_slice(&(720_u32 << 16).to_be_bytes());
        let mut hdlr = vec![0_u8; 24];
        hdlr[8..12].copy_from_slice(b"vide");
        let trak = iso_box(
            b"trak",
            &[
                iso_box(b"tkhd", &tkhd),
                iso_box(b"mdia", &iso_box(b"hdlr", &hdlr)),
            ]
            .concat(),
        );
        let mp4 = [iso_box(b"ftyp", b"isom\0\0\0\0"), iso_box(b"moov", &trak)].concat();
        assert_eq!(display_size(&mp4), Some((1280, 720)));

        let gif = b"GIF89a\x40\x01\xF0\x00".to_vec();
        assert_eq!(display_size(&gif), Some((320, 240)));

        let mut avi = b"RIFF\0\0\0\0AVI LIST\0\0\0\0hdrlavih\x38\0\0\0".to_vec();
        avi.extend_from_slice(&[0; 32]);
        avi.extend_from_slice(&640_u32.to_le_bytes());
        avi.extend_from_slice(&480_u32.to_le_bytes());
        avi.extend_from_slice(&[0; 16]);
        assert_eq!(display_size(&avi), Some((640, 480)));

        // 720x480 with a 16:9 display aspect code.
        let mpeg = [0, 0, 1, 0xBA, 0, 0, 1, 0xB3, 0x2D, 0x01, 0xE0, 0x34];
        assert_eq!(display_size(&mpeg), Some((853, 480)));

        // EBML header, Segment(unknown size) > Tracks > TrackEntry > Video.
        let video = [0xE0, 0x87, 0xB0, 0x82, 0x02, 0x80, 0xBA, 0x81, 0xF0];
        let entry = [&[0xAE, 0x80 | video.len() as u8][..], &video].concat();
        let tracks = [
            &[0x16, 0x54, 0xAE, 0x6B, 0x80 | entry.len() as u8][..],
            &entry,
        ]
        .concat();
        let mkv = [
            &[0x1A, 0x45, 0xDF, 0xA3, 0x80][..],
            &[0x18, 0x53, 0x80, 0x67, 0xFF],
            &tracks,
        ]
        .concat();
        assert_eq!(display_size(&mkv), Some((640, 240)));

        assert_eq!(display_size(b"not a video"), None);
    }

    #[test]
    fn fitting_keeps_proportions_in_mdec_steps() {
        assert_eq!(fit((1920, 1080), 320, 240), (320, 176));
        assert_eq!(fit((640, 480), 320, 240), (320, 240));
        assert_eq!(fit((480, 640), 320, 240), (176, 240));
        assert_eq!(fit((2000, 100), 320, 240), (320, 32));
    }
}
