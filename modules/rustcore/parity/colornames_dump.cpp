// Independent reader of `core/math/color_names.inc`: scans the file with a
// hand-written parser (not the same regex the Rust generator uses) and prints
// every entry, so the generated Rust table can be cross-checked.
//
// Build: cl /nologo /std:c++20 /EHsc /O2 modules\rustcore\parity\colornames_dump.cpp

#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

struct Entry {
	std::string name;
	unsigned value;
};

int main() {
	FILE *f = fopen("core/math/color_names.inc", "rb");
	if (!f) {
		printf("missing color_names.inc\n");
		return 1;
	}
	std::string src;
	char buf[4096];
	size_t n;
	while ((n = fread(buf, 1, sizeof(buf), f)) > 0) {
		src.append(buf, n);
	}
	fclose(f);

	std::vector<Entry> entries;
	const char *p = src.c_str();
	while ((p = strstr(p, "{ \"")) != nullptr) {
		p += 3;
		const char *end = strchr(p, '"');
		if (!end) {
			break;
		}
		std::string name(p, end - p);
		p = end;
		const char *hexpos = strstr(p, "0x");
		if (!hexpos) {
			break;
		}
		unsigned value = (unsigned)strtoul(hexpos, nullptr, 0);
		entries.push_back({ name, value });
		p = hexpos + 2;
	}

	printf("count,%zu\n", entries.size());
	for (const Entry &e : entries) {
		printf("%s,0x%08X\n", e.name.c_str(), e.value);
	}
	return 0;
}
