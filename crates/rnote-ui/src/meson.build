# config.rs
message('Configuring config.rs file')

global_conf = configuration_data()
global_conf.set_quoted('APP_NAME', app_name)
global_conf.set_quoted('APP_NAME_CAPITALIZED', app_name_capitalized)
global_conf.set_quoted('APP_ID', app_id)
global_conf.set_quoted('APP_IDPATH', app_idpath)
global_conf.set_quoted('APP_VERSION', version)
global_conf.set_quoted('APP_VERSION_SUFFIX', version_suffix)
global_conf.set_quoted('APP_AUTHOR_NAME', app_author_name)
global_conf.set_quoted('APP_AUTHORS', app_authors)
global_conf.set_quoted('APP_WEBSITE', app_website)
global_conf.set_quoted('APP_ISSUES_URL', app_issues_url)
global_conf.set_quoted('APP_SUPPORT_URL', app_support_url)
global_conf.set_quoted('APP_DONATE_URL', app_donate_url)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('DATADIR', datadir)
global_conf.set_quoted('LIBDIR', libdir)
global_conf.set_quoted('LOCALEDIR', localedir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: global_conf,
)

# Copy the config.rs output to the source directory
run_command(
    'cp',
    meson.project_build_root() / 'crates' / 'rnote-ui' / 'src' / 'config.rs',
    meson.project_source_root() / 'crates' / 'rnote-ui' / 'src' / 'config.rs',
    check: true,
)

rnote_ui_sources = files(
    'app/appactions.rs',
    'app/mod.rs',
    'appwindow/actions.rs',
    'appwindow/appsettings.rs',
    'appwindow/imp.rs',
    'appwindow/mod.rs',
    'canvas/canvaslayout.rs',
    'canvas/imexport.rs',
    'canvas/input.rs',
    'canvas/mod.rs',
    'canvas/widgetflagsboxed.rs',
    'colorpicker/colorsetter.rs',
    'colorpicker/colorpad.rs',
    'colorpicker/mod.rs',
    'dialogs/export.rs',
    'dialogs/import.rs',
    'dialogs/mod.rs',
    'groupediconpicker/group.rs',
    'groupediconpicker/mod.rs',
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
    'strokewidthpicker/mod.rs',
    'strokewidthpicker/previewstyle.rs',
    'strokewidthpicker/strokewidthpreview.rs',
    'strokewidthpicker/strokewidthsetter.rs',
    'workspacebrowser/filerow/actions/duplicate.rs',
    'workspacebrowser/filerow/actions/mod.rs',
    'workspacebrowser/filerow/actions/open.rs',
    'workspacebrowser/filerow/actions/rename.rs',
    'workspacebrowser/filerow/actions/trash.rs',
    'workspacebrowser/filerow/mod.rs',
    'workspacebrowser/workspaceactions/createfolder.rs',
    'workspacebrowser/workspaceactions/mod.rs',
    'workspacebrowser/workspacesbar/mod.rs',
    'workspacebrowser/workspacesbar/workspacelist.rs',
    'workspacebrowser/workspacesbar/workspacelistentry.rs',
    'workspacebrowser/workspacesbar/workspacerow.rs',
    'workspacebrowser/mod.rs',
    'workspacebrowser/widgethelper.rs',
    'appmenu.rs',
    'canvasmenu.rs',
    'canvaswrapper.rs',
    'config.rs',
    'contextmenu.rs',
    'env.rs',
    'filetype.rs',
    'globals.rs',
    'iconpicker.rs',
    'main.rs',
    'mainheader.rs',
    'overlays.rs',
    'penpicker.rs',
    'sidebar.rs',
    'strokecontentpaintable.rs',
    'strokecontentpreview.rs',
    'unitentry.rs',
    'utils.rs',
)
