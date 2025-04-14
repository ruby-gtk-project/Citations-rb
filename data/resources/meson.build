# Resources
resources_conf = configuration_data()
resources_conf.set('APP_ID', application_id)
resources = gnome.compile_resources(
  'resources',
  configure_file(
    input: files('resources.gresource.xml.in'),
    output: 'resources.gresource.xml',
    configuration: resources_conf,
  ),
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
  dependencies: appdata_file,
)
