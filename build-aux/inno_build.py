#!/usr/bin/env python3

import sys
import os
import shutil
import glob
import itertools

source_root = sys.argv[1]
build_root = sys.argv[2]
# The build environment (from a msys installation - choose from https://www.msys2.org/docs/environments/)
build_environment_path = sys.argv[3]
app_name = sys.argv[4]
app_name_capitalized = sys.argv[5]
app_id = sys.argv[6]
ui_output = sys.argv[7]
inno_script = sys.argv[8]

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
""", file=sys.stderr)

def run_command(command, error_message):
    res = os.system(command)
    if res != 0:
        print(f"{error_message}, code: {res}", file=sys.stderr)
        print(f"command: {command}", file=sys.stderr)
        sys.exit(1)


# Collect DLLs
print("Collecting DLLs...", file=sys.stderr)
dlls_dir = os.path.join(build_root, "dlls")

if os.path.exists(dlls_dir):
    shutil.rmtree(dlls_dir)

os.mkdir(dlls_dir)

# Don't use os.path.join here, because that uses the wrong separators which breaks wildcard expansion.
run_command(
    f"ldd {build_root}/{ui_output} | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dlls_dir}",
    "Collecting app DLLs failed"
)

for loader in glob.glob(f"{build_environment_path}/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll"):
    run_command(
        f"ldd {loader} | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dlls_dir}",
        f"Collecting pixbuf-loader ({loader}) DLLs failed"
    )

for angle_dll in itertools.chain(
    glob.glob(f"{build_environment_path}/bin/libEGL*.dll"),
    glob.glob(f"{build_environment_path}/bin/libGLES*.dll"),
):
    run_command(
        f"cp {angle_dll} {dlls_dir}",
        f"Collecting angle ({angle_dll}) DLLs failed",
    )
    run_command(
        f"ldd {angle_dll} | grep '\\/mingw.*\.dll' -o | xargs -i cp {{}} {dlls_dir}",
        f"Collecting angle dependency ({angle_dll}) DLLs failed",
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

run_command(
    f"iscc {inno_script}",
    "Running ISCC failed"
)

print("### Inno-Setup installer build script finished ###", file=sys.stderr)
