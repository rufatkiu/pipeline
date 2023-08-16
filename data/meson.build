subdir('icons')
subdir('resources')

# Desktop file
install_data(
  '@0@.desktop'.format(application_id),
  install_dir: datadir / 'applications'
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
