global_conf = configuration_data()
cargo_options = [ '--manifest-path', meson.project_source_root() / 'nce-helper' /  'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'nce-helper' /  'src' ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'nce-helper' / 'cargo-home' ]

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: 'nce-helper',
  console: true,
  install: true,
  install_dir: get_option('libexecdir'),
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', 'nce-helper' / 'src' / rust_target / 'nce-helper', '@OUTPUT@',
  ]
)
