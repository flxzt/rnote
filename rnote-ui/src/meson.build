
global_conf = configuration_data()
global_conf.set_quoted('APP_ID', app_id)
global_conf.set_quoted('APP_IDPATH', app_idpath)
global_conf.set_quoted('APP_NAME', app_name)
global_conf.set_quoted('APP_NAME_CAPITALIZED', app_name_capitalized)
global_conf.set_quoted('APP_WEBSITE', app_website)
global_conf.set_quoted('APP_ISSUES_URL', app_issues_url)
global_conf.set_quoted('APP_SUPPORT_URL', app_support_url)
global_conf.set_quoted('APP_AUTHOR_NAME', app_author_name)
global_conf.set_quoted('APP_AUTHORS', app_authors)
global_conf.set_quoted('PKG_DATA_DIR', pkgdatadir)
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
  meson.project_build_root() / 'rnote-ui' / 'src' / 'config.rs',
  meson.project_source_root() / 'rnote-ui' / 'src' / 'config.rs',
  check: true
)

rnote_ui_sources = files(
    'main.rs',
    'config.rs',
    'globals.rs',
    'utils.rs',
    'app/mod.rs',
    'app/appactions.rs',
    'appwindow/mod.rs',
    'appwindow/appsettings.rs',
    'appwindow/appwindowactions.rs',
    'appmenu.rs',
    'canvas/mod.rs',
    'canvas/canvaslayout.rs',
    'canvas/input.rs',
    'canvasmenu.rs',
    'settingspanel/mod.rs',
    'settingspanel/penshortcutrow.rs',
    'settingspanel/penshortcutmodels.rs',
    'mainheader.rs',
    'dialogs.rs',
    'unitentry.rs',
    'iconpicker.rs',
    'penssidebar/mod.rs',
    'penssidebar/brushpage.rs',
    'penssidebar/shaperpage.rs',
    'penssidebar/eraserpage.rs',
    'penssidebar/selectorpage.rs',
    'penssidebar/typewriterpage.rs',
    'penssidebar/toolspage.rs',
    'workspacebrowser/mod.rs',
    'workspacebrowser/filerow.rs',
    'workspacebrowser/workspacerow.rs',
    'workspacebrowser/workspacelist.rs',
    'workspacebrowser/workspacelistentry.rs',
    'colorpicker/mod.rs',
    'colorpicker/colorsetter.rs',
)