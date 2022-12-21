# config.rs
message('Configuring config.rs file')

global_conf = configuration_data()
global_conf.set_quoted('APP_ID', app_id)
global_conf.set_quoted('APP_IDPATH', app_idpath)
global_conf.set_quoted('APP_NAME', app_name)
global_conf.set_quoted('APP_NAME_CAPITALIZED', app_name_capitalized)
global_conf.set_quoted('APP_WEBSITE', app_website)
global_conf.set_quoted('APP_ISSUES_URL', app_issues_url)
global_conf.set_quoted('APP_SUPPORT_URL', app_support_url)
global_conf.set_quoted('APP_DONATE_URL', app_donate_url)
global_conf.set_quoted('APP_AUTHOR_NAME', app_author_name)
global_conf.set_quoted('APP_AUTHORS', app_authors)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version)
global_conf.set_quoted('VERSION_SUFFIX', version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)

# Use relative paths for windows installations to make them portable
if host_machine.system() == 'windows' or host_machine.system() == 'cygwin'
  win_datadir = '../' / get_option('datadir')
  win_pkgdatadir = '../' / get_option('datadir') / app_name
  win_libdir = '../' / get_option('libdir')
  win_localedir = '../' / get_option('localedir')

  global_conf.set_quoted('DATADIR', win_datadir)
  global_conf.set_quoted('PKGDATADIR', win_pkgdatadir)
  global_conf.set_quoted('LIBDIR', win_libdir)
  global_conf.set_quoted('RESOURCES_FILE', win_pkgdatadir / 'resources.gresource')
  global_conf.set_quoted('LOCALEDIR', win_localedir )
else
  global_conf.set_quoted('DATADIR', datadir)
  global_conf.set_quoted('PKGDATADIR', pkgdatadir)
  global_conf.set_quoted('LIBDIR', libdir)
  global_conf.set_quoted('RESOURCES_FILE', pkgdatadir / 'resources.gresource')
  global_conf.set_quoted('LOCALEDIR', localedir)
endif

config = configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: global_conf,
)
# Copy the config.rs output to the source directory.
run_command(
    'cp',
    meson.project_build_root() / 'rnote-ui' / 'src' / 'config.rs',
    meson.project_source_root() / 'rnote-ui' / 'src' / 'config.rs',
    check: true,
)

rnote_ui_sources = files(
    'app/appactions.rs',
    'app/mod.rs',
    'appwindow/appsettings.rs',
    'appwindow/appwindowactions.rs',
    'appwindow/imexport.rs',
    'appwindow/mod.rs',
    'canvas/canvaslayout.rs',
    'canvas/input.rs',
    'canvas/mod.rs',
    'colorpicker/colorsetter.rs',
    'colorpicker/mod.rs',
    'penssidebar/brushpage.rs',
    'penssidebar/eraserpage.rs',
    'penssidebar/mod.rs',
    'penssidebar/selectorpage.rs',
    'penssidebar/shaperpage.rs',
    'penssidebar/toolspage.rs',
    'penssidebar/typewriterpage.rs',
    'settingspanel/mod.rs',
    'settingspanel/penshortcutmodels.rs',
    'settingspanel/penshortcutrow.rs',
    'workspacebrowser/filerow/actions/duplicate.rs',
    'workspacebrowser/filerow/actions/mod.rs',
    'workspacebrowser/filerow/actions/open.rs',
    'workspacebrowser/filerow/actions/rename.rs',
    'workspacebrowser/filerow/actions/trash.rs',
    'workspacebrowser/filerow/mod.rs',
    'workspacebrowser/mod.rs',
    'workspacebrowser/widget_helper/entry_dialog.rs',
    'workspacebrowser/widget_helper/mod.rs',
    'workspacebrowser/workspaceactions/create_folder.rs',
    'workspacebrowser/workspaceactions/mod.rs',
    'workspacebrowser/workspacesbar/mod.rs',
    'workspacebrowser/workspacesbar/workspacelist.rs',
    'workspacebrowser/workspacesbar/workspacelistentry.rs',
    'workspacebrowser/workspacesbar/workspacerow.rs',
    'dialogs/mod.rs',
    'dialogs/import.rs',
    'dialogs/export.rs',
    'appmenu.rs',
    'canvasmenu.rs',
    'canvaswrapper.rs',
    'config.rs',
    'globals.rs',
    'iconpicker.rs',
    'main.rs',
    'mainheader.rs',
    'unitentry.rs',
    'utils.rs',
)
