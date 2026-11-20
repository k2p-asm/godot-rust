// Exhaustive check of the character predicates against the vendored header.
// Every code point in 0..=0x10FFFF is classified on both sides and summarised
// as (count, hash), which keeps the fixture tiny while covering the full range.
//
// Build: cl /nologo /std:c++20 /EHsc /O2 /I . /I platform\windows modules\rustcore\parity\charutils_dump.cpp

#include "core/string/char_utils.h"

#include <cstdio>

template <typename F>
static void check(const char *name, F pred) {
	unsigned long long count = 0;
	unsigned long long hash = 1469598103934665603ULL;
	for (unsigned int c = 0; c <= 0x10FFFF; c++) {
		if (pred((char32_t)c)) {
			count++;
			hash = (hash ^ c) * 1099511628211ULL;
		}
	}
	printf("%s,%llu,%llu\n", name, count, hash);
}

int main() {
	check("cu.is_ascii_upper_case", [](char32_t c) { return is_ascii_upper_case(c); });
	check("cu.is_ascii_lower_case", [](char32_t c) { return is_ascii_lower_case(c); });
	check("cu.is_digit", [](char32_t c) { return is_digit(c); });
	check("cu.is_hex_digit", [](char32_t c) { return is_hex_digit(c); });
	check("cu.is_binary_digit", [](char32_t c) { return is_binary_digit(c); });
	check("cu.is_ascii_alphabet_char", [](char32_t c) { return is_ascii_alphabet_char(c); });
	check("cu.is_ascii_alphanumeric_char", [](char32_t c) { return is_ascii_alphanumeric_char(c); });
	check("cu.is_ascii_identifier_char", [](char32_t c) { return is_ascii_identifier_char(c); });
	check("cu.is_symbol", [](char32_t c) { return is_symbol(c); });
	check("cu.is_control", [](char32_t c) { return is_control(c); });
	check("cu.is_whitespace", [](char32_t c) { return is_whitespace(c); });
	check("cu.is_linebreak", [](char32_t c) { return is_linebreak(c); });
	check("cu.is_punct", [](char32_t c) { return is_punct(c); });
	check("cu.is_underscore", [](char32_t c) { return is_underscore(c); });
	check("cu.is_hyphen", [](char32_t c) { return is_hyphen(c); });
	check("cu.is_unicode_identifier_start", [](char32_t c) { return is_unicode_identifier_start(c); });
	check("cu.is_unicode_identifier_continue", [](char32_t c) { return is_unicode_identifier_continue(c); });
	check("cu.is_unicode_upper_case", [](char32_t c) { return is_unicode_upper_case(c); });
	check("cu.is_unicode_lower_case", [](char32_t c) { return is_unicode_lower_case(c); });
	check("cu.is_unicode_letter", [](char32_t c) { return is_unicode_letter(c); });
	return 0;
}
