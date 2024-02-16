use clap::{ArgGroup, Parser, Subcommand};
use litra::{Device, Litra};
use serde::Serialize;
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

fn multiples_within_range(multiples_of: u16, start_range: u16, end_range: u16) -> Vec<u16> {
    (start_range..=end_range)
        .filter(|n| n % multiples_of == 0)
        .collect()
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

#[derive(Serialize, Debug)]
pub struct DeviceInfo {
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

fn main() -> ExitCode {
    let args = Cli::parse();
    let context = Litra::new().unwrap();

    match &args.command {
        Commands::Devices { json } => {
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
                        is_on: device_handle.is_enabled().ok()?,
                        brightness_in_lumen: device_handle.brightness_in_lumen().ok()?,
                        temperature_in_kelvin: device_handle.temperature_in_kelvin().ok()?,
                        minimum_brightness_in_lumen: device_handle.minimum_brightness_in_lumen(),
                        maximum_brightness_in_lumen: device_handle.maximum_brightness_in_lumen(),
                        minimum_temperature_in_kelvin: device_handle
                            .minimum_temperature_in_kelvin(),
                        maximum_temperature_in_kelvin: device_handle
                            .maximum_temperature_in_kelvin(),
                    })
                })
                .collect();

            if *json {
                println!("{}", serde_json::to_string(&litra_devices).unwrap());
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
            ExitCode::SUCCESS
        }
        Commands::On { serial_number } => {
            let device_handle = match context
                .get_connected_devices()
                .filter(check_serial_number_if_some(serial_number.as_deref()))
                .next()
            {
                Some(dev) => dev.open(&context).unwrap(),
                None => {
                    println!("Device not found");
                    return ExitCode::FAILURE;
                }
            };

            device_handle.set_enabled(true).unwrap();
            ExitCode::SUCCESS
        }
        Commands::Off { serial_number } => {
            let device_handle = match context
                .get_connected_devices()
                .filter(check_serial_number_if_some(serial_number.as_deref()))
                .next()
            {
                Some(dev) => dev.open(&context).unwrap(),
                None => {
                    println!("Device not found");
                    return ExitCode::FAILURE;
                }
            };

            device_handle.set_enabled(false).unwrap();
            ExitCode::SUCCESS
        }
        Commands::Toggle { serial_number } => {
            let device_handle = match context
                .get_connected_devices()
                .filter(check_serial_number_if_some(serial_number.as_deref()))
                .next()
            {
                Some(dev) => dev.open(&context).unwrap(),
                None => {
                    println!("Device not found");
                    return ExitCode::FAILURE;
                }
            };

            let is_enabled = device_handle.is_enabled().unwrap();
            device_handle.set_enabled(!is_enabled).unwrap();
            ExitCode::SUCCESS
        }
        Commands::Brightness {
            serial_number,
            value,
            percentage,
        } => {
            let device_handle = match context
                .get_connected_devices()
                .filter(check_serial_number_if_some(serial_number.as_deref()))
                .next()
            {
                Some(dev) => dev.open(&context).unwrap(),
                None => {
                    println!("Device not found");
                    return ExitCode::FAILURE;
                }
            };

            match (value, percentage) {
                (Some(_), None) => {
                    let brightness_in_lumen = value.unwrap();

                    if brightness_in_lumen < device_handle.minimum_brightness_in_lumen()
                        || brightness_in_lumen > device_handle.maximum_brightness_in_lumen()
                    {
                        println!(
                            "Brightness must be set to a value between {} lm and {} lm",
                            device_handle.minimum_brightness_in_lumen(),
                            device_handle.maximum_brightness_in_lumen()
                        );
                        return ExitCode::FAILURE;
                    }

                    device_handle
                        .set_brightness_in_lumen(brightness_in_lumen)
                        .unwrap();
                }
                (None, Some(_)) => {
                    let brightness_in_lumen = percentage_within_range(
                        percentage.unwrap().into(),
                        device_handle.minimum_brightness_in_lumen().into(),
                        device_handle.maximum_brightness_in_lumen().into(),
                    )
                    .try_into()
                    .unwrap();

                    device_handle
                        .set_brightness_in_lumen(brightness_in_lumen)
                        .unwrap();
                }
                _ => unreachable!(),
            }
            ExitCode::SUCCESS
        }
        Commands::Temperature {
            serial_number,
            value,
        } => {
            let device_handle = match context
                .get_connected_devices()
                .filter(check_serial_number_if_some(serial_number.as_deref()))
                .next()
            {
                Some(dev) => dev.open(&context).unwrap(),
                None => {
                    println!("Device not found");
                    return ExitCode::FAILURE;
                }
            };

            let allowed_temperatures_in_kelvin = multiples_within_range(
                100,
                device_handle.minimum_temperature_in_kelvin(),
                device_handle.maximum_temperature_in_kelvin(),
            );

            if !allowed_temperatures_in_kelvin.contains(value) {
                println!(
                    "Temperature must be set to a multiple of 100 between {} K and {} K",
                    device_handle.minimum_temperature_in_kelvin(),
                    device_handle.maximum_temperature_in_kelvin()
                );
                return ExitCode::FAILURE;
            }

            device_handle.set_temperature_in_kelvin(*value).unwrap();
            ExitCode::SUCCESS
        }
    }
}
