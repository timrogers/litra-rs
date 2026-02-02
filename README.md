# `litra-rs`

💡 Control Logitech Litra lights from the command line, Model Context Protocol (MCP) clients and Rust applications

---

## Features

With this tool, you can:

- 💡 Turn your light on and off
- 👀 Check if the light is on or off
- 🎛️ Set, get, increase and decrease the brightness of your light
- 🌡️ Set, get, increase and decrease the temperature of your light
- 🌈 Control the colorful back side of the Litra Beam LX, turning it on and off, setting the brightness and switching colors

> [!TIP]
> 🖲️ Want to automatically turn your Litra on and off when your webcam turns on and off? Check out [`litra-autotoggle`](https://github.com/timrogers/litra-autotoggle)!

## Supported devices

The following Logitech Litra devices, __connected via USB__, are supported:

* [Logitech Litra Glow](https://www.logitech.com/en-gb/products/lighting/litra-glow.946-000002.html)
* [Logitech Litra Beam](https://www.logitech.com/en-gb/products/lighting/litra-beam.946-000007.html)
* [Logitech Litra Beam LX](https://www.logitechg.com/en-gb/products/cameras-lighting/litra-beam-lx-led-light.946-000015.html)

## Installation

### macOS or Linux via [Homebrew](https://brew.sh/)

1. Install the latest version by running `brew install litra`.
1. Run `litra --help` to check that everything is working and see the available commands.

> [!NOTE]
> If you installed Litra via Homebrew before January 30 2026, you will have installed from the `timrogers/homebrew-tap` tap. `litra` is now available through `homebrew-core`. You should reinstall the tool by running `brew uninstall litra && brew install litra`.

### macOS, Linux or Windows via [Cargo](https://doc.rust-lang.org/cargo/), Rust's package manager

1. Install [Rust](https://www.rust-lang.org/tools/install) on your machine, if it isn't already installed.
1. Install the `litra` crate by running `cargo install litra`.
1. Run `litra --help` to check that everything is working and see the available commands.

### macOS, Linux or Windows via direct binary download

1. Download the [latest release](https://github.com/timrogers/litra-rs/releases/latest) for your platform. macOS, Linux and Windows devices are supported.
2. Add the binary to `$PATH`, so you can execute it from your shell. For the best experience, call it `litra` on macOS and Linux, and `litra.exe` on Windows.
3. Run `litra --help` to check that everything is working and see the available commands.

### Configuring `udev` permissions on Linux

On most Linux operating systems, you will need to manually configure permissions using [`udev`](https://www.man7.org/linux/man-pages/man7/udev.7.html) to allow non-`root` users to access and manage Litra devices.

To allow all users that are part of the `video` group to access the Litra devices, copy the [`99-litra.rules`](99-litra.rules) file into `/etc/udev/rules.d`.
Next, reboot your computer or run the following commands as `root`:

    # udevadm control --reload-rules
    # udevadm trigger

## Usage

### From the command line

The following commands are available for controlling your devices:

- `litra on`: Turn your Logitech Litra device on
- `litra off`: Turn your Logitech Litra device off
- `litra toggle`: Toggles your Logitech Litra device on or off
- `litra brightness`: Sets the brightness of your Logitech Litra device, using either `--value` (measured in lumens) or `--percentage` (as a percentage of the device's maximum brightness). The brightness can be set to any value between the minimum and maximum for the device returned by the `devices` command.
- `litra brightness-up`: Increases the brightness of your Logitech Litra device, using either `--value` (measured in lumens) or `--percentage` (with a number of percentage points to add to the device's brightness)
- `litra brightness-down`: Decreases the brightness of your Logitech Litra device, using either `--value` (measured in lumens) or `--percentage` (with a number of percentage points to subtract from the device's brightness)
- `litra temperature`: Sets the temperature of your Logitech Litra device, using a `--value` measured in kelvin (K). The temperature can be set to any multiple of 100 between the minimum and maximum for the device returned by the `devices` command.
- `litra temperature-up`: Increases the temperature of your Logitech Litra device, using a `--value` measured in kelvin (K). The value must be a multiple of 100.
- `litra temperature-down`: Decreases the temperature of your Logitech Litra device, using a `--value` measured in kelvin (K). The value must be a multiple of 100.
- `litra back-on`: Turn on the colorful backlight on your Logitech Litra Beam LX device
- `litra back-off`: Turn off the colorful backlight on your Logitech Litra Beam LX device
- `litra back-toggle`: Toggles the colorful backlight on your Logitech Litra Beam LX device on or off
- `litra back-brightness`: Sets the brightness of the colorful backlight on your Logitech Litra Beam LX device, using `--percentage` (as a percentage of the maximum brightness)
- `litra back-brightness-up`: Increases the brightness of the colorful backlight on your Logitech Litra Beam LX device, using `--percentage` (with a number of percentage points to add to the backlight's brightness)
- `litra back-brightness-down`: Decreases the brightness of the colorful backlight on your Logitech Litra Beam LX device, using `--percentage` (with a number of percentage points to subtract from the backlight's brightness)
- `litra back-color`: Sets the color of the colorful backlight on your Logitech Litra Beam LX device, using `--color` (a named color) or  `--value` (a hexadecimal color code, e.g. `FF0000` for red). You can optionally target a specific zone with `--zone` (numbered 1 to 7 from left to right).

By default, these commands target all connected Litra devices, but this can be filtered down using one of the following device targeting options:

- `--serial-number`/`-s`: Specify the device to target by its serial number
- `--device-path`/`-p`: Specify the device path to target (useful when devices don't have serial numbers)
- `--device-type`/`-t`: Specify the type of device to target: `glow`, `beam`, or `beam_lx` (not available for `back-*` commands which only support Litra Beam LX devices)

The following commands are also included:

- `litra devices`: List Logitech Litra devices connected to your computer. This will be returned in human-readable format by default, or you can get JSON output with the `--json` flag.

Each CLI command can also be called with `--help` for more detailed documentation.

#### Automatic update check

The CLI automatically checks for updates once per day. If a new version is found, it will print details on how to upgrade.

You can disable this behavior by setting the `LITRA_DISABLE_UPDATE_CHECK` environment variable.

### From a Model Context Protocol (MCP) client

Running the `litra mcp` command starts a local Model Context Protocol (MCP) server, exposing tools to allow you to control your Litra devices from AI applications and agents.

#### Usage with Claude Desktop

To use the MCP server with Claude Desktop:

1. From the Claude app, open the "Developer" menu, then click "Open App Config File...".
1. Add the MCP server to the `mcpServers` key in your config:

```json
{
  "mcpServers": {
    "litra": {
      "command": "litra",
      "args": [
        "mcp"
      ]
    }
  }
}
```

1. Back in the Claude app, open the "Developer" menu, then click "Reload MCP Configuration".
1. To check that the MCP server is running, start a chat, then click the "Search and tools" button under the chat input, and check for a "litra" item in the menu.

#### Available Tools

The following tools are available:

- `litra_devices`: List available Logitech Litra devices
- `litra_on`: Turn your Logitech Litra device on
- `litra_off`: Turn your Logitech Litra device off
- `litra_toggle`: Toggles your Logitech Litra device on or off
- `litra_brightness`: Sets the brightness of your Logitech Litra device to either a specific value measured in lumens (lm) or a percentage of the device's maximum brightness. The brightness can be set to any value between the minimum and maximum for the device returned by the `litra_devices` tool.
- `litra_brightness_up`: Increases the brightness of your Logitech Litra device, using either a specific value (measured in lumens) or a percentage of the device's maximum brightness
- `litra_brightness_down`: Decreases the brightness of your Logitech Litra device, using either a specific value (measured in lumens) or a percentage of the device's maximum brightness
- `litra_temperature`: Sets the temperature of your Logitech Litra device to a specific value measured in kelvin (K). The temperature can be set to any multiple of 100 between the minimum and maximum for the device returned by the `litra_devices` tool.
- `litra_temperature_up`: Increases the temperature of your Logitech Litra device, using a specific value measured in kelvin (K). The value must be a multiple of 100.
- `litra_temperature_down`: Decreases the temperature of your Logitech Litra device, using a specific measured in kelvin (K). The value must be a multiple of 100.
- `litra_back_on`: Turn on the back light on your Logitech Litra Beam LX device
- `litra_back_off`: Turn off the back light on your Logitech Litra Beam LX device
- `litra_back_toggle`: Toggles the back light on your Logitech Litra Beam LX device on or off
- `litra_back_brightness`: Sets the brightness of the back light on your Logitech Litra Beam LX device to a specific percentage of the maximum brightness
- `litra_back_brightness_up`: Increases the brightness of the back light on your Logitech Litra Beam LX device by a specified number of percentage points
- `litra_back_brightness_down`: Decreases the brightness of the back light on your Logitech Litra Beam LX device by a specified number of percentage points
- `litra_back_color`: Sets the color of the back light on your Logitech Litra Beam LX device to a specific hexadecimal color code. Can target a specific zone (1-7) or all zones.

### From a Rust application

The `litra` crate includes functions for interacting with Litra devices from your Rust applications.

To see the full API, check out the documentation on [Docs.rs](https://docs.rs/litra/) or read through [`src/lib.rs`](src/lib.rs).
