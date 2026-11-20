import io, os

path = os.path.join(os.path.dirname(__file__), "PORT_STATE.md")
with io.open(path, "r", encoding="utf-8") as f:
    text = f.read()

# Repair literal backtick-n escapes that earlier shell edits left behind.
text = text.replace("`n", "\n")

lines = text.split("\n")
seen = set()
out = []
for line in lines:
    stripped = line.strip()
    # Drop exact duplicate table rows (the phaser row got inserted twice).
    if stripped.startswith("|") and stripped in seen:
        continue
    if stripped.startswith("|"):
        seen.add(stripped)
    out.append(line)

text = "\n".join(out)

row = ("| `audio/effects_stereo_enhance.rs` | `servers/audio/effects/audio_effect_stereo_enhance.*` | "
       "Pan pull-out towards the centre, the delayed right channel, the surround mode adding/subtracting the "
       "delayed mid, and the engine's next-power-of-two ring sizing from `MAX_DELAY_MS + 2`. Tests cover "
       "defaults, ring sizes, full-pullout identity, pull-out mixing, right-channel delay and surround. "
       "**Non-differential** |")
if "audio/effects_stereo_enhance.rs" not in text:
    marker = "| `audio/effects_phaser.rs` |"
    index = text.find(marker)
    assert index != -1, "phaser row missing"
    text = text[:index] + row + "\n" + text[index:]

with io.open(path, "w", encoding="utf-8", newline="\n") as f:
    f.write(text)

print("ok", text.count("audio/effects_phaser.rs"), text.count("audio/effects_stereo_enhance.rs"))
