project('displays-astal', version: '0.1.0')

headers = run_command('find', meson.project_source_root() / 'include', '-name', '*.h', check: true).stdout().strip().split('\n')

cargo_args = ['cargo', 'build', '--release', '--manifest-path', meson.project_source_root() / 'Cargo.toml']
if get_option('faked')
  cargo_args += ['--features', 'faked']
endif

so_target = custom_target('cargo-build',
  output: 'libastaldisplays.so',
  command: [
    'sh', '-c',
    ' '.join(cargo_args) +
    ' && cp ' + meson.project_source_root() / '..' / 'target/release/libastaldisplays.so' + ' @OUTPUT@',
  ],
  build_by_default: true,
  console: true,
  install: true,
  install_dir: get_option('libdir'),
)

gir = custom_target('gir',
  input: so_target,
  output: 'AstalDisplays-0.1.gir',
  command: [
    'g-ir-scanner', '-v', '--warn-all',
    '--namespace', 'AstalDisplays',
    '--nsversion', '0.1',
    '--identifier-prefix', 'AstalDisplays',
    '--symbol-prefix', 'astal_displays',
    '--c-include', 'astal_displays.h',
    '-I' + meson.project_source_root() / 'include',
    '--library=astaldisplays',
    '--library-path', meson.current_build_dir(),
    '--include', 'GLib-2.0', '--pkg', 'glib-2.0',
    '--include', 'GObject-2.0', '--pkg', 'gobject-2.0',
    '--include', 'Gio-2.0', '--pkg', 'gio-2.0',
    '--pkg-export', 'AstalDisplays-0.1',
    '--output', '@OUTPUT@',
    headers,
  ],
  build_by_default: true,
  install: true,
  install_dir: get_option('datadir') / 'gir-1.0',
)

custom_target('typelib',
  input: gir,
  output: 'AstalDisplays-0.1.typelib',
  command: [
    'g-ir-compiler',
    '--shared-library', 'libastaldisplays.so',
    '@INPUT@', '-o', '@OUTPUT@',
  ],
  build_by_default: true,
  install: true,
  install_dir: get_option('libdir') / 'girepository-1.0',
)
