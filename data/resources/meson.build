subdir('ui')

blueprints = custom_target('blueprints',
  input: files(
    'ui/import_window.blp',
    'ui/shortcuts.blp',
    'ui/video_information_window.blp',
    'ui/about.blp.in',
    'ui/error_label.blp',
    'ui/feed_item.blp',
    'ui/feed_list.blp',
    'ui/feed_page.blp',
    'ui/filter_item.blp',
    'ui/filter_list.blp',
    'ui/filter_page.blp',
    'ui/subscription_item.blp',
    'ui/subscription_list.blp',
    'ui/subscription_page.blp',
    'ui/watch_later.blp',
    'ui/window.blp',
    'ui/thumbnail.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

# Resources
resources = gnome.compile_resources(
  'resources',
  'resources.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
  dependencies: blueprints,
)
