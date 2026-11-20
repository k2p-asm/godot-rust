import os
import sys


def can_build(env, platform):
    return True


def configure(env):
    """Builds the Rust crate and links it into the engine binary.

    Note: this module is plain Python, so SCons' global functions (`Dir`,
    `Glob`) are not available here; the `env.*` methods are used instead.
    """
    rust_dir = env.Dir("#modules/rustcore")
    crate = "godot_core_rs"

    if sys.platform == "win32":
        lib_name = crate + ".lib"
        cargo = "cd /d \"{dir}\" && cargo build --release --quiet"
    else:
        lib_name = "lib" + crate + ".a"
        cargo = "cd \"{dir}\" && cargo build --release --quiet"

    lib_file = rust_dir.File(os.path.join("target", "release", lib_name))

    sources = (
        env.Glob("#modules/rustcore/src/*.rs")
        + env.Glob("#modules/rustcore/src/*/*.rs")
        + [env.File("#modules/rustcore/Cargo.toml")]
    )

    env.Command(lib_file, sources, cargo.format(dir=rust_dir.abspath))
    env.Prepend(LIBS=[lib_file])

    if sys.platform == "win32":
        # Rust's std needs these system libraries. They go into LINKFLAGS, not
        # LIBS: Godot's build decorates LIBS entries with its own platform
        # suffix (`ws2_32.windows.editor.x86_64.lib`), which link.exe cannot find.
        env.Append(
            LINKFLAGS=[
                "ws2_32.lib",
                "advapi32.lib",
                "userenv.lib",
                "ntdll.lib",
                "bcrypt.lib",
                "ole32.lib",
                "shell32.lib",
                "kernel32.lib",
            ]
        )