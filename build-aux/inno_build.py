#!/usr/bin/env python3

import sys
import os
from subprocess import call

inno_script = sys.argv[1]
installer_output = sys.argv[2]
source_root = sys.argv[3]
build_root = sys.argv[4]
msys_path = sys.argv[5]

print(f"""
### executing Inno-Setup installer build script with arguments: ###
    inno_script: {inno_script}
    installer_output: {installer_output}
    source_root: {source_root}
    build_root: {build_root}
    msys_path: {msys_path}
""", file=sys.stderr)

# TODO: add mingw path

# collect dlls, prepare files, ..
if not os.path.exists(f"{build_root}/dlls/"):
    os.mkdir(f"{build_root}/dlls/")

os.system(f"ldd {build_root}/rnote.exe | grep '\/mingw.*\.dll' -o | xargs -i cp {{}} {build_root}/dlls/")
os.system(f"ldd /mingw64/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll | grep '\/mingw.*\.dll' -o | xargs -i cp {{}} {build_root}/dlls/")

# invoke inno-setup
os.system(f"pwsh -c \"iscc /O'{inno_installer_output}' '{inno_script}'\"")