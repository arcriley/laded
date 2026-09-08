#  meson.build  Build specification for laded
#
# Copyright 2026 Emerge Cooperative
#
#   Permission is hereby granted, free of charge, to any person
#   obtaining a copy of this software and associated documentation
#   files (the "Software"), to deal in the Software without
#   restriction, including without limitation the rights to use, copy,
#   modify, merge, publish, distribute, sublicense, and/or sell copies
#   of the Software, and to permit persons to whom the Software is
#   furnished to do so, subject to the following conditions:
#
#   The above copyright notice and this permission notice shall be
#   included in all copies or substantial portions of the Software.
#
#   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
#   EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
#   MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
#   NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
#   BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
#   ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
#   CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
#   SOFTWARE.


project('laded', 'rust',
          version: '0.1.0',
    meson_version: '>= 1.0.0',
  default_options: [ 'warning_level=2', 'werror=false' ],
)

sh_bin = find_program('sh', required: true)
cargo_bin = find_program('cargo', required: true)

cargo_build_dir = meson.project_build_root() / 'src'
cargo_home_dir = meson.project_build_root() / 'cargo-home'
manifest_path = meson.project_source_root() / 'Cargo.toml'

if get_option('buildtype') == 'release'
  cargo_release_flag = '--release'
  rust_target = 'release'
else
  cargo_release_flag = ''
  rust_target = 'debug'
endif

# Binary name matching [[bin]] name in Cargo.toml
binary_name = 'laded'
built_binary_path = cargo_build_dir / rust_target / binary_name

cargo_cmd = ' '.join([
  'CARGO_HOME=' + cargo_home_dir,
  cargo_bin.full_path(), 'build',
  '--manifest-path', manifest_path,
  '--target-dir', cargo_build_dir,
  '--bin', binary_name,
  cargo_release_flag,
  '&&', 'cp', built_binary_path, '@OUTPUT@'
])

cargo_bin = custom_target(
  'laded',
  output: 'laded',
  build_by_default: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    'cargo', 'build',
    '--manifest-path', '@INPUT0@',
    '--release',
    '--bin', 'laded'
  ],
  input: files('Cargo.toml')
)