// Verifies that a C++ translation unit can call the Rust static library through
// the C ABI declared in `modules/rustcore/src/ffi.rs`, when built by SCons.

#include <cstdio>

struct V3 {
	float x, y, z;
};

extern "C" unsigned gcr_abi_version();
extern "C" float gcr_vector3_dot(V3 a, V3 b);
extern "C" V3 gcr_transform3d_xform(const float *rows, V3 origin, V3 v);

int main() {
	V3 a = { 1.0f, 2.0f, 3.0f };
	V3 b = { 4.0f, 5.0f, 6.0f };
	float dot = gcr_vector3_dot(a, b);

	const float rows[9] = { 1, 0, 0, 0, 1, 0, 0, 0, 1 };
	V3 origin = { 1.0f, 2.0f, 3.0f };
	V3 p = { 0.5f, 0.5f, 0.5f };
	V3 out = gcr_transform3d_xform(rows, origin, p);

	printf("abi=%u dot=%.1f xform=(%.1f, %.1f, %.1f)\n", gcr_abi_version(), dot, out.x, out.y, out.z);

	bool ok = gcr_abi_version() == 1 && dot == 32.0f && out.x == 1.5f && out.y == 2.5f && out.z == 3.5f;
	printf(ok ? "ABI OK\n" : "ABI MISMATCH\n");
	return ok ? 0 : 1;
}
