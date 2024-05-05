use clap::{ArgGroup, Parser, Subcommand};
#[cfg(target_os = "linux")]
use inotify::{EventMask, Inotify, WatchMask};
use litra::{Device, DeviceError, DeviceHandle, Litra};
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
    /// List Logitech Litra devices connected to your computer
    Devices {
        #[clap(long, short, action, help = "Return the results in JSON format")]
        json: bool,
    },
    #[cfg(target_os = "linux")]
    /// Automatically turn the Logitech Litra device on when your webcam turns on, and off when the webcam turns off
    AutoToggle {
        #[clap(long, short, help = "The serial number of the Logitech Litra device")]
        serial_number: Option<String>,
    },
}

#[cfg(target_os = "linux")]
fn get_video_device_paths() -> std::io::Result<Vec<std::path::PathBuf>> {
    Ok(std::fs::read_dir("/dev")?
        .filter_map(|entry| entry.ok())
        .filter_map(|e| {
            e.file_name()
                .to_str()
                .filter(|name| name.starts_with("video"))
                .map(|_| e.path())
        })
        .collect())
}

fn percentage_within_range(percentage: u32, start_range: u32, end_range: u32) -> u32 {
    let result = ((percentage - 1) as f64 / (100 - 1) as f64) * (end_range - start_range) as f64
        + start_range as f64;
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
        serial_number.as_ref().map_or(true, |expected| {
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
    #[cfg(target_os = "linux")]
    IoError(std::io::Error),
    SerializationFailed(serde_json::Error),
    BrightnessPercentageCalculationFailed(TryFromIntError),
    DeviceNotFound,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::DeviceError(error) => error.fmt(f),
            #[cfg(target_os = "linux")]
            CliError::IoError(error) => write!(f, "Input/Output error: {}", error),
            CliError::SerializationFailed(error) => error.fmt(f),
            CliError::BrightnessPercentageCalculationFailed(error) => {
                write!(f, "Failed to calculate brightness: {}", error)
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

#[cfg(target_os = "linux")]
impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::IoError(error)
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

fn handle_devices_command(json: bool) -> CliResult {
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

    if json {
        println!(
            "{}",
            serde_json::to_string(&litra_devices).map_err(CliError::SerializationFailed)?
        );
        Ok(())
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

fn handle_temperature_command(serial_number: Option<&str>, value: u16) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;

    device_handle.set_temperature_in_kelvin(value)?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn handle_autotoggle_command(serial_number: Option<&str>) -> CliResult {
    let context = Litra::new()?;
    let device_handle = get_first_supported_device(&context, serial_number)?;

    let mut inotify = Inotify::init()?;
    for path in get_video_device_paths()? {
        match inotify
            .watches()
            .add(&path, WatchMask::OPEN | WatchMask::CLOSE)
        {
            Ok(_) => println!("Watching device {}", path.display()),
            Err(_) => eprintln!("Failed to watch device {}", path.display()),
        }
    }

    let mut num_devices_open: usize = 0;
    loop {
        // Read events that were added with `Watches::add` above.
        let mut buffer = [0; 1024];
        let events = inotify.read_events_blocking(&mut buffer)?;
        for event in events {
            match event.mask {
                EventMask::OPEN => {
                    match event.name.and_then(std::ffi::OsStr::to_str) {
                        Some(name) => println!("Video device opened: {}", name),
                        None => println!("Video device opened"),
                    }
                    num_devices_open = num_devices_open.saturating_add(1);
                }
                EventMask::CLOSE_WRITE | EventMask::CLOSE_NOWRITE => {
                    match event.name.and_then(std::ffi::OsStr::to_str) {
                        Some(name) => println!("Video device closed: {}", name),
                        None => println!("Video device closed"),
                    }
                    num_devices_open = num_devices_open.saturating_sub(1);
                }
                _ => (),
            }
        }
        if num_devices_open == 0 {
            println!("No video devices open, turning off light");
            device_handle.set_on(false)?;
        } else {
            println!("{} video devices open, turning on light", num_devices_open);
            device_handle.set_on(true)?;
        }
    }
}

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
        Commands::Temperature {
            serial_number,
            value,
        } => handle_temperature_command(serial_number.as_deref(), *value),
        #[cfg(target_os = "linux")]
        Commands::AutoToggle { serial_number } => {
            handle_autotoggle_command(serial_number.as_deref())
        }
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
