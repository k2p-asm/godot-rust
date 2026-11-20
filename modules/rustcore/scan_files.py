import io, os, re

root = r"C:\Users\k2p\godot-rust"
rust_dir = os.path.join(root, "modules", "rustcore", "src")
rust_files = {}
for dirpath, _, names in os.walk(rust_dir):
    for name in names:
        if name.endswith(".rs"):
            rust_files[name[:-3]] = os.path.join(dirpath, name)

rust_text = []
for path in rust_files.values():
    with io.open(path, "r", encoding="utf-8", errors="ignore") as f:
        rust_text.append(f.read())
rust_text = "\n".join(rust_text)

for folder in ["scene/3d", "scene/2d", "scene/gui", "scene/resources/3d", "scene/resources/2d"]:
    path = os.path.join(root, *folder.split("/"))
    print("===", folder, "===")
    for name in sorted(os.listdir(path)):
        if not name.endswith(".h"):
            continue
        stem = name[:-2]
        # Skip headers whose Rust module already exists.
        if stem in rust_files:
            continue
        with io.open(os.path.join(path, name), "r", encoding="utf-8", errors="ignore") as f:
            text = f.read()
        m = re.search(r"class\s+(?:\[\[nodiscard\]\]\s+)?([A-Za-z_][A-Za-z0-9_]*)", text)
        cls = m.group(1) if m else "?"
        if cls != "?" and re.search(r"\b" + re.escape(cls) + r"\b", rust_text):
            continue
        print(stem, ":", cls)
