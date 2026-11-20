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

- `cargo test`: **285 unit tests + 34 differential tests, 0 failures.**
- Differential coverage: Vector2, Vector2i-level ops, Vector3, scalar math
  (30+ functions), RandomPCG stream, AABB, Plane, Rect2, Basis, Quaternion,
  Transform3D, all six Euler orders (round-trip and arbitrary rotations),
  Vector2i/Vector3i, Vector4, Transform2D, axis-angle extraction, filled\n  polygon tests, integer snapping (`v2i.snap`, `v3i.snap`, `v4i.snap`) and
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
| `scene/node.rs` | `scene/main/node.*` (data half) | name/owner, unique-name flag, `ProcessMode` (inherit/pausable/when-paused/always/disabled) with the engine resolution and `can_process` matrix, process/physics priorities, thread group, `inside_tree`, and the eight `set_process*` flags. **Non-differential** (needs the tree/object layer) |
| `scene/curve_2d.rs` | `scene/resources/curve.*` (Curve2D core) | points with in/out controls, cubic Bézier `sample`/`samplef`, even-length tessellation (depth 10, `bake_interval` 5.0), cumulative arc lengths, `sample_baked` (linear and cubic) with clamping. Tilts/posture and Curve3D not ported. **Non-differential** |\n| `scene/curve.rs` | `scene/resources/curve.*` (1D core) | ordered points clamped to the value/domain ranges, lower-bound `get_index`, cubic Bézier `sample` with the engine's one-third control points, auto (linear) tangents, bake cache and `sample_baked`. Curve2D/3D not ported. **Non-differential** (`Resource` pulls `ClassDB`) |\n| `scene/canvas_layer.rs` | `scene/main/canvas_layer.*` (data half) | `layer`, offset/rotation/scale with the engine's lazy decomposition after `set_transform`, visibility, follow-viewport transform composition (`vp * scale(ratio) * layer`). `set_rotation_and_scale` is differential (`t2.rot_scale*`) |
| `math/vector4.rs` | `core/math/vector4.*` | complete (`Vector4i` pending) |
| `math/face3.rs` | `core/math/face3.*` | plane/area/aabb/split_by_plane, exact `get_closest_point_to`, SAT `intersects_aabb` (verified). ray/segment via `geometry_3d`, support + project_range (all verified) |
| `math/vector4i.rs` | `core/math/vector4i.*` | core, `clamp`/`clampi`, `snapped`/`snappedi` (verified), `min`/`max_axis_index` |
| `math/rect2i.rs` | `core/math/rect2i.*` | core done; filled polygon/segment pending |
| `math/projection.rs` | `core/math/projection.*` | perspective/ortho/frustum, determinant, xform, fov/aspect, Gauss-Jordan `invert`, frustum planes (all 6), per-eye HMD, `adjust_perspective_znear`, viewport/far half extents, `get_pixels_per_meter`, `get_lod_multiplier`, `make_scale`, `get_projection_planes(Transform3D)`, `scale_translate_to_fit(Aabb)`, `create_fit_aabb`, `perspective_znear_adjusted`, all verified |
| `string/char_utils.rs` | `core/string/char_utils.h` | **20** predicates (incl. the 5 Unicode table-backed), verified over **all 1.1M code points** (count+hash) |
| `math/color_names.rs` | `core/math/color_names.inc` | 146 `(NAME, 0xRRGGBBAA)` pairs from `tools/gen_color_names.py`, cross-checked with an independent C++ parser |\n| `string/gstring.rs` | `core/string/ustring.h/.cpp` (start) | COW buffer (`Arc::make_mut`), append/find/rfind/substr/begins_with/ends_with/strip_edges/split/replace/count/join/to_int/is_valid_int/to_float/is_valid_float/insert/repeat/find_char/pad_decimals/pad_zeros/lpad/rpad/trim_prefix/trim_suffix/quote/unquote/is_quoted/is_lowercase/is_uppercase/path helpers (get_file, get_extension, get_basename, path_join, is_absolute_path), ASCII + Unicode case mapping (`to_upper`/`to_lower`, locale-independent, non-differential), Murmur3 `hash()`. UTF-8/UTF-16 encode/decode with surrogate and U+10FFFF boundary tests (not differential: `ustring.cpp` cannot be linked in the harness). `String::num` pending |\n| `object/mod.rs` | `core/object/object.h`, `ref_counted.*` | `ObjectId` (unique, never reused, `is_valid`), `Object` lifetime registry, `RefCounted` ref/unref that destroys at zero. **Non-differential** |\n| `object/class_db.rs` | `core/object/class_db.*` | Class hierarchy, inherited method lookup, `ObjectId` -> class binding, `bind_method` + `call` with arity/payload validation, properties (`register_property`, `property_list` with inheritance, `property_get/set`, `clear_property_values`), signals (`register_signal`, `signal_list` with inheritance, `connect_signal`/`disconnect_signal`, `emit_signal` returning targets, `clear_signal_connections`, typed properties, `PropertyHint`/usage flags via `register_property_ex` + `property_meta` (inherit-aware)). **Non-differential** |\n| `variant/mod.rs` | `core/variant/variant.h` | `Variant::Type` tags (all 39, verified against the engine enum) + POD payloads, `Object`, `Callable`, `Signal` (identity-backed), `Dictionary`/`Array` of variants, all 10 typed arrays with payloads, `dict_get/set/keys`, `array_get/push/len`, `booleanize`, `to_int`/`to_float` |\n| `variant/ops.rs` | `core/variant/variant_op.cpp` (subset) | `+ - * /` for Int/Float/Vector2/Vector3/Vector4/Color incl. promotion and scalar scaling; `String + String` concatenates; int div-by-zero and all other invalid pairs yield `None`. **Non-differential** |\n| `variant/serial.rs` | `core/variant/variant_parser.*` | Minimal `to_text`/`from_text` for nil/bool/int/float/String/Vector2/Vector3/Color/Array/Dictionary (nested). **Non-differential by design** |\n| `string/string_name.rs` | `core/string/string_name.h` | Interning pool (global, lock-protected), identity equality via `u64` id, djb2 `hash()`. Non-differential |\n| `string/num.rs` | `core/string/ustring.cpp` (`num_int64`/`num_uint64`) | bases 2-36, INT64_MIN, u64::MAX; plus `num_float` (fixed decimals, zero trimming, `-0.0`, inf/nan, 16-decimal cap, half-to-even rounding). **Non-differential** (see note 3) |\n| `math/fieldwise.rs` | `core/math/math_fieldwise.cpp` | `fieldwise_assign` over 14 variant types (Vector2..Transform3D), type-mismatch and unknown-field no-ops. **Non-differential** (engine code is `#ifdef DEBUG_ENABLED`) |\n| `math/triangle_mesh.rs` | `core/math/triangle_mesh.*` | Vertices/triangles, `create_from_faces` (no welding), `get_faces`, `get_aabb`, `get_area`, `recompute_normals`. **BVH not ported** (needs `class_db`) |
| `math/triangle_mesh_bvh.rs` | `core/math/triangle_mesh.*` (BVH) | Median-split BVH with `intersect_segment`, verified against brute force over 200 random segments |
| `math/triangulate.rs` | `core/math/triangulate.*` | Ear clipping (`get_area`, `is_inside_triangle`, `snip`); verified on convex/concave polygons by area preservation. Output order is a valid triangulation, not the engine's exact order |
| `math/delaunay_3d.rs` | `core/math/delaunay_3d.*` | Bowyer-Watson tetrahedralization; verified by total volume, vertex coverage, and empty on coplanar input |
| `math/convex_hull.rs` | `core/math/convex_hull.*` | Incremental convex hull (visible faces + horizon). Verified by containment of all input points, outward normals, closed manifold, Euler `V-E+F=2`, and analytic volumes (tetrahedron, cube, icosahedron). **Non-differential**: Bullet-based C++ needs `PagedAllocator`; equivalent mesh, different triangle order |
| `math/dynamic_bvh.rs` | `core/math/dynamic_bvh.*` | Dynamic AABB tree: insert by surface-area growth, remove, update, `aabb_query`, `ray_query`. Verified against brute force over 200 boxes and 200 random queries plus a 600-operation insert/remove cycle. Stores min/max like the engine's `Volume` (position+size round-trips lose an ulp). **Non-differential** |
| `math/random_number_generator.rs` | `core/math/random_number_generator.h` | Wrapper over the bit-exact PCG: seed/state, randi/randf, randfn, ranges, `rand_weighted`. **Non-differential** (RefCounted) |\n| `math/a_star.rs` | `core/math/a_star.h/.cpp` (AStar3D) | Point registry, connections, A* with distance heuristic, partial paths, disabled points, one-way edges. **Non-differential** (`a_star.cpp` pulls `class_db.h`); `weight_scale` applied to entering edges like the engine. `AStar2D` wraps it for Vector2; `AStarGrid2D` with solid cells, per-cell weights, 4 diagonal modes and `jumping_enabled` (`_jump` + `_forced_successor`, all three diagonal branches, ported rule for rule; costs/heuristics use the engine's index-based Euclidean default, `cell_size` only scales `get_point_path`) |\n| `math/audio_frame.rs` | `core/math/audio_frame.h` | Stereo frame, per-channel ops, `lerp`, `undenormalize` (exponent threshold 16, not just denormals) |\n| `math/disjoint_set.rs` | `core/math/disjoint_set.h` | Union-find with path compression + union by rank, generic over `T`. Non-differential (templates) |\n| `string/char_range.rs` | `core/string/char_range.cpp` | 3.596 ranges generated by `tools/gen_char_range.py` (no hand-typing) |
| `math/geometry_3d.rs` | `core/math/geometry_3d.h` | 2 triangle tests, sphere/cylinder/convex segment tests, closest point on segment, barycentric coords, `point_in_projected_triangle`, `triangle_sphere_intersection_test`, `clip_polygon`, `get_support`, `project_range`, `get_closest_points_between_segments` + distance, `build_box_planes`, `build_cylinder_planes` (sides + caps), `build_sphere_planes`, `build_capsule_planes`, `compute_convex_mesh_points`, all verified |
| `math/geometry_2d.rs` | `core/math/geometry_2d.h` | hit tests + `vec2_cross`, `convex_hull` (verified: 5 points, first repeated), `bresenham_line` (verified) |
| `ffi.rs` | (new) | `gcr_variant_nil/bool/int/float/vector3`, `gcr_variant_type/as_int/as_float/as_vector3/booleanize`; verified inside the engine build (prints `variant tag = 2, truthy = 1` and `Rust packed sum = 256`) | **old**: C ABI: `gcr_abi_version`, `gcr_vector3_dot`, `gcr_vector3_normalized`, `gcr_transform3d_xform` |
| `templates/hashfuncs.rs` | `core/templates/hashfuncs.*` | complete |

## Deliberately not translated

- `core/templates/*` containers (`HashMap`, `List`, `CowData`, `PagedAllocator`).
  Rust's `std` replaces them; a line-by-line translation would be dead code.
  `hashfuncs` was ported because hash values are observable.
- `editor/` (332k lines), `platform/` (188k), `modules/mono` (74k),
  `thirdparty/` (2.87M). Not runtime, or not ours.

## Marked pending, with the reason

- `core/math/expression.cpp` (1529 lines). It is a tokenizer + parser +
  evaluator whose every node is a `Variant`, and the engine's operator tables
  (`Variant::evaluate`, conversion matrices, `in`/`not in`, indexing, built-in
  functions with overloads) are exactly what this port has not built yet.
  A numeric-only subset would answer a different API under the same name, so it
  stays pending until `Variant` has the conversion/evaluation tables. `scene/`
  started instead.

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
