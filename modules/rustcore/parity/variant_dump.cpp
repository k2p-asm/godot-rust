// Prints the engine's own `Variant::Type` values so the Rust enum's
// discriminants can be checked differentially. This needs no linkage beyond the
// header: the enum is compile-time data.
//
// Build: cl /nologo /std:c++20 /Zc:__cplusplus /EHsc /O2 /I . /I platform\windows modules\rustcore\parity\variant_dump.cpp

#include "core/variant/variant.h"

#include <cstdio>

int main() {
	printf("vt.NIL,%d\n", (int)Variant::NIL);
	printf("vt.BOOL,%d\n", (int)Variant::BOOL);
	printf("vt.INT,%d\n", (int)Variant::INT);
	printf("vt.FLOAT,%d\n", (int)Variant::FLOAT);
	printf("vt.STRING,%d\n", (int)Variant::STRING);
	printf("vt.VECTOR2,%d\n", (int)Variant::VECTOR2);
	printf("vt.VECTOR2I,%d\n", (int)Variant::VECTOR2I);
	printf("vt.RECT2,%d\n", (int)Variant::RECT2);
	printf("vt.RECT2I,%d\n", (int)Variant::RECT2I);
	printf("vt.VECTOR3,%d\n", (int)Variant::VECTOR3);
	printf("vt.VECTOR3I,%d\n", (int)Variant::VECTOR3I);
	printf("vt.TRANSFORM2D,%d\n", (int)Variant::TRANSFORM2D);
	printf("vt.VECTOR4,%d\n", (int)Variant::VECTOR4);
	printf("vt.VECTOR4I,%d\n", (int)Variant::VECTOR4I);
	printf("vt.PLANE,%d\n", (int)Variant::PLANE);
	printf("vt.QUATERNION,%d\n", (int)Variant::QUATERNION);
	printf("vt.AABB,%d\n", (int)Variant::AABB);
	printf("vt.BASIS,%d\n", (int)Variant::BASIS);
	printf("vt.TRANSFORM3D,%d\n", (int)Variant::TRANSFORM3D);
	printf("vt.PROJECTION,%d\n", (int)Variant::PROJECTION);
	printf("vt.COLOR,%d\n", (int)Variant::COLOR);
	printf("vt.STRING_NAME,%d\n", (int)Variant::STRING_NAME);
	printf("vt.NODE_PATH,%d\n", (int)Variant::NODE_PATH);
	printf("vt.RID,%d\n", (int)Variant::RID);
	printf("vt.OBJECT,%d\n", (int)Variant::OBJECT);
	printf("vt.CALLABLE,%d\n", (int)Variant::CALLABLE);
	printf("vt.SIGNAL,%d\n", (int)Variant::SIGNAL);
	printf("vt.ARRAY,%d\n", (int)Variant::ARRAY);
	printf("vt.DICTIONARY,%d\n", (int)Variant::DICTIONARY);
	printf("vt.PACKED_BYTE_ARRAY,%d\n", (int)Variant::PACKED_BYTE_ARRAY);
	printf("vt.PACKED_INT32_ARRAY,%d\n", (int)Variant::PACKED_INT32_ARRAY);
	printf("vt.PACKED_INT64_ARRAY,%d\n", (int)Variant::PACKED_INT64_ARRAY);
	printf("vt.PACKED_FLOAT32_ARRAY,%d\n", (int)Variant::PACKED_FLOAT32_ARRAY);
	printf("vt.PACKED_FLOAT64_ARRAY,%d\n", (int)Variant::PACKED_FLOAT64_ARRAY);
	printf("vt.PACKED_STRING_ARRAY,%d\n", (int)Variant::PACKED_STRING_ARRAY);
	printf("vt.PACKED_VECTOR2_ARRAY,%d\n", (int)Variant::PACKED_VECTOR2_ARRAY);
	printf("vt.PACKED_VECTOR3_ARRAY,%d\n", (int)Variant::PACKED_VECTOR3_ARRAY);
	printf("vt.PACKED_COLOR_ARRAY,%d\n", (int)Variant::PACKED_COLOR_ARRAY);
	printf("vt.PACKED_VECTOR4_ARRAY,%d\n", (int)Variant::PACKED_VECTOR4_ARRAY);
	printf("vt.TYPE_MAX,%d\n", (int)Variant::VARIANT_MAX);
	return 0;
}
