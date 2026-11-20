// Reference values for the Okhsl/Okhsv port. Includes the vendored header
// directly, so it needs no engine translation unit and therefore avoids the
// `String`/`HashMap` link wall that blocks color.cpp.
//
// Build: cl /nologo /std:c++20 /EHsc /O2 /I . modules\rustcore\parity\okcolor_dump.cpp

#include "thirdparty/misc/ok_color.h"

#include <cstdio>

static void rgb(const char *name, ok_color::RGB c) {
	printf("%s,%.9g,%.9g,%.9g\n", name, (double)c.r, (double)c.g, (double)c.b);
}

static void hsl(const char *name, ok_color::HSL c) {
	printf("%s,%.9g,%.9g,%.9g\n", name, (double)c.h, (double)c.s, (double)c.l);
}

static void hsv(const char *name, ok_color::HSV c) {
	printf("%s,%.9g,%.9g,%.9g\n", name, (double)c.h, (double)c.s, (double)c.v);
}

int main() {
	rgb("ok.hsl_red", ok_color::okhsl_to_srgb({ 0.0f, 1.0f, 0.5f }));
	rgb("ok.hsl_green", ok_color::okhsl_to_srgb({ 0.333f, 0.8f, 0.6f }));
	rgb("ok.hsl_blue", ok_color::okhsl_to_srgb({ 0.666f, 0.55f, 0.35f }));
	rgb("ok.hsl_mid", ok_color::okhsl_to_srgb({ 0.12f, 0.4f, 0.72f }));

	hsl("ok.to_hsl_1", ok_color::srgb_to_okhsl({ 0.2f, 0.6f, 0.9f }));
	hsl("ok.to_hsl_2", ok_color::srgb_to_okhsl({ 0.9f, 0.1f, 0.05f }));

	rgb("ok.hsv_1", ok_color::okhsv_to_srgb({ 0.25f, 0.7f, 0.8f }));
	rgb("ok.hsv_2", ok_color::okhsv_to_srgb({ 0.8f, 0.35f, 0.5f }));
	hsv("ok.to_hsv_1", ok_color::srgb_to_okhsv({ 0.2f, 0.6f, 0.9f }));
	hsv("ok.to_hsv_2", ok_color::srgb_to_okhsv({ 0.9f, 0.1f, 0.05f }));

	return 0;
}
