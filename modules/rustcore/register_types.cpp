/**************************************************************************/
/*  register_types.cpp                                                    */
/**************************************************************************/
/*                         This file is part of:                          */
/*                             GODOT ENGINE                               */
/*                        https://godotengine.org                         */
/**************************************************************************/

#include "register_types.h"

#include "core/string/print_string.h"
#include "core/string/ustring.h"
#include "core/variant/variant.h"

// Struct shared with `#[repr(C)] struct CGcrVector3` in the Rust crate.
struct CGcrVector3 {
	float x, y, z;
};

// C ABI exposed by the Rust crate (modules/rustcore/src/ffi.rs).
extern "C" unsigned gcr_abi_version();
extern "C" float gcr_vector3_dot(CGcrVector3 a, CGcrVector3 b);
extern "C" CGcrVector3 gcr_vector3_normalized(CGcrVector3 v);

struct CGcrVariant {
	unsigned tag;
	unsigned long long bits;
	float data[4];
};
extern "C" CGcrVariant gcr_variant_int(long long v);
extern "C" int gcr_variant_booleanize(CGcrVariant v);
extern "C" unsigned gcr_variant_type(CGcrVariant v);
extern "C" unsigned long long gcr_packed_byte_sum(const unsigned char *data, unsigned len);

void initialize_rustcore_module(ModuleInitializationLevel p_level) {
	if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
		return;
	}

	// Smoke check: proves the Rust static library is linked and executing.
	CGcrVector3 a = { 1.0f, 2.0f, 3.0f };
	CGcrVector3 b = { 4.0f, 5.0f, 6.0f };
	CGcrVector3 n = gcr_vector3_normalized({ 0.0f, 0.0f, 4.0f });
	CGcrVariant vi = gcr_variant_int(-42);
	print_line(vformat("Rust core module: ABI %d, dot = %.1f, normalized z = %.3f, variant tag = %d, truthy = %d",
			(int)gcr_abi_version(), gcr_vector3_dot(a, b), n.z,
			(int)gcr_variant_type(vi), gcr_variant_booleanize(vi)));
	unsigned char packed[4] = { 1, 2, 3, 250 };
	print_line(vformat("Rust packed sum = %d", (int)gcr_packed_byte_sum(packed, 4)));
}

void uninitialize_rustcore_module(ModuleInitializationLevel p_level) {
}
