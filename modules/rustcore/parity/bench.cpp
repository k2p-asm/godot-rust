// Micro-benchmark: the same hot loops in C++ (engine sources) and in the Rust
// port, so the port's cost can be measured rather than guessed.
//
// Build: modules\rustcore\parity\bench.cmd
// Run:   modules\rustcore\parity\bench.exe

#include "core/math/vector3.h"
#include "core/math/transform_3d.h"
#include "core/math/basis.h"

#include <chrono>
#include <cstdio>

static const int N = 2 * 1000 * 1000;

int main() {
	// Deterministic pseudo-random input, identical in the Rust benchmark.
	Vector3 *pts = new Vector3[N];
	uint32_t state = 0x12345678u;
	auto next = [&]() -> float {
		state ^= state << 13;
		state ^= state >> 17;
		state ^= state << 5;
		return (float)(state & 0xFFFF) / 65535.0f - 0.5f;
	};
	for (int i = 0; i < N; i++) {
		pts[i] = Vector3(next() * 100.0f, next() * 100.0f, next() * 100.0f);
	}

	const Basis basis = Basis::from_euler(Vector3(0.3f, -0.7f, 1.1f));
	const Transform3D xform(basis, Vector3(1.0f, 2.0f, 3.0f));

	double acc_dot = 0.0;
	auto t0 = std::chrono::steady_clock::now();
	for (int i = 0; i < N; i++) {
		acc_dot += pts[i].dot(pts[(i + 1) % N]);
	}
	auto t1 = std::chrono::steady_clock::now();

	double acc_x = 0.0, acc_y = 0.0, acc_z = 0.0;
	auto t2 = std::chrono::steady_clock::now();
	for (int i = 0; i < N; i++) {
		Vector3 v = xform.xform(pts[i]);
		acc_x += v.x;
		acc_y += v.y;
		acc_z += v.z;
	}
	auto t3 = std::chrono::steady_clock::now();

	double acc_len = 0.0;
	auto t4 = std::chrono::steady_clock::now();
	for (int i = 0; i < N; i++) {
		acc_len += pts[i].normalized().length();
	}
	auto t5 = std::chrono::steady_clock::now();

	auto ms = [](auto a, auto b) {
		return std::chrono::duration<double, std::milli>(b - a).count();
	};

	printf("cpp dot        %8.2f ms  (%.3f ns/op)  checksum %.6f\n", ms(t0, t1), ms(t0, t1) * 1e6 / N, acc_dot);
	printf("cpp xform      %8.2f ms  (%.3f ns/op)  checksum %.6f\n", ms(t2, t3), ms(t2, t3) * 1e6 / N, acc_x + acc_y + acc_z);
	printf("cpp normalize  %8.2f ms  (%.3f ns/op)  checksum %.6f\n", ms(t4, t5), ms(t4, t5) * 1e6 / N, acc_len);

	delete[] pts;
	return 0;
}
