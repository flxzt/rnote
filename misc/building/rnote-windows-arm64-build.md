# Build Instructions for Windows arm64

## Prerequisites
### Clone the repository

```bash
git clone https://github.com/flxzt/rnote
git submodule update --init --recursive
```

### Install MSYS2
- Install MSYS2 (using CLANGARM64 environment)
- Update MSYS2:
```Bash
pacman -Suy
```
- Add MSYS2 binary directories `C:\msys64\clangarm64\bin` and `C:\msys64\usr\bin` to the system environment variable Path.

### (Optional) Install Inno Setup (for building the installer)

### Install Dependencies
Run the following command in the MSYS2 CLANGARM64 terminal to install necessary dependencies:

```bash
pacman -S mingw-w64-clang-aarch64-xz mingw-w64-clang-aarch64-pkgconf mingw-w64-clang-aarch64-clang \
mingw-w64-clang-aarch64-toolchain mingw-w64-clang-aarch64-autotools mingw-w64-clang-aarch64-make mingw-w64-clang-aarch64-cmake \
mingw-w64-clang-aarch64-meson mingw-w64-clang-aarch64-desktop-file-utils mingw-w64-clang-aarch64-appstream \
mingw-w64-clang-aarch64-gtk4 mingw-w64-clang-aarch64-libadwaita mingw-w64-clang-aarch64-poppler mingw-w64-clang-aarch64-poppler-data \
mingw-w64-clang-aarch64-angleproject
```
All the above packages are clang-aarch64 versions corresponding to x86_64 versions.
The mingw-w64-x86_64-diffutils package doesn't have a corresponding clang-aarch64 version, so install mingw-w64-x86_64-diffutils directly:

```bash
pacman -S mingw-w64-x86_64-diffutils
```

### Install Rust
Install Rust provided by MSYS2 (important)
```bash
pacman -S mingw-w64-clang-aarch64-rust
```

### Configuration

Add the following line to the end of the `C:\msys64\home\ZhouZhiwu\.bashrc` file to add the Rust binary directory to MSYS2's `PATH`:

```
export PATH=$PATH:/c/msys64/clangarm64/bin
```
If you have installed Inno Setup, add the following line to the end of the `C:\msys64\home\ZhouZhiwu\.bashrc` file:

```
export PATH=$PATH:/c/msys64/clangarm64/bin:/c/Program\ Files\ \(x86\)/Inno\ Setup\ 6
```
Apply the configuration:

```bash
source ~/.bashrc
```
### Rename dll
Rename `"C:\msys64\clangarm64\lib\libpthread.dll.a"` to `"C:\msys64\clangarm64\lib\libpthread.dll.a.bak"`

### Other Preparations before compilation
1. Modify line 25 of the cargo_call statement in `C:\rnote-1\build-aux\cargo_build.py` to:

   ```
   cargo_call = f"env {cargo_env} RUSTFLAGS='-C linker=clang' {cargo_cmd} build {cargo_options}"
   ```
2. Replace the contents of `"C:\rnote\crates\rnote-ui\po\zh_CN.po"` and `"C:\rnote\crates\rnote-ui\po\zh_SG.po"` with the contents of `"C:\rnote\crates\rnote-ui\po\zh_Hans.po"`, but change `"Language: zh_Hans\n"` to `"Language: zh_CN\n"` and `"Language: zh_SG\n"` respectively; replace the contents of `"C:\rnote\crates\rnote-ui\po\zh_TW.po"` and `"C:\rnote\crates\rnote-ui\po\zh_HK.po"` with the contents of `"C:\rnote\crates\rnote-ui\po\zh_Hant.po"`, but change `"Language: zh_Hant\n"` to `"Language: zh_TW\n"` and `"Language: zh_HK\n"` respectively.
# Compile Rnote

1. First, make sure you're in the correct directory:
   ```bash
   cd /c/rnote
   ```
   
2. Clean previous build files:
   ```bash
   rm -rf _mesonbuild
   ```
   
3. Reconfigure the project:
   ```bash
   meson setup --prefix=C:/msys64/clangarm64 _mesonbuild
   ```
    ```bash
    meson setup --prefix=C:/msys64/clangarm64 -Dwin-installer-name='windows_arm64_installer' _mesonbuild
    ```

4. Compile the project:
   ```bash
   meson compile -C _mesonbuild
   ```
   
5. Install the project:
   ```bash
   meson install -C _mesonbuild
   ```
# Build the installer

```bash
meson compile rnote-gmo -C _mesonbuild

meson compile build-installer -C _mesonbuild
```