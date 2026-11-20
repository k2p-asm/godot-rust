import io, os, re

root = r"C:\Users\k2p\godot-rust"
rust_dir = os.path.join(root, "modules", "rustcore", "src")
rust = []
for dirpath, _, names in os.walk(rust_dir):
    for name in names:
        if name.endswith(".rs"):
            with io.open(os.path.join(dirpath, name), "r", encoding="utf-8", errors="ignore") as f:
                rust.append(f.read())
rust = "\n".join(rust)

for folder in ["scene/3d", "scene/2d", "scene/resources", "scene/main"]:
    path = os.path.join(root, *folder.split("/"))
    print("===", folder, "===")
    for name in sorted(os.listdir(path)):
        if not name.endswith(".h"):
            continue
        with io.open(os.path.join(path, name), "r", encoding="utf-8", errors="ignore") as f:
            text = f.read()
        m = re.search(r"class\s+(?:\[\[nodiscard\]\]\s+)?([A-Za-z_][A-Za-z0-9_]*)", text)
        if not m:
            continue
        cls = m.group(1)
        if not re.search(r"\b" + re.escape(cls) + r"\b", rust):
            print(name, ":", cls)
