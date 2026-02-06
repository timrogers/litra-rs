# litra-rs: Year in Review (Feb 2025 - Feb 2026)

It's been a busy year for `litra-rs`! We shipped **8 releases** spanning two major versions, with highlights including full Litra Beam LX backlight support, MCP server improvements, ARM Windows builds, and an automatic update checker. Here's a summary of everything that landed.

## v3.3.0 (February 2, 2026)

**Automatic update checking** — The CLI now checks for new releases automatically and notifies you when a newer version is available. Checks happen at most once a day, and look for releases that have been published for at least 72 hours (to avoid notifying about versions before binaries are fully available). If you'd prefer to opt out, set the `LITRA_DISABLE_UPDATE_CHECK` environment variable. State is stored in `.litra.toml`.

Other changes:
- Updated the readme to reflect availability through `homebrew-core`

## v3.2.0 (January 23, 2026)

**ARM Windows support** — Pre-built binaries are now published for ARM64 Windows devices, not just AMD64. If you're on a Snapdragon-based Windows machine, you can now grab a native build.

## v3.1.1 (January 23, 2026)

Bug fixes and stability improvements:
- Fixed `--device-path` and `--serial-number` filters for `back-*` CLI commands
- `litra devices` and the underlying Rust API now return devices in a stable, deterministic order
- Reverted an earlier experiment with using a shared system `hidapi` library on Linux, which caused compatibility issues

## v3.1.0 (January 10, 2026)

**Pre-set color names for backlight** — The `back-color` command now accepts a `--color` option with pre-set color names (e.g. `red`, `blue`, `green`), making it easier to set colors without specifying raw RGB values.

## v3.0.0 (January 10, 2026)

**Full Litra Beam LX back-side control** — This major release adds comprehensive support for the colorful backlight on the Litra Beam LX:

- New CLI commands: `back-toggle`, `back-brightness-up`, `back-brightness-down`
- New Rust API functions: `is_back_on`, `back_brightness_percentage`
- Back status information is now included in the `devices` command output
- MCP tools for controlling the back side of the Litra Beam LX
- Support for Litra Beam LX RGB zone control

**Breaking change:** The JSON output of `litra devices --json` was restructured — the previous `device_type` field was renamed to `device_type_display`, a new enum-based `device_type` field was added, and a `has_back_side` boolean was introduced.

Also fixed: `serde` and `serde_json` dependencies are now correctly marked as non-optional, fixing usage of `litra` as a library.

## v2.5.1 (December 21, 2025)

Bug fix release:
- Fixed MCP server name and version reporting
- Fixed the "Publish with Cargo to Crates.io" CI build step

## v2.5.0 (December 21, 2025)

**MCP server improvements** — This release focused on improving the Model Context Protocol (MCP) server:

- Added annotations to MCP tools
- `litra_devices` MCP tool now returns structured outputs
- Upgraded to `rmcp` v0.12.0
- Updated Rust toolchain to v1.92.0

## v2.4.1 (December 21, 2025)

Dependency updates:
- Bumped `tabled` from 0.16.0 to 0.20.0
- Bumped `actions/checkout` from v4 to v6

---

## By the numbers

| Metric | Value |
|---|---|
| Releases | 8 |
| Major versions | 2 (v2 → v3) |
| New CLI commands | 4+ (`back-toggle`, `back-brightness-up`, `back-brightness-down`, `back-color --color`) |
| Platforms with new builds | ARM64 Windows |
| New features | Beam LX backlight control, auto update checks, pre-set color names |

## What's next?

We'd love to hear what features or improvements you'd like to see. Drop your ideas in the comments below!

Thanks to everyone who has contributed, filed issues, and used `litra-rs` over the past year.
