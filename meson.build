project(
    'rnote',
    ['cpp', 'rust'],
    version: '0.5.9',
    meson_version: '>= 0.56',
)
# add a patch suffix for alpha or beta version, starting with a dash.
patch = ''

i18n = import('i18n')
gnome = import('gnome')
fs = import('fs')

app_name = 'rnote'
app_name_capitalized = 'Rnote'
app_cli_name = 'rnote-cli'
base_id = 'com.github.flxzt.' + app_name
app_website = 'https://rnote.flxzt.net'
app_issues_url = 'https://github.com/flxzt/rnote/issues'
app_support_url = 'https://github.com/flxzt/rnote/discussions'
app_donate_url = 'https://rnote.flxzt.net/donate/'
app_author_name = 'Felix Zwettler'
app_authors = run_command(
    'cat',
    meson.project_source_root() / 'AUTHORS',
    check: true,
).stdout().strip()

version = meson.project_version()
version_array = version.split('.')
version_major = version_array[0].to_int()
version_minor = version_array[1].to_int()
version_micro = version_array[2].to_int()

if get_option('profile') == 'devel'
    profile = 'devel'
    version_suffix = '-devel'
    app_id = base_id + '.' + 'Devel'
else
    profile = 'default'
    app_id = base_id
    version_suffix = patch
endif

app_idpath = '/com/github/flxzt/' + app_name + '/'

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
libdir = prefix / get_option('libdir')
localedir = prefix / get_option('localedir')
pkgdatadir = datadir / app_name
iconsdir = datadir / 'icons'
gettext_package = app_name

user_home_dir = fs.expanduser('~')

# Dependencies

cpp = meson.get_compiler('cpp')
#add_project_arguments(['-stdlib=stdlibc++'], language : 'cpp')
#add_project_link_arguments(['-stdlib=stdlibc++'], language : 'cpp')

# Rust bindings to C++ using autocxx need libclang
# However when building as flatpak libclang is not found, even though the llvm extension is installed. How to fix this?
#cpp.find_library('clang')

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.7')
dependency('libadwaita-1', version: '>= 1.2')
dependency('poppler-glib', version: '>= 22.07')

cargo = find_program('cargo', required: true)
# cmake is needed by the "ink-stroke-modeler-rs" bindings crate to build the C++ library
cmake = find_program('cmake', required: true)
glib_compile_resources = find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)
update_mime_database = find_program('update-mime-database', required: false)
# easier to build with a script on windows
cargo_build_win_script = find_program('build-aux/cargo_build_win.py')

meson.add_dist_script(
    'build-aux/dist-vendor.sh',
    meson.project_build_root() / 'meson-dist' / app_name + '-' + version,
    meson.project_source_root(),
)

if get_option('profile') == 'devel'
    # Setup pre-commit hook for ensuring coding style is always consistent
    message('Setting up git pre-commit hook..')
    run_command(
        'cp',
        '-f',
        'hooks/pre-commit.hook',
        '.git/hooks/pre-commit',
        check: false,
    )
endif

# workspaces crates
subdir('rnote-compose')
subdir('rnote-fileformats')
subdir('rnote-engine')
subdir('rnote-cli')
subdir('rnote-ui')

cargo_sources = files(
    meson.project_source_root() + '/' + 'Cargo.toml',
    meson.project_source_root() + '/' + 'rnote-compose' + '/' + 'Cargo.toml',
    meson.project_source_root() + '/' + 'rnote-fileformats' + '/' + 'Cargo.toml',
    meson.project_source_root() + '/' + 'rnote-engine' + '/' + 'Cargo.toml',
    meson.project_source_root() + '/' + 'rnote-ui' + '/' + 'Cargo.toml',
    meson.project_source_root() + '/' + 'rnote-ui' + '/' + 'build.rs',
)

sources = [
    rnote_compose_sources,
    rnote_fileformats_sources,
    rnote_engine_sources,
    rnote_ui_sources,
    cargo_sources,
]

cargo_manifest_path = meson.project_source_root() / 'Cargo.toml'
cargo_target_dir = meson.project_build_root() / 'target'
cargo_options = ['--manifest-path', cargo_manifest_path]
cargo_options += ['--target-dir', cargo_target_dir]
cargo_options += ['-p', 'rnote']

if get_option('cli') == true
    cargo_options += ['-p', 'rnote-cli']

    sources += rnote_cli_sources
    sources += files([
        meson.project_source_root() + '/' + 'rnote-cli' + '/' + 'Cargo.toml',
    ])
endif

if get_option('profile') == 'default'
    cargo_options += ['--release']
    rust_target = 'release'
    message('cargo: building in release mode')
else
    rust_target = 'debug'
    message('cargo: building in debug mode')
endif

cargo_env = ['CARGO_HOME=' + meson.project_build_root() / 'cargo-home']

# see https://mesonbuild.com/Reference-tables.html#operating-system-names
if host_machine.system() == 'windows' or host_machine.system() == 'cygwin'
    bin_output = app_name + '.exe'
else
    bin_output = app_name
endif

if host_machine.system() == 'windows' or host_machine.system() == 'cygwin'
    # the script needs one string
    win_cargo_options = (
        '--manifest-path ' + cargo_manifest_path + ' --target-dir ' + cargo_target_dir
    )

    if get_option('profile') == 'default'
        win_cargo_options += ' --release'
    endif

    custom_target(
        'cargo-build-win',
        build_by_default: true,
        input: sources,
        output: bin_output,
        console: true,
        install: true,
        install_dir: bindir,
        depends: gresources,
        command: [
            cargo_build_win_script,
            meson.project_build_root(),
            meson.project_source_root(),
            cargo_env,
            cargo,
            win_cargo_options,
            cargo_target_dir / rust_target / app_name,
            meson.project_build_root() / '@OUTPUT@',
        ],
    )
else
    custom_target(
        'cargo-build',
        build_by_default: true,
        input: sources,
        output: bin_output,
        console: true,
        install: true,
        install_dir: bindir,
        depends: gresources,
        command: [
            'env',
            cargo_env,
            cargo,
            'build',
            cargo_options,
            '&&',
            'cp',
            cargo_target_dir / rust_target / app_name,
            meson.project_build_root() / '@OUTPUT@',
        ],
    )
endif

# TODO: Install the cli

# meson >= 0.64 will include `update-mime-database` in the gnome module.
# Then this script can be replaced by `gnome.post_install(..)`
meson.add_install_script(
    'build-aux/meson_post_install.py',
    datadir,
    bindir,
    app_name,
)
