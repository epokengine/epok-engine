# World and camera coordinates

The PSX renderer no longer imposes an absolute-world +/-128 position limit.
The relevant representations are different:

- Actor translations and chunk origins: signed 32-bit Q12.
- Stored chunk-local vertices: signed 16-bit or 32-bit Q12, selected losslessly per chunk.
- GTE projection input: signed 16-bit Q8; chunk origins that do not fit are
  folded into the translation matrix instead of the vertex input.
- Near/far planes: camera-relative, not absolute map coordinates.

`CameraSpace` retains the camera's world origin separately from its inverse
rotation/scale basis. It computes `basis * (world_position - camera_origin)`.
Rotating two large world positions independently and subtracting afterwards
introduces origin-dependent fixed-point rounding. Subtraction and accumulation
are widened; out-of-range final results saturate rather than wrap. Each product
retains PsyQo's truncation-toward-zero rule. This is render-space preparation,
not a rewrite of scene or mesh coordinates.

Meshes, streamed-page selection, sprites, particles, blob projections and
`Camera.project` share this camera representation. The renderer takes the origin
from the same interpolated world matrix cache used for objects. Hierarchical
inverse bases still compose inverse local transforms in reverse ancestry order;
they do not invert the forward matrix with a new floating-point algorithm.

The authoring +/-10000 local-transform validation, scale/basis budget and
signed-32-bit inherited-position check remain. This change is not a guarantee
that arbitrarily compounded scales or script-generated transforms cannot
overflow elsewhere, nor does it extend draw distance or change GTE clipping.

`tests/runtime/camera_space.cpp` uses real PsyQo arithmetic and the production GTE
matrix uploader against register stubs. It checks 10000 exact translated pairs,
non-diagonal inverse bases, Main's actual distant starting origin and wide
subtraction/cancellation. It tests register preparation, not GTE execution or
rasterization; emulator tests are required separately.

## Imported coordinate handedness

Camera `Mirror X` (`camera_mirror_x` in the Camera3D component; `mirror_x` in
runtime CameraSettings) reflects the horizontal **projection**, not actor
transforms. It defaults to false, leaves depth/near/far planes unchanged, and
adjusts face culling to retain the same front surfaces. It applies to PSX mesh,
sprite/particle and blob rendering and Camera.project, and to Native PC Play.
HUD coordinates and the free Scene observer are unaffected. A mirrored camera
does not enable baked visibility or streaming simulation in Scene View.

This allows an importer to bake a coordinate reflection into vertex positions,
reverse winding together with per-corner data, conjugate camera rotations and
request the matching projection without negative actor scales. Positive-scale
validation remains in force. Camera scale still uses ordinary Q12 arithmetic;
real-number import proofs do not establish pixel-identical fixed-point output.

## Native PC game projection

Native Play uses the same normalized focal ratio as the PSX runtime:
horizontal pixels = `(width/2) * cot(fov/2)` and vertical pixels =
`(height*2/3) * cot(fov/2)`. All supported modes therefore use a 4:3
**projection ratio**, not the raw framebuffer's width/height. The separate
Game-panel Fit/Stretch/Integer setting still controls presentation.

Previously Native Play reused the square-pixel editor observer's target aspect
and converted FOV with an incorrect zoom factor. At 512x240 and 90 degrees it
produced focal lengths (192,192), versus the runtime's (256,160). It now supplies
the game projection ratio explicitly and converts the vertical focal length to
the orbit shader's scale. Scene View, asset/effect previews and their camera
controls retain their existing projection; no streaming filter is enabled.

`native_projection_scale_matches_runtime_across_ntsc_modes` checks the focal
conversion without a GPU. The ignored graphics test
`native_game_projection_matches_runtime_focals_in_every_video_mode` renders
through `render_game` and reads back 120 images: ten modes, six FOV inputs
(including both clamps), and normal/mirrored cameras. It compares a known
quad's screen bounds against the runtime focal equations within one pixel.
It opens no window and does not capture the desktop.

This is a focal/projection correction, not a replacement for the GTE rasterizer
or proof of complete native/PSX camera parity. The native orbit conversion still
does not reproduce arbitrary camera roll or inherited non-uniform camera scale,
and its floating-point projection does not emulate GTE rounding. The PSX code,
framebuffer dimensions, clipping and frame costs are unchanged by this fix.

## Parameterized target-aim kernel

`camera_aim.hpp` provides a standalone integer orientation blend. Its input
angles use 4096 steps per turn, weights use 256 = 1, and all inputs must share
one coordinate convention. Per-axis weights blend the biased target direction
toward an authored base orientation. Optional previous angles apply a clamped
delta before weighted smoothing; asymmetric limits apply relative to the base
after smoothing. The defaults retain the base orientation.

Negative products round down by arithmetic shift. Interpolation chooses the
negative direction on a half-turn tie, while the branch-based final limiter
preserves the sign of a positive half-turn. These details are tested rather
than replaced with floating-point lerp. The helper has no hidden state, heap,
trigonometric table, global game constants or page-selection logic. Callers
own the target-direction calculation, update cadence and cut/reset policy.

The exhaustive host test covers 16,777,216 angle pairs; the PSX cross-compiler
test checks O2/O3 output with no external helpers, divides or floating-point
instructions. This is not a timing measurement. The private reference study also feeds
117 independently replayed normal-mode updates through this exact header,
matching all observed angles. Special pitch effects/transition overrides are
outside that adapter's scope and are explicitly rejected.

### Optional rail integration

`RailNavigationData::aim` is optional and defaults to null. Old authored tracks
retain their existing behavior. An aim-enabled track cooks validated per-sample
weights, target bias, asymmetric limits and smoothing flags into deduplicated
parameter tables. `rail_camera_aim.hpp` owns the independent fixed-cadence state;
`track_camera_pose.hpp` shares pose interpolation and application between PSX
and Native Play. Native Play now advances authored rails too, with pages always
ready. This is gameplay playback, not a Scene View streaming filter.

The generic rig uses positive pitch up and yaw zero toward -Z; the adapter
converts Epok's Q12-degree Euler convention and supports an X-reflected rig.
`camera_direction.hpp` uses a mathematically generated atan table, not bytes
from an imported game. The 1..120 Hz aim cadence excludes paused elapsed time,
bounds catch-up to 100 ms, resets on authored load cuts and clamps the shown
pose between aim ticks without advancing smoothing. Moving target/base inputs
are sampled per render: equal fixed update counts alone do not prove cadence
independence for changing inputs.

Host tests cover atan directions, extreme coordinates, pose overflow/wrap,
fixed-cadence smoothing, reflected rigs, limits, cuts and scene reset. MIPS
O2/O3 checks find no external helpers in the controller, but it uses integer
division; the earlier no-divide claim applies only to the angle-blend kernel.
The ignored Native integration test runs 240 paired frames: raising a target
changes pitch without changing its camera rail position.

The private diagnostic candidate explicitly selects its RailProbe target and
bakes 1650 samples. On 117 verified original updates, independent direction
and one-step blending differ by at most one turn12 step per axis (0.087890625
degrees). 79 directions and 108 blended outputs are exact. Each blend starts
from the observed previous angle; this is not accumulated-drift, fractional
property interpolation, complete camera-mode or performance equivalence.
Main itself has not been migrated to this diagnostic actor/controller.
