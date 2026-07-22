# Build Instructions for Windows on ARM (aarch64)

Rnote builds and runs natively on Windows on ARM using the MSYS2 **CLANGARM64**
environment. This was developed and verified on a Surface Pro 11
(Snapdragon X Elite `X1E80100`, Adreno X1-85 GPU, Windows 11 26H1).

The build does not require any source changes: the same recipes serve MINGW64 and
CLANGARM64, with the environment supplying the architecture-dependent parts.

## Prerequisites

Install [MSYS2](https://www.msys2.org/) and open a **CLANGARM64** shell
(not MINGW64, not MSYS).

Rust must use the `gnullvm` toolchain, which is the one that matches CLANGARM64.
It is Tier 2 with host tools since Rust 1.91, so no cross-compilation is involved:

```bash
rustup toolchain install stable-aarch64-pc-windows-gnullvm
rustup default stable-aarch64-pc-windows-gnullvm
```

Then, from the CLANGARM64 shell:

```bash
cargo install --locked just
just prerequisites-win
```

`prerequisites-win` reads `$MINGW_PACKAGE_PREFIX`, so it installs the
`mingw-w64-clang-aarch64-*` packages here and the `mingw-w64-x86_64-*` packages
under MINGW64.

Two host requirements that are easy to miss:

- **Enable Windows Developer Mode** before cloning. Without it, `core.symlinks`
  stays off and `crates/rnote-ui/po/zh_{CN,SG,HK,TW}.po` are checked out as 10-byte
  text files instead of symlinks; `msgfmt` then fails with
  `keyword "zh_Hant" unknown`. This applies to x86_64 just as much.
- **Use a path without spaces**, e.g. `C:\dev\rnote`. `build-aux/cargo_build.py`
  assembles a shell string for `os.system()` without quoting, so a path such as
  `C:\Users\me\OneDrive - university\…` breaks the build with
  `env: '-': No such file or directory`. Also architecture-independent.

## Building

```bash
just setup-win-dev          # devel build, installs into $MSYSTEM_PREFIX
just build
meson install -C _mesonbuild
```

`meson install` fails on its last step, `update-desktop-database` (exit 126). That
step is a no-op on Windows and everything relevant is installed before it runs.

Run the **installed** binary at `$MSYSTEM_PREFIX/bin/rnote.exe`, not the copy in
`_mesonbuild`: `crates/rnote-ui/src/env.rs` derives `XDG_DATA_DIRS` from
`<exec_dir>/../share`, so only the installed one finds its GSettings schema.
(`just run-ui` has this problem on Windows in general.)

For the installer:

```bash
just setup-win-installer rnote-win-installer-arm64
just build
just build-win-installer
```

## GPU acceleration requires pinned packages

**This is the part that matters most on ARM.** Without the pinning below the app
runs correctly but entirely on the CPU.

GTK's Win32 backend moved its rendering to DirectComposition. From that version on,
GSK cannot realize either GL or Vulkan on a `GdkWin32Toplevel` unless a DComp
device exists, and falls back to `GskCairoRenderer`, i.e. software rendering. On
GTK 4.22.4 on the Adreno X1-85 the log shows:

```
Failed to realize renderer 'GskGLRenderer'     for surface 'GdkWin32Toplevel': OpenGL requires Direct Composition
Failed to realize renderer 'GskVulkanRenderer' for surface 'GdkWin32Toplevel': Vulkan requires Direct Composition
Using renderer 'GskCairoRenderer' for surface 'GdkWin32Toplevel'
```

Note this is not a driver problem: a WGL 4.6 context is created successfully on the
Adreno ("Renderer: D3D12 (Qualcomm(R) Adreno(TM) X1-85 GPU)"). Only the surface
attachment fails.

`prerequisites-win` therefore pins, for the same reason and from the same era as
the existing MINGW64 pinning:

| Package | Version | Why |
| --- | --- | --- |
| `gtk4` | `4.18.6-3` | before the DirectComposition switch |
| `libadwaita` | `1.7.7-1` | matches that GTK |
| `gettext-{runtime,libtextstyle,tools}` | `0.26-1` | those builds import `DllMain` from `libintl-8.dll`, which newer gettext no longer exports |

The gettext pin is not optional. Without it the app dies during loading with
`STATUS_ENTRYPOINT_NOT_FOUND` (`0xC0000139`) and prints nothing at all, which is
easy to misdiagnose. The recipe also adds these packages to `IgnorePkg` in
`/etc/pacman.conf`, so a later `pacman -Syu` does not silently undo the pin — and
with it the GPU acceleration.

Unlike MINGW64, CLANGARM64 needs **no** `libpthread.dll.a` workaround; linking with
`lld` works as is.

## Verifying that the GPU is actually used

```bash
GSK_DEBUG=renderer $MSYSTEM_PREFIX/bin/rnote.exe
```

- `Using renderer 'GskGLRenderer'` — GPU, this is the expected result.
- `Using renderer 'GskCairoRenderer'` — software; the pin was lost or bypassed.

When measuring this, make sure no other instance is running first. Rnote is a
single-instance `GtkApplication`, so a second launch hands off to the existing
process and exits immediately without output, which silently produces misleading
results.

Vulkan remains unavailable on this configuration even with the pinned GTK
(`Not using Vulkan: platform is not Wayland`); GL is the working path.
