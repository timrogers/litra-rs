use clap::{builder::TypedValueParser, ArgGroup, Parser, Subcommand};
use litra::{Device, DeviceError, DeviceHandle, DeviceResult, DeviceType, Litra};
use serde::Serialize;
use std::fmt;
use std::process::ExitCode;
use std::str::FromStr;

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
#[clap(name = "litra", version)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

const SERIAL_NUMBER_ARGUMENT_HELP: &str = "Specify the device to target by its serial number";
const DEVICE_PATH_ARGUMENT_HELP: &str =
    "Specify the device to target by its path (useful for devices that don't show a serial number)";
const DEVICE_TYPE_ARGUMENT_HELP: &str =
    "Specify the device to target by its type (`glow`, `beam` or `beam_lx`)";

#[cfg(feature = "cli")]
#[derive(Debug, Subcommand)]
enum Commands {
    /// Turn your Logitech Litra device on. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    On {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
        device_type: Option<DeviceType>,
    },
    /// Turn your Logitech Litra device off. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    Off {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
        device_type: Option<DeviceType>,
    },
    /// Toggles your Logitech Litra device on or off. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    Toggle {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
        device_type: Option<DeviceType>,
    },
    /// Sets the brightness of your Logitech Litra device. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    #[clap(group = ArgGroup::new("brightness").required(true).multiple(false))]
    Brightness {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
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
            short,
            help = "The brightness to set, as a percentage of the maximum brightness",
            group = "brightness"
        )]
        percentage: Option<u8>,
    },
    /// Increases the brightness of your Logitech Litra device. The command will error if trying to increase the brightness beyond the device's maximum. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    #[clap(group = ArgGroup::new("brightness-up").required(true).multiple(false))]
    BrightnessUp {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
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
            group = "brightness-up"
        )]
        percentage: Option<u8>,
    },
    /// Decreases the brightness of your Logitech Litra device. The command will error if trying to decrease the brightness below the device's minimum. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    #[clap(group = ArgGroup::new("brightness-down").required(true).multiple(false))]
    BrightnessDown {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
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
            group = "brightness-down"
        )]
        percentage: Option<u8>,
    },
    /// Sets the temperature of your Logitech Litra device. By default, all devices are targeted, unless one or more devices are specified with --device-type, --serial-number or --device-path.
    Temperature {
        #[clap(
            long,
            short,
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
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
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
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
            help = SERIAL_NUMBER_ARGUMENT_HELP
        )]
        serial_number: Option<String>,
        #[clap(
            long,
            short('p'),
            help = DEVICE_PATH_ARGUMENT_HELP
        )]
        device_path: Option<String>,
        #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser)]
        device_type: Option<DeviceType>,
        #[clap(
            long,
            short,
            help = "The amount to decrease the temperature by, measured in Kelvin. This must be a multiple of 100."
        )]
        value: u16,
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
    let range = end_range as f64 - start_range as f64;
    let result = (percentage as f64 / 100.0) * range + start_range as f64;
    result.round() as u32
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
    MultipleFilterSpecified,
    MCPError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::DeviceError(error) => error.fmt(f),
            CliError::SerializationFailed(error) => error.fmt(f),
            CliError::DeviceNotFound => write!(f, "Device not found."),
            CliError::MultipleFilterSpecified => write!(f, "Only one filter (--serial-number, --device-path, or --device-type) can be specified at a time."),
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

/// Validates that only one filter is specified
fn validate_single_filter(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
) -> Result<(), CliError> {
    let filter_count = serial_number.is_some() as usize
        + device_path.is_some() as usize
        + device_type.is_some() as usize;

    if filter_count > 1 {
        Err(CliError::MultipleFilterSpecified)
    } else {
        Ok(())
    }
}

/// Get all devices matching the given filters
fn get_all_supported_devices(
    context: &Litra,
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
) -> Result<Vec<DeviceHandle>, CliError> {
    // Validate that only one filter is used
    validate_single_filter(serial_number, device_path, device_type)?;

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

    // Default to all matching devices or explicit filter
    let use_all = serial_number.is_none() && device_path.is_none() && device_type.is_none();

    if use_all {
        // Get all devices
        let devices = get_all_supported_devices(&context, None, None, None)?;
        if devices.is_empty() {
            return Err(CliError::DeviceNotFound);
        }

        for device_handle in devices {
            // Ignore errors for individual devices when targeting all
            let _ = callback(&device_handle);
        }
        Ok(())
    } else {
        // Filtering by one of the options
        let devices = get_all_supported_devices(&context, serial_number, device_path, device_type)?;
        if devices.is_empty() {
            return Err(CliError::DeviceNotFound);
        }

        // Apply to all matched devices
        for device_handle in devices {
            // Ignore errors for individual devices
            let _ = callback(&device_handle);
        }
        Ok(())
    }
}

#[derive(Serialize, Debug)]
struct DeviceInfo {
    pub serial_number: String,
    pub device_path: String,
    pub device_type: String,
    pub is_on: bool,
    pub brightness_in_lumen: u16,
    pub temperature_in_kelvin: u16,
    pub minimum_brightness_in_lumen: u16,
    pub maximum_brightness_in_lumen: u16,
    pub minimum_temperature_in_kelvin: u16,
    pub maximum_temperature_in_kelvin: u16,
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

            Some(DeviceInfo {
                serial_number: serial,
                device_path,
                device_type: device.device_type().to_string(),
                is_on,
                brightness_in_lumen: brightness,
                temperature_in_kelvin: temperature,
                minimum_brightness_in_lumen: device_handle.minimum_brightness_in_lumen(),
                maximum_brightness_in_lumen: device_handle.maximum_brightness_in_lumen(),
                minimum_temperature_in_kelvin: device_handle.minimum_temperature_in_kelvin(),
                maximum_temperature_in_kelvin: device_handle.maximum_temperature_in_kelvin(),
            })
        })
        .collect();
    Ok(litra_devices)
}

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
            for device_info in &litra_devices {
                println!(
                    "- {} ({}): {} {}",
                    device_info.device_type,
                    device_info.serial_number,
                    get_is_on_text(device_info.is_on),
                    get_is_on_emoji(device_info.is_on)
                );
                println!("  - Device path: {}", device_info.device_path);
                println!("  - Brightness: {} lm", device_info.brightness_in_lumen);
                println!(
                    "    - Minimum: {} lm",
                    device_info.minimum_brightness_in_lumen
                );
                println!(
                    "    - Maximum: {} lm",
                    device_info.maximum_brightness_in_lumen
                );
                println!("  - Temperature: {} K", device_info.temperature_in_kelvin);
                println!(
                    "    - Minimum: {} K",
                    device_info.minimum_temperature_in_kelvin
                );
                println!(
                    "    - Maximum: {} K",
                    device_info.maximum_temperature_in_kelvin
                );
            }
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

#[cfg(feature = "mcp")]
fn handle_mcp_command() -> CliResult {
    mcp::handle_mcp_command()
}

#[cfg(feature = "cli")]
fn main() -> ExitCode {
    let args = Cli::parse();

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
        #[cfg(feature = "mcp")]
        Commands::Mcp => handle_mcp_command(),
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
