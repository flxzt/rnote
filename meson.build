project(
    'rnote',
    ['rust', 'cpp'],
    version: '0.10.2',
    meson_version: '>= 1.0',
)
# add a patch suffix for alpha or beta versions in format '-<alpha|beta>.<x>'.
patch = ''

# Modules
i18n = import('i18n')
gnome = import('gnome')

# Options
prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
libdir = prefix / get_option('libdir')
localedir = prefix / get_option('localedir')
sysconfdir = prefix / get_option('sysconfdir')
profile = get_option('profile')
build_ui = get_option('ui')
build_cli = get_option('cli')
win_build_environment_path = get_option('win-build-environment-path')
win_installer_name = get_option('win-installer-name')

app_name = 'rnote'
app_name_capitalized = 'Rnote'
cli_name = 'rnote-cli'
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

pkgdatadir = datadir / app_name
iconsdir = datadir / 'icons'
gettext_package = app_name
app_idpath = '/com/github/flxzt/' + app_name + '/'

if profile == 'devel'
    app_id = base_id + '.' + 'Devel'
    version_suffix = '-devel'
    rust_target_folder = 'debug'
else
    app_id = base_id
    version_suffix = patch
    rust_target_folder = 'release'
endif

# Dependencies
cpp_compiler = meson.get_compiler('cpp')
# Rust bindings to C++ using autocxx need libclang
# However when building as flatpak libclang is not found, even though the llvm extension is installed.
# TODO: How to fix this?
#cpp_compiler.find_library('clang')
dependency('glib-2.0', version: '>= 2.76')
dependency('gio-2.0', version: '>= 2.76')
dependency('cairo', version: '>= 1.18')

cargo = find_program('cargo', required: true)
# cmake is needed by the "ink-stroke-modeler-rs" bindings crate to build the C++ library
find_program('cmake', required: true)
find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)
find_program('fc-cache', required: false)
find_program('update-mime-database', required: false)
cargo_build_script = find_program('build-aux/cargo_build.py', required: true)
inno_build_script = find_program('build-aux/inno_build.py', required: true)

cargo_env = ['CARGO_HOME=' + meson.project_build_root() / 'cargo-home']
cargo_manifest_path = meson.project_source_root() / 'Cargo.toml'
cargo_target_dir = meson.project_build_root() / 'target'

meson.add_dist_script(
    'build-aux/dist-vendor.sh',
    meson.project_build_root() / 'meson-dist' / app_name + '-' + version,
    meson.project_source_root(),
)

if profile == 'devel'
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

# core crates
subdir('crates/rnote-compose')
subdir('crates/rnote-engine')


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
            '--workspace',
            '--manifest-path',
            cargo_manifest_path,
            '--target-dir',
            cargo_target_dir,
        ],
    ],
    env: cargo_env,
)

# cargo clean
run_target(
    'cargo-clean',
    command: [
        cargo,
        [
            'clean',
            '--manifest-path',
            cargo_manifest_path,
            '--target-dir',
            cargo_target_dir,
        ],
    ],
    env: cargo_env,
)

# build/install the ui
if build_ui == true
    # ui dependencies
    dependency('appstream')
    dependency('poppler-glib', version: '>= 22.02')
    dependency('gtk4', version: '>= 4.12')
    dependency('libadwaita-1', version: '>= 1.4')

    # ui crate
    subdir('crates/rnote-ui')

    ui_sources = [
        rnote_compose_sources,
        rnote_engine_sources,
        rnote_ui_sources,
        rnote_ui_gresources_files,
        files(
            meson.project_source_root() + '/crates/rnote-compose/Cargo.toml',
            meson.project_source_root() + '/crates/rnote-engine/Cargo.toml',
            meson.project_source_root() + '/crates/rnote-ui/Cargo.toml',
            meson.project_source_root() + '/crates/rnote-ui/build.rs',
            meson.project_source_root() + '/Cargo.lock',
            meson.project_source_root() + '/Cargo.toml',
        ),
    ]

    ui_cargo_options = ['--manifest-path', cargo_manifest_path]
    ui_cargo_options += ['--target-dir', cargo_target_dir]
    ui_cargo_options += ['-p', 'rnote']

    if profile == 'default'
        ui_cargo_options += ['--release']
    endif

    if host_machine.system() == 'windows'
        ui_output = app_name + '.exe'
    else
        ui_output = app_name
    endif

    # ui cargo check
    run_target(
        'ui-cargo-check',
        command: [
            cargo,
            'check',
            ui_cargo_options,
        ],
        env: cargo_env,
    )

    # ui cargo clippy
    run_target(
        'ui-cargo-clippy',
        command: [
            cargo,
            'clippy',
            ui_cargo_options,
        ],
        env: cargo_env,
    )

    # ui cargo doc
    run_target(
        'ui-cargo-doc',
        command: [
            cargo,
            'doc',
            ui_cargo_options,
        ],
        env: cargo_env,
    )

    # ui cargo build
    app_cargo_build = custom_target(
        'ui-cargo-build',
        build_by_default: true,
        input: ui_sources,
        output: ui_output,
        console: true,
        install: true,
        install_dir: bindir,
        command: [
            cargo_build_script,
            meson.project_build_root(),
            meson.project_source_root(),
            cargo_env,
            cargo,
            ' '.join(ui_cargo_options),
            cargo_target_dir / rust_target_folder / app_name,
            meson.project_build_root() / '@OUTPUT@',
        ],
    )

    meson.add_install_script(
        'build-aux/meson_post_install.py',
        datadir,
        bindir,
        app_name,
    )

    gnome.post_install(
        glib_compile_schemas: true,
        gtk_update_icon_cache: true,
        update_desktop_database: true,
        update_mime_database: true,
    )

    # Windows installer
    message('Configuring Inno-Setup installer script file')

    installer_output = win_installer_name + '.exe'

    inno_script_conf = configuration_data()
    inno_script_conf.set_quoted('MESON_SOURCE_ROOT', meson.project_source_root())
    inno_script_conf.set_quoted('MESON_BUILD_ROOT', meson.project_build_root())
    inno_script_conf.set_quoted('BUILD_ENVIRONMENT_PATH', win_build_environment_path)
    inno_script_conf.set_quoted('APP_NAME', app_name)
    inno_script_conf.set_quoted('APP_NAME_CAPITALIZED', app_name_capitalized)
    inno_script_conf.set_quoted('APP_ID', app_id)
    inno_script_conf.set_quoted('APP_VERSION', version)
    inno_script_conf.set_quoted('APP_VERSION_SUFFIX', version_suffix)
    inno_script_conf.set_quoted('APP_AUTHOR_NAME', app_author_name)
    inno_script_conf.set_quoted('APP_AUTHORS', app_authors)
    inno_script_conf.set_quoted('APP_WEBSITE', app_website)
    inno_script_conf.set_quoted('APP_ISSUES_URL', app_issues_url)
    inno_script_conf.set_quoted('APP_SUPPORT_URL', app_support_url)
    inno_script_conf.set_quoted('APP_DONATE_URL', app_donate_url)
    inno_script_conf.set_quoted('UI_OUTPUT', ui_output)
    inno_script_conf.set_quoted('INSTALLER_NAME', win_installer_name)

    inno_script = configure_file(
        input: 'build-aux/rnote_inno.iss.in',
        output: 'rnote_inno.iss',
        configuration: inno_script_conf,
    )

    if host_machine.system() == 'windows'
        # build installer
        custom_target(
            'build-installer',
            input: inno_script,
            output: installer_output,
            command: [
                inno_build_script,
                meson.project_source_root(),
                meson.project_build_root(),
                win_build_environment_path,
                app_name,
                app_name_capitalized,
                app_id,
                ui_output,
                meson.project_build_root() / '@INPUT@',
            ],
            build_by_default: false,
            depends: app_cargo_build,
        )
    endif
endif

# build/install the cli
if build_cli == true
    # cli crate
    subdir('crates/rnote-cli')

    cli_cargo_options = ['--manifest-path', cargo_manifest_path]
    cli_cargo_options += ['--target-dir', cargo_target_dir]
    cli_cargo_options += ['-p', 'rnote-cli']

    if profile == 'default'
        cli_cargo_options += ['--release']
    endif

    if host_machine.system() == 'windows'
        cli_output = cli_name + '.exe'
    else
        cli_output = cli_name
    endif

    cli_sources = [
        rnote_compose_sources,
        rnote_engine_sources,
        rnote_cli_sources,
        files(
            meson.project_source_root() + '/crates/rnote-compose/Cargo.toml',
            meson.project_source_root() + '/crates/rnote-engine/Cargo.toml',
            meson.project_source_root() + '/crates/rnote-cli/Cargo.toml',
            meson.project_source_root() + '/Cargo.lock',
            meson.project_source_root() + '/Cargo.toml',
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

    # cli cargo doc
    run_target(
        'cli-cargo-doc',
        command: [
            cargo,
            'doc',
            cli_cargo_options,
        ],
        env: cargo_env,
    )

    # cli cargo build
    custom_target(
        'cli-cargo-build',
        build_by_default: true,
        input: cli_sources,
        output: cli_output,
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
