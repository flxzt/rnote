# Resources
resource_conf = configuration_data()
resource_conf.set('APP_IDPATH', app_idpath)
configure_file(
  input: 'resources.gresource.xml.in',
  output: 'resources.gresource.xml',
  configuration: resource_conf,
)
resources = gnome.compile_resources(
  'resources',
  meson.current_build_dir() + '/' + 'resources.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
)

# # D-Bus service file
# dbus_conf = configuration_data()
# dbus_conf.set('APP_ID', app_id)
# dbus_conf.set('APP_NAME', app_name)
# configure_file(
#   input: 'app.service.in',
#   output: app_id + '.service',
#   configuration: dbus_conf,
#   install: true,
#   install_dir: datadir / 'dbus-1' / 'services',
# )

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('APP_ID', app_id)
desktop_conf.set('APP_NAME', app_name)
desktop_file = configure_file(
  input: 'app.desktop.in',
  output: app_id + '.desktop',
  configuration: desktop_conf,
  install: true,
  install_dir: datadir / 'applications',
)

# Validate Desktop file
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [
      desktop_file
    ],
  )
endif

# Gschema
gschema_conf = configuration_data()
gschema_conf.set('APP_ID', app_id)
gschema_conf.set('APP_IDPATH', app_idpath)
gschema_conf.set('APP_NAME', app_name)
gschema_file = configure_file(
  input: 'app.gschema.xml.in',
  output: app_id + '.gschema.xml',
  configuration: gschema_conf,
  install: true,
  install_dir: datadir / 'glib-2.0' / 'schemas',
)

# Validata GSchema
if glib_compile_schemas.found()
  test(
    'validate-gschema', glib_compile_schemas,
    args: [
      '--strict', '--dry-run', datadir / 'glib-2.0' / 'schemas',
    ],
  )
endif

# Appdata
appdata_conf = configuration_data()
appdata_conf.set('APP_ID', app_id)
appdata_conf.set('APP_NAME', app_name)
appdata_file = configure_file(
  input: 'app.metainfo.xml.in',
  output: app_id + '.metainfo.xml',
  configuration: appdata_conf,
  install: true,
  install_dir: datadir / 'metainfo',
)

# Validate Appdata
if appstream_util.found()
  test(
    'validate-appdata', appstream_util,
    args: [
      'validate', '--nonet', appdata_file
    ],
  )
endif

# App-Icon
if profile == 'devel'
  install_data(
    'icons/scalable/apps/' + app_name + '-devel.svg',
    install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps',
    rename: app_id + '.svg',
  )
else
  install_data(
    'icons/scalable/apps/' + app_name + '.svg',
    install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps',
    rename: app_id + '.svg',
  )
endif

# MIME-Type
install_data(
  'app.mimetype.xml.in',
  install_dir: datadir / 'mime' / 'packages',
  rename: app_id + '.xml',
)

# templates
template_files = files(
  'templates/brushstroke-cubicbezier.svg.templ',
  'templates/brushstroke-linear.svg.templ',
)

install_data(
  template_files,
  install_dir: pkgdatadir / 'templates'
)

# sounds
template_files = files(
  'sounds/marker.wav',
  'sounds/brush.wav',
)

install_data(
  template_files,
  install_dir: pkgdatadir / 'sounds'
)