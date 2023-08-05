# sounds
sounds_files = files(
    'sounds/brush/brush.wav',
    'sounds/marker/marker_00.wav',
    'sounds/marker/marker_01.wav',
    'sounds/marker/marker_02.wav',
    'sounds/marker/marker_03.wav',
    'sounds/marker/marker_04.wav',
    'sounds/marker/marker_05.wav',
    'sounds/marker/marker_06.wav',
    'sounds/marker/marker_07.wav',
    'sounds/marker/marker_08.wav',
    'sounds/marker/marker_09.wav',
    'sounds/marker/marker_10.wav',
    'sounds/marker/marker_11.wav',
    'sounds/marker/marker_12.wav',
    'sounds/marker/marker_13.wav',
    'sounds/marker/marker_14.wav',
    'sounds/typewriter/typewriter_00.wav',
    'sounds/typewriter/typewriter_01.wav',
    'sounds/typewriter/typewriter_02.wav',
    'sounds/typewriter/typewriter_03.wav',
    'sounds/typewriter/typewriter_04.wav',
    'sounds/typewriter/typewriter_05.wav',
    'sounds/typewriter/typewriter_06.wav',
    'sounds/typewriter/typewriter_07.wav',
    'sounds/typewriter/typewriter_08.wav',
    'sounds/typewriter/typewriter_09.wav',
    'sounds/typewriter/typewriter_10.wav',
    'sounds/typewriter/typewriter_11.wav',
    'sounds/typewriter/typewriter_12.wav',
    'sounds/typewriter/typewriter_13.wav',
    'sounds/typewriter/typewriter_14.wav',
    'sounds/typewriter/typewriter_15.wav',
    'sounds/typewriter/typewriter_16.wav',
    'sounds/typewriter/typewriter_17.wav',
    'sounds/typewriter/typewriter_18.wav',
    'sounds/typewriter/typewriter_19.wav',
    'sounds/typewriter/typewriter_20.wav',
    'sounds/typewriter/typewriter_21.wav',
    'sounds/typewriter/typewriter_22.wav',
    'sounds/typewriter/typewriter_23.wav',
    'sounds/typewriter/typewriter_24.wav',
    'sounds/typewriter/typewriter_25.wav',
    'sounds/typewriter/typewriter_26.wav',
    'sounds/typewriter/typewriter_27.wav',
    'sounds/typewriter/typewriter_28.wav',
    'sounds/typewriter/typewriter_29.wav',
    'sounds/typewriter/typewriter_bell.wav',
    'sounds/typewriter/typewriter_insert.wav',
    'sounds/typewriter/typewriter_linefeed.wav',
    'sounds/typewriter/typewriter_thump.wav',
)

# install sounds flat in one directory
install_data(sounds_files, install_dir: pkgdatadir / 'sounds')


# fonts
fonts_files = files(
    'fonts/virgil.ttf'
)

# install fonts
install_data(fonts_files, install_dir: pkgdatadir / 'fonts')

# fontconfig file
message('Configuring fontconfig file')

fc_conf = configuration_data()
fc_conf.set('APP_ID', app_id)
fc_conf.set('APP_NAME', app_name)
fc_conf.set('APP_NAME_CAPITALIZED', app_name_capitalized)
fc_conf.set('PKG_DATA_DIR', pkgdatadir)
fc_file_in = configure_file(
    input: '09-app.conf.in',
    output: '09-' + app_name + '.conf',
    configuration: fc_conf,
    install: true,
    install_dir: sysconfdir / 'fonts/conf.d',
)
