"""Extracts the generated Unicode range tables from the fork's char_range.cpp
and emits a Rust source file. The data is copied mechanically: no ranges are
typed by hand.

Usage: python modules/rustcore/tools/gen_char_range.py
"""

import re
import os

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
SRC = os.path.join(ROOT, "core", "string", "char_range.cpp")
OUT = os.path.join(ROOT, "modules", "rustcore", "src", "string", "char_range.rs")

TABLES = [
    ("xid_start", "XID_START"),
    ("xid_continue", "XID_CONTINUE"),
    ("uppercase_letter", "UPPERCASE_LETTER"),
    ("lowercase_letter", "LOWERCASE_LETTER"),
    ("unicode_letter", "UNICODE_LETTER"),
]

text = open(SRC, "r", encoding="utf-8", errors="replace").read()

lines = []
lines.append("//! Generated from `core/string/char_range.cpp` by "
             "`modules/rustcore/tools/gen_char_range.py`.")
lines.append("//! Do not edit by hand: regenerate instead.")
lines.append("")

for c_name, rs_name in TABLES:
    m = re.search(
        r"const\s+CharRange\s+" + c_name + r"\s*\[[^\]]*\]\s*=\s*\{(.*?)\}\s*;",
        text,
        re.S,
    )
    if not m:
        raise SystemExit("table not found: " + c_name)
    pairs = re.findall(r"\{\s*(0x[0-9a-fA-F]+|\d+)\s*,\s*(0x[0-9a-fA-F]+|\d+)\s*\}", m.group(1))
    entries = [(int(a, 0), int(b, 0)) for a, b in pairs]

    lines.append("pub static %s: [(u32, u32); %d] = [" % (rs_name, len(entries)))
    for a, b in entries:
        lines.append("    (0x%X, 0x%X)," % (a, b))
    lines.append("];")
    lines.append("")
    print("%-18s %d ranges" % (c_name, len(entries)))

with open(OUT, "w", encoding="utf-8", newline="\n") as f:
    f.write("\n".join(lines))

print("wrote", OUT)
