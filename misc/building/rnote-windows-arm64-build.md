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

## Why the installer no longer uses `ldd`

`inno_build.py` used to collect the DLLs to package by running `ldd` on the app,
the gdk-pixbuf loaders and the ANGLE libraries. MSYS2's `ldd` resolves imports by
*loading* the binary, and ANGLE's `libGLESv2.dll` blocks while doing so: the
installer build hangs there indefinitely, with no error and no CPU usage, which
looks like a very slow build rather than a deadlock.

Dependencies are now read from the PE import table with `objdump -p` and walked
transitively, which executes nothing. DLLs that do not exist in the build
environment are Windows system DLLs and are deliberately not packaged. This also
made the collection step noticeably faster.

Whether the old code hangs on x86_64 depends only on how that ANGLE build behaves
at load time, so this is not an ARM-specific fix.

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

## Why WGL is disabled on aarch64

Getting `GskGLRenderer` is necessary but not sufficient: *which* OpenGL
implementation serves it matters just as much.

Windows on ARM ships no native desktop OpenGL driver. Going through WGL therefore
lands in Microsoft's `OpenGLOn12.dll` (the `Microsoft.D3DMappingLayers` package,
installed as the "OpenCL and OpenGL Compatibility Pack"), which translates OpenGL
onto Direct3D 12. GTK reports this as:

```
 - Renderer: D3D12 (Qualcomm(R) Adreno(TM) X1-85 GPU)
Using OpenGL backend Windows WGL
```

That layer crashes. After a few minutes of ordinary drawing and changing settings,
the process dies with an access violation, and the Windows event log names the
faulting module:

```
Faulting application name: rnote.exe
Faulting module name: OpenGLOn12.dll
Exception code: 0xc0000005
```

`crates/rnote-ui/src/env.rs` therefore defaults `GDK_DISABLE=wgl` on
`target_arch = "aarch64"`, which makes GDK pick EGL and with it the ANGLE that is
already shipped with the app. ANGLE maps GL ES onto Direct3D 11:

```
 - Vendor: Google Inc. (Qualcomm)
 - Version: 1.5 (ANGLE 2.1.25748)
Using renderer 'GskGLRenderer'
```

GPU rendering is fully preserved, and the crashes are gone (verified over ten
minutes of normal use where the WGL path died after about two). It is set only as
a default, so `GDK_DISABLE` from the environment still takes precedence, and it is
scoped to aarch64 because x86_64 has real OpenGL drivers where ANGLE would be a
detour rather than a fix.
