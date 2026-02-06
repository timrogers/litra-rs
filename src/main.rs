use clap::{builder::TypedValueParser, ArgGroup, Parser, Subcommand, ValueEnum};
use litra::{Device, DeviceError, DeviceHandle, DeviceResult, DeviceType, Litra};
use serde::Serialize;
use std::fmt;
use std::process::ExitCode;
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "cli")]
use colored::Colorize;
#[cfg(feature = "cli")]
use tabled::{Table, Tabled};

// Custom parser for DeviceType
#[derive(Debug, Clone)]
struct DeviceTypeValueParser;

impl TypedValueParser for DeviceTypeValueParser {
    type Value = DeviceType;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value_str = value.to_string_lossy();
        DeviceType::from_str(&value_str).map_err(|_| {
            let mut err = clap::Error::new(clap::error::ErrorKind::InvalidValue);
            if let Some(arg) = arg {
                err.insert(
                    clap::error::ContextKind::InvalidArg,
                    clap::error::ContextValue::String(arg.to_string()),
                );
            }
            err.insert(
                clap::error::ContextKind::Custom,
                clap::error::ContextValue::String(format!("Invalid device type: {}", value_str)),
            );
            err
        })
    }
}

#[cfg(feature = "mcp")]
mod mcp;

/// Control your USB-connected Logitech Litra lights from the command line
#[cfg(feature = "cli")]
#[derive(Debug, Parser)]
#[clap(
    name = "litra",
    version,
    after_long_help = "This CLI automatically checks for updates once per day. To disable update checks, set the LITRA_DISABLE_UPDATE_CHECK environment variable to any value."
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

const SERIAL_NUMBER_ARGUMENT_HELP: &str = "Specify the device to target by its serial number";
const DEVICE_PATH_ARGUMENT_HELP: &str =
    "Specify the device to target by its path (useful for devices that don't show a serial number)";
const DEVICE_TYPE_ARGUMENT_HELP: &str =
    "Specify the device to target by its type (`glow`, `beam` or `beam_lx`)";

/// Named colors for the back-color command
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Copy, ValueEnum)]
enum NamedColor {
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Purple,
    Pink,
    Cyan,
    White,
    Magenta,
}

#[cfg(feature = "cli")]
impl NamedColor {
    fn to_hex(self) -> &'static str {
        match self {
            NamedColor::Red => "FF0000",
            NamedColor::Green => "00FF00",
            NamedColor::Blue => "0000FF",
            NamedColor::Yellow => "FFFF00",
            NamedColor::Orange => "FFA500",
            NamedColor::Purple => "800080",
            NamedColor::Pink => "FFC0CB",
            NamedColor::Cyan => "00FFFF",
            NamedColor::White => "FFFFFF",
            NamedColor::Magenta => "FF00FF",
        }
    }
}

#[cfg(feature = "cli")]
#[derive(Debug, Subcommand)]
enum Commands {
    /// Turn your Logitech Litra device on. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    On {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
    },
    /// Turn your Logitech Litra device off. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    Off {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
    },
    /// Toggles your Logitech Litra device on or off. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    Toggle {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
    },
    /// Sets the brightness of your Logitech Litra device. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    #[clap(group = ArgGroup::new("brightness").required(true).multiple(false))]
    Brightness {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
        #[clap(
            long,
            short,
            help = "The brightness to set, measured in lumens. This can be set to any value between the minimum and maximum for the device returned by the `devices` command.",
            group = "brightness"
        )]
        value: Option<u16>,
        #[clap(
            long,
            short('b'),
            help = "The brightness to set, as a percentage of the maximum brightness",
            group = "brightness",
            value_parser = clap::value_parser!(u8).range(1..=100)
        )]
        percentage: Option<u8>,
    },
    /// Increases the brightness of your Logitech Litra device. The command will error if trying to increase the brightness beyond the device's maximum. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    #[clap(group = ArgGroup::new("brightness-up").required(true).multiple(false))]
    BrightnessUp {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
        #[clap(
            long,
            short,
            help = "The amount to increase the brightness by, measured in lumens.",
            group = "brightness-up"
        )]
        value: Option<u16>,
        #[clap(
            long,
            short,
            help = "The number of percentage points to increase the brightness by",
            group = "brightness-up",
            value_parser = clap::value_parser!(u8).range(1..=100)
        )]
        percentage: Option<u8>,
    },
    /// Decreases the brightness of your Logitech Litra device. The command will error if trying to decrease the brightness below the device's minimum. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    #[clap(group = ArgGroup::new("brightness-down").required(true).multiple(false))]
    BrightnessDown {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
        #[clap(
            long,
            short,
            help = "The amount to decrease the brightness by, measured in lumens.",
            group = "brightness-down"
        )]
        value: Option<u16>,
        #[clap(
            long,
            short,
            help = "The number of percentage points to reduce the brightness by",
            group = "brightness-down",
            value_parser = clap::value_parser!(u8).range(1..=100)
        )]
        percentage: Option<u8>,
    },
    /// Sets the temperature of your Logitech Litra device. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    Temperature {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
        #[clap(
            long,
            short,
            help = "The temperature to set, measured in Kelvin. This can be set to any multiple of 100 between the minimum and maximum for the device returned by the `devices` command."
        )]
        value: u16,
    },
    /// Increases the temperature of your Logitech Litra device. The command will error if trying to increase the temperature beyond the device's maximum. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    TemperatureUp {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
        #[clap(
            long,
            short,
            help = "The amount to increase the temperature by, measured in Kelvin. This must be a multiple of 100."
        )]
        value: u16,
    },
    /// Decreases the temperature of your Logitech Litra device. The command will error if trying to decrease the temperature below the device's minimum. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    TemperatureDown {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with_all = ["device_path", "device_type"]
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with_all = ["serial_number", "device_type"]
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
        device_type: Option<DeviceType>,
        #[clap(
            long,
            short,
            help = "The amount to decrease the temperature by, measured in Kelvin. This must be a multiple of 100."
        )]
        value: u16,
    },
    /// Set the color of one or more zones on the back of your Logitech Litra Beam LX device. By default, all Litra Beam LX devices are targeted, unless a specific device is specified with --serial-number or --device-path.
    #[clap(group = ArgGroup::new("color-input").required(true).multiple(false))]
    BackColor {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with = "device_path"
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with = "serial_number"
        )]
        device_path: Option<String>,
        #[clap(
            long,
            short,
            help = "The hexadecimal color code to use (e.g. FF0000 for red). Either --value or --color must be specified.",
            group = "color-input"
        )]
        value: Option<String>,
        #[clap(
            long,
            short,
            help = "A named color to use. Either --value or --color must be specified.",
            group = "color-input"
        )]
        color: Option<NamedColor>,
        #[clap(
            long,
            short('z'),
            help = "The zone of the light to control, numbered 1 to 7 from left to right. If not specified, all zones will be targeted."
        )]
        zone: Option<u8>,
    },
    /// Set the brightness of the colorful backlight on your Logitech Litra Beam LX device. By default, all Litra Beam LX devices are targeted, unless a specific device is specified with --serial-number or --device-path.
    BackBrightness {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with = "device_path"
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with = "serial_number"
        )]
        device_path: Option<String>,
        #[clap(
            long,
            short('b'),
            help = "The brightness to set, as a percentage of the maximum brightness",
            value_parser = clap::value_parser!(u8).range(1..=100)
        )]
        percentage: u8,
    },
    /// Turn off the colorful backlight on your Logitech Litra Beam LX device. By default, all Litra Beam LX devices are targeted, unless a specific device is specified with --serial-number or --device-path.
    BackOff {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with = "device_path"
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with = "serial_number"
        )]
        device_path: Option<String>,
    },
    /// Turn on the colorful backlight on your Logitech Litra Beam LX device. By default, all Litra Beam LX devices are targeted, unless a specific device is specified with --serial-number or --device-path.
    BackOn {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with = "device_path"
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with = "serial_number"
        )]
        device_path: Option<String>,
    },
    /// Toggles the colorful backlight on your Logitech Litra Beam LX device on or off. By default, all Litra Beam LX devices are targeted, unless a specific device is specified with --serial-number or --device-path.
    BackToggle {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with = "device_path"
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with = "serial_number"
        )]
        device_path: Option<String>,
    },
    /// Increases the brightness of the colorful backlight on your Logitech Litra Beam LX device. The command will error if trying to increase the brightness beyond 100%. By default, all Litra Beam LX devices are targeted, unless a specific device is specified with --serial-number or --device-path.
    BackBrightnessUp {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with = "device_path"
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with = "serial_number"
        )]
        device_path: Option<String>,
        #[clap(
            long,
            short('b'),
            help = "The number of percentage points to increase the brightness by",
            value_parser = clap::value_parser!(u8).range(1..=100)
        )]
        percentage: u8,
    },
    /// Decreases the brightness of the colorful backlight on your Logitech Litra Beam LX device. The command will error if trying to decrease the brightness below 1%. By default, all Litra Beam LX devices are targeted, unless a specific device is specified with --serial-number or --device-path.
    BackBrightnessDown {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP,
            conflicts_with = "device_path"
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP,
            conflicts_with = "serial_number"
        )]
        device_path: Option<String>,
        #[clap(
            long,
            short('b'),
            help = "The number of percentage points to decrease the brightness by",
            value_parser = clap::value_parser!(u8).range(1..=100)
        )]
        percentage: u8,
    },
    /// List Logitech Litra devices connected to your computer
    Devices {
        #[clap(long, short, action, help = "Return the results in JSON format")]
        json: bool,
    },
    /// Start a MCP (Model Context Protocol) server for controlling Litra devices
    #[cfg(feature = "mcp")]
    Mcp,
}

fn percentage_within_range(percentage: u32, start_range: u32, end_range: u32) -> u32 {
    // Handle edge cases: 0% should return exactly start_range, 100% should return exactly end_range
    if percentage == 0 {
        return start_range;
    }
    if percentage == 100 {
        return end_range;
    }

    // For values between 0 and 100, use ceiling to ensure 1% is always > 0%
    // This fixes the bug where small percentages would round back to the minimum
    let range = end_range as f64 - start_range as f64;
    let result = (percentage as f64 / 100.0) * range + start_range as f64;
    result.ceil() as u32
}

fn get_is_on_text(is_on: bool) -> &'static str {
    if is_on {
        "On"
    } else {
        "Off"
    }
}

fn get_is_on_emoji(is_on: bool) -> &'static str {
    if is_on {
        "💡"
    } else {
        "🌑"
    }
}

fn get_is_back_on_emoji(is_on: bool) -> &'static str {
    if is_on {
        "🌈"
    } else {
        "🌑"
    }
}

fn check_device_filters<'a>(
    _context: &'a Litra,
    _serial_number: Option<&'a str>,
    device_path: Option<&'a str>,
    device_type: Option<&'a DeviceType>,
) -> impl Fn(&Device) -> bool + 'a {
    move |device| {
        // Check device path if specified
        if let Some(path) = device_path {
            return device.device_path() == path;
        }

        // Check device type if specified
        if let Some(expected_type) = device_type {
            if device.device_type() != *expected_type {
                return false;
            }
        }

        // If a serial number is specified, we'll filter by it after opening the device
        // since serial numbers are only accessible after opening
        true
    }
}

#[derive(Debug)]
enum CliError {
    DeviceError(DeviceError),
    SerializationFailed(serde_json::Error),
    DeviceNotFound,
    MCPError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::DeviceError(error) => error.fmt(f),
            CliError::SerializationFailed(error) => error.fmt(f),
            CliError::DeviceNotFound => write!(f, "Device not found."),
            CliError::MCPError(message) => write!(f, "MCP server error: {}", message),
        }
    }
}

impl From<DeviceError> for CliError {
    fn from(error: DeviceError) -> Self {
        CliError::DeviceError(error)
    }
}

type CliResult = Result<(), CliError>;

/// Get all devices matching the given filters
fn get_all_supported_devices(
    context: &Litra,
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
) -> Result<Vec<DeviceHandle>, CliError> {
    // Filter by various criteria
    let potential_devices: Vec<Device> = context
        .get_connected_devices()
        .filter(check_device_filters(
            context,
            serial_number,
            device_path,
            device_type,
        ))
        .collect();

    // If we need to filter by serial, open devices and check
    if let Some(serial) = serial_number {
        let mut handles = Vec::new();
        for device in potential_devices {
            if let Ok(handle) = device.open(context) {
                if let Ok(Some(actual_serial)) = handle.serial_number() {
                    if actual_serial == serial {
                        handles.push(handle);
                    }
                }
            }
        }
        Ok(handles)
    } else {
        // No serial filter, include all devices that matched the other filters
        Ok(potential_devices
            .into_iter()
            .filter_map(|dev| dev.open(context).ok())
            .collect())
    }
}

/// Apply a command to device(s)
fn with_device<F>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    callback: F,
) -> CliResult
where
    F: Fn(&DeviceHandle) -> DeviceResult<()>,
{
    let context = Litra::new()?;

    let devices = get_all_supported_devices(&context, serial_number, device_path, device_type)?;
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    for device_handle in devices {
        // Ignore device-specific errors (e.g. unsupported device type) but propagate
        // validation errors (e.g. invalid brightness) since those indicate user input errors
        if let Err(e) = callback(&device_handle) {
            if is_user_input_error(&e) {
                return Err(e.into());
            }
        }
    }
    Ok(())
}

/// Returns true if the error is caused by invalid user input and should be propagated
fn is_user_input_error(error: &DeviceError) -> bool {
    matches!(
        error,
        DeviceError::InvalidBrightness(_)
            | DeviceError::InvalidTemperature(_)
            | DeviceError::InvalidZone(_)
            | DeviceError::InvalidColor(_)
            | DeviceError::InvalidPercentage(_)
    )
}

#[cfg_attr(feature = "cli", derive(Tabled))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[derive(Serialize, Debug)]
pub struct DeviceInfo {
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub device_type: DeviceType,
    #[cfg_attr(feature = "cli", tabled(rename = "Type"))]
    pub device_type_display: String,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub has_back_side: bool,
    #[cfg_attr(feature = "cli", tabled(rename = "Serial Number"))]
    pub serial_number: String,
    #[cfg_attr(feature = "cli", tabled(rename = "Device Path"))]
    pub device_path: String,
    #[cfg_attr(feature = "cli", tabled(rename = "Status"))]
    pub status_display: String,
    #[cfg_attr(feature = "cli", tabled(rename = "Brightness (lm)"))]
    pub brightness_display: String,
    #[cfg_attr(feature = "cli", tabled(rename = "Temperature (K)"))]
    pub temperature_display: String,
    #[cfg_attr(feature = "cli", tabled(rename = "Back Status"))]
    pub back_status_display: String,
    #[cfg_attr(feature = "cli", tabled(rename = "Back Brightness (%)"))]
    pub back_brightness_display: String,
    // Keep original fields for JSON output
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub is_on: bool,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub brightness_in_lumen: u16,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub temperature_in_kelvin: u16,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub minimum_brightness_in_lumen: u16,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub maximum_brightness_in_lumen: u16,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub minimum_temperature_in_kelvin: u16,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub maximum_temperature_in_kelvin: u16,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub is_back_on: Option<bool>,
    #[cfg_attr(feature = "cli", tabled(skip))]
    pub back_brightness_percentage: Option<u8>,
}

fn get_connected_devices() -> Result<Vec<DeviceInfo>, CliError> {
    let context = Litra::new()?;

    let litra_devices: Vec<DeviceInfo> = context
        .get_connected_devices()
        .filter_map(|device| {
            let device_handle = match device.open(&context) {
                Ok(handle) => handle,
                Err(_e) => {
                    return None;
                }
            };

            // Get the device path
            let device_path = device.device_path();

            // Get serial number if available
            let serial = match device_handle.serial_number() {
                Ok(Some(s)) => s,
                Ok(None) => "UNKNOWN".to_string(),
                Err(_e) => "UNKNOWN".to_string(),
            };

            // Try to get attributes, log errors
            let is_on = match device_handle.is_on() {
                Ok(on) => on,
                Err(_e) => {
                    return None;
                }
            };

            let brightness = match device_handle.brightness_in_lumen() {
                Ok(b) => b,
                Err(_e) => {
                    return None;
                }
            };

            let temperature = match device_handle.temperature_in_kelvin() {
                Ok(t) => t,
                Err(_e) => {
                    return None;
                }
            };

            // Get back light status for Litra Beam LX devices
            let (
                is_back_on,
                back_brightness_percentage,
                back_status_display,
                back_brightness_display,
            ) = if device.device_type() == DeviceType::LitraBeamLX {
                let back_on = device_handle.is_back_on().ok();
                let back_brightness = device_handle.back_brightness_percentage().ok();
                let status_display = match back_on {
                    Some(on) => format!("{} {}", get_is_on_text(on), get_is_back_on_emoji(on)),
                    None => "Unknown".to_string(),
                };
                let brightness_display = match back_brightness {
                    Some(b) => format!("{}%", b),
                    None => "Unknown".to_string(),
                };
                (back_on, back_brightness, status_display, brightness_display)
            } else {
                (None, None, "N/A".to_string(), "N/A".to_string())
            };

            Some(DeviceInfo {
                device_type: device.device_type(),
                device_type_display: device.device_type().to_string(),
                has_back_side: device.device_type().has_back_side(),
                serial_number: serial,
                device_path,
                status_display: format!("{} {}", get_is_on_text(is_on), get_is_on_emoji(is_on)),
                brightness_display: format!(
                    "{}/{}",
                    brightness,
                    device_handle.maximum_brightness_in_lumen()
                ),
                temperature_display: format!(
                    "{}/{}",
                    temperature,
                    device_handle.maximum_temperature_in_kelvin()
                ),
                back_status_display,
                back_brightness_display,
                // Keep original fields for JSON output
                is_on,
                brightness_in_lumen: brightness,
                temperature_in_kelvin: temperature,
                minimum_brightness_in_lumen: device_handle.minimum_brightness_in_lumen(),
                maximum_brightness_in_lumen: device_handle.maximum_brightness_in_lumen(),
                minimum_temperature_in_kelvin: device_handle.minimum_temperature_in_kelvin(),
                maximum_temperature_in_kelvin: device_handle.maximum_temperature_in_kelvin(),
                is_back_on,
                back_brightness_percentage,
            })
        })
        .collect();
    Ok(litra_devices)
}

#[cfg(feature = "cli")]
fn handle_devices_command(json: bool) -> CliResult {
    let litra_devices = get_connected_devices()?;

    if json {
        println!(
            "{}",
            serde_json::to_string(&litra_devices).map_err(CliError::SerializationFailed)?
        );
        Ok(())
    } else {
        if litra_devices.is_empty() {
            println!("No Logitech Litra devices found");
        } else {
            let table = Table::new(&litra_devices);
            println!("{}", table);
        }

        Ok(())
    }
}

fn handle_on_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
) -> CliResult {
    with_device(serial_number, device_path, device_type, |device_handle| {
        device_handle.set_on(true)
    })
}

fn handle_off_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
) -> CliResult {
    with_device(serial_number, device_path, device_type, |device_handle| {
        device_handle.set_on(false)
    })
}

fn handle_toggle_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
) -> CliResult {
    // Get context to work with devices
    let context = Litra::new()?;

    // Get all matched devices
    let devices = get_all_supported_devices(&context, serial_number, device_path, device_type)?;
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    // Toggle each device individually
    for device_handle in devices {
        // Toggle each device individually, ignoring errors
        if let Ok(is_on) = device_handle.is_on() {
            let _ = device_handle.set_on(!is_on);
        }
    }
    Ok(())
}

/// Create a general purpose function to handle brightness setting
fn with_brightness_setting<F>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    brightness_fn: F,
) -> CliResult
where
    F: Fn(&DeviceHandle) -> Result<u16, DeviceError>,
{
    let context = Litra::new()?;

    // Get all matched devices
    let devices = get_all_supported_devices(&context, serial_number, device_path, device_type)?;
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    for device_handle in devices {
        if let Ok(brightness) = brightness_fn(&device_handle) {
            let _ = device_handle.set_brightness_in_lumen(brightness);
        }
    }
    Ok(())
}

fn handle_brightness_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    match (value, percentage) {
        (Some(brightness), None) => {
            with_device(serial_number, device_path, device_type, |device_handle| {
                device_handle.set_brightness_in_lumen(brightness)
            })
        }
        (None, Some(pct)) => {
            with_brightness_setting(serial_number, device_path, device_type, |device_handle| {
                let brightness_in_lumen = percentage_within_range(
                    pct.into(),
                    device_handle.minimum_brightness_in_lumen().into(),
                    device_handle.maximum_brightness_in_lumen().into(),
                );

                // Convert to u16, handling any potential conversion errors
                // DeviceError doesn't have a constructor for this error type,
                // so we'll use InvalidBrightness as the closest match
                brightness_in_lumen
                    .try_into()
                    .map_err(|_| DeviceError::InvalidBrightness(0))
            })
        }
        _ => unreachable!(),
    }
}

fn handle_brightness_up_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    match (value, percentage) {
        (Some(brightness_to_add), None) => {
            with_brightness_setting(serial_number, device_path, device_type, |device_handle| {
                let current_brightness = device_handle.brightness_in_lumen()?;
                let new_brightness = current_brightness + brightness_to_add;
                Ok(new_brightness)
            })
        }
        (None, Some(pct)) => {
            with_brightness_setting(serial_number, device_path, device_type, |device_handle| {
                let current_brightness = device_handle.brightness_in_lumen()?;
                let brightness_to_add = percentage_within_range(
                    pct.into(),
                    device_handle.minimum_brightness_in_lumen().into(),
                    device_handle.maximum_brightness_in_lumen().into(),
                ) as u16
                    - device_handle.minimum_brightness_in_lumen();

                let new_brightness = current_brightness + brightness_to_add;
                Ok(new_brightness)
            })
        }
        _ => unreachable!(),
    }
}

fn handle_brightness_down_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    match (value, percentage) {
        (Some(brightness_to_subtract), None) => {
            with_brightness_setting(serial_number, device_path, device_type, |device_handle| {
                let current_brightness = device_handle.brightness_in_lumen()?;

                if current_brightness <= brightness_to_subtract {
                    // Skip this device by returning an error which will be ignored
                    return Err(DeviceError::InvalidBrightness(0));
                }

                let new_brightness = current_brightness - brightness_to_subtract;
                Ok(new_brightness)
            })
        }
        (None, Some(pct)) => {
            with_brightness_setting(serial_number, device_path, device_type, |device_handle| {
                let current_brightness = device_handle.brightness_in_lumen()?;

                let brightness_to_subtract = percentage_within_range(
                    pct.into(),
                    device_handle.minimum_brightness_in_lumen().into(),
                    device_handle.maximum_brightness_in_lumen().into(),
                ) as u16
                    - device_handle.minimum_brightness_in_lumen();

                let new_brightness = current_brightness as i16 - brightness_to_subtract as i16;

                if new_brightness <= 0 {
                    // Skip this device by returning an error which will be ignored
                    return Err(DeviceError::InvalidBrightness(0));
                }

                Ok(new_brightness as u16)
            })
        }
        _ => unreachable!(),
    }
}

fn handle_temperature_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    value: u16,
) -> CliResult {
    with_device(serial_number, device_path, device_type, |device_handle| {
        device_handle.set_temperature_in_kelvin(value)
    })
}

fn handle_temperature_up_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    value: u16,
) -> CliResult {
    with_device(serial_number, device_path, device_type, |device_handle| {
        let current_temperature = device_handle.temperature_in_kelvin()?;
        let new_temperature = current_temperature + value;

        // Check if new temperature would exceed maximum
        if new_temperature > device_handle.maximum_temperature_in_kelvin() {
            return Err(DeviceError::InvalidTemperature(new_temperature));
        }

        device_handle.set_temperature_in_kelvin(new_temperature)
    })
}

fn handle_temperature_down_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    value: u16,
) -> CliResult {
    with_device(serial_number, device_path, device_type, |device_handle| {
        let current_temperature = device_handle.temperature_in_kelvin()?;

        // Check if new temperature would be below minimum
        if current_temperature <= value {
            // Skip this device by returning an error which will be ignored
            return Err(DeviceError::InvalidTemperature(0));
        }

        let new_temperature = current_temperature - value;
        device_handle.set_temperature_in_kelvin(new_temperature)
    })
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), String> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err("Hex color must be exactly 6 characters long".into());
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| "Failed to parse red component from hex color")?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| "Failed to parse green component from hex color")?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| "Failed to parse blue component from hex color")?;

    Ok((r, g, b))
}

fn handle_back_color_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    hex: &str,
    zone_id: Option<u8>,
) -> CliResult {
    with_device(
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
        |device_handle| match hex_to_rgb(hex) {
            Ok((r, g, b)) => match zone_id {
                None => {
                    for i in 1..=7 {
                        device_handle.set_back_color(i, r, g, b)?;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Ok(())
                }
                Some(id) => {
                    device_handle.set_back_color(id, r, g, b)?;
                    Ok(())
                }
            },
            Err(error) => Err(DeviceError::InvalidColor(error)),
        },
    )
}

fn handle_back_brightness_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    brightness: u8,
) -> CliResult {
    with_device(
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
        |device_handle| device_handle.set_back_brightness_percentage(brightness),
    )
}

fn handle_back_off_command(serial_number: Option<&str>, device_path: Option<&str>) -> CliResult {
    with_device(
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
        |device_handle| device_handle.set_back_on(false),
    )
}

fn handle_back_on_command(serial_number: Option<&str>, device_path: Option<&str>) -> CliResult {
    with_device(
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
        |device_handle| device_handle.set_back_on(true),
    )
}

fn handle_back_toggle_command(serial_number: Option<&str>, device_path: Option<&str>) -> CliResult {
    // Get context to work with devices
    let context = Litra::new()?;

    // Get all matched devices (only Litra Beam LX supports back light)
    let devices = get_all_supported_devices(
        &context,
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
    )?;
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    // Toggle each device individually
    for device_handle in devices {
        // Toggle each device individually, ignoring errors
        if let Ok(is_on) = device_handle.is_back_on() {
            let _ = device_handle.set_back_on(!is_on);
        }
    }
    Ok(())
}

fn handle_back_brightness_up_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    // Get context to work with devices
    let context = Litra::new()?;

    // Get all matched devices (only Litra Beam LX supports back light)
    let devices = get_all_supported_devices(
        &context,
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
    )?;
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    for device_handle in devices {
        if let Ok(current_brightness) = device_handle.back_brightness_percentage() {
            let new_brightness = current_brightness.saturating_add(percentage).min(100);
            let _ = device_handle.set_back_brightness_percentage(new_brightness);
        }
    }
    Ok(())
}

fn handle_back_brightness_down_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    // Get context to work with devices
    let context = Litra::new()?;

    // Get all matched devices (only Litra Beam LX supports back light)
    let devices = get_all_supported_devices(
        &context,
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
    )?;
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    for device_handle in devices {
        if let Ok(current_brightness) = device_handle.back_brightness_percentage() {
            let new_brightness = current_brightness.saturating_sub(percentage).max(1);
            let _ = device_handle.set_back_brightness_percentage(new_brightness);
        }
    }
    Ok(())
}

#[cfg(feature = "mcp")]
fn handle_mcp_command() -> CliResult {
    mcp::handle_mcp_command()
}

/// The current version of the CLI, extracted from Cargo.toml
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub API URL for fetching releases (list endpoint, limited to 10 most recent)
const GITHUB_API_URL: &str = "https://api.github.com/repos/timrogers/litra-rs/releases?per_page=10";

/// Timeout for update check requests in seconds
const UPDATE_CHECK_TIMEOUT_SECS: u64 = 2;

/// Response structure for GitHub releases API
#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    published_at: String,
    draft: bool,
    prerelease: bool,
}

/// Configuration file name
const CONFIG_FILE_NAME: &str = ".litra.toml";

/// Number of seconds in a day (24 hours)
const SECONDS_PER_DAY: u64 = 86400;

/// Configuration structure for litra.toml
#[derive(serde::Deserialize, serde::Serialize, Default)]
struct Config {
    #[serde(default)]
    update_check: UpdateCheckConfig,
}

/// Update check configuration
#[derive(serde::Deserialize, serde::Serialize, Default)]
struct UpdateCheckConfig {
    /// Unix timestamp of the last update check
    last_check_timestamp: Option<u64>,
}

/// Returns the path to the litra.toml config file in the user's home directory
#[cfg(feature = "cli")]
fn get_config_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| home.join(CONFIG_FILE_NAME))
}

/// Reads the config file, returning a default config if the file doesn't exist or can't be read
#[cfg(feature = "cli")]
fn read_config() -> Config {
    let Some(config_path) = get_config_path() else {
        return Config::default();
    };

    match std::fs::read_to_string(&config_path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

/// Writes the config to the config file, silently ignoring errors
#[cfg(feature = "cli")]
fn write_config(config: &Config) {
    let Some(config_path) = get_config_path() else {
        return;
    };

    if let Ok(contents) = toml::to_string_pretty(config) {
        let _ = std::fs::write(&config_path, contents);
    }
}

/// Returns the current Unix timestamp in seconds
#[cfg(feature = "cli")]
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Checks if enough time has passed since the last update check (at least one day)
#[cfg(feature = "cli")]
fn should_check_for_updates(config: &Config) -> bool {
    let Some(last_check) = config.update_check.last_check_timestamp else {
        return true; // Never checked before
    };

    let now = current_timestamp();
    now.saturating_sub(last_check) >= SECONDS_PER_DAY
}

/// Checks if a release is old enough to be considered for update notifications (at least 72 hours)
/// Uses chrono for ISO 8601 parsing and comparison
#[cfg(feature = "cli")]
fn is_release_old_enough(published_at: &str) -> bool {
    use chrono::{DateTime, Duration, Utc};

    // Parse the release timestamp
    let Ok(release_time) = DateTime::parse_from_rfc3339(published_at) else {
        return false; // If we can't parse the timestamp, skip this release
    };

    // Calculate the cutoff time (72 hours ago)
    let cutoff = Utc::now() - Duration::hours(72);

    // Check if the release is older than the cutoff
    release_time < cutoff
}

/// Environment variable to disable update checks
const DISABLE_UPDATE_CHECK_ENV: &str = "LITRA_DISABLE_UPDATE_CHECK";

/// Checks for updates by fetching releases from GitHub.
/// Returns the latest version tag if a newer version is available, None otherwise.
/// This function will timeout after 2 seconds and log a warning, but will not
/// disrupt the CLI's normal operation.
/// Set the LITRA_DISABLE_UPDATE_CHECK environment variable to any value to disable this check.
/// The check is performed at most once per day, with the last check time stored in ~/.litra.toml.
/// Only releases that are at least 72 hours old are considered.
#[cfg(feature = "cli")]
fn check_for_updates() -> Option<String> {
    // Check if update check is disabled via environment variable
    if std::env::var(DISABLE_UPDATE_CHECK_ENV).is_ok() {
        return None;
    }

    // Read config and check if we should perform the update check
    let mut config = read_config();
    if !should_check_for_updates(&config) {
        return None;
    }

    // Update the last check timestamp regardless of the result
    config.update_check.last_check_timestamp = Some(current_timestamp());
    write_config(&config);

    let timeout = Duration::from_secs(UPDATE_CHECK_TIMEOUT_SECS);

    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .build(),
    );

    let mut response = match agent
        .get(GITHUB_API_URL)
        .header("User-Agent", format!("litra-rs/{}", CURRENT_VERSION))
        .header("Accept", "application/vnd.github.v3+json")
        .call()
    {
        Ok(response) => response,
        Err(e) => {
            if let ureq::Error::Timeout(_) = e {
                eprintln!(
                    "Warning: Update check timed out after {} seconds",
                    UPDATE_CHECK_TIMEOUT_SECS
                );
            }
            // Silently ignore other errors to not disrupt CLI operation
            return None;
        }
    };

    let releases: Vec<GitHubRelease> = match response.body_mut().read_json() {
        Ok(releases) => releases,
        Err(_) => return None,
    };

    find_best_update_version(&releases, CURRENT_VERSION)
}

/// Finds the best (highest) version from a list of releases that is newer than the current version.
/// Filters out drafts, pre-releases, and releases that are less than 72 hours old.
fn find_best_update_version(releases: &[GitHubRelease], current_version: &str) -> Option<String> {
    let mut best_version: Option<String> = None;

    for release in releases {
        // Skip draft and pre-release versions
        if release.draft || release.prerelease {
            continue;
        }

        // Skip releases that are too new (less than 72 hours old)
        if !is_release_old_enough(&release.published_at) {
            continue;
        }

        // Extract version from tag_name (e.g., "v3.2.0" -> "3.2.0")
        let release_version = release.tag_name.trim_start_matches('v');

        // Check if this release is newer than the current version
        if is_newer_version(release_version, current_version) {
            // Check if this is better than our current best
            match &best_version {
                None => best_version = Some(release.tag_name.clone()),
                Some(current_best) => {
                    let current_best_version = current_best.trim_start_matches('v');
                    if is_newer_version(release_version, current_best_version) {
                        best_version = Some(release.tag_name.clone());
                    }
                }
            }
        }
    }

    best_version
}

/// Compares two semantic version strings to determine if `latest` is newer than `current`.
/// Returns true if `latest` is a newer version.
fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() >= 3 {
            Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            ))
        } else if parts.len() == 2 {
            Some((parts[0].parse().ok()?, parts[1].parse().ok()?, 0))
        } else if parts.len() == 1 {
            Some((parts[0].parse().ok()?, 0, 0))
        } else {
            None
        }
    };

    match (parse_version(latest), parse_version(current)) {
        (Some((l_major, l_minor, l_patch)), Some((c_major, c_minor, c_patch))) => {
            (l_major, l_minor, l_patch) > (c_major, c_minor, c_patch)
        }
        _ => false,
    }
}

/// Generates the update notification message for the given version
#[cfg(feature = "cli")]
fn format_update_message(latest_version: &str) -> String {
    format!(
        "A new version of litra is available: {} (current: v{})\n\
         If you installed Litra from Homebrew, you can upgrade by running `brew upgrade litra`\n\
         Otherwise, you can download the latest release at  https://github.com/timrogers/litra-rs/releases/tag/{}",
        latest_version, CURRENT_VERSION, latest_version
    )
    .green()
    .to_string()
}

#[cfg(feature = "cli")]
fn main() -> ExitCode {
    let args = Cli::parse();

    // Check for updates after parsing args so --help/--version aren't delayed
    let update_message = check_for_updates().map(|v| format_update_message(&v));

    let result = match &args.command {
        Commands::Devices { json } => handle_devices_command(*json),
        Commands::On {
            serial_number,
            device_path,
            device_type,
        } => handle_on_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
        ),
        Commands::Off {
            serial_number,
            device_path,
            device_type,
        } => handle_off_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
        ),
        Commands::Toggle {
            serial_number,
            device_path,
            device_type,
        } => handle_toggle_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
        ),
        Commands::Brightness {
            serial_number,
            device_path,
            device_type,
            value,
            percentage,
        } => handle_brightness_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
            *value,
            *percentage,
        ),
        Commands::BrightnessUp {
            serial_number,
            device_path,
            device_type,
            value,
            percentage,
        } => handle_brightness_up_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
            *value,
            *percentage,
        ),
        Commands::BrightnessDown {
            serial_number,
            device_path,
            device_type,
            value,
            percentage,
        } => handle_brightness_down_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
            *value,
            *percentage,
        ),
        Commands::Temperature {
            serial_number,
            device_path,
            device_type,
            value,
        } => handle_temperature_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
            *value,
        ),
        Commands::TemperatureUp {
            serial_number,
            device_path,
            device_type,
            value,
        } => handle_temperature_up_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
            *value,
        ),
        Commands::TemperatureDown {
            serial_number,
            device_path,
            device_type,
            value,
        } => handle_temperature_down_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            device_type.as_ref(),
            *value,
        ),
        Commands::BackColor {
            serial_number,
            device_path,
            value,
            color,
            zone: zone_id,
        } => {
            let hex = match (value, color) {
                (Some(v), None) => v.clone(),
                (None, Some(c)) => c.to_hex().to_string(),
                _ => unreachable!("clap ensures exactly one of value or color is provided"),
            };
            handle_back_color_command(
                serial_number.as_deref(),
                device_path.as_deref(),
                &hex,
                *zone_id,
            )
        }
        Commands::BackBrightness {
            serial_number,
            device_path,
            percentage,
        } => handle_back_brightness_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            *percentage,
        ),
        Commands::BackOff {
            serial_number,
            device_path,
        } => handle_back_off_command(serial_number.as_deref(), device_path.as_deref()),
        Commands::BackOn {
            serial_number,
            device_path,
        } => handle_back_on_command(serial_number.as_deref(), device_path.as_deref()),
        Commands::BackToggle {
            serial_number,
            device_path,
        } => handle_back_toggle_command(serial_number.as_deref(), device_path.as_deref()),
        Commands::BackBrightnessUp {
            serial_number,
            device_path,
            percentage,
        } => handle_back_brightness_up_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            *percentage,
        ),
        Commands::BackBrightnessDown {
            serial_number,
            device_path,
            percentage,
        } => handle_back_brightness_down_command(
            serial_number.as_deref(),
            device_path.as_deref(),
            *percentage,
        ),
        #[cfg(feature = "mcp")]
        Commands::Mcp => handle_mcp_command(),
    };

    let exit_code = if let Err(error) = result {
        eprintln!("{}", error);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    };

    // Print update notification after command output so it doesn't delay or obscure results
    if let Some(message) = update_message {
        eprintln!("{}", message);
    }

    exit_code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_within_range_zero_percent() {
        // 0% should return exactly the minimum value
        assert_eq!(percentage_within_range(0, 20, 250), 20);
        assert_eq!(percentage_within_range(0, 30, 400), 30);
        assert_eq!(percentage_within_range(0, 100, 200), 100);
    }

    #[test]
    fn test_percentage_within_range_hundred_percent() {
        // 100% should return exactly the maximum value
        assert_eq!(percentage_within_range(100, 20, 250), 250);
        assert_eq!(percentage_within_range(100, 30, 400), 400);
        assert_eq!(percentage_within_range(100, 100, 200), 200);
    }

    #[test]
    fn test_percentage_within_range_one_percent_greater_than_zero() {
        // 1% should return a value greater than 0%
        // This is the key bug fix: 1% should never equal the minimum
        let zero_pct = percentage_within_range(0, 20, 250);
        let one_pct = percentage_within_range(1, 20, 250);
        assert!(
            one_pct > zero_pct,
            "1% ({}) should be greater than 0% ({})",
            one_pct,
            zero_pct
        );

        // Test with Litra Beam range
        let zero_pct_beam = percentage_within_range(0, 30, 400);
        let one_pct_beam = percentage_within_range(1, 30, 400);
        assert!(
            one_pct_beam > zero_pct_beam,
            "1% ({}) should be greater than 0% ({})",
            one_pct_beam,
            zero_pct_beam
        );

        // Test with a small range where the bug is most apparent
        let zero_pct_small = percentage_within_range(0, 20, 30);
        let one_pct_small = percentage_within_range(1, 20, 30);
        assert!(
            one_pct_small > zero_pct_small,
            "1% ({}) should be greater than 0% ({}) even with small range",
            one_pct_small,
            zero_pct_small
        );
    }

    #[test]
    fn test_percentage_within_range_midpoint() {
        // 50% should return approximately the midpoint
        // For range 20-250: midpoint is 135
        let result = percentage_within_range(50, 20, 250);
        assert_eq!(result, 135);

        // For range 30-400: midpoint is 215
        let result = percentage_within_range(50, 30, 400);
        assert_eq!(result, 215);
    }

    #[test]
    fn test_percentage_within_range_litra_glow() {
        // Test with Litra Glow's actual brightness range (20-250lm)
        assert_eq!(percentage_within_range(0, 20, 250), 20);
        assert!(percentage_within_range(1, 20, 250) > 20);
        assert_eq!(percentage_within_range(100, 20, 250), 250);

        // Verify that small percentages are distinguishable
        let one = percentage_within_range(1, 20, 250);
        let two = percentage_within_range(2, 20, 250);
        let three = percentage_within_range(3, 20, 250);

        // Each should be at least the minimum
        assert!(one >= 20);
        assert!(two >= 20);
        assert!(three >= 20);

        // And 1% should be greater than 0%
        assert!(one > 20);
    }

    #[test]
    fn test_percentage_within_range_litra_beam() {
        // Test with Litra Beam's actual brightness range (30-400lm)
        assert_eq!(percentage_within_range(0, 30, 400), 30);
        assert!(percentage_within_range(1, 30, 400) > 30);
        assert_eq!(percentage_within_range(100, 30, 400), 400);

        // Verify that small percentages are distinguishable
        let one = percentage_within_range(1, 30, 400);
        let two = percentage_within_range(2, 30, 400);

        // Each should be greater than minimum
        assert!(one > 30);
        assert!(two > 30);
    }

    #[test]
    fn test_percentage_within_range_small_range() {
        // Test with a small range where rounding issues are most apparent
        // This is the case that exposes the original bug
        let range_start = 20;
        let range_end = 30;

        assert_eq!(
            percentage_within_range(0, range_start, range_end),
            range_start
        );
        assert!(
            percentage_within_range(1, range_start, range_end) > range_start,
            "1% should be greater than start even with small range"
        );
        assert!(
            percentage_within_range(5, range_start, range_end) > range_start,
            "5% should be greater than start even with small range"
        );
        assert_eq!(
            percentage_within_range(100, range_start, range_end),
            range_end
        );
    }

    #[test]
    fn test_percentage_within_range_monotonic() {
        // Verify that the function is monotonically increasing
        // (higher percentages should never give lower values)
        let range_start = 20;
        let range_end = 250;

        let mut prev_value = percentage_within_range(0, range_start, range_end);
        for pct in 1..=100 {
            let current_value = percentage_within_range(pct, range_start, range_end);
            assert!(
                current_value >= prev_value,
                "{}% ({}) should be >= {}% ({})",
                pct,
                current_value,
                pct - 1,
                prev_value
            );
            prev_value = current_value;
        }
    }

    #[test]
    fn test_is_newer_version_major() {
        assert!(is_newer_version("4.0.0", "3.2.0"));
        assert!(is_newer_version("2.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "2.0.0"));
        assert!(!is_newer_version("3.0.0", "4.0.0"));
    }

    #[test]
    fn test_is_newer_version_minor() {
        assert!(is_newer_version("3.3.0", "3.2.0"));
        assert!(is_newer_version("1.2.0", "1.1.0"));
        assert!(!is_newer_version("1.1.0", "1.2.0"));
        assert!(!is_newer_version("3.2.0", "3.3.0"));
    }

    #[test]
    fn test_is_newer_version_patch() {
        assert!(is_newer_version("3.2.1", "3.2.0"));
        assert!(is_newer_version("1.0.5", "1.0.4"));
        assert!(!is_newer_version("1.0.4", "1.0.5"));
        assert!(!is_newer_version("3.2.0", "3.2.1"));
    }

    #[test]
    fn test_is_newer_version_same_version() {
        assert!(!is_newer_version("3.2.0", "3.2.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_newer_version_edge_cases() {
        // Two-part version
        assert!(is_newer_version("3.3", "3.2"));
        assert!(!is_newer_version("3.2", "3.3"));
        // One-part version
        assert!(is_newer_version("4", "3"));
        assert!(!is_newer_version("3", "4"));
        // Invalid version format
        assert!(!is_newer_version("invalid", "3.2.0"));
        assert!(!is_newer_version("3.2.0", "invalid"));
        assert!(!is_newer_version("", "3.2.0"));
    }

    #[test]
    fn test_should_check_for_updates_never_checked() {
        // Should check if never checked before (no timestamp)
        let config = Config::default();
        assert!(should_check_for_updates(&config));
    }

    #[test]
    fn test_should_check_for_updates_checked_recently() {
        // Should not check if checked less than a day ago
        let mut config = Config::default();
        config.update_check.last_check_timestamp = Some(current_timestamp());
        assert!(!should_check_for_updates(&config));
    }

    #[test]
    fn test_should_check_for_updates_checked_long_ago() {
        // Should check if checked more than a day ago
        let mut config = Config::default();
        // Set timestamp to more than a day ago
        config.update_check.last_check_timestamp = Some(current_timestamp() - SECONDS_PER_DAY - 1);
        assert!(should_check_for_updates(&config));
    }

    #[test]
    fn test_should_check_for_updates_exactly_one_day() {
        // Should check if exactly one day has passed
        let mut config = Config::default();
        config.update_check.last_check_timestamp = Some(current_timestamp() - SECONDS_PER_DAY);
        assert!(should_check_for_updates(&config));
    }

    #[test]
    fn test_is_release_old_enough() {
        // A release from far in the past should be old enough
        assert!(is_release_old_enough("2020-01-01T00:00:00Z"));

        // A release from far in the future should not be old enough
        assert!(!is_release_old_enough("2099-01-01T00:00:00Z"));

        // An invalid timestamp should not be old enough (returns false)
        assert!(!is_release_old_enough("invalid"));
    }

    #[test]
    fn test_format_update_message() {
        let message = format_update_message("v3.3.0");
        assert!(message.contains("v3.3.0"));
        assert!(message.contains(CURRENT_VERSION));
        assert!(message.contains("https://github.com/timrogers/litra-rs/releases/tag/v3.3.0"));
    }

    /// Helper to create a GitHubRelease for testing
    fn make_release(tag: &str, published_at: &str, draft: bool, prerelease: bool) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag.to_string(),
            published_at: published_at.to_string(),
            draft,
            prerelease,
        }
    }

    #[test]
    fn test_find_best_update_version_skips_drafts() {
        let releases = vec![make_release("v99.0.0", "2020-01-01T00:00:00Z", true, false)];
        assert_eq!(find_best_update_version(&releases, "1.0.0"), None);
    }

    #[test]
    fn test_find_best_update_version_skips_prereleases() {
        let releases = vec![make_release("v99.0.0", "2020-01-01T00:00:00Z", false, true)];
        assert_eq!(find_best_update_version(&releases, "1.0.0"), None);
    }

    #[test]
    fn test_find_best_update_version_returns_newest_stable() {
        let releases = vec![
            make_release("v5.0.0", "2020-01-01T00:00:00Z", false, true), // prerelease
            make_release("v4.0.0", "2020-01-01T00:00:00Z", false, false), // stable
            make_release("v3.5.0", "2020-01-01T00:00:00Z", true, false), // draft
            make_release("v3.1.0", "2020-01-01T00:00:00Z", false, false), // stable but lower
        ];
        assert_eq!(
            find_best_update_version(&releases, "3.0.0"),
            Some("v4.0.0".to_string())
        );
    }

    #[test]
    fn test_find_best_update_version_skips_too_new_releases() {
        let releases = vec![make_release(
            "v99.0.0",
            "2099-01-01T00:00:00Z",
            false,
            false,
        )];
        assert_eq!(find_best_update_version(&releases, "1.0.0"), None);
    }

    #[test]
    fn test_find_best_update_version_no_newer_version() {
        let releases = vec![make_release("v1.0.0", "2020-01-01T00:00:00Z", false, false)];
        assert_eq!(find_best_update_version(&releases, "2.0.0"), None);
    }

    #[test]
    fn test_find_best_update_version_empty_releases() {
        let releases: Vec<GitHubRelease> = vec![];
        assert_eq!(find_best_update_version(&releases, "1.0.0"), None);
    }

    #[test]
    fn test_find_best_update_version_picks_highest_among_multiple() {
        let releases = vec![
            make_release("v4.2.0", "2020-01-01T00:00:00Z", false, false),
            make_release("v4.0.0", "2020-01-01T00:00:00Z", false, false),
            make_release("v4.1.0", "2020-01-01T00:00:00Z", false, false),
        ];
        assert_eq!(
            find_best_update_version(&releases, "3.0.0"),
            Some("v4.2.0".to_string())
        );
    }
}
