#!/usr/bin/env python3

import sys
import os

project_build_root = sys.argv[1]
project_src_root = sys.argv[2]
cargo_env = sys.argv[3]
cargo_cmd = sys.argv[4]
cargo_options = sys.argv[5]
bin_output = sys.argv[6]
output_file = sys.argv[7]

print(f"""
### executing cargo build script with arguments: ###
    project_build_root: {project_build_root}
    project_src_root: {project_src_root}
    cargo_env: {cargo_env}
    cargo_cmd: {cargo_cmd}
    cargo_options: {cargo_options}
    bin_output: {bin_output}
    output_file: {output_file}
""", file=sys.stderr)

cargo_call = f"env {cargo_env} {cargo_cmd} build {cargo_options}"
cp_call = f"cp {bin_output} {output_file}"

print(cargo_call, file=sys.stderr)
res = os.system(cargo_call)
if res != 0:
    print(f"cargo call failed, code {res}", file=sys.stderr)
    sys.exit(1)

print(cp_call, file=sys.stderr)
res = os.system(cp_call)
if res != 0:
    print(f"cp call failed, code {res}", file=sys.stderr)
    sys.exit(1)

print("### cargo build script finished ###", file=sys.stderr)
