use clap::{ArgGroup, Parser, Subcommand};
use litra::{Device, DeviceError, DeviceHandle, DeviceResult, Litra};
use serde::Serialize;
use std::fmt;
use std::num::TryFromIntError;
use std::process::ExitCode;

/// Control your USB-connected Logitech Litra lights from the command line
#[derive(Debug, Parser)]
#[clap(name = "litra", version)]
struct Cli {
    // Test
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Turn your Logitech Litra device on
    On {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
    },
    /// Turn your Logitech Litra device off
    Off {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
    },
    /// Toggles your Logitech Litra device on or off
    Toggle {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
    },
    /// Sets the brightness of your Logitech Litra device
    #[clap(group = ArgGroup::new("brightness").required(true).multiple(false))]
    Brightness {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
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
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
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
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
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
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
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
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
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
        #[clap(long, help = "Select devices by their type (LitraGlow, LitraBeam, LitraBeamLX)")]
        device_type: Option<String>,
        #[clap(long, help = "Apply command to all connected devices", default_value = "false")]
        all_devices: bool,
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
    serial_number: Option<&'a str>,
    device_type: Option<&'a str>,
) -> impl Fn(&Device) -> bool + 'a {
    move |device| {
        // Check device type if specified
        let type_match = device_type.as_ref().map_or(true, |expected| {
            // Convert both to strings without spaces and compare
            let device_type_str = format!("{}", device.device_type())
                .replace(" ", "")
                .to_lowercase();
            
            let expected_type = expected.replace(" ", "").to_lowercase();
            
            // Check if the expected type is contained in the device type (to be more flexible)
            device_type_str.contains(&expected_type) || 
                expected_type.contains(&device_type_str)
        });
        
        // If a serial number is specified and type matches, then try to check it
        if serial_number.is_some() && type_match {
            true  // We'll filter by serial number after opening the device
        } else {
            // No serial specified, just use type match
            type_match
        }
    }
}

#[derive(Debug)]
enum CliError {
    DeviceError(DeviceError),
    SerializationFailed(serde_json::Error),
    BrightnessPercentageCalculationFailed(TryFromIntError),
    InvalidBrightness(i16),
    DeviceNotFound,
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
    device_type: Option<&str>,
) -> Result<DeviceHandle, CliError> {
    // First filter by device type
    let potential_devices: Vec<Device> = context
        .get_connected_devices()
        .filter(check_device_filters(context, serial_number, device_type))
        .collect();
    
    // If we need to filter by serial, open devices and check
    if let Some(serial) = serial_number {
        for device in potential_devices {
            if let Ok(handle) = device.open(context) {
                if let Ok(Some(actual_serial)) = handle.serial_number() {
                    if actual_serial == serial {
                        return Ok(handle);
                    }
                }
            }
        }
        Err(CliError::DeviceNotFound)
    } else if let Some(device) = potential_devices.into_iter().next() {
        // No serial filter, just return the first device that matched the type filter
        device.open(context).map_err(CliError::DeviceError)
    } else {
        Err(CliError::DeviceNotFound)
    }
}

/// Get all devices matching the given filters
fn get_all_supported_devices(
    context: &Litra,
    serial_number: Option<&str>,
    device_type: Option<&str>,
) -> Vec<DeviceHandle> {
    // First filter by device type
    let potential_devices: Vec<Device> = context
        .get_connected_devices()
        .filter(check_device_filters(context, serial_number, device_type))
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
        handles
    } else {
        // No serial filter, include all devices that matched the type filter
        potential_devices
            .into_iter()
            .filter_map(|dev| dev.open(context).ok())
            .collect()
    }
}

/// Apply a command to device(s) based on filters and all_devices flag
fn with_device<F>(
    serial_number: Option<&str>,
    device_type: Option<&str>,
    all_devices: bool,
    callback: F,
) -> CliResult
where
    F: Fn(&DeviceHandle) -> DeviceResult<()>,
{
    let context = Litra::new()?;
    
    if all_devices {
        let devices = get_all_supported_devices(&context, serial_number, device_type);
        if devices.is_empty() {
            return Err(CliError::DeviceNotFound);
        }
        
        for device_handle in devices {
            // Ignore errors for individual devices when using all_devices
            let _ = callback(&device_handle);
        }
        Ok(())
    } else {
        let device_handle = get_first_supported_device(&context, serial_number, device_type)?;
        callback(&device_handle)?;
        Ok(())
    }
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

fn handle_devices_command(json: bool) -> CliResult {
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
            
            let serial = match device_handle.serial_number() {
                Ok(Some(s)) => s,
                Ok(None) => {
                    "UNKNOWN".to_string()
                },
                Err(_e) => {
                    "UNKNOWN".to_string()
                }
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
                Err(e) => {
                    return None;
                }
            };
            
            Some(DeviceInfo {
                serial_number: serial,
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

fn handle_on_command(
    serial_number: Option<&str>,
    device_type: Option<&str>,
    all_devices: bool,
) -> CliResult {
    with_device(serial_number, device_type, all_devices, |device_handle| {
        device_handle.set_on(true)
    })
}

fn handle_off_command(
    serial_number: Option<&str>,
    device_type: Option<&str>,
    all_devices: bool,
) -> CliResult {
    with_device(serial_number, device_type, all_devices, |device_handle| {
        device_handle.set_on(false)
    })
}

fn handle_toggle_command(
    serial_number: Option<&str>,
    device_type: Option<&str>,
    all_devices: bool,
) -> CliResult {
    // For toggle we need special logic since we need to get state first
    let context = Litra::new()?;
    
    if all_devices {
        let devices = get_all_supported_devices(&context, serial_number, device_type);
        if devices.is_empty() {
            return Err(CliError::DeviceNotFound);
        }
        
        for device_handle in devices {
            // Toggle each device individually, ignoring errors
            if let Ok(is_on) = device_handle.is_on() {
                let _ = device_handle.set_on(!is_on);
            }
        }
        Ok(())
    } else {
        with_device(serial_number, device_type, false, |device_handle| {
            let is_on = device_handle.is_on()?;
            device_handle.set_on(!is_on)
        })
    }
}

/// Create a general purpose function to handle brightness setting
fn with_brightness_setting<F>(
    serial_number: Option<&str>,
    device_type: Option<&str>,
    all_devices: bool,
    brightness_fn: F,
) -> CliResult
where
    F: Fn(&DeviceHandle) -> Result<u16, DeviceError>,
{
    let context = Litra::new()?;
    
    if all_devices {
        let devices = get_all_supported_devices(&context, serial_number, device_type);
        if devices.is_empty() {
            return Err(CliError::DeviceNotFound);
        }
        
        for device_handle in devices {
            if let Ok(brightness) = brightness_fn(&device_handle) {
                let _ = device_handle.set_brightness_in_lumen(brightness);
            }
        }
        Ok(())
    } else {
        let device_handle = get_first_supported_device(&context, serial_number, device_type)?;
        let brightness = brightness_fn(&device_handle)?;
        device_handle.set_brightness_in_lumen(brightness)?;
        Ok(())
    }
}

fn handle_brightness_command(
    serial_number: Option<&str>,
    device_type: Option<&str>,
    all_devices: bool,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    match (value, percentage) {
        (Some(brightness), None) => {
            with_device(serial_number, device_type, all_devices, |device_handle| {
                device_handle.set_brightness_in_lumen(brightness)
            })
        }
        (None, Some(pct)) => {
            with_brightness_setting(serial_number, device_type, all_devices, |device_handle| {
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
    device_type: Option<&str>,
    all_devices: bool,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    match (value, percentage) {
        (Some(brightness_to_add), None) => {
            with_brightness_setting(serial_number, device_type, all_devices, |device_handle| {
                let current_brightness = device_handle.brightness_in_lumen()?;
                let new_brightness = current_brightness + brightness_to_add;
                Ok(new_brightness)
            })
        }
        (None, Some(pct)) => {
            with_brightness_setting(serial_number, device_type, all_devices, |device_handle| {
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
    device_type: Option<&str>,
    all_devices: bool,
    value: Option<u16>,
    percentage: Option<u8>,
) -> CliResult {
    match (value, percentage) {
        (Some(brightness_to_subtract), None) => {
            with_brightness_setting(serial_number, device_type, all_devices, |device_handle| {
                let current_brightness = device_handle.brightness_in_lumen()?;
                
                if current_brightness <= brightness_to_subtract {
                    if all_devices {
                        // When in all_devices mode, just skip this device
                        return Err(DeviceError::InvalidBrightness(0));
                    } else {
                        return Err(DeviceError::InvalidBrightness(0));
                    }
                }
                
                let new_brightness = current_brightness - brightness_to_subtract;
                Ok(new_brightness)
            })
        }
        (None, Some(pct)) => {
            with_brightness_setting(serial_number, device_type, all_devices, |device_handle| {
                let current_brightness = device_handle.brightness_in_lumen()?;
                
                let brightness_to_subtract = percentage_within_range(
                    pct.into(),
                    device_handle.minimum_brightness_in_lumen().into(),
                    device_handle.maximum_brightness_in_lumen().into(),
                ) as u16
                    - device_handle.minimum_brightness_in_lumen();
                
                let new_brightness = current_brightness as i16 - brightness_to_subtract as i16;
                
                if new_brightness <= 0 {
                    if all_devices {
                        // When in all_devices mode, just skip this device
                        return Err(DeviceError::InvalidBrightness(0));
                    } else {
                        return Err(DeviceError::InvalidBrightness(0));
                    }
                }
                
                Ok(new_brightness as u16)
            })
        }
        _ => unreachable!(),
    }
}

fn handle_temperature_command(
    serial_number: Option<&str>,
    device_type: Option<&str>,
    all_devices: bool,
    value: u16
) -> CliResult {
    with_device(serial_number, device_type, all_devices, |device_handle| {
        device_handle.set_temperature_in_kelvin(value)
    })
}

fn handle_temperature_up_command(
    serial_number: Option<&str>,
    device_type: Option<&str>,
    all_devices: bool,
    value: u16
) -> CliResult {
    with_device(serial_number, device_type, all_devices, |device_handle| {
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
    device_type: Option<&str>,
    all_devices: bool,
    value: u16
) -> CliResult {
    with_device(serial_number, device_type, all_devices, |device_handle| {
        let current_temperature = device_handle.temperature_in_kelvin()?;
        
        // Check if new temperature would be below minimum
        if current_temperature <= value {
            if all_devices {
                // When in all_devices mode, just skip this device
                return Err(DeviceError::InvalidTemperature(0));
            } else {
                return Err(DeviceError::InvalidTemperature(current_temperature - value));
            }
        }
        
        let new_temperature = current_temperature - value;
        device_handle.set_temperature_in_kelvin(new_temperature)
    })
}

fn main() -> ExitCode {
    let args = Cli::parse();

    let result = match &args.command {
        Commands::Devices { json } => handle_devices_command(*json),
        Commands::On { serial_number, device_type, all_devices } => 
            handle_on_command(serial_number.as_deref(), device_type.as_deref(), *all_devices),
        Commands::Off { serial_number, device_type, all_devices } => 
            handle_off_command(serial_number.as_deref(), device_type.as_deref(), *all_devices),
        Commands::Toggle { serial_number, device_type, all_devices } => 
            handle_toggle_command(serial_number.as_deref(), device_type.as_deref(), *all_devices),
        Commands::Brightness {
            serial_number,
            device_type,
            all_devices,
            value,
            percentage,
        } => handle_brightness_command(serial_number.as_deref(), device_type.as_deref(), *all_devices, *value, *percentage),
        Commands::BrightnessUp {
            serial_number,
            device_type,
            all_devices,
            value,
            percentage,
        } => handle_brightness_up_command(serial_number.as_deref(), device_type.as_deref(), *all_devices, *value, *percentage),
        Commands::BrightnessDown {
            serial_number,
            device_type,
            all_devices,
            value,
            percentage,
        } => handle_brightness_down_command(serial_number.as_deref(), device_type.as_deref(), *all_devices, *value, *percentage),
        Commands::Temperature {
            serial_number,
            device_type,
            all_devices,
            value,
        } => handle_temperature_command(serial_number.as_deref(), device_type.as_deref(), *all_devices, *value),
        Commands::TemperatureUp {
            serial_number,
            device_type,
            all_devices,
            value,
        } => handle_temperature_up_command(serial_number.as_deref(), device_type.as_deref(), *all_devices, *value),
        Commands::TemperatureDown {
            serial_number,
            device_type,
            all_devices,
            value,
        } => handle_temperature_down_command(serial_number.as_deref(), device_type.as_deref(), *all_devices, *value),
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
