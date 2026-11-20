import io, os

path = os.path.join(os.path.dirname(__file__), "PORT_STATE.md")
with io.open(path, "r", encoding="utf-8") as f:
    text = f.read()

# Repair literal backtick-n sequences left by shell edits.
text = text.replace("`n", "\n")

with io.open(path, "w", encoding="utf-8", newline="\n") as f:
    f.write(text)

print("repaired", text.count("line_builder.rs"), text.count("1577 unit"))
