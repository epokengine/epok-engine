//! Streaming ABI for prepared native packets. Each primitive record and all of
//! its vertices occupy ONE page. Tokens directly address the record, so PSX
//! RAM never needs a resident primitive-to-chunk lookup table.
//!
//! A vertex is stored once per projection span. In a compact span it is two
//! words: signed Q8 X/Y relative to the span center (the GTE VXY layout), and
//! relative Q8 Z in the low half with the three dropped Q12 fraction nibbles
//! above it, which the GTE ignores. Q12 = (relative + center) * 16 + fraction
//! exactly, so the CPU paths keep full precision. A span whose relative values
//! do not fit 16 bits keeps absolute Q12 words. Face records carry their
//! global span index above the two topology bits.
use crate::{
    ordered_draw::{Program, Resolved},
    scene::Scene,
};
use std::collections::{BTreeMap, BTreeSet};

const PAGE: usize = crate::streaming::PAGE_BYTES;
const ALIGN: usize = 32;
const JOB_PRIMITIVES: usize = 64;
pub const ABI: u32 = 3;

#[derive(Clone, Debug)]
pub struct Block {
    pub page: u32,
    pub offset: u16,
    pub first: u16,
}
#[derive(Clone, Debug)]
pub struct ProjectionBlock {
    pub first: u32,
    pub count: usize,
    pub center: [i32; 3],
    pub compact: bool,
    pub low: [i16; 3],
    pub high: [i16; 3],
}
pub struct Cooked {
    pub bytes: Vec<u8>,
    pub blocks: Vec<Block>,
    pub sample_blocks: Vec<u16>,
    pub sets: Vec<BTreeMap<u32, ()>>,
    pub actors: BTreeSet<usize>,
    pub actor_count: usize,
    pub max_packets: usize,
    pub primitive_tokens: Vec<u32>,
    pub projections: Vec<ProjectionBlock>,
}

fn aligned(n: usize) -> usize {
    (n + ALIGN - 1) & !(ALIGN - 1)
}
fn put16(out: &mut [u8], at: usize, value: u16) {
    out[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(out: &mut [u8], at: usize, value: u32) {
    out[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

struct Job {
    first: usize,
    count: usize,
    /// Byte offset after the records for each (span, point).
    vertices: BTreeMap<(usize, [i32; 3]), usize>,
    /// Span of each primitive in the job.
    spans: Vec<usize>,
    length: usize,
    projections: Vec<ProjectionSpan>,
}

struct ProjectionSpan {
    first: usize,
    count: usize,
    center: [i32; 3],
    compact: bool,
    low: [i16; 3],
    high: [i16; 3],
}

/// Signed Q8 offsets of one span's points from its center, if all fit 16 bits.
fn compact_range(points: &[[[i32; 3]; 4]], center: [i32; 3]) -> Option<([i16; 3], [i16; 3])> {
    let mut low = [i16::MAX; 3];
    let mut high = [i16::MIN; 3];
    for point in points.iter().flatten() {
        for axis in 0..3 {
            let relative =
                i16::try_from(i64::from(point[axis] >> 4) - i64::from(center[axis])).ok()?;
            low[axis] = low[axis].min(relative);
            high[axis] = high[axis].max(relative);
        }
    }
    Some((low, high))
}

fn compact_vertex(point: [i32; 3], center: [i32; 3]) -> [u32; 2] {
    let relative = |axis: usize| u32::from(((point[axis] >> 4) - center[axis]) as i16 as u16);
    let fraction = |axis: usize| (point[axis] & 15) as u32;
    [
        relative(0) | relative(1) << 16,
        relative(2) | fraction(0) << 16 | fraction(1) << 20 | fraction(2) << 24,
    ]
}

fn projection_spans(points: &[[[i32; 3]; 4]]) -> Vec<ProjectionSpan> {
    let mut spans = Vec::new();
    let mut first = 0;
    while first < points.len() {
        let mut low = [i32::MAX; 3];
        let mut high = [i32::MIN; 3];
        let mut count = 0;
        while count < JOB_PRIMITIVES && first + count < points.len() {
            let (mut next_low, mut next_high) = (low, high);
            for point in &points[first + count] {
                for axis in 0..3 {
                    next_low[axis] = next_low[axis].min(point[axis] >> 4);
                    next_high[axis] = next_high[axis].max(point[axis] >> 4);
                }
            }
            // 240 units plus at most 8 units of camera-relative lattice
            // rounding on either side fits the signed Q8 vector. A single
            // oversized primitive remains intact and uses the CPU fallback.
            let wide = (0..3).any(|axis| i64::from(next_high[axis]) - i64::from(next_low[axis]) > 240 * 256 - 2);
            if count != 0 && wide { break; }
            low = next_low;
            high = next_high;
            count += 1;
            if wide { break; }
        }
        let center = std::array::from_fn(|axis| ((i64::from(low[axis]) + i64::from(high[axis])) >> 1) as i32);
        let range = compact_range(&points[first..first + count], center);
        spans.push(ProjectionSpan { first, count, center, compact: range.is_some(),
            low: range.map_or([0; 3], |r| r.0), high: range.map_or([0; 3], |r| r.1) });
        first += count;
    }
    spans
}

fn draw_jobs(points: &[[[i32; 3]; 4]]) -> (Vec<Job>, Vec<usize>) {
    // Residency and vertex sharing retain the original 64-primitive jobs.
    // Spatial projection bounds subdivide DESCRIPTORS, never storage jobs:
    // changing a GTE origin must not silently repack every subsequent page.
    let jobs = points.chunks(JOB_PRIMITIVES).enumerate().map(|(id, chunk)| {
        let projections = projection_spans(chunk);
        let mut vertices = BTreeMap::new();
        let mut spans = vec![0; chunk.len()];
        let mut bytes = 0;
        for (index, span) in projections.iter().enumerate() {
            let size = if span.compact { 8 } else { 12 };
            for local in span.first..span.first + span.count {
                spans[local] = index;
                for point in &chunk[local] {
                    vertices.entry((index, *point)).or_insert_with(|| { bytes += size; bytes - size });
                }
            }
        }
        let length = aligned(chunk.len() * 32 + bytes);
        Job { first: id * JOB_PRIMITIVES, count: chunk.len(), vertices, spans, length, projections }
    }).collect();
    (jobs, (0..points.len()).map(|i| i / JOB_PRIMITIVES).collect())
}

pub fn cook(
    scene: &Scene,
    program: &Program,
    resolved: &Resolved,
    first_page: usize,
    pool_pages: usize,
    texture_ids: &[uuid::Uuid],
    palette: &crate::vertex_colors::Palette,
) -> Result<Cooked, String> {
    if pool_pages < 3 || pool_pages > 8 {
        return Err("Ordered drawing needs 3..8 streaming pages".into());
    }
    let layout = crate::texture::layout(scene)?.textures;
    let (jobs, membership) = draw_jobs(&resolved.points);
    if jobs.iter().any(|job| job.length > PAGE) { return Err("Ordered draw job exceeds a page".into()); }
    let track = scene
        .visibility_track
        .as_ref()
        .ok_or("Missing ordered visibility track")?;
    let pairs = track.residency_pairs();
    let mut views = vec![BTreeSet::new(); jobs.len()];
    let sample_jobs: Vec<BTreeSet<_>> = program
        .samples
        .iter()
        .map(|list| {
            list.iter()
                .map(|token| membership[(token >> 2) as usize])
                .collect()
        })
        .collect();
    for (sample, selected) in sample_jobs.iter().enumerate() {
        for &job in selected {
            views[job].insert(sample);
        }
    }
    for (edge, &(a, b)) in pairs.iter().enumerate() {
        for &job in sample_jobs[a].union(&sample_jobs[b]) {
            views[job].insert(program.samples.len() + edge);
        }
    }
    let lengths: Vec<_> = jobs.iter().map(|j| j.length).collect();
    let mut initial = Vec::new();
    let (mut page, mut used) = (0, 0);
    for &length in &lengths {
        if used + length > PAGE {
            page += 1;
            used = 0;
        }
        initial.push(page);
        used += length;
    }
    // Two order pages are a conservative reservation for any continuous edge.
    // The exact sets below still verify every sample and navigation edge.
    let plan = crate::visibility_packing::refine(
        &lengths,
        &views
            .iter()
            .map(|v| v.iter().copied().collect())
            .collect::<Vec<_>>(),
        &initial,
        PAGE,
        pool_pages - 2,
    );
    let mut bins = BTreeMap::new();
    let mut offsets = BTreeMap::<usize, usize>::new();
    let mut locations = Vec::new();
    for (job, bin) in jobs.iter().zip(plan) {
        let next = bins.len();
        let page = *bins.entry(bin).or_insert(next);
        let offset = offsets.entry(page).or_default();
        if *offset + job.length > PAGE {
            return Err("Ordered draw packer overfilled a page".into());
        }
        locations.push((page, *offset));
        *offset += job.length;
    }
    let mut out = Cooked {
        bytes: vec![0; bins.len() * PAGE],
        blocks: vec![],
        sample_blocks: vec![],
        sets: vec![],
        actors: resolved.actors.clone(),
        actor_count: scene.actors.len(),
        max_packets: program.samples.iter().map(Vec::len).max().unwrap_or(0),
        primitive_tokens: vec![0; program.primitives.len()],
        projections: vec![],
    };
    for (job, &(page, offset)) in jobs.iter().zip(&locations) {
        let start = page * PAGE + offset;
        let first = (first_page + page).checked_mul(PAGE)
            .and_then(|n| n.checked_add(offset)).and_then(|n| u32::try_from(n).ok())
            .ok_or("Ordered draw projection address overflow")? >> 3;
        for span in &job.projections {
            out.projections.push(ProjectionBlock { first: first + (span.first * 4) as u32,
                count: span.count, center: span.center, compact: span.compact,
                low: span.low, high: span.high });
        }
        let vertex_start = offset + job.count * 32;
        for (&(span, point), &at) in &job.vertices {
            let at = page * PAGE + vertex_start + at;
            let span = &job.projections[span];
            if span.compact {
                for (word, value) in compact_vertex(point, span.center).into_iter().enumerate() {
                    put32(&mut out.bytes, at + word * 4, value);
                }
            } else {
                for axis in 0..3 { put32(&mut out.bytes, at + axis * 4, point[axis] as u32); }
            }
        }
        for local in 0..job.count {
            let primitive = job.first + local;
            let q = &resolved.quads[primitive];
            let actor = &scene.actors[*resolved
                .actors
                .first()
                .ok_or("Ordered draw has no actors")?];
            let indices = resolved.points[primitive]
                .map(|v| (vertex_start + job.vertices[&(job.spans[local], v)]) as u16);
            let texture = if let Some(id) = q.material.texture {
                texture_ids
                    .iter()
                    .position(|&candidate| candidate == id)
                    .ok_or("Missing ordered texture")?;
                let data = scene
                    .textures
                    .get(&id)
                    .ok_or("Unresolved ordered texture")?;
                let location = layout
                    .iter()
                    .find(|(candidate, _)| *candidate == id)
                    .ok_or("Unplaced ordered texture")?;
                Some(crate::baked_draw::ResolvedTexture::new(location.1, data))
            } else {
                None
            };
            let mut record = crate::baked_draw::encode_with_palette(
                q, indices, actor, texture, palette,
            );
            // Topology is authoring data. Coincident points do not transform a
            // source quad into a triangle or change the tagged half namespace.
            put16(
                &mut record,
                30,
                if program.primitives[primitive].second.is_some() {
                    0
                } else {
                    1
                },
            );
            out.bytes[start + local * 32..start + (local + 1) * 32].copy_from_slice(&record);
            let address = (first_page + page)
                .checked_mul(PAGE)
                .and_then(|a| a.checked_add(offset + local * 32))
                .and_then(|a| u32::try_from(a).ok())
                .ok_or("Ordered draw archive exceeds 4 GiB")?;
            out.primitive_tokens[primitive] = address >> 3;
        }
    }
    out.projections.sort_by_key(|block| block.first);
    // The runtime reaches a face's span directly instead of searching blocks.
    for (index, block) in out.projections.iter().enumerate() {
        let index = u16::try_from(index).ok().filter(|&i| i < 1 << 14)
            .ok_or("Too many ordered projection spans")?;
        for face in 0..block.count {
            let at = ((block.first as usize + face * 4) << 3) - first_page * PAGE + 30;
            let topology = u16::from_le_bytes([out.bytes[at], out.bytes[at + 1]]);
            put16(&mut out.bytes, at, topology | index << 2);
        }
    }
    let lists: Vec<Vec<u32>> = program
        .samples
        .iter()
        .map(|list| {
            list.iter()
                .map(|&token| out.primitive_tokens[(token >> 2) as usize] | (token & 3))
                .collect()
        })
        .collect();
    // Blocks are page-local: restart a checkpoint early rather than overflow.
    let (spans, payload) = crate::draw_order_codec::pack_within(
        lists
            .iter()
            .zip(&track.samples)
            .map(|(l, s)| (l.as_slice(), s.cut)),
        PAGE - 32,
    )?;
    let mut first = 0;
    let mut used = PAGE;
    while first < spans.len() {
        let end = (first + 1..spans.len())
            .find(|&i| spans[i].reset)
            .unwrap_or(spans.len());
        let begin = spans[first].offset as usize;
        let finish = spans[end - 1].offset as usize + spans[end - 1].bytes as usize;
        let header = 8 + (end - first) * 8;
        let length = aligned(header + finish - begin);
        if length > PAGE {
            return Err("Ordered draw checkpoint block exceeds one page".into());
        }
        if used + length > PAGE {
            out.bytes.resize(out.bytes.len() + PAGE, 0);
            used = 0;
        }
        let page = out.bytes.len() / PAGE - 1;
        let start = page * PAGE + used;
        let block_index =
            u16::try_from(out.blocks.len()).map_err(|_| "Too many ordered draw blocks")?;
        out.blocks.push(Block {
            page: u32::try_from(first_page + page).map_err(|_| "Ordered page overflow")?,
            offset: used as u16,
            first: first as u16,
        });
        put32(&mut out.bytes, start, 0x3152444f); // ODR1
        put16(&mut out.bytes, start + 4, (end - first) as u16);
        put16(&mut out.bytes, start + 6, header as u16);
        for (i, span) in spans[first..end].iter().enumerate() {
            let offset = header + span.offset as usize - begin;
            put16(&mut out.bytes, start + 8 + i * 8, offset as u16);
            put16(
                &mut out.bytes,
                start + 10 + i * 8,
                u16::try_from(span.bytes).map_err(|_| "Ordered sample exceeds a page")?,
            );
            put16(&mut out.bytes, start + 12 + i * 8, span.count);
            put16(&mut out.bytes, start + 14 + i * 8, u16::from(span.reset));
            out.sample_blocks.push(block_index);
        }
        out.bytes[start + header..start + header + finish - begin]
            .copy_from_slice(&payload[begin..finish]);
        used += length;
        first = end;
    }
    out.sets = lists
        .iter()
        .enumerate()
        .map(|(sample, list)| {
            let mut pages: BTreeMap<_, _> = list.iter().map(|token| (token >> 13, ())).collect();
            pages.insert(out.blocks[usize::from(out.sample_blocks[sample])].page, ());
            pages
        })
        .collect();
    for (a, b) in pairs.into_iter().chain((0..out.sets.len()).map(|i| (i, i))) {
        let union: BTreeSet<_> = out.sets[a].keys().chain(out.sets[b].keys()).collect();
        if union.len() > pool_pages {
            return Err(format!(
                "Ordered draw transition {a}->{b} requires {} pages; pool has {pool_pages}",
                union.len()
            ));
        }
    }
    Ok(out)
}

impl Cooked {
    pub fn header(&self, actors: usize) -> String {
        format!(
            "static_assert(ordered_draw_abi=={},\"Re-cook the ordered draw ABI\");\ninline constexpr OrderedDrawBlock ordered_blocks[]={{{}}};\ninline constexpr uint16_t ordered_sample_blocks[]={{{}}};\ninline constexpr uint8_t ordered_actors[]={{{}}};\ninline OrderedProjectionBlock ordered_projections[]={{{}}};\ninline constexpr OrderedDrawData ordered_data={{ordered_blocks,ordered_sample_blocks,ordered_actors,{},{},{},{},ordered_projections,{}}};\n",
            ABI,
            self.blocks
                .iter()
                .map(|b| format!("{{{}u,{},{}}}", b.page, b.offset, b.first))
                .collect::<Vec<_>>()
                .join(","),
            self.sample_blocks
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(","),
            (0..actors)
                .map(|i| if self.actors.contains(&i) { "1" } else { "0" })
                .collect::<Vec<_>>()
                .join(","),
            self.projections.iter().map(|b| format!("{{{}u,{},{},{{{},{},{}}},{{{},{},{}}},{{{},{},{}}}}}",
                b.first, b.count, u16::from(b.compact), b.center[0], b.center[1], b.center[2],
                b.low[0], b.low[1], b.low[2], b.high[0], b.high[1], b.high[2])).collect::<Vec<_>>().join(","),
            self.blocks.len(),
            self.sample_blocks.len(),
            self.max_packets,
            actors,
            self.projections.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_spans_do_not_change_storage_jobs_or_split_primitives() {
        let mut points=vec![[[0;3];4];70];
        points[64]=[[1000*4096,0,0];4];
        points[65]=[[i32::MAX,0,0];4];
        points[66]=[[i32::MIN,0,0];4];
        points[67]=[[0,0,0];4];points[67][3][0]=1000*4096;
        let (jobs,membership)=draw_jobs(&points);
        assert_eq!(jobs.len(),2);
        assert_eq!(jobs[0].count,64);
        // 64 coincident points share one compact 8-byte vertex.
        assert_eq!(jobs[0].length,aligned(64*32+8));
        assert_eq!(jobs[1].first,64);
        assert_eq!(jobs[1].count,6);
        assert_eq!(membership,(0..70).map(|i|i/64).collect::<Vec<_>>());
        assert_eq!(jobs.iter().map(|j|j.count).sum::<usize>(),points.len());
        for (id,job) in jobs.iter().enumerate(){
            for index in job.first..job.first+job.count { assert_eq!(membership[index],id); }
            let mut end=0;
            for span in &job.projections {
                assert_eq!(span.first,end);
                end+=span.count;assert!(end<=job.count);
                let selected=&points[job.first+span.first..job.first+end];
                for axis in 0..3 {
                    let lo=selected.iter().flatten().map(|v|v[axis]>>4).min().unwrap();
                    let hi=selected.iter().flatten().map(|v|v[axis]>>4).max().unwrap();
                    assert_eq!(span.center[axis],((i64::from(lo)+i64::from(hi))>>1) as i32);
                    assert!(span.count==1||i64::from(hi)-i64::from(lo)<=240*256-2);
                }
            }
            assert_eq!(end,job.count);
        }
        let span=|index:usize|{let job=&jobs[membership[index]];job.projections.iter().find(|s|job.first+s.first==index).unwrap()};
        assert_eq!(span(67).count,1); // Oversized quad stays whole.
        assert!(!span(67).compact); // ...and keeps absolute Q12 vertices.
        assert!(span(65).compact&&span(66).compact&&span(64).compact);
        assert_eq!(span(65).center[0],i32::MAX>>4);
        assert_eq!(span(66).center[0],i32::MIN>>4);
    }
    #[test]
    fn packet_vertices_lists_and_blocks_share_exact_page_residency() {
        let scene = crate::ordered_draw::tests::fixture();
        let program = scene.ordered_draw.as_ref().unwrap();
        let resolved = crate::ordered_draw::resolve(&scene, program).unwrap();
        let palette = crate::vertex_colors::Palette::scene(&scene).unwrap();
        let cooked = cook(&scene, program, &resolved, 40, 3, &[], &palette).unwrap();
        assert_eq!(cooked.bytes.len(), 2 * PAGE);
        assert_eq!(cooked.blocks.len(), 1);
        assert_eq!(cooked.primitive_tokens, [40 << 13]);
        assert_eq!(cooked.projections.len(),1);
        assert_eq!(cooked.projections[0].first,40<<13);
        assert_eq!(cooked.projections[0].count,1);
        assert_eq!(cooked.blocks[0].page, 41);
        assert_eq!(cooked.sample_blocks, [0, 0, 0, 0]);
        for sample in 0..3 {
            assert_eq!(
                cooked.sets[sample].keys().copied().collect::<Vec<_>>(),
                [40, 41]
            );
        }
        assert_eq!(cooked.sets[3].keys().copied().collect::<Vec<_>>(), [41]);
        let block = &cooked.projections[0];
        assert!(block.compact);
        assert_eq!(u16::from_le_bytes([cooked.bytes[30], cooked.bytes[31]]) >> 2, 0);
        for corner in 0..4 {
            let at = usize::from(u16::from_le_bytes(
                cooked.bytes[corner * 2..corner * 2 + 2].try_into().unwrap(),
            ));
            let word = |i: usize| u32::from_le_bytes(cooked.bytes[at + i * 4..at + i * 4 + 4].try_into().unwrap());
            let (xy, z) = (word(0), word(1));
            let relative = [xy as u16 as i16, (xy >> 16) as u16 as i16, z as u16 as i16];
            for axis in 0..3 {
                assert!(block.low[axis] <= relative[axis] && relative[axis] <= block.high[axis]);
                let q12 = (i32::from(relative[axis]) + block.center[axis]) * 16
                    + ((z >> (16 + 4 * axis)) & 15) as i32;
                assert_eq!(q12, resolved.points[0][corner][axis]);
            }
        }
        assert!(cooked.header(2).contains("OrderedDrawData ordered_data="));
        assert!(cook(&scene, program, &resolved, 0, 2, &[], &palette).is_err());
    }
    #[test]
    fn compact_vertices_are_lossless_across_signs_and_fractions() {
        let center = [-5, 7, 100_000];
        for point in [[-81, 113, 1_600_000], [-80 - 32768 * 16, 112 + 32767 * 16 + 15, 1_600_015],
                      [-1, -16, 1_599_999], [0, 15, 1_600_001]] {
            let [xy, z] = compact_vertex(point, center);
            let relative = [xy as u16 as i16, (xy >> 16) as u16 as i16, z as u16 as i16];
            for axis in 0..3 {
                assert_eq!((i32::from(relative[axis]) + center[axis]) * 16
                    + ((z >> (16 + 4 * axis)) & 15) as i32, point[axis]);
            }
        }
        assert!(compact_range(&[[[0; 3], [0; 3], [0; 3], [32768 * 16, 0, 0]]], [0; 3]).is_none());
        assert_eq!(compact_range(&[[[-16; 3], [0; 3], [0; 3], [32767 * 16, 0, 0]]], [0; 3]),
                   Some(([-1, -1, -1], [32767, 0, 0])));
    }
    #[test]
    fn cuts_and_periodic_checkpoints_remain_page_local() {
        let mut scene = crate::ordered_draw::tests::fixture();
        let sample = scene.visibility_track.as_ref().unwrap().samples[0].clone();
        scene.visibility_track.as_mut().unwrap().samples = vec![sample; 100];
        scene.visibility_track.as_mut().unwrap().samples[17].cut = true;
        scene.ordered_draw.as_mut().unwrap().samples = vec![vec![3]; 100];
        let program = scene.ordered_draw.as_ref().unwrap();
        let resolved = crate::ordered_draw::resolve(&scene, program).unwrap();
        let palette = crate::vertex_colors::Palette::scene(&scene).unwrap();
        let cooked = cook(&scene, program, &resolved, 0, 3, &[], &palette).unwrap();
        assert_eq!(
            cooked.blocks.iter().map(|b| b.first).collect::<Vec<_>>(),
            [0, 17, 32, 64, 96]
        );
        for (i, &block) in cooked.sample_blocks.iter().enumerate() {
            let b = &cooked.blocks[usize::from(block)];
            assert!(i - usize::from(b.first) < 32);
            let at = b.page as usize * PAGE + usize::from(b.offset);
            assert_eq!(&cooked.bytes[at..at + 4], b"ODR1");
        }
    }
}
