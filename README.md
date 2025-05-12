# `litra-rs`

💡 Control your Logitech Litra light from the command line

---

## Features

With this tool, you can:

- Turn your light on and off
- Check if the light is on or off
- Set, get, increase and decrease the brightness of your light
- Set, get, increase and decrease the temperature of your light

> [!TIP]
> 🖲️ Want to automatically turn your Litra on and off when your webcam turns on and off? Check out [`litra-autotoggle`](https://github.com/timrogers/litra-autotoggle)!

## Supported devices

The following Logitech Litra devices, __connected via USB__, are supported:

* [Logitech Litra Glow](https://www.logitech.com/en-gb/products/lighting/litra-glow.946-000002.html)
* [Logitech Litra Beam](https://www.logitech.com/en-gb/products/lighting/litra-beam.946-000007.html)
* [Logitech Litra Beam LX](https://www.logitechg.com/en-gb/products/cameras-lighting/litra-beam-lx-led-light.946-000015.html)

## Installation

### macOS or Linux via [Homebrew](https://brew.sh/)

1. Install the latest version by running `brew tap timrogers/tap && brew install litra`.
1. Run `litra --help` to check that everything is working and see the available commands.

### macOS, Linux or Windows via [Cargo](https://doc.rust-lang.org/cargo/), Rust's package manager

1. Install [Rust](https://www.rust-lang.org/tools/install) on your machine, if it isn't already installed.
1. Install the `litra` crate by running `cargo install litra`.
1. Run `litra --help` to check that everything is working and see the available commands.

### macOS, Linux or Windows via direct binary download

1. Download the [latest release](https://github.com/timrogers/litra-rs/releases/latest) for your platform. macOS, Linux and Windows devices are supported.
2. Add the binary to `$PATH`, so you can execute it from your shell. For the best experience, call it `litra` on macOS and Linux, and `litra.exe` on Windows.
3. Run `litra --help` to check that everything is working and see the available commands.

## Configuring `udev` permissions on Linux

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
- `litra temperature`: Sets the temperature of your Logitech Litra device, using a `--value` measured in kelvin (K). The temperature be set to any multiple of 100 between the minimum and maximum for the device returned by the `devices` command.
- `litra temperature-up`: Increases the temperature of your Logitech Litra device, using a `--value` measured in kelvin (K). The value must be a multiple of 100.
- `litra temperature-down`: Decreases the temperature of your Logitech Litra device, using a `--value` measured in kelvin (K). The value must be a multiple of 100.

All of these commands support the following device targeting options:

- `--serial-number`/`-s`: Specify the device to target by its serial number
- `--device-path`/`-p`: Specify the device path to target (useful when devices don't have serial numbers)
- `--device-type`/`-t`: Specify the type of device to target (`LitraGlow`, `LitraBeam`, or `LitraBeamLX`)

Important usage notes:

- You can use only one of these options at a time.
- If you don't specify any targeting option, the command will apply to all connected devices.
- The device-type filter accepts both the full name (e.g., "LitraGlow") and shortened names (e.g., "Glow") in a case-insensitive manner.

The following commands are also included:

- `litra devices`: List Logitech Litra devices connected to your computer. This will be returned in human-readable format by default, or you can get JSON output with the `--json` flag.

Each CLI command can also be called with `--help` for more detailed documentation.

### From a Rust application

The `litra` crate includes functions for interacting with Litra devices from your Rust applications.

To see the full API, check out the documentation on [Docs.rs](https://docs.rs/litra/) or read through [`src/lib.rs`](src/lib.rs).
