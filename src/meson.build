global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
is_devel = get_option('profile') == 'development'
if is_devel
  global_conf.set('IS_DEVEL', 'true')
else
  global_conf.set('IS_DEVEL', 'false')
endif
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true
)

if get_option('profile') == 'default'
  cargo_release_options = [ '--release' ]
  cargo_clippy_deny_warnings = []
  rust_target = 'release'
  message('Building in release mode')
else
  cargo_release_options = []
  cargo_clippy_deny_warnings = [
    '--',
    '--deny',
    'warnings',
  ]
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_build_options = [
  '--manifest-path', meson.project_source_root() / 'Cargo.toml',
  cargo_release_options,
]
cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  env: cargo_env,
  command: [
    cargo, 'build',
    cargo_build_options,
    '&&',
    'cp', cargo_target / rust_target / meson.project_name(), '@OUTPUT@',
  ]
)

cargo_extra_options = [
  cargo_build_options,
  '--all-targets',
  '--all-features',
  '--workspace',
]

cargo_test_options = cargo_extra_options
test(
  'cargo-test',
  cargo,
  args: [
    'test',
    cargo_test_options,
  ],
  env: cargo_env,
  timeout: 400,
)

cargo_clippy_options = [
  cargo_extra_options,
  cargo_clippy_deny_warnings,
]
test(
  'cargo-clippy',
  cargo,
  args: [
    'clippy',
    cargo_clippy_options,
  ],
  env: cargo_env,
  timeout: 400, # cargo might take a bit of time sometimes
)
