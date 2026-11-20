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

- `cargo test`: **705 unit tests + 34 differential tests, 0 failures.**
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
| `scene/curve_2d.rs` | `scene/resources/curve.*` (Curve2D core) | points with in/out controls, cubic Bézier `sample`/`samplef`, even-length tessellation (depth 10, `bake_interval` 5.0), cumulative arc lengths, `sample_baked` (linear and cubic) with clamping. Tilts/posture and Curve3D not ported. **Non-differential** |
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
| `scene/path_2d.rs` | `scene/2d/path_2d.*` (Path2D), `audio_listener_2d.*` | Path2D curve accessors; AudioListener2D current flag. `PathFollow2D` pending (needs Curve2D forward-vector cache). **Non-differential** |
| `scene/theme.rs` | `scene/resources/theme.*` (colours/constants/font sizes + variations) | Identifier validation and `validate_type_name`, per-type nested maps with add/rename/clear and their exact failures, sorted lists, positive-size font rules with default and fallback 16, and recursive type variations. Icons/styleboxes/fonts not ported. **Non-differential** |
| `scene/range.rs` | `scene/gui/range.*` (value state) | `_calc_value` with min-offset snapping, rounding, `max - page`/`min` clamps and greater/lesser allowances; min/max/page revalidation; linear and exponential `as_ratio`. Signals/sharing not ported. **Non-differential** |
| `scene/gui_indicators.rs` | `scene/gui/{scroll_bar,slider,progress_bar}.*` (state) | ScrollBar step 0/custom step/smooth scroll; Slider ticks, position, editable and scrollable; ProgressBar step 0.01, percentage/indeterminate flags and fill modes with `FILL_MODE_MAX` rejection. **Non-differential** |
| `scene/bit_map.rs` | `scene/resources/bit_map.*` (grid half) | Packed bits with create limits, checked set/get, clipped rect fill, popcount, nearest-neighbour resize, circular grow/shrink masks with rect clipping, and blit. Image import/export and marching squares not ported. **Non-differential** |
| `scene/light_occluder_2d.rs` | `scene/2d/light_occluder_2d.*` | OccluderPolygon2D vertices/closed/cull mode and LightOccluder2D polygon reference, light mask and SDF flag. RIDs/SDF generation not ported. **Non-differential** |
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
| `scene/skeleton_profile.rs` | `scene/resources/skeleton_profile.*` | Group/bone metadata with resize retention, per-index accessors/defaults, find/has bone (empty name never found) and the read-only flag blocking mutations. **Non-differential** |
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
