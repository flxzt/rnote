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

Add the following line to the end of the `C:\msys64\home\<user>\.bashrc` file to add the Rust binary directory to MSYS2's `PATH`:

```
export PATH=$PATH:/c/msys64/clangarm64/bin
```
If you have installed Inno Setup, add the following line to the end of the `C:\msys64\home\<user>\.bashrc` file:

```
export PATH=$PATH:/c/msys64/clangarm64/bin:/c/Program\ Files\ \(x86\)/Inno\ Setup\ 6
```
Apply the configuration:

```bash
source ~/.bashrc
```

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
    meson setup --prefix="C:/msys64/clangarm64" -Dwin-installer-name='windows_arm64_installer' -Dwin-build-environment-path='C:\\msys64\\clangarm64' _mesonbuild
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
```

```bash
meson compile build-installer -C _mesonbuild
```
