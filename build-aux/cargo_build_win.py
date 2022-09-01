#!/usr/bin/python

import sys
import os

project_build_root = sys.argv[1]
project_src_root = sys.argv[2]
cargo_env = sys.argv[3]
cargo_cmd = sys.argv[4]
cargo_options = sys.argv[5]
cargo_generated_bin = sys.argv[6]
output_file = sys.argv[7]

cargo_call = f"env {cargo_env} {cargo_cmd} build {cargo_options}"
cp_call = f"cp {cargo_generated_bin} {output_file}"

print(cargo_call)
os.system(cargo_call)
print(cp_call)
os.system(cp_call)