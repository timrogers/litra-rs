use litra::{Device, DeviceError, DeviceHandle, Litra};
use serde::Serialize;
use std::fmt;
use std::num::TryFromIntError;
use std::process::ExitCode;

#[cfg(feature = "cli")]
use clap::{ArgGroup, Parser, Subcommand};

#[cfg(feature = "mcp")]
mod mcp;

/// Control your USB-connected Logitech Litra lights from the command line
#[cfg(feature = "cli")]
#[derive(Debug, Parser)]
#[clap(name = "litra", version)]
struct Cli {
    // Test
    #[clap(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Debug, Subcommand)]
enum Commands {
    /// Turn your Logitech Litra device on
    On {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
    },
    /// Turn your Logitech Litra device off
    Off {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
    },
    /// Toggles your Logitech Litra device on or off
    Toggle {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
    },
    /// Sets the brightness of your Logitech Litra device
    #[clap(group = ArgGroup::new("brightness").required(true).multiple(false))]
    Brightness {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
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
    /// Increases the brightness of your Logitech Litra device. The command will error if trying to increase the brightness beyond the device's maximum.
    #[clap(group = ArgGroup::new("brightness-up").required(true).multiple(false))]
    BrightnessUp {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
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
    /// Decreases the brightness of your Logitech Litra device. The command will error if trying to decrease the brightness below the device's minimum.
    #[clap(group = ArgGroup::new("brightness-down").required(true).multiple(false))]
    BrightnessDown {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
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
    /// Sets the temperature of your Logitech Litra device
    Temperature {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
        #[clap(
            long,
            short,
            help = "The temperature to set, measured in Kelvin. This can be set to any multiple of 100 between the minimum and maximum for the device returned by the `devices` command."
        )]
        value: u16,
    },
    /// Increases the temperature of your Logitech Litra device. The command will error if trying to increase the temperature beyond the device's maximum.
    TemperatureUp {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
        #[clap(
            long,
            short,
            help = "The amount to increase the temperature by, measured in Kelvin. This must be a multiple of 100."
        )]
        value: u16,
    },
    /// Decreases the temperature of your Logitech Litra device. The command will error if trying to decrease the temperature below the device's minimum.
    TemperatureDown {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
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

fn check_serial_number_if_some(serial_number: Option<&str>) -> impl Fn(&Device) -> bool + '_ {
    move |device| {
        serial_number.as_ref().is_none_or(|expected| {
            device
                .device_info()
                .serial_number()
                .is_some_and(|actual| &actual == expected)
        })
    }
}

#[derive(Debug)]
enum CliError {
    DeviceError(DeviceError),
    SerializationFailed(serde_json::Error),
    BrightnessPercentageCalculationFailed(TryFromIntError),
    InvalidBrightness(i16),
    DeviceNotFound,
    MCPError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::DeviceError(error) => error.fmt(f),
            CliError::SerializationFailed(error) => error.fmt(f),
            CliError::BrightnessPercentageCalculationFailed(error) => {
                write!(f, "Failed to calculate brightness: {}", error)
            }
            CliError::InvalidBrightness(brightness) => {
                write!(f, "Brightness {} lm is not supported", brightness)
            }
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

fn get_first_supported_device(
    context: &Litra,
    serial_number: Option<&str>,
) -> Result<DeviceHandle, CliError> {
    context
        .get_connected_devices()
        .find(check_serial_number_if_some(serial_number))
        .ok_or(CliError::DeviceNotFound)
        .and_then(|dev| dev.open(context).map_err(CliError::DeviceError))
}

#[derive(Serialize, Debug)]
struct DeviceInfo {
    pub serial_number: String,
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
            let device_handle = device.open(&context).ok()?;
            Some(DeviceInfo {
                serial_number: device
                    .device_info()
                    .serial_number()
                    .unwrap_or("")
                    .to_string(),
                device_type: device.device_type().to_string(),
                is_on: device_handle.is_on().ok()?,
                brightness_in_lumen: device_handle.brightness_in_lumen().ok()?,
                temperature_in_kelvin: device_handle.temperature_in_kelvin().ok()?,
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

fn handle_on_command(serial_number: Option<&str>) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;
    device_handle.set_on(true)?;
    Ok(())
}

fn handle_off_command(serial_number: Option<&str>) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;
    device_handle.set_on(false)?;
    Ok(())
}

fn handle_toggle_command(serial_number: Option<&str>) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;
    let is_on = device_handle.is_on()?;
    device_handle.set_on(!is_on)?;
    Ok(())
}

fn handle_brightness_command(
    serial_number: Option<&str>,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;

    match (value, percentage) {
        (Some(_), None) => {
            let brightness_in_lumen = value.unwrap();
            device_handle.set_brightness_in_lumen(brightness_in_lumen)?;
        }
        (None, Some(_)) => {
            let brightness_in_lumen = percentage_within_range(
                percentage.unwrap().into(),
                device_handle.minimum_brightness_in_lumen().into(),
                device_handle.maximum_brightness_in_lumen().into(),
            )
            .try_into()
            .map_err(CliError::BrightnessPercentageCalculationFailed)?;

            device_handle.set_brightness_in_lumen(brightness_in_lumen)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn handle_brightness_up_command(
    serial_number: Option<&str>,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;
    let current_brightness = device_handle.brightness_in_lumen()?;

    match (value, percentage) {
        (Some(_), None) => {
            let brightness_to_add = value.unwrap();
            let new_brightness = current_brightness + brightness_to_add;
            device_handle.set_brightness_in_lumen(new_brightness)?;
        }
        (None, Some(_)) => {
            let brightness_to_add = percentage_within_range(
                percentage.unwrap().into(),
                device_handle.minimum_brightness_in_lumen().into(),
                device_handle.maximum_brightness_in_lumen().into(),
            ) as u16
                - device_handle.minimum_brightness_in_lumen();

            let new_brightness = current_brightness + brightness_to_add;

            device_handle.set_brightness_in_lumen(new_brightness)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn handle_brightness_down_command(
    serial_number: Option<&str>,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;
    let current_brightness = device_handle.brightness_in_lumen()?;

    match (value, percentage) {
        (Some(_), None) => {
            let brightness_to_subtract = value.unwrap();
            let new_brightness = current_brightness - brightness_to_subtract;
            device_handle.set_brightness_in_lumen(new_brightness)?;
        }
        (None, Some(_)) => {
            let brightness_to_subtract = percentage_within_range(
                percentage.unwrap().into(),
                device_handle.minimum_brightness_in_lumen().into(),
                device_handle.maximum_brightness_in_lumen().into(),
            ) as u16
                - device_handle.minimum_brightness_in_lumen();

            let new_brightness = current_brightness as i16 - brightness_to_subtract as i16;

            if new_brightness < 0 {
                Err(CliError::InvalidBrightness(new_brightness))?;
            }

            device_handle.set_brightness_in_lumen(new_brightness as u16)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn handle_temperature_command(serial_number: Option<&str>, value: u16) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;

    device_handle.set_temperature_in_kelvin(value)?;
    Ok(())
}

fn handle_temperature_up_command(serial_number: Option<&str>, value: u16) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;
    let current_temperature = device_handle.temperature_in_kelvin()?;
    let new_temperature = current_temperature + value;

    device_handle.set_temperature_in_kelvin(new_temperature)?;
    Ok(())
}

fn handle_temperature_down_command(serial_number: Option<&str>, value: u16) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;
    let current_temperature = device_handle.temperature_in_kelvin()?;
    let new_temperature = current_temperature - value;

    device_handle.set_temperature_in_kelvin(new_temperature)?;
    Ok(())
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
        Commands::On { serial_number } => handle_on_command(serial_number.as_deref()),
        Commands::Off { serial_number } => handle_off_command(serial_number.as_deref()),
        Commands::Toggle { serial_number } => handle_toggle_command(serial_number.as_deref()),
        Commands::Brightness {
            serial_number,
            value,
            percentage,
        } => handle_brightness_command(serial_number.as_deref(), *value, *percentage),
        Commands::BrightnessUp {
            serial_number,
            value,
            percentage,
        } => handle_brightness_up_command(serial_number.as_deref(), *value, *percentage),
        Commands::BrightnessDown {
            serial_number,
            value,
            percentage,
        } => handle_brightness_down_command(serial_number.as_deref(), *value, *percentage),
        Commands::Temperature {
            serial_number,
            value,
        } => handle_temperature_command(serial_number.as_deref(), *value),
        Commands::TemperatureUp {
            serial_number,
            value,
        } => handle_temperature_up_command(serial_number.as_deref(), *value),
        Commands::TemperatureDown {
            serial_number,
            value,
        } => handle_temperature_down_command(serial_number.as_deref(), *value),
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
