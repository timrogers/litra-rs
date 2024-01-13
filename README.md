# `litra-rs`

💡 Control your Logitech Litra light from the command line

The following Logitech Litra devices, connected via USB, are supported:

* [Logitech Litra Glow](https://www.logitech.com/en-gb/products/lighting/litra-glow.946-000002.html)
* [Logitech Litra Beam](https://www.logitech.com/en-gb/products/lighting/litra-beam.946-000007.html) 
* [Logitech Litra Beam LX](https://www.logitechg.com/en-gb/products/cameras-lighting/litra-beam-lx-led-light.946-000015.html?&utm_source=Google&utm_medium=Paid-Search&utm_campaign=Dialect_FY24_Q3_GBR_GA_G_DTX-LogiG-Creator_Google_na&gad_source=1&gclid=CjwKCAiAp5qsBhAPEiwAP0qeJs7jOdlBu8DCsEoOFt1_BK1HLABI0l2jglDweTnNDddt5neXm_vpyRoCic4QAvD_BwE)

With this driver, you can:

- Turn your light on and off
- Check if the light is on or off
- Set and get the brightness of your light
- Set and get the temperature of your light

## Installation

1. Download the [latest release](https://github.com/timrogers/litra-rs/releases/latest) for your platform. macOS, Linux and Windows devices are supported.
2. Add the binary to your path, so you can execute it from your shell. For the best experience, call it `litra` on macOS and Linux, and `litra.exe` on Windows.
3. Run `litra --help` to check that everything is working, and see the available commands.

## Usage

The following commands are available for controlling your devices:

- `litra on`: Turn your Logitech Litra device on

All of the these commands support a `--serial-number`/`-s` argument to specify the serial number of the device you want to target. If you only have one Litra device, you can omit this argument. If you have multiple devices, we recommend specifying it. If it isn't specified, the "first" device will be picked, but this isn't guaranteed to be stable between command runs.

The following commands are also included:

- `litra list-devices`: List Logitech Litra devices connected to your computer. This will be returned in human-readable format by default, or you can get JSON output with the `--json` flag.

Each CLI command can also be called with `--help` for more detailed documentation.