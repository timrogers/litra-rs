# Looking Back: One Year of litra-rs Progress 🎉

Hi everyone! 👋

As we approach the one-year anniversary since February 2025, I wanted to take a moment to reflect on the incredible journey litra-rs has been on. This project has grown significantly, and I'm excited to share all the amazing improvements we've made together.

## Major Milestones 🚀

### Version 3.0: Litra Beam LX Support
The biggest feature addition this year was **full support for the Logitech Litra Beam LX**, including its colorful backlight! This was made possible through the fantastic work of @aleixpol and the generous financial support from @liyuwen356-collab via GitHub Sponsors.

**New capabilities include:**
- Control the RGB backlight (on/off, brightness, color)
- Set colors using hex codes or convenient named colors
- Target individual zones (1-7) or control all zones at once
- New CLI commands: `back-toggle`, `back-color`, `back-brightness`, `back-brightness-up`, `back-brightness-down`
- Corresponding MCP tools and Rust API methods

### Model Context Protocol (MCP) Integration
We've embraced the emerging Model Context Protocol, allowing AI assistants like Claude to control your Litra lights directly! The `litra mcp` command starts a local MCP server with comprehensive tools for:
- Listing devices
- Toggling lights on/off
- Adjusting brightness and temperature
- Controlling the Litra Beam LX backlight

### Improved Distribution 📦

#### Homebrew Core Integration
Previously available through a custom tap, **litra is now available directly in Homebrew core**! This means easier installation and better integration with the Homebrew ecosystem.

```bash
brew install litra  # That's it!
```

#### Multi-Architecture Support
- Added ARM64 Windows support, extending compatibility beyond x86_64
- Continued support for macOS, Linux, and Windows across different architectures

### Automatic Update Checks
Version 3.3.0 introduced automatic update checking! The CLI now:
- Checks for new versions once per day (respecting a 72-hour release maturity window)
- Displays helpful upgrade instructions when updates are available
- Can be disabled via the `LITRA_DISABLE_UPDATE_CHECK` environment variable

## Device Selection Improvements 🎯

Thanks to @joegoldin's contributions, device targeting is now much more flexible:

- **`--device-type`**: Filter by device type (e.g., `glow`, `beam`, `beam_lx`)
- **`--device-path`**: Select devices by platform-specific path (especially useful for devices without serial numbers)
- **Stable ordering**: Devices are now returned in a consistent order
- **Table output**: The `litra devices` command now displays results in a clean, readable table format

## Quality of Life Improvements ✨

### User Experience
- Better validation error messages
- Fixed percentage validation for brightness commands
- Improved MCP tool descriptions
- Enhanced documentation throughout

### Developer Experience
- Fixed issues with using litra as a Rust library (correctly marking serde dependencies)
- Improved crate publishing to Crates.io
- Better-organized codebase

## Bug Fixes 🐛

- Fixed `--device-path` and `--serial-number` filters for `back-*` commands
- Resolved various edge cases and validation issues
- Improved cross-platform compatibility

## Dependency Maintenance 🔧

We've kept dependencies up-to-date throughout the year with regular updates to:
- hidapi, clap, tokio, serde, rmcp, schemars, and more
- Rust toolchain updates to 1.92.0

## Release Cadence 📅

Over the past year, we've shipped:
- **12 releases** from v2.3.0 to v3.3.0
- A major version bump (v3.0.0) for Litra Beam LX support
- Regular minor and patch releases with improvements and fixes

## Version History (Feb 2025 - Feb 2026)

| Version | Date | Highlights |
|---------|------|------------|
| v3.3.0 | Feb 2, 2026 | Automatic update checking |
| v3.2.0 | Jan 23, 2026 | ARM Windows support |
| v3.1.1 | Jan 23, 2026 | Bug fixes for device filters, stable device ordering |
| v3.1.0 | Jan 10, 2026 | Named colors for `back-color` command |
| v3.0.0 | Jan 10, 2026 | Litra Beam LX backlight support |
| v2.5.2 | Jan 10, 2026 | Fix for Rust library usage |
| v2.5.1 | Dec 21, 2025 | Maintenance release |
| v2.5.0 | Dec 21, 2025 | Various improvements |
| v2.4.1 | Dec 21, 2025 | Bug fixes |
| v2.4.0 | Aug 23, 2025 | Device-type and device-path selection, table output |
| v2.3.1 | Jul 14, 2025 | Rust crate publishing fix |
| v2.3.0 | Jun 30, 2025 | Various improvements |

## Thank You! 🙏

A huge thank you to:
- **@aleixpol** for the incredible work on Litra Beam LX support
- **@joegoldin** for device selection improvements
- **@liyuwen356-collab** and other GitHub Sponsors for financial support
- **@chenrui333** and the Homebrew maintainers for bringing litra to homebrew-core
- **All contributors** who opened issues, submitted PRs, and helped improve the project
- **Everyone using litra** and providing feedback

## Looking Forward 🔮

The project continues to evolve! Some areas we're exploring:
- Further improving the MCP integration as the protocol matures
- Expanding device support as new Litra models are released
- Continuing to enhance the CLI and Rust API based on user feedback

## Get Involved 💬

Whether you're a user, contributor, or just interested in the project:
- Try out the latest features and share your feedback
- Report bugs or suggest improvements via GitHub Issues
- Consider sponsoring the project if you find it valuable
- Spread the word about litra-rs!

Thank you all for making this past year so productive and exciting. Here's to another great year ahead! 🎊

---

*Built with Rust 🦀 | Available via Homebrew, Cargo, and direct downloads*
