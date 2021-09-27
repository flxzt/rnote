
global_conf = configuration_data()
global_conf.set_quoted('APP_ID', app_id)
global_conf.set_quoted('APP_IDPATH', app_idpath)
global_conf.set_quoted('APP_NAME', app_name)
global_conf.set_quoted('RESOURCES_FILE', pkgdatadir / 'resources.gresource')
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

rust_sources = files(
    'main.rs',
    'config.rs',
    'globals.rs',
    'app.rs',
    'utils.rs',
    'sheet/mod.rs',
    'sheet/format.rs',
    'sheet/selection.rs',
    'pens/mod.rs',
    'pens/marker.rs',
    'pens/brush.rs',
    'pens/eraser.rs',
    'pens/selector.rs',
    'strokes/mod.rs',
    'strokes/compose.rs',
    'strokes/render.rs',
    'strokes/markerstroke.rs',
    'strokes/brushstroke.rs',
    'strokes/vectorimage.rs',
    'strokes/bitmapimage.rs',
    'ui/mod.rs',
    'ui/appwindow.rs',
    'ui/actions.rs',
    'ui/appmenu.rs',
    'ui/canvas.rs',
    'ui/canvasmenu.rs',
    'ui/mainheader.rs',
    'ui/penssidebar.rs',
    'ui/dialogs.rs',
    'ui/templatechooser.rs',
    'ui/workspacebrowser.rs',
    'ui/selectionmodifier/mod.rs',
    'ui/selectionmodifier/modifiernode.rs',
    'ui/colorpicker/mod.rs',
    'ui/colorpicker/colorsetter.rs',
)

sources = [rust_sources, rough_rs_rust_sources, cargo_sources]

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