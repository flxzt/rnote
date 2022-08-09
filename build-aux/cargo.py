#!/usr/bin/env python3

import sys
import os
from subprocess import call

print("--- entering build script ---")

cwd = os.getcwd()
meson_build_root = sys.argv[1]
meson_source_root = sys.argv[2]
output = sys.argv[3]
profile = sys.argv[4]
app_bin = sys.argv[5]
cargo_target_dir = os.path.join(meson_build_root, "target")
cargo_home = os.path.join(meson_build_root, "cargo-home")

os.environ["CARGO_TARGET_DIR"] = cargo_target_dir
os.environ["CARGO_HOME"] = cargo_home

if profile == "devel":
    print("\n --- DEVEL PROFILE ---\n")
    call(["cargo", "build", "--manifest-path", os.path.join(meson_source_root, "Cargo.toml")])
    call(["cp", os.path.join(cargo_target_dir, "debug", app_bin), os.path.join(cwd, output)])
else:
    print("\n --- RELEASE PROFILE ---\n")
    call(["cargo", "build", "--manifest-path", os.path.join(meson_source_root, "Cargo.toml"), "--release"])
    call(["cp", os.path.join(cargo_target_dir, "release", app_bin), os.path.join(cwd, output)])
