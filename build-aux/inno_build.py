#!/usr/bin/env python3

import sys
import os
import re
import shutil
import glob
import itertools
import subprocess
from subprocess import CalledProcessError

source_root = sys.argv[1]
build_root = sys.argv[2]
# The build environment (from a msys installation - choose from https://www.msys2.org/docs/environments/)
build_environment_path = sys.argv[3]
app_name = sys.argv[4]
app_name_capitalized = sys.argv[5]
app_id = sys.argv[6]
ui_output = sys.argv[7]
inno_script = sys.argv[8]
cli_name = sys.argv[9]

print(f"""
### executing Inno-Setup installer build script with arguments: ###
    source_root: {source_root}
    build_root: {build_root}
    build_environment_path: {build_environment_path}
    app_name: {app_name}
    app_name_capitalized: {app_name_capitalized}
    app_id: {app_id}
    ui_output: {ui_output}
    inno_script: {inno_script}
    cli_name: {cli_name} (empty if not packaged)
""", file=sys.stderr)

def run_command(command, error_message):
    try:
        subprocess.run(command, shell=True, check=True)
    except CalledProcessError as e:
        print(f"{error_message}: {e}", file=sys.stderr)
        print(f"command: {command}", file=sys.stderr)
        sys.exit(1)


env_bin_dir = f"{build_environment_path}/bin"


def collect_dependencies(binary, out_dir):
    """Copy every DLL from the build environment that `binary` needs, transitively.

    This reads the PE import table with objdump rather than using `ldd`. `ldd`
    resolves dependencies by actually loading the binary, which deadlocks on
    libraries that do work at load time - notably ANGLE's libGLESv2.dll, where
    the installer build would hang indefinitely.

    DLLs that do not exist in the build environment are Windows system DLLs and
    are deliberately not packaged.
    """
    seen = set()
    queue = [binary]

    while queue:
        current = queue.pop()
        result = subprocess.run(
            ["objdump", "-p", current], capture_output=True, text=True, errors="replace"
        )
        if result.returncode != 0:
            print(f"Reading imports of {current} failed", file=sys.stderr)
            sys.exit(1)

        for name in re.findall(r"DLL Name:\s*(\S+)", result.stdout):
            key = name.lower()
            if key in seen:
                continue
            seen.add(key)

            candidate = os.path.join(env_bin_dir, name)
            if os.path.exists(candidate):
                shutil.copy(candidate, out_dir)
                queue.append(candidate)


# Collect DLLs
print("Collecting DLLs...", file=sys.stderr)
dlls_dir = os.path.join(build_root, "dlls")

if os.path.exists(dlls_dir):
    shutil.rmtree(dlls_dir)

os.mkdir(dlls_dir)

collect_dependencies(f"{build_root}/{ui_output}", dlls_dir)

for loader in glob.glob(f"{build_environment_path}/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll"):
    collect_dependencies(loader, dlls_dir)

# ANGLE is loaded at runtime, so it is not in the import table of anything above
# and has to be packaged explicitly, together with what it depends on.
for angle_dll in itertools.chain(
    glob.glob(f"{env_bin_dir}/libEGL*.dll"),
    glob.glob(f"{env_bin_dir}/libGLES*.dll"),
):
    shutil.copy(angle_dll, dlls_dir)
    collect_dependencies(angle_dll, dlls_dir)

# add the openssl runtime. The file name carries the architecture
# (libcrypto-3-x64.dll on x86_64, libcrypto-3-arm64.dll on aarch64), so glob it.
for openssl_pattern in ("libcrypto-3-*.dll", "libssl-3-*.dll"):
    matches = glob.glob(f"{build_environment_path}/bin/{openssl_pattern}")
    if not matches:
        print(f"Could not find any openssl dll matching {openssl_pattern}", file=sys.stderr)
        sys.exit(1)
    for openssl_dll in matches:
        run_command(
            f"cp {openssl_dll} {dlls_dir}",
            f"Collecting openssl ({openssl_dll}) failed",
        )

# Collect necessary GSchema Xml's and compile them into a `gschemas.compiled`
print("Collecting and compiling GSchemas...", file=sys.stderr)
gschemas_dir = os.path.join(build_root, "gschemas")

if os.path.exists(gschemas_dir):
    shutil.rmtree(gschemas_dir)

os.mkdir(gschemas_dir)

for src in glob.glob(f"{build_environment_path}/share/glib-2.0/schemas/org.gtk.*"):
    shutil.copy(src, gschemas_dir)

shutil.copy(f"{build_root}/crates/rnote-ui/data/{app_id}.gschema.xml", gschemas_dir)

# generate `gschemas.compiled` in the same directory
run_command(
    f"glib-compile-schemas {gschemas_dir}",
    "Compiling schemas failed"
)

# Collect locale
print("Collecting locale...", file=sys.stderr)
locale_dir = os.path.join(build_root, "locale")

if os.path.exists(locale_dir):
    shutil.rmtree(locale_dir)

# app locale
app_mo_dir = os.path.join(build_root, 'crates/rnote-ui/po')
shutil.copytree(app_mo_dir, locale_dir)

# system locale
for file in os.listdir(app_mo_dir):
    current_lang = os.fsdecode(file)
    current_locale_out_dir = os.path.join(locale_dir, current_lang, "LC_MESSAGES")
    current_system_locale_dir = os.path.join(build_environment_path, "share/locale", current_lang, "LC_MESSAGES")

    if not os.path.exists(current_locale_out_dir):
        os.mkdir(current_locale_out_dir)

    glib_locale = os.path.join(current_system_locale_dir, "glib20.mo")
    if os.path.exists(glib_locale):
        shutil.copy(glib_locale, current_locale_out_dir)

    gtk4_locale = os.path.join(current_system_locale_dir, "gtk40.mo")
    if os.path.exists(gtk4_locale):
        shutil.copy(gtk4_locale, current_locale_out_dir)

    adw_locale = os.path.join(current_system_locale_dir, "libadwaita.mo")
    if os.path.exists(adw_locale):
        shutil.copy(adw_locale, current_locale_out_dir)

    # TODO: do we need any other system locales?

# Build installer
print("Running ISCC...", file=sys.stderr)

# the inno script will package the cli if the variable MyAppCliExeName
# is defined. This is done with an additional /DMyAppCliExeName=cli_name.exe
# argument 
define_cli_output = ""
if cli_name:
    define_cli_output = "/DMyAppCliExeName=" + \
        cli_name + ".exe"
run_command(
    f"iscc {inno_script} {define_cli_output}",
    "Running ISCC failed"
)

print("### Inno-Setup installer build script finished ###", file=sys.stderr)
