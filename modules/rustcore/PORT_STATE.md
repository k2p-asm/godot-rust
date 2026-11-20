# Port state — Godot 4.7.2 → Rust

Checkpoint written by the agent so any later session can resume without re-deriving
context. Updated as work lands; every number below is measured, not estimated.

## Layout

```
C:\Users\k2p\godot-rust\                 forked engine tree (4.7.2-stable, unmodified C++)
    modules\rustcore\                    the Rust port lives here
        Cargo.toml                       crate `godot_core_rs`, MIT
        src\math\{funcs,pcg,vector2,vector3,color,rect2,plane,aabb,
                  quaternion,basis,transform_3d}.rs
        src\templates\hashfuncs.rs
        parity\                          C++ differential harness
            dump.cpp                     prints engine-computed values
            run.cmd                      builds dump.exe and regenerates the fixture
            rng_stub.cpp, string_stub.cpp  link stubs (documented, not behaviour changes)
        tests\parity.rs                  compares Rust against the C++ fixture
        tests\fixtures\cpp_math_parity.txt
```

## Engine-level proof (verified 19/11 06:00)

```
scons platform=windows target=editor -j 8 accesskit=no d3d12=no
  -> scons: done building targets.  (full build 2h22m, incremental relink 21s)
  -> bin\godot.windows.editor.x86_64.exe  165,334,528 bytes

.\bin\godot.windows.editor.x86_64.console.exe --headless --quit
  -> Rust core module: ABI 1, dot = 32.0, normalized z = 1.000
```

`modules\rustcore\verify_link.cmd` automates that check and exits 0.

## Commands

```powershell
cd C:\Users\k2p\godot-rust\modules\rustcore
cargo test                     # unit + differential tests
cargo build --release
modules\rustcore\parity\run.cmd  # regenerate the C++ fixture (needs MSVC)
```

## Git

The tree is a git repository whose first parent is the upstream
`4.7.2-stable` tag (commit `ed1daf0`), branch `rust-port`. `git status` on a
clean tree shows nothing, which proves the rest of the engine is byte-identical
to upstream: only `modules/rustcore/` is ours.

```powershell
git status
git add -A
git commit -m "Rust port: <unit>"
git push origin rust-port
```

`modules/rustcore/.gitignore` excludes `target/`, `parity/*.exe` and the
`scons_check/` build products, so no binaries enter history.

## Verified state (last run)

- `cargo test`: **1064 unit tests + 34 differential tests, 0 failures.**
- Differential coverage: Vector2, Vector2i-level ops, Vector3, scalar math
  (30+ functions), RandomPCG stream, AABB, Plane, Rect2, Basis, Quaternion,
  Transform3D, all six Euler orders (round-trip and arbitrary rotations),
  Vector2i/Vector3i, Vector4, Transform2D, axis-angle extraction, filled
  polygon tests, integer snapping (`v2i.snap`, `v3i.snap`, `v4i.snap`) and
  axis indices. Hash values compared bit-exactly (`v2.hash`, `v3.hash`,
  `aabb.hash`, `v3i.hash`, `v2i.hash`), PCG stream compared word-by-word.
- Fixture generated from the fork's own C++ sources compiled with MSVC 19.x,
  `/O2`, `std:c++20`. The Rust side matches within 1e-5 relative for `f32`
  paths and 1e-12 for `f64` paths.

## Benchmarks (first numbers, both languages, same loops)

`parity/bench.cpp` vs `examples/bench.rs`, 2,000,000 elements, `-O2` / `--release`, best of 3:

| Loop | C++ | Rust |
|---|---|---|
| `Vector3::dot` | 1.45 ns/op | 1.11-1.45 ns/op |
| `Transform3D::xform` | 2.41 ns/op | 1.47-1.95 ns/op |
| `Vector3::normalized().length()` | 10.2 ns/op | 4.7-7.2 ns/op |

Caveat measured, not assumed: the 2M accumulate ends with slightly different
checksums (1999999.96 vs 2000000.00), so single operations can differ in the last
ulp. Hashes and the PCG stream are bit-exact; other float math is verified to
1e-5 relative, not bit-identical.

## SCons integration notes (learned the hard way)

- `modules/<name>/config.py` is a **plain Python module**, not a SConscript:
  `Dir()`, `Glob()` and `File()` do not exist there. Use `env.Dir`, `env.Glob`,
  `env.File`.
- Godot's module init hooks take a parameter:
  `void initialize_<name>_module(ModuleInitializationLevel p_level)`, declared
  with `#include "modules/register_module_types.h"` (generated at build time).
- `vformat` lives in `core/variant/variant.h`.
- System libraries must go into `LINKFLAGS`, not `LIBS`: Godot decorates `LIBS`
  entries with its platform suffix, producing
  `ws2_32.windows.editor.x86_64.lib` and `LNK1181`.
- A static library whose symbols are never referenced is dropped by the linker.
  `modules/rustcore/register_types.cpp` exists to call into Rust at startup,
  which is also the proof that the crate is really inside the binary.
- Build command used: `python -m SCons platform=windows target=editor -j 8
  accesskit=no d3d12=no` (accesskit and d3d12 need extra downloads).

## Next step, ready to execute (Color OK-HSL)

`thirdparty/misc/ok_color.h` (18.9 KB, header-only, namespace `ok_color`) can be
ported **without** touching `color.cpp`, which is what blocks Color parity:

1. New `parity/okcolor_dump.cpp`: only `#include "thirdparty/misc/ok_color.h"`,
   prints `ok_color::okhsl_to_srgb({h,s,l})` and `srgb_to_okhsl({r,g,b})` for a
   fixed table. Compiles alone, no `String`/`HashMap` dependency.
2. New `src/math/ok_color.rs`, porting in this order (each is needed by the next):
   `clamp`, `sgn`, `srgb_transfer_function(_inv)`, `linear_srgb_to_oklab`,
   `oklab_to_linear_srgb`, `compute_max_saturation`, `find_cusp`,
   `find_gamut_intersection` (both overloads), `toe`, `toe_inv`, `to_ST`,
   `get_ST_mid`, `get_Cs`, `okhsl_to_srgb`, `srgb_to_okhsl`.
3. Wire `Color::set_ok_hsl` / `from_ok_hsl` (and the OK-HSV pair) to that module.
4. Add the dump to `parity/run.cmd` and a `ok_color_matches_cpp` test.

Constants must be copied digit for digit: `compute_max_saturation` switches
coefficient sets by region (red/green/blue branches) and `get_ST_mid` is an
optimisation-fitted polynomial - any "cleanup" changes the result.
## Ported so far (~6.2k lines of Rust)

| Unit | C++ source | Status |
|---|---|---|
| `math/expression.rs` | `core/math/expression.*` (subset) | tokenizer (numbers/strings/idents/operators), precedence-climbing parser, `Variant`-based evaluator: arithmetic, `%`, `**`, comparisons, `and/or/not` with short-circuit, bitwise/shifts, indexing, array/dict literals, variables, and ~25 math builtins. No `self`/member access, assignment or `in`. **Non-differential** |
| `math/funcs.rs` | `core/math/math_funcs.h/.cpp` | complete except RNG globals |
| `math/pcg.rs` | `core/math/random_pcg.*`, `thirdparty/misc/pcg.*` | complete, bit-exact |
| `math/vector2.rs` | `core/math/vector2.*` | complete |
| `math/vector2i.rs` | `core/math/vector2i.*` | complete (`snapped`/`snappedi` verified against the C++ `Math::snapped`) |
| `math/vector3.rs` | `core/math/vector3.*` | complete |
| `math/vector3i.rs` | `core/math/vector3i.*` | complete (`snapped`/`snappedi` verified against the C++ `Math::snapped`) |
| `math/rect2.rs` | `core/math/rect2.*` | core done; `intersects_transformed` and `intersects_filled_polygon` verified |
| `math/plane.rs` | `core/math/plane.*` | complete |
| `math/aabb.rs` | `core/math/aabb.*` | complete |
| `math/color.rs` | `core/math/color.*` | numeric + `html` + OK-HSL/HSV + named colors (146, table generated) |
| `math/ok_color.rs` | `thirdparty/misc/ok_color.h` | Okhsl/Okhsv both directions, verified against the header via `parity/okcolor_dump.cpp` |
| `math/quaternion.rs` | `core/math/quaternion.*` | complete: rotation math, `get_euler` (6 orders, differential), `log`/`exp`, `slerp`/`slerpni`, spherical cubic interpolate (+ in-time), all verified against `quaternion.cpp` |
| `math/basis.rs` | `core/math/basis.*` | core + all 6 Euler orders + axis-angle + `diagonalize`, `rotate_sh`, `rotref_posscale_decomposition` |
| `math/transform_3d.rs` | `core/math/transform_3d.*` | complete incl. batch `xform_array`/`xform_array_inv` (verified) |
| `math/transform_interpolator.rs` | `core/math/transform_interpolator.*` | `find_method`/`_test_basis` (lerp vs slerp vs scaled slerp, shear and zero-scale fallbacks), basis and transform interpolation, 2D handedness flip, checksum. **Differential**: `ti.*` keys generated from `transform_interpolator.cpp` |
| `math/transform_2d.rs` | `core/math/transform_2d.*` | complete |
| `scene/node_2d.rs` | `scene/2d/node_2d.*` (data half) | position/rotation/scale/skew with the engine's lazy decomposition of `set_transform`, zero-scale clamp, `rotate`/`translate`, relative transform. `set_rotation_scale_and_skew` is differential (`t2.rs_skew*`). Parenting and notifications need the object layer |
| `scene/canvas_item.rs` | `scene/main/canvas_item.*` (data half) | visibility with the cached `parent_visible_in_tree`, `modulate`/`self_modulate` + ancestor composition, `z_index` range check (`-4096..=4096`) and relative composition, `y_sort_enabled`, `show_behind_parent`, clip mode, light/visibility layers |
| `scene/tile_set.rs` | `scene/resources/2d/tile_set.*` (square cells) | `map_to_local` returns the cell center `(cell+0.5)*tile_size`, `local_to_map` floors `local/tile_size` (so -0.1 maps to cell -1). `map_to_local` also covers the `STACKED`/`STACKED_OFFSET` half-offset layouts in both offset axes for isometric/hexagon/half-offset square (with the 0.5/0.75 overlap ratios); `local_to_map` implements the smart floor plus top/bottom triangle correction for all three half-offset shapes in both axes, verified by cell-center round-trips; `STAIRS_*`/`DIAMOND_*` remain rejected. **Non-differential** |
| `scene/parallax_layer.rs` | `scene/2d/parallax_layer.*` | motion scale/offset, mirroring clamped to `>= 0`, captured origin, and `compute_offset_scale` implementing `base*scale + offset*view + origin*view` plus the `ceil`-based mirroring wrap. Rendering-server mirroring not ported. **Non-differential** |
| `scene/sprite_frames.rs` | `scene/resources/sprite_frames.*` + `scene/2d/animated_sprite_2d.*` | named animations (speed, loop mode, per-frame duration), add/remove/rename, duration fallbacks (1.0), and `AnimatedSprite2DData::advance` with linear wrap, non-loop stop-on-last-frame and ping-pong reversal, including the `speed_scale` divisor and zero-speed freeze. Textures reduced to size; drawing not ported. **Non-differential** |
| `scene/polygon_2d.rs` | `scene/2d/polygon_2d.*` (data + outline triangulation) | outline, hole list accessors, UVs, vertex colors, texture offset/scale/rotation, invert + border (clamped at 0), poly-as-lines, internal vertex count. `triangulate_outer` uses the ported ear clipping; the hole-aware Clipper path is rejected rather than ignored. **Non-differential** |
| `scene/line_2d.rs` | `scene/2d/line_2d.*` (state + width) | point list with append/insert/remove, closed flag, width clamped at 0, `Curve`-scaled `get_width_at`, default color, optional `Gradient`, joint/cap modes, sharp limit, round precision (>= 1) and texture mode. `_draw`'s triangle strip is not ported. **Non-differential** |
| `scene/camera_2d.rs` | `scene/2d/camera_2d.*` (state + `align`) | anchor mode, offset, zoom with zero rejection and reciprocal `zoom_scale`, drag margins/offsets/enabled flags, limits, smoothing speeds (clamped to >= 0), process callback, and the `align` position math for both anchors, plus `get_camera_transform` with limit clamping (left/right/top/bottom, split-centering when the viewport is wider than the limits), zoom on the basis, optional rotation and position smoothing, and `get_camera_screen_center`. **Non-differential** |
| `scene/node.rs` | `scene/main/node.*` (data half) | name/owner, unique-name flag, `ProcessMode` (inherit/pausable/when-paused/always/disabled) with the engine resolution and `can_process` matrix, process/physics priorities, thread group, `inside_tree`, and the eight `set_process*` flags. **Non-differential** (needs the tree/object layer) |
| `scene/sprite_2d.rs` | `scene/2d/sprite_2d.*` (geometry) | Texture reduced to its size; frame grid (`hframes`x`vframes`) with the engine frame/column retention rules, `frame_coords`, centered/offset/flip, region rect and filter clip, `get_rect`/`get_rects`. Drawing and `is_pixel_opaque` need pixels. **Non-differential** |
| `scene/gradient.rs` | `scene/resources/gradient.*` | Offset-sorted stops, the engine binary search with its `middle--` correction and end clamping, Constant/Linear/Cubic modes, and sRGB / linear-sRGB / Oklab interpolation. **Non-differential** (`Resource`) |
| `scene/curve_3d.rs` | `scene/resources/curve.*` (Curve3D core) | same tessellation as Curve2D with `Vector3`, plus `closed` wrapping (the last interval ends at point 0). Tilts/up vectors/rotation sampling not ported. **Non-differential** |
| `scene/curve_2d.rs` | `scene/resources/curve.*` (Curve2D core) | points with in/out controls, cubic Bézier `sample`/`samplef`, even-length tessellation (depth 10, `bake_interval` 5.0), cumulative arc lengths, `sample_baked` (linear and cubic) with clamping, and `sample_baked_with_rotation` backed by the baked tangent cache (`_calculate_tangent` corner cases + slerp). Tilts/posture and Curve3D not ported. **Non-differential** |
| `scene/curve.rs` | `scene/resources/curve.*` (1D core) | ordered points clamped to the value/domain ranges, lower-bound `get_index`, cubic Bézier `sample` with the engine's one-third control points, auto (linear) tangents, bake cache and `sample_baked`. Curve2D/3D not ported. **Non-differential** (`Resource` pulls `ClassDB`) |
| `scene/canvas_layer.rs` | `scene/main/canvas_layer.*` (data half) | `layer`, offset/rotation/scale with the engine's lazy decomposition after `set_transform`, visibility, follow-viewport transform composition (`vp * scale(ratio) * layer`). `set_rotation_and_scale` is differential (`t2.rot_scale*`) |
| `math/vector4.rs` | `core/math/vector4.*` | complete (`Vector4i` pending) |
| `math/face3.rs` | `core/math/face3.*` | plane/area/aabb/split_by_plane, exact `get_closest_point_to`, SAT `intersects_aabb` (verified). ray/segment via `geometry_3d`, support + project_range (all verified) |
| `math/vector4i.rs` | `core/math/vector4i.*` | core, `clamp`/`clampi`, `snapped`/`snappedi` (verified), `min`/`max_axis_index` |
| `math/rect2i.rs` | `core/math/rect2i.*` | core done; filled polygon/segment pending |
| `math/projection.rs` | `core/math/projection.*` | perspective/ortho/frustum, determinant, xform, fov/aspect, Gauss-Jordan `invert`, frustum planes (all 6), per-eye HMD, `adjust_perspective_znear`, viewport/far half extents, `get_pixels_per_meter`, `get_lod_multiplier`, `make_scale`, `get_projection_planes(Transform3D)`, `scale_translate_to_fit(Aabb)`, `create_fit_aabb`, `perspective_znear_adjusted`, all verified |
| `string/char_utils.rs` | `core/string/char_utils.h` | **20** predicates (incl. the 5 Unicode table-backed), verified over **all 1.1M code points** (count+hash) |
| `math/color_names.rs` | `core/math/color_names.inc` | 146 `(NAME, 0xRRGGBBAA)` pairs from `tools/gen_color_names.py`, cross-checked with an independent C++ parser |
| `string/gstring.rs` | `core/string/ustring.h/.cpp` (start) | COW buffer (`Arc::make_mut`), append/find/rfind/substr/begins_with/ends_with/strip_edges/split/replace/count/join/to_int/is_valid_int/to_float/is_valid_float/insert/repeat/find_char/pad_decimals/pad_zeros/lpad/rpad/trim_prefix/trim_suffix/quote/unquote/is_quoted/is_lowercase/is_uppercase/path helpers (get_file, get_extension, get_basename, path_join, is_absolute_path), ASCII + Unicode case mapping (`to_upper`/`to_lower`, locale-independent, non-differential), Murmur3 `hash()`. UTF-8/UTF-16 encode/decode with surrogate and U+10FFFF boundary tests (not differential: `ustring.cpp` cannot be linked in the harness). `String::num` pending |
| `object/mod.rs` | `core/object/object.h`, `ref_counted.*` | `ObjectId` (unique, never reused, `is_valid`), `Object` lifetime registry, `RefCounted` ref/unref that destroys at zero. **Non-differential** |
| `scene/virtual_joystick.rs` | `scene/gui/virtual_joystick.*` (state + stick math) | Three joystick modes, sizes, deadzone/clamp ratios, offset ratio, the four action names, visibility mode, plus `_update_joystick` ported exactly: following-mode base drag gated by `has_point`, radius clamp, `offset/radius` raw vector, deadzone rescale through `inverse_lerp` and the one-shot `flick_canceled` transition. `Input` calls and drawing not ported. **Non-differential** |
| `scene/texture_progress_bar.rs` | `scene/gui/texture_progress_bar.*` (state) | Nine fill modes with the `FILL_MODE_MAX`/negative rejection, progress and radial offsets, initial angle/fill degrees, three textures, nine-patch stretch with side-validated margins and the three tints. Drawing not ported. **Non-differential** |
| `scene/label.rs` | `scene/gui/label.*` (state) | Paragraph separator `"\n"`-as-two-chars, language/direction/structured-text parser, visible-character behaviour, the count↔ratio coupling (`-1`/empty means full, truncating), negative line skip rejection and the line limit/tab stops. Shaping and drawing not ported. **Non-differential** |
| `scene/material_extras.rs` | `scene/resources/3d/fog_material.*`, `scene/resources/blit_material.*` | FogMaterial (density 1, white albedo, opaque-black emission from the constructor, height falloff, edge fade 0.1, density texture) and BlitMaterial's five blend modes. Shader selection not ported. **Non-differential** |
| `scene/texture_button.rs` | `scene/gui/texture_button.*` (state + hit test) | Five texture handles, click-mask `BitMap`, `ignore_texture_size`, seven stretch modes and flips; minimum size (absolute) and `has_point` ported branch for branch: mask-size fallback, tile wrapping with truncated indices, scaled/offset mapping with the aspect-covered region correction. Drawing/texture-size tracking not ported. **Non-differential** |
| `scene/sky.rs` | `scene/resources/sky.*`, `scene/resources/3d/sky_material.*` | Sky material handle, radiance ladder `32..=2048` with the `MAX` rejection and pixel mapping, process modes; ProceduralSkyMaterial constructor colours/curves/energies plus the derived `inv_*_curve` and `cos(sun_angle_max)` shader values; PanoramaSkyMaterial panorama/energy/filter; PhysicalSkyMaterial rayleigh/mie/turbidity/sun-disk/ground/energy/debanding/night-sky (its header's `exposure_value` pair has no definition in this tree, so it is skipped). Shader compilation and server parameters not ported. **Non-differential** |
| `scene/environment.rs` | `scene/resources/environment.*` | Every Environment parameter: background (mode/sky/fov/rotation/colour/layer/feed/energy/intensity), ambient and reflection sources, tonemap (incl. the AgX 16.29/1.25 defaults), SSR, SSAO, SSIL, SDFGI with the derived `cascade0 = min_cell_size*64` and `max = cascade0*2^cascades`, glow with the constructor's `[0,0.8,0.4,0.1,0,0,0]` levels, fog + depth fog, volumetric fog and colour adjustment. Only `set_sdfgi_cascades` (1..=8) and `set_glow_level` (<7) validate, like the C++. Server updates not ported. **Non-differential** |
| `scene/cpu_particles_2d.rs` | `scene/2d/cpu_particles_2d.*` (parameters) | Constructor defaults, the 12 min/max parameter tables (only `scale` starts at 1) with per-parameter curve slots, particle flags, seven emission shapes with their per-shape data, colour/initial ramps, split scale curves, and `amount >= 1` / `lifetime > 0` plus sentinel rejection. Simulation and multimesh buffer not ported. **Non-differential** |
| `scene/tab_bar.rs` | `scene/gui/tab_bar.*` (state) | Tab records (text/tooltip/icon/disabled/hidden), selection with the engine's removal and move adjustments, `_can_deselect` (enabled, or every tab disabled/hidden), first-tab auto-selection, alignment/clip/close-policy/switch-on-release/max-width/scrolling settings and enum sentinels. Drawing and tree staging not ported. **Non-differential** |
| `scene/base_button.rs` | `scene/gui/{base_button,button}.*` (state) | Interaction status and configuration with the derived rules: toggle-only `set_pressed` (reporting `toggled`), toggle-mode clearing on disable, `pressed_down_with_focus` reporting `button_up`, `is_pressed` switching between toggle state and press attempt, and `get_draw_mode` branch for branch (including the pressed/excluded inversion and `keep_pressed_outside`). Button text/icon/alignment state. `ButtonGroup` not ported (needs the shared object graph). **Non-differential** |
| `io/path_utils.rs` | `core/string/ustring.cpp` (`simplify_path`, `get_base_dir`, `is_network_share_path`) | Drive/scheme/UNC/root prefix splitting, backslash normalisation, repeated-separator collapsing and the `.`/`..` rules — including the two engine exceptions: `res://` keeps leading `..` (glTF importer) and absolute non-`res://` paths drop it. `get_base_dir` follows the same four prefixes in the engine's order. **Non-differential** |
| `scene/importer_mesh.rs` | `scene/resources/3d/importer_mesh.*` (surfaces) | Blend-shape names sanitised (`:` → `_`) and only addable before any surface, `add_surface` validating `ARRAY_MAX` (13) entries, one blend-shape array list per declared shape, non-empty vertex array and matching blend-shape vertex counts; per-surface primitive/name/flags/material/LOD reads, LOD writes via the add call, and `clear`. ArrayMesh conversion, LOD generation and lightmap unwrap not ported. **Non-differential** |
| `scene/particles_3d.rs` | `scene/3d/gpu_particles_3d.*` (state) | GPUParticles3D constructor defaults (amount 8, lifetime 1, **fixed FPS 30**, AABB `(-4,-4,-4)+(8,8,8)`, index draw order, collision size 0.01), draw-pass list with index checks, transform-align mode/channel/axis, trails and the same amount/lifetime/trail bounds; `interp_to_end` clamps `0..=1`. Rendering/RID not ported. **Non-differential** |
| `scene/shapes_3d.rs` | `scene/resources/3d/*_shape_3d.*` (state) | Box `(1,1,1)`, sphere/cylinder radius 0.5, capsule 0.5/2 with the height↔radius coupling and `mid_height`, separation ray length 1, world boundary plane `(0,1,0,0)`, convex/concave polygons, and the heightmap's resize rules (sizes `< 1` ignored, data length must match `width*depth`). Physics server calls not ported. **Non-differential** |
| `scene/particles_2d.rs` | `scene/2d/gpu_particles_2d.*` (state) | GPUParticles2D with the constructor's own defaults (emitting, amount 8, lifetime 1, fractional delta, visibility rect `(-100,-100,200,200)`), trails off (0.3 s, 8 sections, 4 subdivisions) and the engine's validation: `amount >= 1`, `lifetime > 0`, `trail_lifetime >= 0.01 - CMP_EPSILON`, sections `2..=128`, subdivisions `1..=1024`, while `amount_ratio` assigns unclamped. Rendering/RID/seed RNG not ported. **Non-differential** |
| `io/resource_uid.rs` | `core/io/resource_uid.*` | Base-34 text form (`a`..`y`, `0`..`8`, `uid://` prefix, `<invalid>` sentinel, `0x7FFF…` mask) both directions, and the id→path table with add/set/remove/get refusal rules, `uid_to_path`, `ensure_path` and `create_id` retrying on collisions with an injected random source. On-disk cache, reverse cache and the MD5-seeded `create_id_for_path` not ported. **Non-differential** |
| `object/resource.rs` | `core/io/resource.*` (identity/emission) | Name/path/path cache/scene unique id/local-to-scene, built-in path detection and the emit-changed block/pending/unblock state machine with a counter. Cache takeover, duplication and signal delivery not ported. **Non-differential** |
| `object/class_db.rs` | `core/object/class_db.*` | Class hierarchy, inherited method lookup, `ObjectId` -> class binding, `bind_method` + `call` with arity/payload validation, properties (`register_property`, `property_list` with inheritance, `property_get/set`, `clear_property_values`), signals (`register_signal`, `signal_list` with inheritance, `connect_signal`/`disconnect_signal`, `emit_signal` returning targets, `clear_signal_connections`, typed properties, `PropertyHint`/usage flags via `register_property_ex` + `property_meta` (inherit-aware)). **Non-differential** |
| `variant/mod.rs` | `core/variant/variant.h` | `Variant::Type` tags (all 39, verified against the engine enum) + POD payloads, `Object`, `Callable`, `Signal` (identity-backed), `Dictionary`/`Array` of variants, all 10 typed arrays with payloads, `dict_get/set/keys`, `array_get/push/len`, `booleanize`, `to_int`/`to_float` |
| `variant/compare.rs` | `core/variant/variant_op.cpp` (comparisons/conversions) | `compare` (numeric cross-type, bool, string, Vector2/3 lexicographic), `equals`, `can_convert`/`convert` for the numeric/bool/string subset, `stringify`, and `get_index` for Array/Dictionary. Feeds the pending Expression work. **Non-differential** |
| `variant/ops.rs` | `core/variant/variant_op.cpp` (subset) | `+ - * /` for Int/Float/Vector2/Vector3/Vector4/Color incl. promotion and scalar scaling; `String + String` concatenates; int div-by-zero and all other invalid pairs yield `None`. **Non-differential** |
| `variant/serial.rs` | `core/variant/variant_parser.*` | Minimal `to_text`/`from_text` for nil/bool/int/float/String/Vector2/Vector3/Color/Array/Dictionary (nested). **Non-differential by design** |
| `string/string_name.rs` | `core/string/string_name.h` | Interning pool (global, lock-protected), identity equality via `u64` id, djb2 `hash()`. Non-differential |
| `string/num.rs` | `core/string/ustring.cpp` (`num_int64`/`num_uint64`) | bases 2-36, INT64_MIN, u64::MAX; plus `num_float` (fixed decimals, zero trimming, `-0.0`, inf/nan, 16-decimal cap, half-to-even rounding). **Non-differential** (see note 3) |
| `math/fieldwise.rs` | `core/math/math_fieldwise.cpp` | `fieldwise_assign` over 14 variant types (Vector2..Transform3D), type-mismatch and unknown-field no-ops. **Non-differential** (engine code is `#ifdef DEBUG_ENABLED`) |
| `math/triangle_mesh.rs` | `core/math/triangle_mesh.*` | Vertices/triangles, `create_from_faces` (no welding), `get_faces`, `get_aabb`, `get_area`, `recompute_normals`. **BVH not ported** (needs `class_db`) |
| `math/triangle_mesh_bvh.rs` | `core/math/triangle_mesh.*` (BVH) | Median-split BVH with `intersect_segment`, verified against brute force over 200 random segments |
| `math/triangulate.rs` | `core/math/triangulate.*` | Ear clipping (`get_area`, `is_inside_triangle`, `snip`); verified on convex/concave polygons by area preservation. Output order is a valid triangulation, not the engine's exact order |
| `math/delaunay_3d.rs` | `core/math/delaunay_3d.*` | Bowyer-Watson tetrahedralization; verified by total volume, vertex coverage, and empty on coplanar input |
| `math/convex_hull.rs` | `core/math/convex_hull.*` | Incremental convex hull (visible faces + horizon). Verified by containment of all input points, outward normals, closed manifold, Euler `V-E+F=2`, and analytic volumes (tetrahedron, cube, icosahedron). **Non-differential**: Bullet-based C++ needs `PagedAllocator`; equivalent mesh, different triangle order |
| `math/dynamic_bvh.rs` | `core/math/dynamic_bvh.*` | Dynamic AABB tree: insert by surface-area growth, remove, update, `aabb_query`, `ray_query`. Verified against brute force over 200 boxes and 200 random queries plus a 600-operation insert/remove cycle. Stores min/max like the engine's `Volume` (position+size round-trips lose an ulp). **Non-differential** |
| `math/random_number_generator.rs` | `core/math/random_number_generator.h` | Wrapper over the bit-exact PCG: seed/state, randi/randf, randfn, ranges, `rand_weighted`. **Non-differential** (RefCounted) |
| `math/a_star.rs` | `core/math/a_star.h/.cpp` (AStar3D) | Point registry, connections, A* with distance heuristic, partial paths, disabled points, one-way edges. **Non-differential** (`a_star.cpp` pulls `class_db.h`); `weight_scale` applied to entering edges like the engine. `AStar2D` wraps it for Vector2; `AStarGrid2D` with solid cells, per-cell weights, 4 diagonal modes and `jumping_enabled` (`_jump` + `_forced_successor`, all three diagonal branches, ported rule for rule; costs/heuristics use the engine's index-based Euclidean default, `cell_size` only scales `get_point_path`) |
| `math/audio_frame.rs` | `core/math/audio_frame.h` | Stereo frame, per-channel ops, `lerp`, `undenormalize` (exponent threshold 16, not just denormals) |
| `math/disjoint_set.rs` | `core/math/disjoint_set.h` | Union-find with path compression + union by rank, generic over `T`. Non-differential (templates) |
| `string/char_range.rs` | `core/string/char_range.cpp` | 3.596 ranges generated by `tools/gen_char_range.py` (no hand-typing) |
| `math/geometry_3d.rs` | `core/math/geometry_3d.h` | 2 triangle tests, sphere/cylinder/convex segment tests, closest point on segment, barycentric coords, `point_in_projected_triangle`, `triangle_sphere_intersection_test`, `clip_polygon`, `get_support`, `project_range`, `get_closest_points_between_segments` + distance, `build_box_planes`, `build_cylinder_planes` (sides + caps), `build_sphere_planes`, `build_capsule_planes`, `compute_convex_mesh_points`, all verified |
| `math/geometry_2d.rs` | `core/math/geometry_2d.h` | hit tests + `vec2_cross`, `convex_hull` (verified: 5 points, first repeated), `bresenham_line` (verified) |
| `ffi.rs` | (new) | `gcr_variant_nil/bool/int/float/vector3`, `gcr_variant_type/as_int/as_float/as_vector3/booleanize`; verified inside the engine build (prints `variant tag = 2, truthy = 1` and `Rust packed sum = 256`) | **old**: C ABI: `gcr_abi_version`, `gcr_vector3_dot`, `gcr_vector3_normalized`, `gcr_transform3d_xform` |
| `templates/lru.rs` | `core/templates/lru.h` | Front-ordered LRU with `insert` (replacing a key, trimming from the tail), `get`/`get_ptr` recency updates, `peek`, `erase`/`clear`, and the engine's capacity rules (default 64, `set_capacity` ignored at zero). **Non-differential** |
| `math/rng.rs` | `core/math/random_number_generator.*` | Seed/state accessors, `randomize`, and the six draws (`randi` raw bits, `randf`, inclusive `randf_range`/`randi_range` with bounds swap, `randfn`, `rand_weighted`) delegating to the verified `RandomPcg`. The constructor randomizes like the engine; `with_seed` exists for deterministic tests. **Non-differential** |
| `string/plural_rules.rs` | `core/string/plural_rules.*` | `parse` of `nplurals=<N>; plural=<expr>;` with the engine's exact error texts, ternary-tree splitting at unbalanced `?`/`:` (parenthesised regions skipped, one wrapping pair stripped), per-node `Expression` evaluation with input `n`, leaf values and a capacity-64 `LruCache` memo. Bare conditions resolve to 1/0. **Non-differential** |
| `string/translation.rs` | `core/string/translation.*` | `(context, source)` → plural-form store with `add_message`, `add_plural_message` (empty list rejected), `get_message` (first form), `erase_message`, the sorted `context \x04 source` list, count/singular-translation listings and the rules-override string. Documented limits: locale is stored verbatim (no `standardize_locale`) and plural evaluation uses the one/other rule instead of parsing gettext expressions. **Non-differential** |
| `io/config_file.rs` | `core/io/config_file.*` (parse/encode) | Section headers with `\]` unescaping, `key = value` lines where values are Variants (multi-line arrays included via `variant_parser::parse_prefix`), `;` comments, section-less keys under `""`, `set_value` erasing on `Nil` and dropping emptied sections, `get_value` defaults, section/key erase with the engine's failures, the section/key listings, and `encode_to_text` with `property_name_encode` keys and sorted sections. Writing to disk not ported. **Non-differential** |
| `io/xml_parser.rs` | `core/io/xml_parser.*` | The irrXML pull parser: byte-offset and line tracking, text nodes with the engine's fewer-than-three/whitespace rule and XML unescaping, elements with single/double-quoted unescaped attributes, self-closing tags, end tags, `<?...?>` definitions, comments (`<!-- -->` and bracket-balanced `<!...>`), CDATA, `skip_section`, `seek` and `close`, with the engine's empty-string returns for wrong node kinds and bad indices. Encoding sniffing (`open`) not ported. **Non-differential** |
| `io/variant_parser.rs` | `core/variant/variant_parser.cpp` (text values + writer) | Tokenizer with `;` comments, the symbol set, `#RRGGBB[AA]` colours, `&"name"`/`@"name"` string names, quoted strings with the full escape set (`\uXXXX`, `\UXXXXXX`, surrogate pairs) and the int/float `READING_*` state machine (including `-inf` as an identifier); `parse_value` for literals, arrays, dictionaries and the Vector2/2i, Rect2/2i, Vector3/3i, Vector4/4i, Plane, Quaternion, AABB, Basis, Transform2D/3D, Projection, Color, NodePath and StringName constructors with their arity checks; and `write_variant` in the engine's text (floats through the port's `num_float` placeholder, `[a, b]` arrays, sorted `{\n"k": v\n}` dictionaries). Dictionary keys must be strings (port container limit) and NodePath/StringName decode to `Variant::String`. Objects/resources/RIDs/tags not ported. **Non-differential** |
| `io/plist.rs` | `core/io/plist.*` (XML) | `load_string`'s tag scanner (skips `<?xml?>`/`<!DOCTYPE>`, requires `<plist>`, stack of open containers, mismatched-tag errors, values only inside containers, pending keys dropped) and `save_text`'s exact formatting (header/doctype/tabs, `<true/>`, indented `<data>` block, two trailing newlines), with a `PlistNode` tree and dict/array lookups. Binary `bplist00` reading and ASN.1 writing not ported. **Non-differential** |
| `string/slices.rs` | `core/string/ustring.cpp` (slice helpers) | `get_slice_count` (0 for an empty text or splitter), `get_slice` (whole text when the splitter is absent, empty for negative/missing), `get_slicec`, `split` with `allow_empty`/`maxsplit` (rest in the last element), and `split_floats` through `to_float`, all on character indices. **Non-differential** |
| `scene/primitive_meshes.rs` | `scene/resources/3d/primitive_meshes.*` (parameters) | PrimitiveMesh base (flip faces/UV2/padding/texel size) plus the constructor defaults and guards for Capsule (ings >= 0), Box, Cylinder (ings >= 0), Plane/Quad, Prism, Sphere (ings >= 1) and Torus (ings/ring_segments >= 3, equal radii rejected). Mesh array generation not ported. **Non-differential** |
| `scene/kinematic_collision.rs` | `scene/2d/physics/kinematic_collision_2d.*`, `scene/3d/physics/kinematic_collision_3d.*` (records) | 2D record fields (position/normal/travel/remainder/depth/shapes/collider ids/velocity) with `get_angle`, and the 3D list variant with per-index accessors returning zero values for bad indices. Motion generation not ported. **Non-differential** |
| `scene/gpu_particles_collision.rs` | `scene/3d/gpu_particles_collision_3d.*` (sphere/box/SDF state) | Cull mask and collision handle, sphere radius 1, box size (2,2,2), and the SDF parameters (size, thickness, `cell_size >= 0.01`, field texture, bake mask with 1..=32 bits, texture size, range). Heightfield and baking pending. **Non-differential** |
| `scene/spring_arm_3d.rs` | `scene/3d/physics/spring_arm_3d.*` (state) | Spring length 1, hit length 0, mask 1, margin 0.01, optional shape and the excluded-object list with add/remove/clear; `get_hit_length` mirrors the cast result (settable here). Shape cast/process not ported. **Non-differential** |
| `scene/worlds.rs` | `scene/resources/world_2d.*`, `scene/resources/3d/world_3d.*` (state) | Canvas/navigation-map/space handles and viewport count for 2D; scenario/space/navigation-map plus environment, fallback environment, camera attributes, compositor and the registered camera list for 3D. Server/lazy-map creation not ported. **Non-differential** |
| `scene/popup_menu.rs` | `scene/gui/popup_menu.*` (items) | Item store with text/xl-text/icon+width+modulate, checkable type (box/radio), checked, multistate count/state, visibility, separator/disabled, id, metadata, submenu, tooltip and indent; all add variants incl. icon check/radio and separators; id/text lookups; every field accessor with the engine's empty returns; removal shifting current/focused; and the popup options (three hide-on-selection flags, search, native preference, system menu, search-bar trio). Shortcuts/native menu/drawing not ported. **Non-differential** |
| `io/ip_address.rs` | `core/io/ip_address.*` | IPv4 parsing (three dots, 0..=255 octets, no junk), IPv6 parsing (four hex digits per group, one `::`, the engine's group-shift for the shortened range, `::::FFFF:`-mapped IPv4 tails), the string constructor rules (`*`, `:`→IPv6 under 40 chars, IPv4 under 16), `operator String()` (wildcard/invalid/dotted/lowercase hex groups), `is_valid_ip_address`, `set_ipv4`/`set_ipv6`, and the engine's equality (valid and all 16 bytes; invalid never equals). **Non-differential** |
| `scene/menu_bar.rs` | `scene/gui/menu_bar.*` (state) | Menu entries (title/tooltip/hidden/disabled/popup handle) with title/tooltip/flag accessors and index guards, plus the bar options: switch-on-hover (on), shortcut disabling, native preference (on), flat, `start_index` (-1), text direction/language and focused/selected/active indices. `add_menu`/`remove_menu` are port-level list helpers (this fork builds the list from child popups); native menus/drawing/input not ported. **Non-differential** |
| `scene/item_list.rs` | `scene/gui/item_list.*` (items/selection) | Item fields (text, icon+region+modulate, selectable/selected/disabled, tooltip enable/text, metadata, custom fg/bg, column), add/remove/clear, every field accessor with the engine's empty returns, and the selection rules: single/`single` calls refuse unselectable/disabled and replace the whole selection, multi/toggle only set, `deselect` clears the current index in single mode, `deselect_all`, plus the constructor's column/icon/search/autosize options and `SelectMode`/`IconMode` index guards. Layout/drawing/input not ported. **Non-differential** |
| `scene/joints_2d.rs` | `scene/2d/physics/joints/*` (state) | Joint2D body paths/bias/exclusion/handles, PinJoint2D (softness, angular limits, motor), GrooveJoint2D (length 50, offset 25) and DampedSpringJoint2D (stiffness 20, damping 1, rest 0, length 50) constructor defaults with plain setters. Servers not ported. **Non-differential** |
| `scene/collision_polygon.rs` | `scene/2d/physics/collision_polygon_2d.*`, `scene/3d/physics/collision_polygon_3d.*` (state) | 2D build modes (solids/segments, index-guarded), polygon storage with the engine's AABB recompute (30% expansion, default 20×20 when empty), disabled/one-way settings; 3D depth, polygon, debug colour/fill and disabled with the node-side extruded extent. Shape-owner/server calls not ported. **Non-differential** |
| `scene/physics_body_exceptions.rs` | `PhysicsBody2D`/`PhysicsBody3D` exception lists | Local add/remove/clear/query with duplicate refusal and membership reporting, mirroring the server-forwarded calls. **Non-differential** |
| `scene/collision_shape.rs` | `scene/2d/physics/collision_shape_2d.*`, `scene/3d/physics/collision_shape_3d.*` (state) | Shape handle, disabled flag, debug colour; the 2D editor rect (-10,-10,20,20) and one-way collision settings (margin 1, direction (0,1)). Owner registration/debug drawing not ported. **Non-differential** |
| `scene/ray_cast.rs` | `scene/2d/physics/ray_cast_2d.*`, `scene/3d/physics/ray_cast_3d.*` (state) | Enabled/mask (1..=32 helpers)/exceptions/parent-exclusion/area-body flags/hit-from-inside, 2D target (0,50) and 3D target (0,-1,0) with the debug-shape trio, plus collision result fields and set/clear helpers standing in for the space query. **Non-differential** |
| `scene/flow_container.rs` | `scene/gui/flow_container.*` (lines) | Vertical/reverse/alignment/last-wrap settings (orientation rejected once fixed) and the first pass of `_resort`: child accumulation until overflow, per-line child count, tallest child, consumed length, remaining stretch space, stretch-ratio total (horizontal or vertical expand flag) and the `is_filled` rule, with cached size/line counts. The expansion/alignment placement passes are pending. **Non-differential** |
| `scene/physics_body_2d.rs` | `scene/2d/physics/{area_2d,character_body_2d,rigid_body_2d,static_body_2d}.*` (state) | Area2D's constructor gravity 980/direction (0,1) and enabled monitoring, five space-override modes, damp/priority/point-gravity settings; CharacterBody2D's 0.08 margin, four slides, 1.0 snap, 45°/15° angles, up (0,-1) and the same guards as 3D; RigidBody2D with 2D CCD modes and StaticBody2D's constant velocities/material. Servers and motion resolution not ported. **Non-differential** |
| `scene/split_container.rs` | `scene/gui/split_container.*` (offsets/clamping) | Orientation (rejected once fixed), collapsed flag, three dragger visibilities, per-dragger split offsets, drag-area options, and `_update_dragger_positions` ported as scalar math: default+offset clamped to valid ranges, overlap push-right without a priority, left/right constraint propagation with the active dragger (including child maximum sizes), split-offset write-back, collapsed handling and horizontal RTL mirroring. Child rect fitting/theme/input not ported. **Non-differential** |
| `scene/physics_material.rs` | `scene/resources/physics_material.*` | Friction 1 / rough off / bounce 0 / absorbent off, unclamped setters and `computed_friction` (negated when rough). **Non-differential** |
| `scene/navigation_nodes_2d.rs` | `scene/2d/navigation/{navigation_link_2d,navigation_obstacle_2d}.*` (state) | Link enabled/bidirectional/layers/offsets/costs with negative rejection, obstacle radius guard, avoidance flags, velocity latch, affect/carve and the derived winding/validity flags through the ported Geometry2D helpers. Servers/maps not ported. **Non-differential** |
| `scene/joints_3d.rs` | `scene/3d/physics/joints/*` (state) | Joint3D body paths/solver priority/collision-exclusion/handle, and the Pin (±bias table), Hinge (bias/limits/softness/relaxation/motor with both flags), Slider (22-entry linear+angular table) and ConeTwist (span/bias/softness/relaxation) constructor defaults, with `MAX` sentinel rejection for params and flags. `Generic6DOFJoint3D` included with its 22-entry per-axis tables and six per-axis flags; servers not ported. **Non-differential** |
| `scene/canvas_extras.rs` | `scene/2d/canvas_group.*`, `scene/2d/visible_on_screen_notifier_2d.*`, `scene/2d/navigation/navigation_region_2d.*` (state) | CanvasGroup's 10 px margins/mipmaps, the 2D notifier's `(-10,-10,20,20)` rect with show/on-screen flags, the enabler's mode/path, and NavigationRegion2D's enabled/edges/layers/costs (negative rejection)/polygon/map/bounds. Drawing/visibility tests/server not ported. **Non-differential** |
| `scene/physics_body_3d.rs` | `scene/3d/physics/{character_body,static_body,rigid_body}_3d.*` (state) | CharacterBody3D defaults (grounded, six slides, 0.1 snap, 45°/15° angles, full floor layers), its `max_slides >= 1` and `floor_snap_length >= 0` guards, the motion-state getters and `get_floor_angle`; StaticBody3D's constant velocities and material; RigidBody3D mass/gravity/damp modes/freeze/CCD/contacts/custom-integrator/centre-of-mass/velocities. `move_and_slide` and the physics server not ported. **Non-differential** |
| `scene/mesh_resource.rs` | `scene/resources/mesh.*` (base) | PrimitiveType/ArrayType/ArrayCustomFormat/ArrayFormat bits/BlendShapeMode, the surface table with arrays/format/primitive/material/name/blend shapes/LODs and the base accessors (array/index lengths, material/name setters), blend-shape names/mode, lightmap hint, cached AABB and `get_faces` expanding triangle and strip soups. Subclass surface creation/shadow meshes not ported. **Non-differential** |
| `scene/tab_container.rs` | `scene/gui/tab_container.*` (state) | Tabs visible/position (`POSITION_MAX` rejected), hidden tabs in minimum size, drag-to-rearrange and its group, popup handle, and the staged `setup_current_tab` (-2 sentinel) forwarded to the owned `TabBarData` on entering the tree. Drawing/popup/repaint plumbing not ported. **Non-differential** |
| `scene/cpu_particles_3d.rs` | `scene/3d/cpu_particles_3d.*` (parameters) | Constructor defaults (amount 8, scale 1, ring axis/height/radius, gravity `(0,-9.8,0)`, white colour), the 12-entry min/max tables with curve slots, flags, seven emission shapes with per-shape data, ramps, split scale curves, and the engine's validations (`amount >= 1`, `lifetime > 0`, sentinel rejection for draw order/param/flag/shape). Simulation/multimesh not ported. **Non-differential** |
| `scene/collision_object.rs` | `scene/3d/physics/collision_object_3d.*` (state) | Layer/mask with the `1..=32` bit helpers and their error rules, collision priority, disable mode, body mode, area flag, ray-pickable and input-capture flags, plus the shape-owner table (create/remove/transform/disabled). Physics server, debug shapes and picking not ported. **Non-differential** |
| `scene/visual_helpers.rs` | `scene/2d/marker_2d.*`, `scene/3d/marker_3d.*`, `scene/3d/visible_on_screen_notifier_3d.*` | Marker gizmo extents (10 / 0.25), the notifier's `(-1,-1,-1)+(2,2,2)` AABB with its on-screen flag, and the enabler's mode/`../AnimationPlayer` path. Drawing/notifications not ported. **Non-differential** |
| `scene/scroll_bar.rs` | `scene/gui/scroll_bar.*` | Range plus orientation, custom step, the increment/decrement highlight and active flags, the drag record, drag-node/scrolling/smooth-scroll flags, and the three derived helpers (`get_grabber_size` with the zero-range rule, `get_area_size` over caller-supplied theme metrics, `get_grabber_offset` = area × ratio). Drag-node input, animation, theme and drawing not ported. **Non-differential** |
| `scene/slider.rs` | `scene/gui/slider.*` | Range plus four tick positions, tick count, mouse-inside, custom step, editable/scrollable flags (`set_editable` cancels an active grab) and the grab record. Drawing/input not ported. **Non-differential** |
| `scene/progress_bar.rs` | `scene/gui/progress_bar.*` | Four fill directions with `FILL_MODE_MAX`/negative rejection and the stripe reset, percentage/indeterminate/preview flags, the 200 px/s default speed with the wrapping `advance_indeterminate` step, and `get_percentage` over `Range::get_as_ratio`. Drawing/theme/timer not ported. **Non-differential** |
| `scene/spin_box.rs` | `scene/gui/spin_box.*` (text options) | Prefix/suffix, update_on_text_changed, select_all_on_focus, editable and horizontal alignment (delegated to an owned `LineEditData`), custom arrow step/rounding and `compose_text` (prefix+value+suffix; locale grouping not ported). Arrows, timer, `apply` and theming not ported. **Non-differential** |
| `scene/toggles.rs` | `scene/gui/{check_box,check_button}.*` (state) | Constructor toggle mode and left text alignment plus `set_checked`/`is_checked` over `BaseButton::set_pressed`. Drawing/icon states not ported. **Non-differential** |
| `scene/canvas_modulate.rs` | `scene/2d/canvas_modulate.*` | White default colour and the canvas-membership/visibility/active flags with leave-canvas deactivation. Server calls not ported. **Non-differential** |
| `io/logger.rs` | `core/io/logger.*` (base helpers + sink) | `ErrorType` strings/indents, `LogEntry` headline/location formatting and a `MemoryLogger` sink with error/warning/message counts and text rendering. File rotation/stdout/backtraces not ported. **Non-differential** |
| `scene/line_edit.rs` | `scene/gui/line_edit.*` (text/caret/selection) | Constructor defaults (bullet secret char, expand mode, selection/menu/keyboard flags), `set_text` clearing selection then re-inserting at column 0, caret-clamped insertion honouring `max_length` (returning the rejected overflow), delete-char/delete-text rules, select/select-all/deselect with clamped columns and the `selecting_enabled` gate, selection text/columns, caret clamping and the secret-character truncation. Shaping, IME, undo, menus and drawing not ported. **Non-differential** |
| `scene/gui_simple.rs` | `scene/gui/{color_rect,separator,reference_rect,nine_patch_rect,popup}.*` (state) | ColorRect's colour; the three separators' heading orientations; ReferenceRect's red 1 px editor-only border; NinePatchRect's centre flag, four side-validated margins, region/texture and stretch axes; Popup's popped-up flag and hide reason. Drawing/theming/window plumbing not ported. **Non-differential** |
| `scene/gui_buttons.rs` | `scene/gui/{menu_button,link_button}.*` (state) | MenuButton's constructor state (flat, toggle mode, press-mode actions) plus switch-on-hover, shortcut disabling and the popup handle; LinkButton's underline modes, text/xl-text, URI, language/direction/parser/overrun, `…` ellipsis and alignments. Drawing/popup behaviour not ported. **Non-differential** |
| `scene/navigation_nodes_3d.rs` | `scene/3d/navigation/{navigation_region,navigation_link,navigation_obstacle}_3d.*` (state) | Region (enabled, edge connections, layers, enter/travel costs with negative rejection, mesh/map handles, bounds), link (enabled, bidirectional, layers, start/end offsets, same cost rules) and obstacle (height/radius with the engine's messages, avoidance flags, 2D/3D mode, velocity latch, affect/carve, plus the derived clockwise/valid vertex flags from the `(x, z)` outline using Godot's own winding sum; validity is the real `triangulate_polygon` result, now that `Geometry2D::triangulate_polygon`/`is_polygon_clockwise` are ported). Server calls, maps and debug meshes not ported. **Non-differential** |
| `scene/navigation_polygon.rs` | `scene/resources/2d/navigation_polygon.*` (settings + geometry) | Cell size 1 m (stored unvalidated like the engine), border 0 and agent radius 10 with negative rejection, the three enums with sentinel rejection, layer mask `0xFFFFFFFF` with `1..=32` bits, group name, and outline/polygon/vertex storage with indexed accessors. Baking not ported. **Non-differential** |
| `scene/navigation_mesh.rs` | `scene/resources/navigation_mesh.*` (parameters + geometry) | Cell 0.25/0.25, agent 1.5/0.5/0.25/45°, regions 2/20, edges 0/1.3, six vertices per polygon, detail 6/1, watershed partition, mesh-instance parsing, root-children sources, mask `0xFFFFFFFF` and group `navigation_mesh_source_group`, plus every setter's validation (positive cells, non-negative distances, slope `0..=90`, `>= 3` vertices, `>= 0.1` detail distance, layers `1..=32`, enum sentinels) and the vertex/polygon store. Baking not ported. **Non-differential** |
| `scene/animated_sprite_2d.rs` | `scene/2d/animated_sprite_2d.*` (playback state) | Frames assignment with animation-name repair (empty → `""` and autoplay cleared, missing → alphabetically first), frame clamping and `1/duration` speed scale, `play`/`play_backwards` with the engine's direction and boundary conditions, `pause`/`stop` differences, playing-speed reporting, and the centred/offset/flip flags. Signals/processing/drawing not ported. **Non-differential** |
| `scene/option_button.rs` | `scene/gui/option_button.*` (items + selection) | Item store (text/icon/id/disabled/checked/radio/checkable/submenu/tooltip/metadata), add variants incl. separator and submenu, lookup by id and text, removal shifting the selection, and the selection rules: first selectable item auto-selected, separators never select themselves, `NONE_SELECTED` clears the text, reselect gated by `allow_reselect`, index validation. Popups/drawing not ported. **Non-differential** |
| `string/escape.rs` | `core/string/ustring.cpp` (escaping) | `uri_encode` (unreserved set, upper-case hex), `uri_decode` with the engine's quirks (only upper-case hex accepted, `+` → space, `%` dropped when the second digit is invalid), `c_escape`/`c_unescape`, `c_escape_multiline`, `json_escape`, `xml_escape`/`xml_unescape` (named + decimal + hex entities, malformed left alone), `property_name_encode` and `get_invalid_node_name_characters`. **Non-differential** |
| `string/case.rs` | `core/string/ustring.cpp` (`_separate_compound_words`, case conversions) | Compound-word splitting at `aA`, `AAa`/`2Aa`, `2aa` and `A2`/`a2` boundaries, whitespace/underscore/hyphen → space and lowercasing, then `capitalize`, `to_camel_case`, `to_pascal_case`, `to_snake_case`, `to_kebab_case`. **Non-differential** |
| `templates/hashfuncs.rs` | `core/templates/hashfuncs.*` | complete |
| `scene/node_3d.rs` | `scene/3d/node_3d.*` (data half) | Local transform, Euler rotation and scale as separate authoritative states with lazy derivation; six Euler orders and the engine's order-change rules; `RotationEditMode` with orthogonalize when leaving Basis mode; position/basis/quaternion accessors, rotate helpers, `orthonormalize`, `set_identity`, `look_at(_from_position)` preserving scale, and global compose/inverse with a supplied parent plus `top_level`. **Non-differential** |
| `scene/light_2d.rs` | `scene/2d/light_2d.*` (state) | Light2D/PointLight2D header defaults, shadow-filter/blend enums, `SHADOW_FILTER_MAX` rejection, zero texture scale -> `CMP_EPSILON`. Rendering not ported. **Non-differential** |
| `scene/shapes_2d.rs` | `scene/resources/2d/*_shape_2d.*` | Rectangle, Circle, Capsule, Segment, WorldBoundary, SeparationRay, ConvexPolygon (hull via `Geometry2D::convex_hull`, `>= 3` points, repeated first point kept) and ConcavePolygon: defaults, dimension validation and `get_rect`/`get_enclosing_radius` copied from the C++. **Non-differential** |
| `scene/node2d_extras.rs` | `scene/2d/{remote_transform_2d,canvas_modulate,back_buffer_copy,marker_2d,canvas_group}.*` | RemoteTransform2D flags/path, CanvasModulate colour, BackBufferCopy rect/mode, Marker2D extents, CanvasGroup margins with negative rejection. **Non-differential** |
| `scene/gui_misc.rs` | `scene/gui/{texture_rect,nine_patch_rect,color_rect,separator}.*` | TextureRect expand/stretch enums and flips, NinePatchRect margins/region/axis modes, ColorRect colour, Separator orientation. **Non-differential** |
| `scene/style_box.rs` | `scene/resources/style_box.*`, `style_box_flat.*` | Content margins with the `-1` fallback to the subclass style margin, minimum size/offset, and the flat box's colours, borders, expand margins, corner radii, draw center, blend, skew, anti-aliasing and shadow, with `corner_detail` 1..=20 and `aa_size` 0.01..=10 clamps. **Non-differential** |
| `scene/animation_library.rs` | `scene/resources/animation_library.*` | Name validation, `validate_library_name`, add-or-replace, remove, rename with all three rejections, lookup and the alphabetically sorted list (`RBMap` order). **Non-differential** |
| `scene/gradient_texture.rs` | `scene/resources/gradient_texture.*` | 1D/2D dimension validation `1..=16384`, and `_get_gradient_offset_at` reproduced per texel: signed linear projection, radial, Chebyshev square, positive conic, then none/repeat/mirror. **Non-differential** |
| `scene/texture_misc.rs` | `scene/resources/curve_texture.*`, `placeholder_textures.*` | CurveTexture/CurveXYZTexture width `32..=4096`, RGB/RED mode and `ensure_default_setup` flat curve; PlaceholderTexture2D unvalidated size. **Non-differential** |
| `scene/path_2d.rs` | `scene/2d/path_2d.*` (Path2D + PathFollow2D), `audio_listener_2d.*` | Path2D curve accessors; AudioListener2D current flag; PathFollow2D progress with the engine's loop-wrap (exact-zero becomes full length) and clamp rules, ratio conversion, offsets, cubic/rotation flags and `update_transform` (tangent rotation or offset-only). **Non-differential** |
| `scene/theme.rs` | `scene/resources/theme.*` (colours/constants/font sizes + variations) | Identifier validation and `validate_type_name`, per-type nested maps with add/rename/clear and their exact failures, sorted lists, positive-size font rules with default and fallback 16, and recursive type variations. Icons/styleboxes/fonts not ported. **Non-differential** |
| `scene/range.rs` | `scene/gui/range.*` (value state) | `_calc_value` with min-offset snapping, rounding, `max - page`/`min` clamps and greater/lesser allowances; min/max/page revalidation; linear and exponential `as_ratio`. Signals/sharing not ported. **Non-differential** |
| `scene/gui_indicators.rs` | `scene/gui/{scroll_bar,slider,progress_bar}.*` (state) | ScrollBar step 0/custom step/smooth scroll; Slider ticks, position, editable and scrollable; ProgressBar step 0.01, percentage/indeterminate flags and fill modes with `FILL_MODE_MAX` rejection. **Non-differential** |
| `scene/bit_map.rs` | `scene/resources/bit_map.*` (grid half) | Packed bits with create limits, checked set/get, clipped rect fill, popcount, nearest-neighbour resize, circular grow/shrink masks with rect clipping, and blit. Image import/export and marching squares not ported. **Non-differential** |
| `scene/light_occluder_2d.rs` | `scene/2d/light_occluder_2d.*` | OccluderPolygon2D vertices/closed/cull mode and LightOccluder2D polygon reference, light mask and SDF flag. RIDs/SDF generation not ported. **Non-differential** |
| `os/time.rs` | `core/os/time.*` (conversions) | Proleptic-Gregorian unix↔datetime (floor-correct for negatives), Sunday-based weekday, canonical date/time/datetime strings and parsing, offset `±HH:MM`. System clock/ticks not ported. **Non-differential** |
| `io/json_native.rs` | `core/io/json.*` (`from_native`/`to_native`) | Scalar prefixes, untyped arrays, dictionaries wrapped with key/value pairs like the engine, the ten packed arrays in their encoded forms, and Vector2/2i, Rect2/2i, Vector3/3i, Transform2D, Vector4/4i, Plane, Quaternion, AABB, Basis, Transform3D, Projection, Color in engine argument order. Typed Array/Dictionary metadata and resource serialization not ported (type-only RID/Callable/Signal dicts decode to Nil). **Non-differential** |
| `io/json.rs` | `core/io/json.*` (parse/stringify) | Engine token rules (floats for all numbers), full string escapes incl. surrogate pairs, exact error messages/lines, 1024 depth cap; stringify with sorted keys, optional indent, 14-magnitude fixed precision, `0.0`/inf/NaN handling and JSON escaping. Native conversions not ported. **Non-differential** |
| `scene/panel_container.rs` | `scene/gui/panel_container.*` (layout) | Minimum `child + stylebox` and child cell `Rect2(offset, size - style_min)` with the child's flags via `Control::layout_in_cell`. Theme lookup not ported. **Non-differential** |
| `scene/label_settings.rs` | `scene/resources/label_settings.*` | Line/paragraph spacing, font handle, size 16, colours, shadow defaults and the ordered stacked outline/shadow lists with the engine's add/move/remove/count/validation rules. Text layout not ported. **Non-differential** |
| `scene/bone_map.rs` | `scene/resources/bone_map.*` | Profile revalidation (insert missing, drop unknown, null clears), mapping only existing profile bones, reverse lookup and count. **Non-differential** |
| `scene/velocity_tracker_3d.rs` | `scene/3d/velocity_tracker_3d.*` | Four-frame position history, same-frame overwrite, physics/idle time conversion, 0.2 s averaging window and reset; ticks passed explicitly (no Engine singleton). **Non-differential** |
| `scene/visual_instance_3d.rs` | `scene/3d/visual_instance_3d.*` | VisualInstance layers/sorting (bits 1..20) and GeometryInstance shadow/materials/visibility range/transparency clamp/LOD/instance params/cull margin/custom AABB/lightmap/GI/occlusion state. RIDs not ported. **Non-differential** |
| `scene/occluder_3d.rs` | `scene/3d/{marker_3d,occluder_instance_3d}.*` | Marker gizmo extents 0.25, array/quad/box/sphere/polygon occluders with size/radius clamps, and the instance's occluder handle, bake mask (bits 1..20) and simplification distance. Baking not ported. **Non-differential** |
| `scene/light_3d.rs` | `scene/3d/light_3d.*` | The 21 light params with constructor defaults, colour/masks/bake/temperature, and the directional/omni/spot subclass overrides and enums. RIDs and physical-unit colour mixing not ported. **Non-differential** |
| `scene/reflection_probe.rs` | `scene/3d/reflection_probe.*` | Probe defaults, ambient/update modes, masks, and the origin-offset clamp to `max(size/2, 0.01) - 0.01` on size/offset changes. RID not ported. **Non-differential** |
| `scene/volumes_3d.rs` | `scene/3d/{fog_volume,decal}.*` | FogVolume size clamp/shape/material and Decal size clamp, four textures, mix/modulate/cull mask/fades/distance fade. RIDs not ported. **Non-differential** |
| `scene/world_environment.rs` | `scene/3d/world_environment.*` | Environment/camera-attributes/compositor handles with independent assignment and clearing. Viewport propagation not ported. **Non-differential** |
| `scene/visible_on_screen_notifier_3d.rs` | `scene/3d/visible_on_screen_notifier_3d.*` | AABB `(-1,-1,-1)+(2,2,2)`, on-screen hook, visual state, and the enabler modes with default `..` path. Server registration/signals not ported. **Non-differential** |
| `scene/camera_3d.rs` | `scene/3d/camera_3d.*` (state) | Projection modes, FOV `1..=179`, size `> CMP_EPSILON`, near/far/frustum/h/v offsets, keep-aspect, cull mask `0xfffff` with layer bits `1..=20`, environment/attributes/compositor handles, Doppler mode and current flag. Projection matrix/RID not ported. **Non-differential** |
| `scene/compositor.rs` | `scene/resources/compositor.*` | CompositorEffect defaults (enabled, post-transparent stage, four access flags) and the ordered effect list. Render callbacks/RIDs not ported. **Non-differential** |
| `scene/shader.rs` | `scene/resources/shader.*` | Mode enum (6 modes + Max), code/include path, and indexed default textures where null at index 0 erases the parameter while other indices keep a null entry. Compilation/uniforms/RID not ported. **Non-differential** |
| `scene/material.rs` | `scene/resources/material.*` (base + ShaderMaterial) | Next pass, render priority validated to -128..=127, shader handle and a parameter map where `Nil` erases; sorted parameter list. RenderingServer side not ported. **Non-differential** |
| `scene/skeleton_3d.rs` | `scene/3d/skeleton_3d.*` (core) | Bone add/clear/rename with the engine's name rules, parent validation/ancestry, rests, pose position/rotation/scale, global rest/pose composition with cycle guards, global-pose setting, reset and show-rest-only. Dirty caches/skins/modifiers/overrides not ported. **Non-differential** |
| `scene/skeleton_profile.rs` | `scene/resources/skeleton_profile.*` | Group/bone metadata with resize retention, per-index accessors/defaults, find/has bone (empty name never found) and the read-only flag blocking mutations. **Non-differential** |
| `scene/mesh_instance_3d.rs` | `scene/3d/mesh_instance_3d.*` | Mesh/skin handles, skeleton path, blend-shape tracks with name lookup and index validation, and per-surface override materials. **Non-differential** |
| `scene/importer_mesh_instance_3d.rs` | `scene/3d/importer_mesh_instance_3d.*` | Importer mesh/skin/path, growing surface material list with negative rejection, layer mask, shadow setting and visibility range/fade. **Non-differential** |
| `scene/mesh_library.rs` | `scene/resources/3d/mesh_library.*` | Sorted integer items with create/remove/clear, name/mesh/transform/shadow/preview/navigation data, shape pairs, last-unused id and find-by-name; engine defaults for missing items. **Non-differential** |
| `scene/skin.rs` | `scene/resources/3d/skin.*` | Bind defaults (-1 bone, empty name, identity pose), count resize rejecting negatives, add_bind/add_named_bind, index-validated setters and engine-default reads, clear_binds. **Non-differential** |
| `scene/immediate_mesh.rs` | `scene/resources/immediate_mesh.*` (capture) | Surface begin/end with the double-begin and empty-end rejections, attribute setters that back-fill existing vertices when first enabled, 2D/3D mixing rejection, per-surface primitive/material/attribute arrays and `clear_surfaces`. Packed buffers/format/AABB not ported. **Non-differential** |
| `scene/audio_stream_polyphonic.rs` | `scene/resources/audio_stream_polyphonic.*` (descriptor) | Voices default 32 with `0..=128` validation, default playback type and the meta-stream/monophonic flags. Playback object not ported. **Non-differential** |
| `scene/placeholder_3d.rs` | `scene/resources/placeholder_textures.*` (3D/layered) | 3D `(1,1,1)` size and layered `1x1`/1 layer with the constructor-fixed layered type; unvalidated setters. **Non-differential** |
| `scene/texture_classes.rs` | `scene/resources/atlas_texture.*`, `animated_texture.*` | AtlasTexture atlas/region (floored size)/margin/filter-clip; AnimatedTexture with 256 frame slots, per-frame texture/duration, count 1..=256, current-frame and speed-scale (`-1000` allowed, `1000` rejected) validation. Playback/proxy/image access not ported. **Non-differential** |
| `scene/canvas_item_material.rs` | `scene/resources/canvas_item_material.*` | Six blend modes, three light modes, particles-animation flag and the constructor's 1x1 frames/loop-off state. Shader/RID plumbing not ported. **Non-differential** |
| `scene/camera_attributes.rs` | `scene/resources/camera_attributes.*` | Base/practical/physical attributes with constructor defaults (including the practical `0..800` and physical `-8..10` auto-exposure ranges), DOF fields, the physical FOV from the 24 mm sensor and focal length, practical `sensitivity/3072007` and physical `1/(aperture²·shutter·(100/sensitivity)·1.2)` normalizations. RID/DOF upload not ported. **Non-differential** |
| `scene/mesh_instance_2d.rs` | `scene/2d/{mesh_instance_2d,multimesh_instance_2d}.*` | Mesh/MMesh resource handles plus optional texture. Drawing and navigation callbacks not ported. **Non-differential** |
| `scene/resource_extras.rs` | `scene/resources/physics_material.*`, `color_palette.*` | PhysicsMaterial friction 1 / bounce 0 / rough / absorbent with the negating `computed_*` helpers, and the ColorPalette array. **Non-differential** |
| `scene/aspect_ratio_container.rs` | `scene/gui/aspect_ratio_container.*` (layout) | Ratio box `(ratio, 1)` scaled by width-controls-height / height-controls-width / fit / cover, raised to the child minimum, then aligned by the two alignment fractions; minimum size is the largest child minimum. RTL and the proportional-TextureRect warning not ported. **Non-differential** |
| `scene/parallax_2d.rs` | `scene/2d/parallax_2d.*` (state) | Scroll scale/offset, screen offset, repeat size (`maxf(0)`) and times (`>= 1`), ±10,000,000 limits, autoscroll, follow/ignore flags, group name. Camera-moved math not ported. **Non-differential** |
| `scene/visible_on_screen_notifier_2d.rs` | `scene/2d/visible_on_screen_notifier_2d.*` (state) | Notifier rect/show-rect/on-screen hook and enabler enable modes with default `..` path. Server registration/signals not ported. **Non-differential** |
| `servers/audio_effect_{reverb,delay,chorus,distortion,eq,filter,misc}.rs` | `servers/audio/effects/audio_effect_*.{h,cpp}` | Parameter state for every effect class: Reverb, Delay, Chorus, Distortion, EQ6/10/21, the five filters, Limiter, Amplify, Panner, StereoEnhance, Compressor and PitchShift. Defaults copied from each constructor or in-class initialiser; setters replicate the C++ validation (EQ band index, chorus voice index/count, pitch scale `> 0`, oversampling `>= 4`, FFT size `< MAX`, reverb feedback `0..0.98`) and never clamp where the engine does not. `Amplify` converts through `linear_to_db`/`db_to_linear`. DSP/`process()` is not ported. **Non-differential** (`AudioEffect` is `Resource`-backed) |
| `servers/audio_capture.rs` | `servers/audio/effects/audio_effect_{capture,record}.*` | Capture ring sized by `nearest_shift(mix_rate * seconds)`, size-1 capacity, whole-block push/discard counters, validated exact reads; Record WAV format (16-bit default) and recording flag. IO thread/stream creation not ported. **Non-differential** |
| `servers/audio_bus.rs` | `servers/audio/audio_bus_layout.*` (state) | Master bus that cannot be removed, add/remove with send reindexing, volume/mute/solo/bypass, validated send targets, and `is_bus_active` with the engine's solo rule. Mixing not ported. **Non-differential** |

## Deliberately not translated

- `core/templates/*` containers (`HashMap`, `List`, `CowData`, `PagedAllocator`).
  Rust's `std` replaces them; a line-by-line translation would be dead code.
  `hashfuncs` was ported because hash values are observable.
- `editor/` (332k lines), `platform/` (188k), `modules/mono` (74k),
  `thirdparty/` (2.87M). Not runtime, or not ours.

## Marked pending, with the reason

- `core/math/expression.cpp` is **partially ported** (`math/expression.rs`):
  tokenizer, precedence-climbing parser, array/dictionary literals, variables,
  indexing, unary `- ! ~`, `+ - * / % **`, comparisons, `and/or` with
  short-circuit, bitwise/shifts, `in`/`not in`, arrays/dictionaries, `self`,
  member reads and object method calls, and a math builtin subset. Still
  missing: the engine's full ~100-function builtin table and assignment (which
  the engine's `Expression` does not implement either).

## Known open items

1. ~~`Basis::get_euler`~~ **done and verified for all 6 orders** (wave 5).
2. Color OK-HSL/OK-HSV needs `thirdparty/misc/ok_color.h` ported.
3. `String::num` (integer and float formatting) is blocked the same way as Color
   parity: it lives in `ustring.cpp`. A standalone dump including only
   `ustring.h` cannot provide reference values; the wall is the `.cpp`, not the
   header. Unblocking it means porting `CowData`/`String` far enough to link, or
   accepting non-differential tests.
4. Color differential parity is blocked: `color.cpp` pulls `String` + `HashMap`
   machinery. Needs either a `ustring` build or an extracted numeric TU.
4. `Geometry3D::tetrahedralize_delaunay` is now wired to `delaunay_3d::tetrahedralize` (Bowyer-Watson).
   `core/string`: UTF-8/16 codecs live inside `ustring.cpp` (String), so the
   next Phase 2 step is the `String`/`CowData` pair itself, not a smaller slice. `Projection` plane extraction pending.
   `Basis::diagonalize`, `rotate_sh`, `rotref_posscale_decomposition` pending.
5. ~~SCons integration~~ **DONE AND PROVEN**: the editor binary links the Rust
   static library and executes it. `modules\rustcore\verify_link.cmd` prints
   `OK: Rust core module is linked and running inside the engine binary.`
6. No benchmarks yet; parity is correctness only.

## Rules that must not be broken

- Only POD crosses the FFI boundary (`#[repr(C)]`), never `Variant`, `String`,
  `Object*`.
- Every ported function keeps the exact arithmetic order of the C++ source;
  "cleaner" math that changes rounding is a regression.
- Directories that link `ustring.cpp` in the parity harness are a dead end;
  add a stub only when the symbol is never executed by the dump.
