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
install_data(
  '@0@.metainfo.xml'.format(application_id),
  install_dir: datadir / 'metainfo'
)

install_data(
  '@0@.gschema.xml'.format(application_id),
  install_dir: datadir / 'glib-2.0' / 'schemas'
)

# Compile schemas
settings_schemas = [ '@0@.gschema.xml'.format(application_id) ]
gnome.compile_schemas(depend_files: files(settings_schemas))
