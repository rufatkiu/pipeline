subdir('icons')
subdir('resources')

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('app-id', application_id)
desktop_conf.set('bindir', bindir)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(application_id),
  type: 'desktop',
  po_dir: podir,
  install: true,
  install_dir: join_paths(datadir, 'applications')
)

# Metainfo
appstream_file = i18n.merge_file(
        input: '@0@.metainfo.xml.in'.format(base_id),
       output: '@0@.metainfo.xml'.format(application_id),
       po_dir: '../po',
      install: true,
  install_dir: get_option('datadir') / 'metainfo'
)

if appstream_cli.found()
  test('Validate appstream file', appstream_cli,
      args: ['validate', '--no-net', '--explain', appstream_file])
endif

install_data(
  '@0@.gschema.xml'.format(application_id),
  install_dir: datadir / 'glib-2.0' / 'schemas'
)

# Compile schemas
settings_schemas = [ '@0@.gschema.xml'.format(application_id) ]
gnome.compile_schemas(depend_files: files(settings_schemas))
