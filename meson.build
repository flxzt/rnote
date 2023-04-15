project(
    'rnote',
    ['rust', 'cpp'],
    version: '0.5.18',
    meson_version: '>= 0.57',
)
# add a patch suffix for alpha or beta version, starting with a dash.
patch = ''

i18n = import('i18n')
gnome = import('gnome')
fs = import('fs')

app_name = 'rnote'
app_name_capitalized = 'Rnote'
# the name of the cli crate
cli_name = 'rnote-cli'
# We need to rename the cli binary, because else the output file name would collide with the rnote-cli folder
cli_output_name = 'rnote_cli'
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
    rust_target_folder = 'debug'
else
    profile = 'default'
    app_id = base_id
    version_suffix = patch
    rust_target_folder = 'release'
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
cargo_build_script = find_program('build-aux/cargo_build.py')
inno_build_script = find_program('build-aux/inno_build.py', required: true)

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

# workspace crates
subdir('rnote-compose')
subdir('rnote-fileformats')
subdir('rnote-engine')
subdir('rnote-ui')

cargo_manifest_path = meson.project_source_root() / 'Cargo.toml'
cargo_target_dir = meson.project_build_root() / 'target'
cargo_env = ['CARGO_HOME=' + meson.project_build_root() / 'cargo-home']

# cargo fmt check
run_target(
    'cargo-fmt-check',
    command: [
        cargo,
        [
            'fmt',
            '--all',
            '--check',
            '--manifest-path',
            cargo_manifest_path,
        ],
    ],
    env: cargo_env,
)

# cargo test
run_target(
    'cargo-test',
    command: [
        cargo,
        [
            'test',
            '--manifest-path',
            cargo_manifest_path,
        ],
    ],
    env: cargo_env,
)

# build / install the app
app_sources = [
    rnote_compose_sources,
    rnote_fileformats_sources,
    rnote_engine_sources,
    rnote_ui_sources,
    files(
        meson.project_source_root() + '/Cargo.toml',
        meson.project_source_root() + '/Cargo.lock',
        meson.project_source_root() + '/rnote-compose/Cargo.toml',
        meson.project_source_root() + '/rnote-fileformats/Cargo.toml',
        meson.project_source_root() + '/rnote-engine/Cargo.toml',
        meson.project_source_root() + '/rnote-ui/Cargo.toml',
        meson.project_source_root() + '/rnote-ui/build.rs',
    ),
]

app_cargo_options = ['--manifest-path', cargo_manifest_path]
app_cargo_options += ['--target-dir', cargo_target_dir]
app_cargo_options += ['-p', 'rnote']

if get_option('profile') == 'default'
    app_cargo_options += ['--release']
endif

# see https://mesonbuild.com/Reference-tables.html#operating-system-names
if host_machine.system() == 'windows' or host_machine.system() == 'cygwin'
    app_bin_output = app_name + '.exe'
else
    app_bin_output = app_name
endif

# app cargo check
run_target(
    'app-cargo-check',
    command: [
        cargo,
        'check',
        app_cargo_options,
    ],
    env: cargo_env,
)

# app cargo clippy
run_target(
    'app-cargo-clippy',
    command: [
        cargo,
        'clippy',
        app_cargo_options,
    ],
    env: cargo_env,
)

# app cargo build
app_cargo_build = custom_target(
    'app-cargo-build',
    build_by_default: true,
    input: app_sources,
    output: app_bin_output,
    console: true,
    install: true,
    install_dir: bindir,
    depends: gresources,
    command: [
        cargo_build_script,
        meson.project_build_root(),
        meson.project_source_root(),
        cargo_env,
        cargo,
        ' '.join(app_cargo_options),
        cargo_target_dir / rust_target_folder / app_name,
        meson.project_build_root() / '@OUTPUT@',
    ],
)

# build / install the cli
if get_option('cli') == true
    # cli crate
    subdir('rnote-cli')

    cli_cargo_options = ['--manifest-path', cargo_manifest_path]
    cli_cargo_options += ['--target-dir', cargo_target_dir]
    cli_cargo_options += ['-p', 'rnote-cli']

    if get_option('profile') == 'default'
        cli_cargo_options += ['--release']
    endif

    if host_machine.system() == 'windows' or host_machine.system() == 'cygwin'
        cli_bin_output = cli_output_name + '.exe'
    else
        cli_bin_output = cli_output_name
    endif

    cli_sources = [
        rnote_compose_sources,
        rnote_fileformats_sources,
        rnote_engine_sources,
        rnote_cli_sources,
        files(
            meson.project_source_root() + '/Cargo.toml',
            meson.project_source_root() + '/Cargo.lock',
            meson.project_source_root() + '/rnote-compose/Cargo.toml',
            meson.project_source_root() + '/rnote-fileformats/Cargo.toml',
            meson.project_source_root() + '/rnote-engine/Cargo.toml',
            meson.project_source_root() + '/rnote-cli/Cargo.toml',
        ),
    ]

    # cli cargo check
    run_target(
        'cli-cargo-check',
        command: [
            cargo,
            'check',
            cli_cargo_options,
        ],
        env: cargo_env,
    )

    # cli cargo clippy
    run_target(
        'cli-cargo-clippy',
        command: [
            cargo,
            'clippy',
            cli_cargo_options,
        ],
        env: cargo_env,
    )

    # cli cargo build
    custom_target(
        'cli-cargo-build',
        build_by_default: true,
        input: cli_sources,
        output: cli_bin_output,
        console: true,
        install: true,
        install_dir: bindir,
        command: [
            cargo_build_script,
            meson.project_build_root(),
            meson.project_source_root(),
            cargo_env,
            cargo,
            ' '.join(cli_cargo_options),
            cargo_target_dir / rust_target_folder / cli_name,
            meson.project_build_root() / '@OUTPUT@',
        ],
    )
endif

meson.add_install_script(
    'build-aux/meson_post_install.py',
    datadir,
    bindir,
    app_name,
)

# meson >= 0.64 will include `update-mime-database` in the gnome module.
# As soon as that is available on supported platforms ( fedora 37, msys2, flatpak gnome platform )
# the post-install script can be replaced by:
#gnome.post_install(
#    glib_compile_schemas: true,
#    gtk_update_icon_cache: true,
#    update_desktop_database: true,
#    update_mime_database: true,
#)

# windows installer

inno_installer_name = 'rnote_installer-@0@'.format(host_machine.cpu_family())
inno_installer_output = inno_installer_name + '.exe'

message('Configuring Inno-Setup installer script file')

inno_script_conf = configuration_data()
inno_script_conf.set_quoted('APP_ID', app_id)
inno_script_conf.set_quoted('APP_NAME', app_name)
inno_script_conf.set_quoted('APP_NAME_CAPITALIZED', app_name_capitalized)
inno_script_conf.set_quoted('APP_VERSION', version)
inno_script_conf.set_quoted('APP_VERSION_SUFFIX', version_suffix)
inno_script_conf.set_quoted('APP_AUTHOR_NAME', app_author_name)
inno_script_conf.set_quoted('APP_AUTHORS', app_authors)
inno_script_conf.set_quoted('APP_WEBSITE', app_website)
inno_script_conf.set_quoted('APP_ISSUES_URL', app_issues_url)
inno_script_conf.set_quoted('APP_SUPPORT_URL', app_support_url)
inno_script_conf.set_quoted('APP_DONATE_URL', app_donate_url)
inno_script_conf.set_quoted('APP_BIN_OUTPUT', app_bin_output)
inno_script_conf.set_quoted('MESON_SOURCE_ROOT', meson.project_source_root())
inno_script_conf.set_quoted('MESON_BUILD_ROOT', meson.project_build_root())
inno_script_conf.set_quoted('MESON_INSTALL_PREFIX', prefix)
inno_script_conf.set_quoted('INSTALLER_NAME', inno_installer_name)
# TODO: make this configurable
inno_script_conf.set_quoted('MINGW_PATH', prefix)

inno_script = configure_file(
    input: 'build-aux/rnote_inno.iss.in',
    output: 'rnote_inno.iss',
    configuration: inno_script_conf,
)

if host_machine.system() == 'windows' or host_machine.system() == 'cygwin'
    # TODO: make the msys/mingw64 installation an explicit dependency

    custom_target('build-inno-installer',
        input : inno_script,
        output : inno_installer_output,
        command : [
            inno_build_script,
            meson.project_source_root() / '@INPUT@',
            meson.project_build_root() / '@OUTPUT@',
            meson.project_source_root(),
            meson.project_build_root(),
        ],
        build_by_default: false,
        depends: app_cargo_build,
    )
endif