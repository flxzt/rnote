project(
    'rnote',
    'rust',
    version: '0.4.0',
    meson_version: '>= 0.56',
)
i18n = import('i18n')
gnome = import('gnome')
fs = import('fs')

app_name = 'rnote'
app_name_capitalized = 'Rnote'
base_id = 'com.github.flxzt.' + app_name
app_website = 'https://github.com/flxzt/rnote'
app_authors = 'Felix Zwettler'

if get_option('profile') == 'devel'
  profile = 'devel'
  version_suffix = '-devel'
  app_id = base_id + '.' + 'Devel'
else
  profile = 'default'
  version_suffix = ''
  app_id = base_id
endif

app_idpath = '/com/github/flxzt/' + app_name + '/'

version = meson.project_version()
version_array = version.split('.')
version_major = version_array[0].to_int()
version_minor = version_array[1].to_int()
version_micro = version_array[2].to_int()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')
datadir = prefix / get_option('datadir')
pkgdatadir = datadir / app_name
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
gettext_package = app_name

user_home_dir = fs.expanduser('~')

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.6')
dependency('libadwaita-1', version: '>= 1.0')
dependency('poppler-glib', version: '>= 21.08')
dependency('gstreamer-1.0', version: '>= 1.18')

cargo = find_program('cargo', required: true)
cargo_script = find_program('build-aux/cargo.sh')
glib_compile_resources = find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)
update_mime_database = find_program('update-mime-database', required: false)

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_build_root() / 'meson-dist' / app_name + '-' + version,
  meson.project_source_root()
)

# local crates
cargo_sources= files(
  meson.project_source_root() + '/' + 'Cargo.toml',
  meson.project_source_root() + '/' + 'rnote-engine' + '/' + 'Cargo.toml',
  meson.project_source_root() + '/' + 'rnote-ui' + '/' + 'Cargo.toml',
  meson.project_source_root() + '/' + 'rnote-fileformats' + '/' + 'Cargo.toml',
)

subdir('rnote-engine')
subdir('rnote-ui')
subdir('rnote-fileformats')

sources = [rnote_fileformats_sources, rnote_ui_sources, cargo_sources]

custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: app_name,
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.project_build_root(),
    meson.project_source_root(),
    '@OUTPUT@',
    profile,
    app_name,
  ]
)

meson.add_install_script('build-aux/meson_post_install.py')
