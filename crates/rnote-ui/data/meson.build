# Desktop file
message('Configuring desktop file')

desktop_file_name_entry = app_name_capitalized
if profile == 'devel'
    desktop_file_name_entry += ' (Development)'
endif

desktop_conf = configuration_data()
desktop_conf.set('APP_ID', app_id)
desktop_conf.set('APP_NAME', app_name)
desktop_conf.set('DESKTOP_FILE_NAME_ENTRY', desktop_file_name_entry)
desktop_file_in = configure_file(
    input: 'app.desktop.in.in',
    output: '@BASENAME@',
    configuration: desktop_conf,
)

desktop_file = i18n.merge_file(
    input: desktop_file_in,
    output: app_id + '.desktop',
    type: 'desktop',
    po_dir: '../po',
    install: true,
    install_dir: datadir / 'applications',
)

# Validate Desktop file
if desktop_file_validate.found()
    test(
        'validate-desktop',
        desktop_file_validate,
        args: [desktop_file],
    )
endif

# GSchema
message('Configuring settings GSchema file')

gschema_conf = configuration_data()
gschema_conf.set('APP_ID', app_id)
gschema_conf.set('APP_IDPATH', app_idpath)
gschema_conf.set('APP_NAME', app_name)
configure_file(
    input: 'app.gschema.xml.in',
    output: app_id + '.gschema.xml',
    configuration: gschema_conf,
    install: true,
    install_dir: datadir / 'glib-2.0' / 'schemas',
)

# Validate installed GSchema
if glib_compile_schemas.found()
    test(
        'validate-gschema',
        glib_compile_schemas,
        args: [
            '--dry-run',
            datadir / 'glib-2.0' / 'schemas',
        ],
    )
endif

# Appdata
if host_machine.system() != 'windows'
    message('Configuring appdata file')

    appdata_conf = configuration_data()
    appdata_conf.set('APP_ID', app_id)
    appdata_conf.set('APP_NAME', app_name)
    appdata_conf.set('APP_NAME_CAPITALIZED', app_name_capitalized)
    appdata_conf.set('APP_AUTHOR_NAME', app_author_name)
    appdata_conf.set('APP_WEBSITE', app_website)
    appdata_conf.set('APP_ISSUES_URL', app_issues_url)
    appdata_conf.set('APP_SUPPORT_URL', app_support_url)
    appdata_conf.set('APP_DONATE_URL', app_donate_url)
    appdata_file_in = configure_file(
        input: 'app.metainfo.xml.in.in',
        output: '@BASENAME@',
        configuration: appdata_conf,
    )

    # Currently this would fail on windows mingw with error:
    #
    # "C:/msys64/mingw64/bin/msgfmt.EXE: cannot locate ITS rules for crates/rnote-ui/data/app.metainfo.xml.in"
    #
    # Even though the appdata package is installed and the rule "metainfo.its" is present in
    # "/mingw64/share/gettext/its". But because we don't need the appdata file, we can circumvent this error by just not
    # processing the metainfo file, we don't need it anyway.
    appdata_file = i18n.merge_file(
        input: appdata_file_in,
        output: app_id + '.metainfo.xml',
        po_dir: '../po',
        install: true,
        install_dir: datadir / 'metainfo',
    )

    # Validate Appdata
    if appstream_util.found()
        test(
            'validate-appdata',
            appstream_util,
            args: ['validate', '--nonet', appdata_file],
        )
    endif
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

# Symbolic App-Icon
install_data(
    'icons/symbolic/apps/' + app_name + '-symbolic.svg',
    install_dir: iconsdir / 'hicolor' / 'symbolic' / 'apps',
    rename: app_id + '-symbolic.svg',
)

# MIME-Type icon
# Unfortunately this is currently not displayed by nautilus (and possibly other file managers?) when the
# Adwaita theme is used. Instead the "application-x-generic" icon gets picked.
# See: https://gitlab.gnome.org/GNOME/nautilus/-/issues/2190
install_data(
    'icons/scalable/mimetypes/' + 'application-' + app_name + '.svg',
    install_dir: iconsdir / 'hicolor' / 'scalable' / 'mimetypes',
)

# MIME-Type
message('Configuring MIME-type file')

mimetype_conf = configuration_data()
mimetype_conf.set('APP_ID', app_id)
mimetype_conf.set('APP_NAME', app_name)
mimetype_conf.set('APP_NAME_CAPITALIZED', app_name_capitalized)
configure_file(
    input: 'app.mimetype.xml.in',
    output: app_id + '.xml',
    configuration: mimetype_conf,
    install: true,
    install_dir: datadir / 'mime' / 'packages',
)

# # D-Bus service file
# message('Configuring D-Bus service file')
#
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

rnote_ui_gresources_icons_files = files(
    'icons/scalable/actions/add-page-symbolic.svg',
    'icons/scalable/actions/appwindow-fullscreen-symbolic.svg',
    'icons/scalable/actions/canvasmenu-symbolic.svg',
    'icons/scalable/actions/clipboard-copy-symbolic.svg',
    'icons/scalable/actions/clipboard-paste-symbolic.svg',
    'icons/scalable/actions/cursor-beam-large.svg',
    'icons/scalable/actions/cursor-beam-medium.svg',
    'icons/scalable/actions/cursor-beam-small.svg',
    'icons/scalable/actions/cursor-crosshair-large.svg',
    'icons/scalable/actions/cursor-crosshair-medium.svg',
    'icons/scalable/actions/cursor-crosshair-small.svg',
    'icons/scalable/actions/cursor-dot-large.svg',
    'icons/scalable/actions/cursor-dot-medium.svg',
    'icons/scalable/actions/cursor-dot-small.svg',
    'icons/scalable/actions/cursor-invisible.svg',
    'icons/scalable/actions/cursor-teardrop-n-large.svg',
    'icons/scalable/actions/cursor-teardrop-n-medium.svg',
    'icons/scalable/actions/cursor-teardrop-n-small.svg',
    'icons/scalable/actions/cursor-teardrop-ne-large.svg',
    'icons/scalable/actions/cursor-teardrop-ne-medium.svg',
    'icons/scalable/actions/cursor-teardrop-ne-small.svg',
    'icons/scalable/actions/cursor-teardrop-nw-large.svg',
    'icons/scalable/actions/cursor-teardrop-nw-medium.svg',
    'icons/scalable/actions/cursor-teardrop-nw-small.svg',
    'icons/scalable/actions/dir-actions-symbolic.svg',
    'icons/scalable/actions/dir-down-symbolic.svg',
    'icons/scalable/actions/dir-left-symbolic.svg',
    'icons/scalable/actions/dir-right-symbolic.svg',
    'icons/scalable/actions/dir-up-symbolic.svg',
    'icons/scalable/actions/doc-create-symbolic.svg',
    'icons/scalable/actions/doc-save-symbolic.svg',
    'icons/scalable/actions/dock-left-symbolic.svg',
    'icons/scalable/actions/dock-right-symbolic.svg',
    'icons/scalable/actions/dot-symbolic.svg',
    'icons/scalable/actions/drawing-pad-button-1-symbolic.svg',
    'icons/scalable/actions/drawing-pad-button-2-symbolic.svg',
    'icons/scalable/actions/drawing-pad-button-3-symbolic.svg',
    'icons/scalable/actions/drawing-pad-button-4-symbolic.svg',
    'icons/scalable/actions/edit-redo-symbolic.svg',
    'icons/scalable/actions/edit-symbolic.svg',
    'icons/scalable/actions/edit-undo-symbolic.svg',
    'icons/scalable/actions/emojichooser-symbolic.svg',
    'icons/scalable/actions/fill-color-symbolic.svg',
    'icons/scalable/actions/focus-mode-symbolic.svg',
    'icons/scalable/actions/keyboard-ctrl-space-shortcut-symbolic.svg',
    'icons/scalable/actions/minus-symbolic.svg',
    'icons/scalable/actions/misc-menu-symbolic.svg',
    'icons/scalable/actions/mouse-button-primary-symbolic.svg',
    'icons/scalable/actions/mouse-button-secondary-symbolic.svg',
    'icons/scalable/actions/pen-brush-style-marker-symbolic.svg',
    'icons/scalable/actions/pen-brush-style-solid-symbolic.svg',
    'icons/scalable/actions/pen-brush-style-textured-symbolic.svg',
    'icons/scalable/actions/pen-brush-symbolic.svg',
    'icons/scalable/actions/pen-eraser-split-colliding-strokes-symbolic.svg',
    'icons/scalable/actions/pen-eraser-symbolic.svg',
    'icons/scalable/actions/pen-eraser-trash-colliding-strokes-symbolic.svg',
    'icons/scalable/actions/pen-selector-intersectingpath-symbolic.svg',
    'icons/scalable/actions/pen-selector-polygon-symbolic.svg',
    'icons/scalable/actions/pen-selector-rectangle-symbolic.svg',
    'icons/scalable/actions/pen-selector-single-symbolic.svg',
    'icons/scalable/actions/pen-selector-symbolic.svg',
    'icons/scalable/actions/pen-shaper-style-rough-symbolic.svg',
    'icons/scalable/actions/pen-shaper-style-smooth-symbolic.svg',
    'icons/scalable/actions/pen-shaper-symbolic.svg',
    'icons/scalable/actions/pen-tools-offsetcameratool-symbolic.svg',
    'icons/scalable/actions/pen-tools-symbolic.svg',
    'icons/scalable/actions/pen-tools-verticalspacetool-symbolic.svg',
    'icons/scalable/actions/pen-tools-zoomtool-symbolic.svg',
    'icons/scalable/actions/pen-typewriter-fontchooser-symbolic.svg',
    'icons/scalable/actions/pen-typewriter-symbolic.svg',
    'icons/scalable/actions/plus-symbolic.svg',
    'icons/scalable/actions/preferences-color-symbolic.svg',
    'icons/scalable/actions/remove-page-symbolic.svg',
    'icons/scalable/actions/reset-state-symbolic.svg',
    'icons/scalable/actions/resize-to-fit-content-symbolic.svg',
    'icons/scalable/actions/return-origin-page-symbolic.svg',
    'icons/scalable/actions/selection-deselect-all-symbolic.svg',
    'icons/scalable/actions/selection-duplicate-symbolic.svg',
    'icons/scalable/actions/selection-invert-color-symbolic.svg',
    'icons/scalable/actions/selection-resize-lock-aspectratio-symbolic.svg',
    'icons/scalable/actions/selection-select-all-symbolic.svg',
    'icons/scalable/actions/selection-trash-symbolic.svg',
    'icons/scalable/actions/settings-symbolic.svg',
    'icons/scalable/actions/shapebuilder-arrow-symbolic.svg',
    'icons/scalable/actions/shapebuilder-coordsystem2d-symbolic.svg',
    'icons/scalable/actions/shapebuilder-coordsystem3d-symbolic.svg',
    'icons/scalable/actions/shapebuilder-cubbez-symbolic.svg',
    'icons/scalable/actions/shapebuilder-ellipse-symbolic.svg',
    'icons/scalable/actions/shapebuilder-fociellipse-symbolic.svg',
    'icons/scalable/actions/shapebuilder-grid-symbolic.svg',
    'icons/scalable/actions/shapebuilder-line-symbolic.svg',
    'icons/scalable/actions/shapebuilder-polygon-symbolic.svg',
    'icons/scalable/actions/shapebuilder-polyline-symbolic.svg',
    'icons/scalable/actions/shapebuilder-quadbez-symbolic.svg',
    'icons/scalable/actions/shapebuilder-quadrantcoordsystem2d-symbolic.svg',
    'icons/scalable/actions/shapebuilder-rectangle-symbolic.svg',
    'icons/scalable/actions/sidebar-reveal-symbolic.svg',
    'icons/scalable/actions/stroke-color-symbolic.svg',
    'icons/scalable/actions/stylus-button-eraser-symbolic.svg',
    'icons/scalable/actions/stylus-button-primary-symbolic.svg',
    'icons/scalable/actions/stylus-button-secondary-symbolic.svg',
    'icons/scalable/actions/tab-new-symbolic.svg',
    'icons/scalable/actions/text-align-end-symbolic.svg',
    'icons/scalable/actions/text-align-fill-symbolic.svg',
    'icons/scalable/actions/text-align-start-symbolic.svg',
    'icons/scalable/actions/text-bold-symbolic.svg',
    'icons/scalable/actions/text-indent-less-symbolic.svg',
    'icons/scalable/actions/text-indent-more-symbolic.svg',
    'icons/scalable/actions/text-italic-symbolic.svg',
    'icons/scalable/actions/text-strikethrough-symbolic.svg',
    'icons/scalable/actions/text-underline-symbolic.svg',
    'icons/scalable/actions/touch-two-finger-long-press-symbolic.svg',
    'icons/scalable/actions/trash-symbolic.svg',
    'icons/scalable/actions/window-close-symbolic.svg',
    'icons/scalable/actions/workspacebrowser-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-bandaid-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-bank-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-book-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-bookmark-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-bread-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-calendar-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-camera-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-chip-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-clock-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-code-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-compose-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-crop-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-dictionary-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-document-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-drinks-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-flag-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-folder-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-footprints-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-gamepad-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-gear-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-globe-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-hammer-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-heart-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-hourglass-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-key-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-language-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-library-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-lightbulb-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-math-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-meeting-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-money-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-musicnote-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-nature-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-open-book-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-paintbrush-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-pencilandpaper-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-people-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-person-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-projector-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-science-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-scratchpad-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-shapes-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-shopping-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-speechbubble-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-speedometer-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-star-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-terminal-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-text-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-travel-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-weather-symbolic.svg',
    'icons/scalable/actions/workspacelistentryicon-weight-symbolic.svg',
    'icons/scalable/actions/zoom-fit-width-symbolic.svg',
    'icons/scalable/actions/zoom-in-symbolic.svg',
    'icons/scalable/actions/zoom-out-symbolic.svg',
    'icons/scalable/apps/rnote-devel.svg',
    'icons/scalable/apps/rnote.svg',
    'icons/scalable/mimetypes/application-rnote.svg',
    'icons/symbolic/apps/rnote-symbolic.svg',
)

rnote_ui_gresources_ui_files = files(
    'ui/dialogs/dialogs.ui',
    'ui/dialogs/export.ui',
    'ui/dialogs/import.ui',
    'ui/groupediconpicker/groupediconpicker.ui',
    'ui/groupediconpicker/groupediconpickergroup.ui',
    'ui/penssidebar/brushpage.ui',
    'ui/penssidebar/eraserpage.ui',
    'ui/penssidebar/penssidebar.ui',
    'ui/penssidebar/selectorpage.ui',
    'ui/penssidebar/shaperpage.ui',
    'ui/penssidebar/toolspage.ui',
    'ui/penssidebar/typewriterpage.ui',
    'ui/workspacesbar/workspacerow.ui',
    'ui/workspacesbar/workspacesbar.ui',
    'ui/appmenu.ui',
    'ui/appwindow.ui',
    'ui/canvasmenu.ui',
    'ui/canvaswrapper.ui',
    'ui/colorpicker.ui',
    'ui/contextmenu.ui',
    'ui/filerow.ui',
    'ui/iconpicker.ui',
    'ui/mainheader.ui',
    'ui/overlays.ui',
    'ui/penpicker.ui',
    'ui/penshortcutrow.ui',
    'ui/settingspanel.ui',
    'ui/shortcuts.ui',
    'ui/sidebar.ui',
    'ui/strokecontentpreview.ui',
    'ui/strokewidthpicker.ui',
    'ui/style.css',
    'ui/unitentry.ui',
    'ui/workspacebrowser.ui',
)

rnote_ui_gresources_files = [
    rnote_ui_gresources_icons_files,
    rnote_ui_gresources_ui_files,
    files(
        'resources.gresource.xml',
    ),
]
