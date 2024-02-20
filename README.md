# `litra-rs`

💡 Control your Logitech Litra light from the command line

---

## Features

With this tool, you can:

- Turn your light on and off
- Check if the light is on or off
- Set and get the brightness of your light
- Set and get the temperature of your light

## Supported devices

The following Logitech Litra devices, __connected via USB__, are supported:

* [Logitech Litra Glow](https://www.logitech.com/en-gb/products/lighting/litra-glow.946-000002.html)
* [Logitech Litra Beam](https://www.logitech.com/en-gb/products/lighting/litra-beam.946-000007.html)
* [Logitech Litra Beam LX](https://www.logitechg.com/en-gb/products/cameras-lighting/litra-beam-lx-led-light.946-000015.html)

## Installation

### macOS with [Homebrew](https://brew.sh/)

1. Install the latest version by running `brew tap timrogers/tap && brew install litra`.
1. Run `litra --help` to check that everything is working and see the available commands.

### All other platforms (using Cargo)

1. Install [Rust](https://www.rust-lang.org/tools/install) on your machine, if it isn't already installed.
1. Install the `litra` crate by running `cargo install litra`.
1. Run `litra --help` to check that everything is working and see the available commands.

### All other platforms (via binary)

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
- `litra brightness`:  Sets the brightness of your Logitech Litra device, using either `--value` (measured in lumens) or `--percentage` (as a percentage of the device's maximum brightness). The brightness can be set to any value between the minimum and maximum for the device returned by the `devices` command.
- `litra temperature`:  Sets the temperature of your Logitech Litra device, using a `--value` measured in kelvin (K). The temperature be set to any multiple of 100 between the minimum and maximum for the device returned by the `devices` command.

All of the these commands support a `--serial-number`/`-s` argument to specify the serial number of the device you want to target. If you only have one Litra device, you can omit this argument. If you have multiple devices, we recommend specifying it. If it isn't specified, the "first" device will be picked, but this isn't guaranteed to be stable between command runs.

The following commands are also included:

- `litra devices`: List Logitech Litra devices connected to your computer. This will be returned in human-readable format by default, or you can get JSON output with the `--json` flag.

Each CLI command can also be called with `--help` for more detailed documentation.

### From a Rust application

The `litra` crate includes functions for interacting with Litra devices from your Rust applications. 

To see the full API, check out the documentation on [Docs.rs](https://docs.rs/litra/) or read through [`src/lib.rs`](src/lib.rs).
