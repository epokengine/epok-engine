//! Deterministic refinement of page layouts over distinct working-set constraints.
//! No geometry is omitted or duplicated. This heuristic is not an optimality proof;
//! the caller must validate the resulting working set against its actual pool.
use std::collections::BTreeMap;

pub fn refine(
    lengths: &[usize],
    source_views: &[Vec<usize>],
    initial: &[usize],
    capacity: usize,
    target: usize,
) -> Vec<usize> {
    if lengths.len() < 2 {
        return initial.to_vec();
    }
    // Identical working sets are one constraint, irrespective of how many
    // camera samples repeat them. Rare graph edges must not be outweighed by
    // hundreds of identical stationary/intro views.
    let mut by_view = BTreeMap::<usize, Vec<usize>>::new();
    for (job, views) in source_views.iter().enumerate() {
        for v in views {
            by_view.entry(*v).or_default().push(job);
        }
    }
    let mut unique = BTreeMap::new();
    for jobs in by_view.into_values() {
        unique.insert(jobs, ());
    }
    let sets: Vec<_> = unique.into_keys().collect();
    let mut views = vec![Vec::new(); lengths.len()];
    for (v, jobs) in sets.iter().enumerate() {
        for &job in jobs {
            views[job].push(v);
        }
    }
    let mut rng = 0x5eeda11u32;
    let mut random = || {
        rng ^= rng << 13;
        rng ^= rng >> 17;
        rng ^= rng << 5;
        rng as usize
    };
    let mut best = initial.to_vec();
    let score = |counts: &[usize]| {
        (
            counts.iter().copied().max().unwrap_or(0),
            counts
                .iter()
                .map(|&n| n.saturating_sub(target).pow(2))
                .sum::<usize>(),
            counts.iter().map(|n| n * n).sum::<usize>(),
        )
    };
    let count_plan = |plan: &[usize]| {
        sets.iter()
            .map(|jobs| {
                let mut pages = jobs.iter().map(|&j| plan[j]).collect::<Vec<_>>();
                pages.sort_unstable();
                pages.dedup();
                pages.len()
            })
            .collect::<Vec<_>>()
    };
    let mut best_score = score(&count_plan(&best));
    if best_score.0 <= target {
        return best;
    }
    let mut heavy: Vec<_> = (0..sets.len()).collect();
    heavy.sort_by_key(|&v| usize::MAX - sets[v].iter().map(|&j| lengths[j]).sum::<usize>());
    for strategy in 0..8 {
        let mut rank = vec![usize::MAX; lengths.len()];
        for (priority, &v) in heavy.iter().enumerate() {
            for &j in &sets[v] {
                rank[j] = rank[j].min(priority);
            }
        }
        let mut order: Vec<_> = (0..lengths.len()).collect();
        order.sort_by_key(|&j| match strategy {
            1 => (
                usize::MAX - views[j].len(),
                *views[j].first().unwrap_or(&usize::MAX),
                j,
            ),
            2 => (views[j].len(), *views[j].first().unwrap_or(&usize::MAX), j),
            3 => (
                usize::MAX - lengths[j],
                *views[j].first().unwrap_or(&usize::MAX),
                j,
            ),
            4..=7 => (rank[j], usize::MAX - lengths[j], j),
            _ => (
                *views[j].first().unwrap_or(&usize::MAX),
                usize::MAX - views[j].len(),
                j,
            ),
        });
        let mut pending = vec![true; lengths.len()];
        let mut plan = vec![0; lengths.len()];
        let mut page = 0;
        for &seed in &order {
            if !pending[seed] {
                continue;
            }
            let mut mask = vec![false; sets.len()];
            let mut used = 0;
            let mut next = Some(seed);
            let mut total = 0;
            while let Some(j) = next {
                pending[j] = false;
                plan[j] = page;
                used += lengths[j];
                for &v in &views[j] {
                    if !mask[v] {
                        mask[v] = true;
                        total += 1;
                    }
                }
                next = order
                    .iter()
                    .copied()
                    .filter(|&k| pending[k] && used + lengths[k] <= capacity)
                    .min_by_key(|&k| {
                        let intersection = views[k].iter().filter(|&&v| mask[v]).count();
                        let union = total + views[k].len() - intersection;
                        (
                            if strategy >= 4 && rank[k] < [1, 4, 16, usize::MAX][strategy - 4] {
                                rank[k]
                            } else {
                                usize::MAX
                            },
                            if strategy == 4 {
                                0
                            } else {
                                (union - intersection) * 1_000_000 / (union + 1)
                            },
                            usize::MAX - lengths[k],
                            k,
                        )
                    });
            }
            page += 1;
        }
        let candidate = score(&count_plan(&plan));
        if candidate < best_score {
            best_score = candidate;
            best = plan;
        }
    }
    for spare in [1, 4, 8] {
        let pages = best.iter().max().unwrap() + 1 + spare;
        let mut plan = best.clone();
        let mut loads = vec![0; pages];
        let mut members = vec![vec![0i32; sets.len()]; pages];
        for (j, &p) in plan.iter().enumerate() {
            loads[p] += lengths[j];
            for &v in &views[j] {
                members[p][v] += 1;
            }
        }
        let mut counts = count_plan(&plan);
        let mut weights = vec![1i64; sets.len()];
        let penalty = |n: usize| n.saturating_sub(target).pow(2) as i64 * 10000 + (n * n) as i64;
        let mut changes = Vec::new();
        for round in 0..80 {
            let bad: Vec<_> = counts
                .iter()
                .enumerate()
                .filter(|(_, n)| **n > target)
                .map(|(v, _)| v)
                .collect();
            for &v in &bad {
                weights[v] += 1;
            }
            for _ in 0..10000 {
                let a = if !bad.is_empty() && random() % 2 == 0 {
                    let v = bad[random() % bad.len()];
                    sets[v][random() % sets[v].len()]
                } else {
                    random() % plan.len()
                };
                let b = random() % plan.len();
                let pa = plan[a];
                let swap = random() % 4 != 0;
                let pb = if swap { plan[b] } else { random() % pages };
                if pa == pb {
                    continue;
                }
                let la = lengths[a];
                let lb = if swap { lengths[b] } else { 0 };
                if loads[pa] - la + lb > capacity || loads[pb] - lb + la > capacity {
                    continue;
                }
                changes.clear();
                let mut delta = 0i64;
                let av = &views[a];
                let bv = if swap { views[b].as_slice() } else { &[] };
                let (mut ai, mut bi) = (0, 0);
                while ai < av.len() || bi < bv.len() {
                    let va = av.get(ai).copied().unwrap_or(usize::MAX);
                    let vb = bv.get(bi).copied().unwrap_or(usize::MAX);
                    if va == vb {
                        ai += 1;
                        bi += 1;
                        continue;
                    }
                    let (v, da) = if va < vb {
                        ai += 1;
                        (va, -1)
                    } else {
                        bi += 1;
                        (vb, 1)
                    };
                    let ma = members[pa][v] + da;
                    let mb = members[pb][v] - da;
                    let n = (counts[v] as i32 + i32::from(ma > 0) + i32::from(mb > 0)
                        - i32::from(members[pa][v] > 0)
                        - i32::from(members[pb][v] > 0)) as usize;
                    delta += (penalty(n) - penalty(counts[v])) * weights[v];
                    changes.push((v, ma, mb, n));
                }
                // Small, deterministic uphill steps escape a full-bin swap
                // local minimum. The best measured plan is retained separately.
                let temperature = if round % 10 < 3 {
                    100000 * (round / 10 + 1)
                } else {
                    0
                };
                if delta > 0
                    && (temperature == 0
                        || delta > temperature
                        || random() % 1000 >= ((temperature - delta) * 1000 / temperature) as usize)
                {
                    continue;
                }
                for &(v, ma, mb, n) in &changes {
                    members[pa][v] = ma;
                    members[pb][v] = mb;
                    counts[v] = n;
                }
                loads[pa] = loads[pa] - la + lb;
                loads[pb] = loads[pb] - lb + la;
                plan[a] = pb;
                if swap {
                    plan[b] = pa;
                }
                let candidate = score(&counts);
                if candidate < best_score {
                    best_score = candidate;
                    best.clone_from(&plan);
                    if best_score.0 <= target {
                        return best;
                    }
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refinement_preserves_capacity_and_is_deterministic() {
        let sizes = vec![8; 24];
        let views = (0..24)
            .map(|i| vec![i % 4, 4 + (i + 1) % 4])
            .collect::<Vec<_>>();
        let initial = (0..24).map(|i| i / 4).collect::<Vec<_>>();
        let result = refine(&sizes, &views, &initial, 32, 2);
        assert_eq!(result, refine(&sizes, &views, &initial, 32, 2));
        assert_eq!(result.len(), sizes.len());
        let mut loads = BTreeMap::new();
        for (&p, &n) in result.iter().zip(&sizes) {
            *loads.entry(p).or_insert(0) += n;
        }
        assert!(loads.values().all(|&n| n <= 32));
        let peak = |plan: &[usize]| {
            (0..8)
                .map(|v| {
                    views
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| s.contains(&v))
                        .map(|(i, _)| plan[i])
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                })
                .max()
                .unwrap()
        };
        assert!(peak(&result) <= peak(&initial));
        // Repeating the same constraints cannot outweigh a rare graph edge.
        let repeated = views
            .iter()
            .map(|vs| vs.iter().flat_map(|&v| [v, v + 8, v + 16]).collect())
            .collect::<Vec<_>>();
        assert_eq!(result, refine(&sizes, &repeated, &initial, 32, 2));
    }
}
