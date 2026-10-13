#pragma once
// World2D navigation (World2D initiative, phase 4): NavigationRegion2D and
// NavigationAgent2D.
//
// A region is either a grid (cooked from a tile layer's solid tiles and the map's static
// bodies, one bit per cell) or a navmesh (a triangulated outline). Regions are cooked in
// world pixels where they were authored. Agents ask the region under them for a path:
//   * grids: A* over cells with integer costs 2 (straight) and 3 (diagonal), never
//     cutting a blocked corner, then line-of-sight pruning through cell centres;
//   * navmeshes: A* over triangles, then the funnel algorithm through shared edges.
// An unreachable target gives the path to the closest reachable point. The A* scratch
// lives in the World2D service, sized by the largest cooked grid; the service plans at
// most one path per simulation step.
//
// MIPS constraints apply: C++20 without RTTI, exceptions, heap or STL containers.
#include "world2d_skeleton.hpp"

namespace epok {
class NavigationRegion2DComponent;
class NavigationAgent2DComponent;

namespace world2d {
inline List<NavigationRegion2DComponent> nav_regions;
inline List<NavigationAgent2DComponent> nav_agents;
inline constexpr size_t nav_path_limit = 32;
inline constexpr size_t nav_triangle_limit = 64;

struct NavGrid2D {
    const uint8_t* blocked = nullptr;  // one bit per cell, row-major
    uint16_t columns = 0, rows = 0;
    Fixed origin[2] = {}, cell[2] = {};
    bool diagonal = true;
};
struct NavMesh2D {
    const Fixed* vertices = nullptr;      // x, y pairs, world pixels
    const uint8_t* triangles = nullptr;   // three vertex indices each, positive area2
    const int8_t* neighbours = nullptr;   // across (v0 v1), (v1 v2), (v2 v0); -1 none
    uint8_t triangle_count = 0;
};
// Grid search scratch, owned by the service.
struct NavScratch {
    uint16_t* g = nullptr;
    uint8_t* info = nullptr;
    uint32_t* heap = nullptr;
    size_t cells = 0, heap_capacity = 0;
};
inline NavScratch nav_scratch;

namespace nav_detail {
inline constexpr int8_t steps[8][2] = {{1, 0}, {0, 1}, {-1, 0}, {0, -1}, {1, 1}, {-1, 1}, {-1, -1}, {1, -1}};
inline int32_t floor_div(int32_t a, int32_t b) { return a >= 0 ? a / b : -((-a + b - 1) / b); }
inline bool walkable(const NavGrid2D& g, int32_t x, int32_t y) {
    if (x < 0 || y < 0 || x >= g.columns || y >= g.rows) return false;
    const uint32_t i = uint32_t(y) * g.columns + uint32_t(x);
    return !(g.blocked[i >> 3] & (1u << (i & 7)));
}
inline void cell_of(const NavGrid2D& g, const Fixed* p, int32_t* c) {
    for (int k = 0; k < 2; ++k) c[k] = floor_div(p[k].raw() - g.origin[k].raw(), g.cell[k].raw() > 0 ? g.cell[k].raw() : 1);
}
inline void centre(const NavGrid2D& g, int32_t x, int32_t y, Fixed* out) {
    out[0] = Fixed(g.origin[0].raw() + x * g.cell[0].raw() + g.cell[0].raw() / 2, Fixed::RAW);
    out[1] = Fixed(g.origin[1].raw() + y * g.cell[1].raw() + g.cell[1].raw() / 2, Fixed::RAW);
}
inline uint16_t octile(int32_t ax, int32_t ay, int32_t bx, int32_t by) {
    const int32_t dx = ax > bx ? ax - bx : bx - ax, dy = ay > by ? ay - by : by - ay;
    const int32_t h = 2 * (dx > dy ? dx : dy) + (dx > dy ? dy : dx);
    return uint16_t(h > 0xffff ? 0xffff : h);
}
inline void heap_push(NavScratch& s, size_t& count, uint32_t key) {
    if (count >= s.heap_capacity) return;  // full: the search degrades, never overflows
    size_t i = count++;
    s.heap[i] = key;
    while (i && s.heap[(i - 1) / 2] > s.heap[i]) {
        const uint32_t t = s.heap[i];
        s.heap[i] = s.heap[(i - 1) / 2];
        s.heap[(i - 1) / 2] = t;
        i = (i - 1) / 2;
    }
}
inline uint32_t heap_pop(NavScratch& s, size_t& count) {
    const uint32_t top = s.heap[0];
    s.heap[0] = s.heap[--count];
    size_t i = 0;
    for (;;) {
        size_t l = 2 * i + 1, r = l + 1, m = i;
        if (l < count && s.heap[l] < s.heap[m]) m = l;
        if (r < count && s.heap[r] < s.heap[m]) m = r;
        if (m == i) break;
        const uint32_t t = s.heap[i];
        s.heap[i] = s.heap[m];
        s.heap[m] = t;
        i = m;
    }
    return top;
}
// Area of (b - a) x (c - a), in raw Q12 squared.
inline int64_t area2(const Fixed* a, const Fixed* b, const Fixed* c) {
    return int64_t(b[0].raw() - a[0].raw()) * (c[1].raw() - a[1].raw()) - int64_t(b[1].raw() - a[1].raw()) * (c[0].raw() - a[0].raw());
}
inline bool same(const Fixed* a, const Fixed* b) { return a[0].raw() == b[0].raw() && a[1].raw() == b[1].raw(); }
}  // namespace nav_detail

// Whether the straight line between two cell centres crosses only walkable cells and
// squeezes past no blocked corner.
inline bool grid_line_of_sight(const NavGrid2D& g, int32_t ax, int32_t ay, int32_t bx, int32_t by) {
    const int32_t dx = bx > ax ? bx - ax : ax - bx, dy = by > ay ? by - ay : ay - by;
    const int32_t sx = bx > ax ? 1 : bx < ax ? -1 : 0, sy = by > ay ? 1 : by < ay ? -1 : 0;
    int32_t x = ax, y = ay, ix = 0, iy = 0;
    while (ix < dx || iy < dy) {
        const int32_t decision = (1 + 2 * ix) * dy - (1 + 2 * iy) * dx;
        if (decision == 0) {
            if (!nav_detail::walkable(g, x + sx, y) || !nav_detail::walkable(g, x, y + sy)) return false;
            x += sx;
            y += sy;
            ++ix;
            ++iy;
        } else if (decision < 0) {
            x += sx;
            ++ix;
        } else {
            y += sy;
            ++iy;
        }
        if (!nav_detail::walkable(g, x, y)) return false;
    }
    return true;
}

// Waypoints from `from` to `to` on a grid (the start excluded), at most `capacity`.
inline size_t grid_path(const NavGrid2D& g, const Fixed* from, const Fixed* to, Fixed (*out)[2], size_t capacity, bool& reached) {
    using namespace nav_detail;
    reached = false;
    NavScratch& s = nav_scratch;
    const size_t cells = size_t(g.columns) * g.rows;
    if (!g.blocked || !cells || cells > s.cells || !s.g || !s.info || !s.heap || !capacity) return 0;
    int32_t start[2], goal[2], raw_goal[2];
    cell_of(g, from, start);
    cell_of(g, to, raw_goal);
    if (!walkable(g, start[0], start[1])) {
        bool found = false;
        for (int32_t ring = 1; ring <= 3 && !found; ++ring)
            for (int32_t dy = -ring; dy <= ring && !found; ++dy)
                for (int32_t dx = -ring; dx <= ring && !found; ++dx)
                    if ((dx == ring || dx == -ring || dy == ring || dy == -ring) && walkable(g, start[0] + dx, start[1] + dy)) {
                        start[0] += dx;
                        start[1] += dy;
                        found = true;
                    }
        if (!found) return 0;
    }
    goal[0] = raw_goal[0] < 0 ? 0 : raw_goal[0] >= g.columns ? g.columns - 1 : raw_goal[0];
    goal[1] = raw_goal[1] < 0 ? 0 : raw_goal[1] >= g.rows ? g.rows - 1 : raw_goal[1];
    for (size_t i = 0; i < cells; ++i) {
        s.g[i] = 0xffff;
        s.info[i] = 0;
    }
    // info: bits 0-2 parent step, bit 3 has a parent, bit 4 closed.
    const uint32_t first = uint32_t(start[1]) * g.columns + uint32_t(start[0]);
    const uint32_t target = uint32_t(goal[1]) * g.columns + uint32_t(goal[0]);
    s.g[first] = 0;
    size_t open = 0;
    heap_push(s, open, (uint32_t(octile(start[0], start[1], goal[0], goal[1])) << 16) | first);
    uint32_t best = first;
    uint16_t best_h = octile(start[0], start[1], goal[0], goal[1]);
    bool found = false;
    while (open) {
        const uint32_t cell = heap_pop(s, open) & 0xffff;
        if (s.info[cell] & 16) continue;
        s.info[cell] |= 16;
        const int32_t x = int32_t(cell % g.columns), y = int32_t(cell / g.columns);
        const uint16_t h = octile(x, y, goal[0], goal[1]);
        if (h < best_h) {
            best_h = h;
            best = cell;
        }
        if (cell == target) {
            found = true;
            break;
        }
        for (uint8_t d = 0; d < 8; ++d) {
            const int32_t nx = x + steps[d][0], ny = y + steps[d][1];
            if (d >= 4 && (!g.diagonal || !walkable(g, nx, y) || !walkable(g, x, ny))) continue;
            if (!walkable(g, nx, ny)) continue;
            const uint32_t n = uint32_t(ny) * g.columns + uint32_t(nx);
            if (s.info[n] & 16) continue;
            const uint32_t cost = uint32_t(s.g[cell]) + (d >= 4 ? 3 : 2);
            if (cost >= s.g[n] || cost > 0xfffe) continue;
            s.g[n] = uint16_t(cost);
            s.info[n] = uint8_t(8 | d);
            const uint32_t f = cost + octile(nx, ny, goal[0], goal[1]);
            heap_push(s, open, ((f > 0xffff ? 0xffff : f) << 16) | n);
        }
    }
    // The cell path, end first, written over the cost array (no longer needed).
    uint32_t end = found ? target : best;
    size_t length = 0;
    for (uint32_t c = end;;) {
        s.g[length++] = uint16_t(c);
        if (c == first || !(s.info[c] & 8) || length >= cells) break;
        const uint8_t d = s.info[c] & 7;
        const int32_t x = int32_t(c % g.columns) - steps[d][0], y = int32_t(c / g.columns) - steps[d][1];
        c = uint32_t(y) * g.columns + uint32_t(x);
    }
    for (size_t i = 0; i < length / 2; ++i) {
        const uint16_t t = s.g[i];
        s.g[i] = s.g[length - 1 - i];
        s.g[length - 1 - i] = t;
    }
    // Line-of-sight pruning: from each waypoint, the farthest visible cell (64 ahead).
    size_t count = 0, at = 0;
    while (at + 1 < length && count < capacity) {
        size_t next = at + 1;
        const size_t window = at + 64 < length - 1 ? at + 64 : length - 1;
        const int32_t ax = s.g[at] % g.columns, ay = s.g[at] / g.columns;
        for (size_t k = window; k >= at + 2; --k)
            if (grid_line_of_sight(g, ax, ay, s.g[k] % g.columns, s.g[k] / g.columns)) {
                next = k;
                break;
            }
        centre(g, s.g[next] % g.columns, s.g[next] / g.columns, out[count++]);
        at = next;
    }
    reached = found && walkable(g, raw_goal[0], raw_goal[1]);
    if (reached) {
        if (!count) ++count;
        out[count - 1][0] = to[0];
        out[count - 1][1] = to[1];
    }
    return count;
}

namespace nav_detail {
inline void corner(const NavMesh2D& m, uint8_t t, int k, Fixed* out) {
    const uint8_t v = m.triangles[t * 3 + k];
    out[0] = m.vertices[v * 2];
    out[1] = m.vertices[v * 2 + 1];
}
inline bool contains(const NavMesh2D& m, uint8_t t, const Fixed* p) {
    Fixed c[3][2];
    for (int k = 0; k < 3; ++k) corner(m, t, k, c[k]);
    // A hair outside an edge still counts: shared edges belong to both triangles.
    for (int e = 0; e < 3; ++e)
        if (area2(c[e], c[(e + 1) % 3], p) < -int64_t(4096) * 16) return false;
    return true;
}
inline void centroid(const NavMesh2D& m, uint8_t t, Fixed* out) {
    Fixed c[3][2];
    for (int k = 0; k < 3; ++k) corner(m, t, k, c[k]);
    for (int k = 0; k < 2; ++k) out[k] = Fixed((c[0][k].raw() + c[1][k].raw() + c[2][k].raw()) / 3, Fixed::RAW);
}
inline void closest_on_segment(const Fixed* p, const Fixed* a, const Fixed* b, Fixed* out) {
    const int64_t dx = b[0].raw() - a[0].raw(), dy = b[1].raw() - a[1].raw();
    const int64_t length = dx * dx + dy * dy;
    int64_t t = length > 0 ? ((int64_t(p[0].raw() - a[0].raw()) * dx + int64_t(p[1].raw() - a[1].raw()) * dy) * 4096) / length : 0;
    t = t < 0 ? 0 : t > 4096 ? 4096 : t;
    out[0] = Fixed(a[0].raw() + int32_t(dx * t / 4096), Fixed::RAW);
    out[1] = Fixed(a[1].raw() + int32_t(dy * t / 4096), Fixed::RAW);
}
// The triangle holding `p` (true), or the closest triangle and point on it (false).
inline bool locate(const NavMesh2D& m, const Fixed* p, uint8_t& triangle, Fixed* point) {
    for (uint8_t t = 0; t < m.triangle_count; ++t)
        if (contains(m, t, p)) {
            triangle = t;
            point[0] = p[0];
            point[1] = p[1];
            return true;
        }
    int64_t best = -1;
    for (uint8_t t = 0; t < m.triangle_count; ++t) {
        Fixed c[3][2];
        for (int k = 0; k < 3; ++k) corner(m, t, k, c[k]);
        for (int e = 0; e < 3; ++e) {
            Fixed q[2];
            closest_on_segment(p, c[e], c[(e + 1) % 3], q);
            const int64_t dx = q[0].raw() - p[0].raw(), dy = q[1].raw() - p[1].raw();
            const int64_t d = dx * dx + dy * dy;
            if (best < 0 || d < best) {
                best = d;
                triangle = t;
                point[0] = q[0];
                point[1] = q[1];
            }
        }
    }
    return false;
}
}  // namespace nav_detail

// Waypoints from `from` to `to` on a navmesh (the start excluded), at most `capacity`.
inline size_t mesh_path(const NavMesh2D& m, const Fixed* from, const Fixed* to, Fixed (*out)[2], size_t capacity, bool& reached) {
    using namespace nav_detail;
    reached = false;
    if (!m.triangle_count || m.triangle_count > nav_triangle_limit || !capacity) return 0;
    uint8_t start_t = 0, goal_t = 0;
    Fixed start[2], goal[2];
    locate(m, from, start_t, start);
    const bool on_mesh = locate(m, to, goal_t, goal);
    int32_t g[nav_triangle_limit];
    int8_t parent[nav_triangle_limit];
    bool closed[nav_triangle_limit];
    for (uint8_t t = 0; t < m.triangle_count; ++t) {
        g[t] = 0x7fffffff;
        parent[t] = -1;
        closed[t] = false;
    }
    g[start_t] = 0;
    bool found = start_t == goal_t;
    while (!found) {
        int32_t pick = -1;
        int64_t pick_f = 0;
        for (uint8_t t = 0; t < m.triangle_count; ++t) {
            if (closed[t] || g[t] == 0x7fffffff) continue;
            Fixed c[2];
            centroid(m, t, c);
            const int64_t f = int64_t(g[t]) + length_2d(goal[0] - c[0], goal[1] - c[1]).raw();
            if (pick < 0 || f < pick_f) {
                pick = t;
                pick_f = f;
            }
        }
        if (pick < 0) break;
        closed[pick] = true;
        if (pick == goal_t) {
            found = true;
            break;
        }
        Fixed here[2];
        centroid(m, uint8_t(pick), here);
        for (int e = 0; e < 3; ++e) {
            const int8_t n = m.neighbours[pick * 3 + e];
            if (n < 0 || closed[n]) continue;
            Fixed there[2];
            centroid(m, uint8_t(n), there);
            const int32_t cost = g[pick] + length_2d(there[0] - here[0], there[1] - here[1]).raw();
            if (cost < g[n]) {
                g[n] = cost;
                parent[n] = int8_t(pick);
            }
        }
    }
    if (!found) return 0;
    // The corridor, start first.
    uint8_t corridor[nav_triangle_limit];
    size_t length = 0;
    for (int32_t t = goal_t; t >= 0 && length < nav_triangle_limit; t = t == start_t ? -1 : parent[t]) corridor[length++] = uint8_t(t);
    for (size_t i = 0; i < length / 2; ++i) {
        const uint8_t t = corridor[i];
        corridor[i] = corridor[length - 1 - i];
        corridor[length - 1 - i] = t;
    }
    // Portals (left, right): the start, each shared edge, the goal.
    Fixed portals[nav_triangle_limit + 1][2][2];
    size_t portal_count = 0;
    portals[0][0][0] = portals[0][1][0] = start[0];
    portals[0][0][1] = portals[0][1][1] = start[1];
    portal_count = 1;
    for (size_t i = 0; i + 1 < length; ++i) {
        const uint8_t a = corridor[i], b = corridor[i + 1];
        int e = 0;
        while (e < 3 && m.neighbours[a * 3 + e] != int8_t(b)) ++e;
        if (e == 3) return 0;
        // Triangles wind with positive area2, so the edge's second vertex is on the left.
        corner(m, a, (e + 1) % 3, portals[portal_count][0]);
        corner(m, a, e, portals[portal_count][1]);
        ++portal_count;
    }
    portals[portal_count][0][0] = portals[portal_count][1][0] = goal[0];
    portals[portal_count][0][1] = portals[portal_count][1][1] = goal[1];
    ++portal_count;
    // The funnel.
    size_t count = 0;
    Fixed apex[2] = {start[0], start[1]}, left[2] = {start[0], start[1]}, right[2] = {start[0], start[1]};
    size_t apex_i = 0, left_i = 0, right_i = 0;
    auto emit = [&](const Fixed* p) {
        if (count && same(out[count - 1], p)) return;
        if (count < capacity) {
            out[count][0] = p[0];
            out[count][1] = p[1];
            ++count;
        }
    };
    for (size_t i = 1; i < portal_count;) {
        const Fixed* l = portals[i][0];
        const Fixed* r = portals[i][1];
        if (area2(apex, right, r) >= 0) {
            if (same(apex, right) || area2(apex, left, r) < 0) {
                right[0] = r[0];
                right[1] = r[1];
                right_i = i;
            } else {
                emit(left);
                apex[0] = right[0] = left[0];
                apex[1] = right[1] = left[1];
                apex_i = right_i = left_i;
                i = apex_i + 1;
                continue;
            }
        }
        if (area2(apex, left, l) <= 0) {
            if (same(apex, left) || area2(apex, right, l) > 0) {
                left[0] = l[0];
                left[1] = l[1];
                left_i = i;
            } else {
                emit(right);
                apex[0] = left[0] = right[0];
                apex[1] = left[1] = right[1];
                apex_i = left_i = right_i;
                i = apex_i + 1;
                continue;
            }
        }
        ++i;
    }
    emit(goal);
    reached = on_mesh;
    return count;
}
}  // namespace world2d

// ---- NavigationRegion2D ------------------------------------------------------------------
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=navigation2d, Id="1aa4252c-57a2-4d29-b92c-33a976a8ff5f") NavigationRegion2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("1aa4252c-57a2-4d29-b92c-33a976a8ff5f");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::nav_regions.add(this); }
    ~NavigationRegion2DComponent() override { world2d::nav_regions.remove(this); }
    world2d::Link<NavigationRegion2DComponent> link2d;

    world2d::NavGrid2D grid;
    world2d::NavMesh2D mesh;
    bool is_mesh = false, enabled = true;

    EPOK_FUNCTION(BlueprintCallable, Id="2d4081e2-6e76-48cd-bd9e-6a37f237bdae") void set_enabled(bool value) { enabled = value; }
    EPOK_FUNCTION(BlueprintPure, Id="e408cfb1-f137-4e9b-a3b4-47f2ad1c46fd") bool is_enabled() const { return enabled; }
    // Whether a world point is inside the region and free to walk on.
    EPOK_FUNCTION(BlueprintPure, Id="80a37d7f-3c14-4e39-b8e7-a1e264b090fa") bool is_walkable(Fixed x, Fixed y) const {
        const Fixed p[2] = {x, y};
        if (is_mesh) {
            for (uint8_t t = 0; t < mesh.triangle_count; ++t)
                if (world2d::nav_detail::contains(mesh, t, p)) return true;
            return false;
        }
        if (!grid.blocked) return false;
        int32_t c[2];
        world2d::nav_detail::cell_of(grid, p, c);
        return world2d::nav_detail::walkable(grid, c[0], c[1]);
    }
    // Whether the region is the one to plan from `p`: inside the grid, or on the mesh.
    bool covers(const Fixed* p) const {
        if (!enabled) return false;
        if (is_mesh) return is_walkable(p[0], p[1]);
        if (!grid.blocked) return false;
        int32_t c[2];
        world2d::nav_detail::cell_of(grid, p, c);
        return c[0] >= 0 && c[1] >= 0 && c[0] < grid.columns && c[1] < grid.rows;
    }
    size_t find_path(const Fixed* from, const Fixed* to, Fixed (*out)[2], size_t capacity, bool& reached) const {
        return is_mesh ? world2d::mesh_path(mesh, from, to, out, capacity, reached)
                       : world2d::grid_path(grid, from, to, out, capacity, reached);
    }
};

// ---- NavigationAgent2D -------------------------------------------------------------------
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=navigation2d, Id="a9d058fe-bf11-4aee-834e-13dae4281bcc") NavigationAgent2DComponent : public ActorComponent {
public:
    static constexpr uint64_t static_class_id = detail::compact_class_id("a9d058fe-bf11-4aee-834e-13dae4281bcc");
    uint64_t class_id() const override { return m_runtime_class_id ? m_runtime_class_id : static_class_id; }
    void level_attached() override { world2d::nav_agents.add(this); }
    ~NavigationAgent2DComponent() override { world2d::nav_agents.remove(this); }
    world2d::Link<NavigationAgent2DComponent> link2d;

    // Authored.
    ObjectId target_actor;
    Fixed max_speed = 64.0, arrive_distance = 4.0, repath_interval = 0.5;
    bool drives_body = true;
    // State.
    Fixed target[2] = {}, planned_for[2] = {}, velocity[2] = {}, clock = 0.0;
    Fixed path[world2d::nav_path_limit][2] = {};
    uint8_t path_count = 0, path_index = 0;
    bool has_target = false, dirty = false, reachable = false, finished = true, arrived = false;

    EPOK_FUNCTION(BlueprintCallable, Id="df839829-42e3-4202-8495-2ca9e2cea348") void set_target_position(Fixed x, Fixed y) {
        target_actor = ObjectId{};
        target[0] = x;
        target[1] = y;
        has_target = true;
        dirty = true;
        finished = false;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="393bddb0-9e97-4bb6-842c-248c2c31c21c") void set_target_actor(ObjectId actor) {
        target_actor = actor;
        has_target = actor.valid();
        dirty = has_target;
        finished = !has_target;
    }
    EPOK_FUNCTION(BlueprintPure, Id="5cde0b6b-5193-4940-8d6a-7e8a5b4c7a6c") ObjectId get_target_actor() const { return target_actor; }
    EPOK_FUNCTION(BlueprintPure, Id="ad181c61-7f9d-4dfa-a40b-6a04e13fc703") Fixed get_target_x() const { return target[0]; }
    EPOK_FUNCTION(BlueprintPure, Id="861cec66-b363-46a4-87f0-1a93e8ae2ba0") Fixed get_target_y() const { return target[1]; }
    // Stops following: no target, no path, no velocity.
    EPOK_FUNCTION(BlueprintCallable, Id="5b3ff7bc-2eae-46a1-a5d1-e09847f0b364") void stop() {
        has_target = false;
        target_actor = ObjectId{};
        path_count = path_index = 0;
        finished = true;
        velocity[0] = velocity[1] = 0.0;
    }
    EPOK_FUNCTION(BlueprintPure, Id="73bb3809-f1ac-4b1f-af64-2d2a85533e15") bool is_navigation_finished() const { return finished; }
    EPOK_FUNCTION(BlueprintPure, Id="e3b735b3-a7ab-4487-a216-50ecc89b5358") bool is_target_reachable() const { return reachable; }
    // True once when the agent reaches a reachable target.
    EPOK_FUNCTION(BlueprintCallable, Id="fc5583bb-3b93-40b3-839e-41493350f015") bool take_arrived() {
        const bool was = arrived;
        arrived = false;
        return was;
    }
    EPOK_FUNCTION(BlueprintPure, Id="85cd8e14-dec3-4cec-99c3-676d3e6fbfe5") Fixed get_next_x() const { return path_index < path_count ? path[path_index][0] : target[0]; }
    EPOK_FUNCTION(BlueprintPure, Id="0bfe0349-b779-4fb2-8e82-b345742211fc") Fixed get_next_y() const { return path_index < path_count ? path[path_index][1] : target[1]; }
    EPOK_FUNCTION(BlueprintPure, Id="f68e811b-2a88-40b0-a6d4-e85f7671b34a") Fixed get_velocity_x() const { return velocity[0]; }
    EPOK_FUNCTION(BlueprintPure, Id="70f2e5ec-4889-4cec-a550-597230e70922") Fixed get_velocity_y() const { return velocity[1]; }
    EPOK_FUNCTION(BlueprintPure, Id="db251a32-0a42-4e99-a428-273e7b9f4d63") int32_t get_path_point_count() const { return path_count - path_index; }
    EPOK_FUNCTION(BlueprintCallable, Id="9e5a7ae4-6447-4b3d-baff-082622520eb1") void set_max_speed(Fixed value) { max_speed = value.raw() < 0 ? Fixed(0.0) : value; }
    EPOK_FUNCTION(BlueprintPure, Id="c3a00f54-6810-4a9e-be92-b46ce6cd866e") Fixed get_max_speed() const { return max_speed; }
};
}  // namespace epok
