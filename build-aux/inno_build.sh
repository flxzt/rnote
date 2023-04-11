#!/bin/bash

inno_script="$1"
inno_installer_output="$2"
source_root="$3"
build_root="$4"

echo "
### executing Inno-Setup installer build script with arguments: ###
  inno_script: $1
  inno_installer_output: $2
  source_root: $3
  build_root: $4
"

# TODO: collect dlls, prepare files, ..

# TODO: invoke inno-setup
