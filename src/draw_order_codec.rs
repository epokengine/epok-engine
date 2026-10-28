//! Ordered, lossless display-list deltas for offline prepared scenery.
//! Unlike visibility bitsets, copies address arbitrary spans in the PREVIOUS
//! list, preserving moves, duplicates of geometry with different part tags,
//! and exact submission order. Values are opaque 29-bit addresses/part IDs.
//! GPU traversal direction and OT policy belong to the renderer, not the codec.
use std::collections::BTreeMap;

// A 32-byte-aligned byte address in a <=4 GiB archive, shifted by three,
// leaves its low two bits available for the primitive part.
pub const MAX_TOKEN: u32 = 0x1fff_ffff;
pub const MAX_COUNT: usize = u16::MAX as usize;

#[derive(Debug, PartialEq)]
pub struct Span {
    pub offset: u32,
    pub bytes: u32,
    pub count: u16,
    pub reset: bool,
}

fn varint(out: &mut Vec<u8>, mut value: u32) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}

/// Command = (count-1)*2 + literal. Copy commands then encode the old index;
/// literals encode count IDs. Every integer is a bounded, canonical varint.
pub fn encode(previous: &[u32], next: &[u32]) -> Result<Vec<u8>, String> {
    for list in [previous, next] {
        if list.len() > MAX_COUNT || list.iter().any(|&v| v > MAX_TOKEN) {
            return Err("Ordered draw list exceeds token/count limits".into());
        }
        let mut unique = std::collections::BTreeSet::new();
        if list.iter().any(|&v| !unique.insert(v)) {
            return Err("Ordered draw list contains duplicate tokens".into());
        }
    }
    let lookup: BTreeMap<_, _> = previous.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let mut out = Vec::new();
    let mut cursor = 0;
    while cursor < next.len() {
        if let Some(&old) = lookup.get(&next[cursor]) {
            let mut count = 1;
            while cursor + count < next.len()
                && old + count < previous.len()
                && next[cursor + count] == previous[old + count]
            {
                count += 1;
            }
            varint(&mut out, ((count - 1) * 2) as u32);
            varint(&mut out, old as u32);
            cursor += count;
        } else {
            let begin = cursor;
            cursor += 1;
            while cursor < next.len() && !lookup.contains_key(&next[cursor]) {
                cursor += 1;
            }
            varint(&mut out, (((cursor - begin - 1) * 2) | 1) as u32);
            for &value in &next[begin..cursor] {
                varint(&mut out, value);
            }
        }
    }
    Ok(out)
}

/// Caller explicitly marks camera cuts/route anchors as reset checkpoints.
/// Periodic checkpoints bound random seek/reverse replay to 31 deltas.
#[cfg(test)]
pub fn pack<'a>(
    lists: impl IntoIterator<Item = (&'a [u32], bool)>,
) -> Result<(Vec<Span>, Vec<u8>), String> {
    pack_within(lists, usize::MAX)
}
/// As `pack`, also starting a checkpoint early wherever the current block
/// (an 8-byte header, 8 bytes per sample and its deltas) would otherwise
/// exceed `block_budget` bytes, so every block still fits one page when
/// consecutive lists change a lot.
pub fn pack_within<'a>(
    lists: impl IntoIterator<Item = (&'a [u32], bool)>,
    block_budget: usize,
) -> Result<(Vec<Span>, Vec<u8>), String> {
    let mut spans = Vec::new();
    let mut bytes = Vec::new();
    let mut previous: &[u32] = &[];
    let (mut block_rows, mut block_bytes) = (0usize, 0usize);
    for (index, (list, cut)) in lists.into_iter().enumerate() {
        if index >= MAX_COUNT {
            return Err("Ordered draw track exceeds 65535 samples".into());
        }
        let mut reset = cut || index % 32 == 0 || block_rows >= 32;
        let mut delta = encode(if reset { &[] } else { previous }, list)?;
        if !reset && 8 + (block_rows + 1) * 8 + block_bytes + delta.len() > block_budget {
            reset = true;
            delta = encode(&[], list)?;
        }
        if reset {
            block_rows = 0;
            block_bytes = 0;
        }
        block_rows += 1;
        block_bytes += delta.len();
        u32::try_from(
            bytes
                .len()
                .checked_add(delta.len())
                .ok_or("Ordered draw size overflow")?,
        )
        .map_err(|_| "Ordered draw archive exceeds 4 GiB")?;
        spans.push(Span {
            offset: u32::try_from(bytes.len()).map_err(|_| "Ordered draw archive exceeds 4 GiB")?,
            bytes: u32::try_from(delta.len()).map_err(|_| "Ordered draw sample exceeds 4 GiB")?,
            count: list.len() as u16,
            reset,
        });
        bytes.extend(delta);
        previous = list;
    }
    Ok((spans, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn decode(data: &[u8], previous: &[u32]) -> Vec<u32> {
        let mut cursor = 0;
        fn read(data: &[u8], cursor: &mut usize) -> u32 {
            let mut result = 0;
            for shift in [0, 7, 14, 21, 28] {
                let v = data[*cursor];
                *cursor += 1;
                result |= u32::from(v & 127) << shift;
                if v & 128 == 0 {
                    return result;
                }
            }
            panic!("unbounded varint")
        }
        let mut result = Vec::new();
        while cursor < data.len() {
            let code = read(data, &mut cursor);
            let count = (code >> 1) as usize + 1;
            if code & 1 == 0 {
                let begin = read(data, &mut cursor) as usize;
                result.extend_from_slice(&previous[begin..begin + count]);
            } else {
                for _ in 0..count {
                    result.push(read(data, &mut cursor));
                }
            }
        }
        result
    }
    #[test]
    fn blocks_restart_early_to_stay_within_budget() {
        // Lists that share nothing make every delta a full literal.
        let lists: Vec<Vec<u32>> = (0..40u32)
            .map(|i| (0..100).map(|v| i * 1000 + v).collect())
            .collect();
        let (spans, _) = pack_within(lists.iter().map(|l| (l.as_slice(), false)), 2000).unwrap();
        let mut start = 0;
        while start < spans.len() {
            assert!(spans[start].reset);
            let end = (start + 1..spans.len())
                .find(|&i| spans[i].reset)
                .unwrap_or(spans.len());
            let payload: u32 = spans[start..end].iter().map(|s| s.bytes).sum();
            assert!(8 + (end - start) * 8 + payload as usize <= 2000 || end - start == 1);
            start = end;
        }
        assert!(spans.iter().filter(|s| s.reset).count() > 2);
        // Without pressure the periodic checkpoints are unchanged.
        let (plain, _) = pack(lists.iter().map(|l| (l.as_slice(), false))).unwrap();
        assert_eq!(plain.iter().filter(|s| s.reset).count(), 2);
    }
    #[test]
    fn reorder_insert_delete_and_empty_preserve_exact_order() {
        let old = [0, 7, 1000, 2000, MAX_TOKEN];
        for new in [
            vec![],
            vec![MAX_TOKEN, 0, 7, 2000, 17],
            old.to_vec(),
            vec![1, 2, 3],
        ] {
            assert_eq!(decode(&encode(&old, &new).unwrap(), &old), new);
        }
        assert_eq!(encode(&old, &old).unwrap(), [8, 0]);
        assert!(encode(&old, &[]).unwrap().is_empty());
    }
    #[test]
    fn rejects_ambiguous_or_unrepresentable_lists() {
        assert!(encode(&[], &[1, 1]).is_err());
        assert!(encode(&[1, 1], &[]).is_err());
        assert!(encode(&[], &[MAX_TOKEN + 1]).is_err());
        assert!(encode(&[], &(0..65536).collect::<Vec<_>>()).is_err());
        let maximum: Vec<_> = (0..65535).collect();
        assert_eq!(decode(&encode(&[], &maximum).unwrap(), &[]), maximum);
    }
    #[test]
    fn randomized_moves_and_half_state_changes_roundtrip() {
        let mut old = Vec::new();
        let mut seed = 94244u32;
        for _ in 0..300 {
            let mut next: Vec<_> = (0..500)
                .filter_map(|id| {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    (seed & 3 != 0).then_some((id << 2) | (seed >> 17 & 3))
                })
                .collect();
            for i in (1..next.len()).rev() {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                next.swap(i, seed as usize % (i + 1));
            }
            assert_eq!(decode(&encode(&old, &next).unwrap(), &old), next);
            old = next;
        }
    }
    #[test]
    fn checkpoints_bound_seeking_and_reset_route_anchors() {
        let lists: Vec<Vec<_>> = (0..100).map(|i| vec![i, 1000, 2000]).collect();
        let (spans, bytes) = pack(
            lists
                .iter()
                .enumerate()
                .map(|(i, v)| (v.as_slice(), i == 17)),
        )
        .unwrap();
        for target in 0..lists.len() {
            let begin = (0..=target).rev().find(|&i| spans[i].reset).unwrap();
            assert!(target - begin < 32);
            let mut old = Vec::new();
            for span in &spans[begin..=target] {
                let at = span.offset as usize;
                old = decode(&bytes[at..at + span.bytes as usize], &old);
                assert_eq!(old.len(), usize::from(span.count));
            }
            assert_eq!(old, lists[target]);
        }
        assert!(spans[17].reset);
    }
}
