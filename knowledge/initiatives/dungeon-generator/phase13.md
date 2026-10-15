# Phase 13: Seed bench

Delivered on `features/dungeon-minimap-seeds-volumes`.

## What shipped

| Area | Delivered |
| --- | --- |
| Measuring | `dungeon_seeds::sample` lays out one seed of a generator (2D drawing handmade rooms as generated ones, as generation does), evaluates its style and computes the band's budget: actors, quads, collision boxes and navigation nodes in 3D, tiles in 2D. A seed goes over when it passes 64 actors, 3,500 quads, the compound collider's boxes, 512 navigation nodes (or navigation would be skipped) or the tile layer's cells; recipe layouts report their attempts (`Layout.attempts`, from `graph_counted`). |
| Editor | **Test seeds...** under the band's meter opens the Seed bench: from seed and count, Run and Stop with a progress bar, a slice of seeds per frame, the summary, histograms of recipe attempts, quads (the limit drawn as a red line when seeds pass it) or tiles, and actors, a table of the seeds that do not fit or that fit with **Use** (sets the seed and generates, one undo step), and export or copy of the seeds that fit. |
| Export | CSV in `exports/dungeon-seeds/<map>-<generator>-<first>-<count>.csv`: seed, rooms, attempts, actors, quads, collision boxes, navigation nodes, tiles. |
| Automation | `--dungeon-seed-bench N` (headless; from the generator's seed or `--dungeon-seed`) and `--open-dungeon-seeds N` (screenshots). |

## Verification

| Check | Result |
| --- | --- |
| Tests | 12 seeds of the built-in recipe all fit, with attempts and rooms, the map untouched and the same seed measured the same twice; a crowded rooms-and-corridors grid goes over quads on every seed; a missing recipe fails every seed with its reason; the export holds the header and exactly the seeds that fit; histograms put values past the top in the last bucket; the summary names the limits passed. |
| Dungeon template | 500 seeds (1-500): all fit, at most 3,305 quads, 26 actors and 11 recipe attempts; 300 seeds from 1207: all fit, at most 3,421 quads. 11.3 s for 500 seeds headless in a debug build, about 22 ms a seed. |
| Crowded grid | 48x42 cells, 19 rooms of 4-8: 65 of 200 seeds fit; 56x48, 22 rooms: none, over quads and navigation. |

## Limits and follow-ups

- The bench measures the band's budget, not console RAM, which also depends on the rest
  of the project; building checks it.
- Seeds are measured one after another; a parallel run would cut the wait on large
  counts.
