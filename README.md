# Embedded Terminal Emulator

[![Actions Status](https://github.com/AlexCharlton/embedded-temu/workflows/CI/badge.svg)](https://github.com/AlexCharlton/embedded-temu/actions)

A terminal emulator for [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics). Forked from [embedded-term](https://github.com/rcore-os/embedded-term) to provide a more flexible, stylable API. [Ratatui](https://ratatui.rs/) backend support is provided.

This crate is `no_std` compatible.

## Examples
Outputs png files.

**1. basic**
```sh
cargo run --example basic
```

**2. replay**
```sh
cargo run --example replay -- examples/test-escapes.txt
```

**3. ratatui**
```sh
cargo run --example ratatui --features ratatui-backend
```

**4. fontdue**

Adds TrueType and OpenType font rendering, via [fontdue](https://crates.io/crates/fontdue).
```sh
cargo run --example fontdue
```

## Optional features
- `log`: Enable built-in logging
- `ratatui-backend`: Allow this to be used as a Ratatui backend
