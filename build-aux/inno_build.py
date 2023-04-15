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

dll_directory = os.path.join(build_root, "dlls/")

if not os.path.exists(dll_directory):
    print("Creating DLL directory...", file=sys.stderr)
    os.mkdir(dll_directory)

print("Collecting DLLs...", file=sys.stderr)
os.system(f"ldd {os.path.join(build_root, 'rnote.exe')} | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dll_directory}")
os.system(f"ldd {msys_path}/mingw64/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dll_directory}")

print("Running ISCC...", file=sys.stderr)
# TODO: maybe use chocolatey to install in workflow? it's not added to PATH by default, hardcoding is a bad idea.
os.system(f"{mingw_path}/../usr/bin/bash -c \"'C:\Program Files (x86)\Inno Setup 6\ISCC.exe' '{inno_script}'\"")