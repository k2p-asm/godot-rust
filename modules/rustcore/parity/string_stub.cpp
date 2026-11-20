// Stubs for the engine symbols that the math sources reference but that the
// harness never executes: error reporting and String formatting (only used by
// `operator String()`, which the dump does not call). Keeping them here means
// the *numeric* code paths compiled are the untouched engine sources.

#include "core/error/error_macros.h"
#include "core/os/memory.h"
#include "core/string/ustring.h"
#include "core/variant/variant.h"

#include <cstdlib>
#include <cstring>

void _err_print_error(const char *, const char *, int, const char *, bool, ErrorHandlerType) {}
void _err_print_error(const char *, const char *, int, const String &, bool, ErrorHandlerType) {}
void _err_print_error(const char *, const char *, int, const char *, const char *, bool, ErrorHandlerType) {}
void _err_print_error(const char *, const char *, int, const String &, const char *, bool, ErrorHandlerType) {}
void _err_print_error(const char *, const char *, int, const char *, const String &, bool, ErrorHandlerType) {}
void _err_print_error(const char *, const char *, int, const String &, const String &, bool, ErrorHandlerType) {}

void _err_print_index_error(const char *, const char *, int, int64_t, int64_t, const char *, const char *, const char *, bool, bool) {}
void _err_print_index_error(const char *, const char *, int, int64_t, int64_t, const char *, const char *, const String &, bool, bool) {}

Variant::Variant(const Vector3 &) {
	type = NIL;
}

void Variant::_clear_internal() {}

void _err_flush_stdout(void) {}

bool is_print_verbose_enabled(void) {
	return false;
}

void *operator new(unsigned long long p_size, DefaultAllocator) {
	return ::malloc((size_t)p_size);
}

namespace Memory {
template <bool p_ensure_zero>
void *alloc_static(size_t p_bytes, bool) {
	void *p = ::malloc(p_bytes);
	if (p_ensure_zero && p) {
		::memset(p, 0, p_bytes);
	}
	return p;
}
void *realloc_static(void *p_memory, size_t p_bytes, bool) {
	return ::realloc(p_memory, p_bytes);
}
void free_static(void *, bool) {}
} // namespace Memory

template void *Memory::alloc_static<false>(size_t, bool);
template void *Memory::alloc_static<true>(size_t, bool);

String String::operator+(const String &) const {
	return String();
}

String String::operator+(const char *) const {
	return String();
}

String String::operator+(char32_t) const {
	return String();
}

String String::num_real(float, bool) {
	return String();
}

String String::num_real(double, bool) {
	return String();
}

String itos(int64_t) {
	return String();
}

String operator+(const char *, const String &) {
	return String();
}
