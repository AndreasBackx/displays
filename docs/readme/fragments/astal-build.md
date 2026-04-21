## Building

Default build, using the real backend:

```sh
cargo build -p displays_astal --release
meson setup build
meson compile -C build
```

Fake build, using the `faked` Cargo feature:

```sh
cargo build -p displays_astal --release --features faked
meson setup build-faked -Dfaked=true
meson compile -C build-faked
```
