
global_conf = configuration_data()
global_conf.set_quoted('APP_ID', app_id)
global_conf.set_quoted('APP_IDPATH', app_idpath)
global_conf.set_quoted('APP_NAME', app_name)
global_conf.set_quoted('RESOURCES_FILE', pkgdatadir / 'resources.gresource')
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version)
global_conf.set_quoted('VERSION_SUFFIX', version_suffix)
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
    'input.rs',
    'app.rs',
    'utils.rs',
    'compose/mod.rs',
    'compose/geometry.rs',
    'compose/shapes.rs',
    'compose/curves.rs',
    'compose/solid.rs',
    'compose/textured.rs',
    'render.rs',
    'drawbehaviour.rs',
    'audioplayer.rs',
    'sheet/mod.rs',
    'sheet/format.rs',
    'sheet/background.rs',
    'pens/mod.rs',
    'pens/penbehaviour.rs',
    'pens/marker.rs',
    'pens/brush.rs',
    'pens/shaper.rs',
    'pens/eraser.rs',
    'pens/selector.rs',
    'pens/tools.rs',
    'strokes/mod.rs',
    'strokes/strokestyle.rs',
    'strokes/strokebehaviour.rs',
    'strokes/markerstroke.rs',
    'strokes/brushstroke.rs',
    'strokes/shapestroke.rs',
    'strokes/vectorimage.rs',
    'strokes/bitmapimage.rs',
    'strokesstate/mod.rs',
    'strokesstate/chrono_comp.rs',
    'strokesstate/render_comp.rs',
    'strokesstate/selection_comp.rs',
    'strokesstate/trash_comp.rs',
    'ui/mod.rs',
    'ui/appwindow.rs',
    'ui/appsettings.rs',
    'ui/actions.rs',
    'ui/develactions.rs',
    'ui/appmenu.rs',
    'ui/canvas/mod.rs',
    'ui/canvas/canvaslayout.rs',
    'ui/canvasmenu.rs',
    'ui/settingspanel.rs',
    'ui/mainheader.rs',
    'ui/dialogs.rs',
    'ui/unitentry.rs',
    'ui/penssidebar/mod.rs',
    'ui/penssidebar/markerpage.rs',
    'ui/penssidebar/brushpage.rs',
    'ui/penssidebar/shaperpage.rs',
    'ui/penssidebar/eraserpage.rs',
    'ui/penssidebar/selectorpage.rs',
    'ui/penssidebar/toolspage.rs',
    'ui/workspacebrowser/mod.rs',
    'ui/workspacebrowser/filerow.rs',
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