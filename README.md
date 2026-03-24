# AUTD3

[![build](https://github.com/shinolab/autd3-rs/workflows/build/badge.svg)](https://github.com/shinolab/autd3-rs/actions)
[![codecov](https://codecov.io/gh/shinolab/autd3-rs/graph/badge.svg?precision=2)](https://codecov.io/gh/shinolab/autd3-rs)
[![Crate.io version](https://img.shields.io/crates/v/autd3)](https://crates.io/crates/autd3)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

The Rust implementation of [**AUTD3**](https://github.com/shinolab/autd3) library.

## Project Structure

This workspace contains several crates:

- [autd3](./autd3): The main library for AUTD3.
- [autd3-core](./autd3-core): Core components of AUTD3.
- [autd3-derive](./autd3-derive): Macros for AUTD3.
- [autd3-driver](./autd3-driver): Driver components.
- [autd3-firmware-emulator](./autd3-firmware-emulator): Emulator for AUTD3 firmware.
- [autd3-gain-holo](./autd3-gain-holo): Multiple focal point gain.
- [autd3-link-*]: Various link implementations (EtherCrab, TwinCAT, Remote, etc.).

## Development

This project uses `cargo xtask` for common development tasks.

- **Build all**: `cargo xtask build`
- **Test all**: `cargo xtask test`
- **Lint**: `cargo xtask lint`
- **Format**: `cargo xtask format`
- **Check all**: `cargo xtask check` (format + lint + build + test)
- **Code Coverage**: `cargo xtask cov` (requires `grcov`)
- **Documentation**: `cargo xtask doc`

## Examples

```bash
cargo xtask run <example_name>
```

Available examples:
- `twincat`: Example using local TwinCAT.
- `remote_twincat`: Example using remote TwinCAT.
- `ethercrab`: Example using EtherCrab.
- `remote`: Example using Remote for simulator or remote server.
- `remote_server`: Example for remote server.
- `async`: Example using async API.
- (`nop`: Example that does nothing (for testing).)

## License

This project is licensed under the MIT License, see the [LICENSE](./LICENSE) for details.

## Author

Shun Suzuki, 2022-2026
