"""Extracts the named-color table from `core/math/color_names.inc` and emits a
Rust source file, so the data is never typed by hand.

Usage: python modules/rustcore/tools/gen_color_names.py
"""

import os
import re

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
SRC = os.path.join(ROOT, "core", "math", "color_names.inc")
OUT = os.path.join(ROOT, "modules", "rustcore", "src", "math", "color_names.rs")

text = open(SRC, "r", encoding="utf-8", errors="replace").read()

entries = re.findall(r'\{\s*"([A-Z0-9_]+)"\s*,\s*Color::hex\(\s*0x([0-9A-Fa-f]+)\s*\)\s*\}', text)
if not entries:
    raise SystemExit("no entries parsed")

lines = []
lines.append("//! Generated from `core/math/color_names.inc` by")
lines.append("//! `modules/rustcore/tools/gen_color_names.py`. Do not edit by hand.")
lines.append("")
lines.append("/// `(NAME, 0xRRGGBBAA)` pairs, in the engine's order.")
lines.append("pub static NAMED_COLORS: [(&str, u32); %d] = [" % len(entries))
for name, hexv in entries:
    lines.append('    ("%s", 0x%08X),' % (name, int(hexv, 16)))
lines.append("];")
lines.append("")

with open(OUT, "w", encoding="utf-8", newline="\n") as f:
    f.write("\n".join(lines))

print("parsed %d named colors" % len(entries))
print("wrote", OUT)
