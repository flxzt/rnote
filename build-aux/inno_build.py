#!/usr/bin/env python3

import sys
import os

inno_script = sys.argv[1]
inno_installer_output = sys.argv[2]
source_root = sys.argv[3]
build_root = sys.argv[4]

print(f"""
### executing Inno-Setup installer build script with arguments: ###
    inno_script: {inno_script}
    inno_installer_output: {inno_installer_output}
    source_root: {source_root}
    build_root: {build_root}
""", file=sys.stderr)

# TODO: collect dlls, prepare files, ..

# TODO: invoke inno-setup
