use clap::{ArgGroup, Parser, Subcommand};
use litra::{
    get_connected_devices, set_brightness_in_lumen, set_temperature_in_kelvin, turn_off, turn_on,
};

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

fn main() {
    let args = Cli::parse();
    let api = hidapi::HidApi::new().unwrap();

    match &args.command {
        Commands::Devices { json } => {
            let litra_devices = get_connected_devices(api, None);

            if *json {
                println!("{}", serde_json::to_string(&litra_devices).unwrap());
            } else {
                for device in &litra_devices {
                    println!(
                        "- {} ({}): {} {}",
                        device.device_type,
                        device.serial_number,
                        get_is_on_text(device.is_on),
                        get_is_on_emoji(device.is_on)
                    );

                    println!("  - Brightness: {} lm", device.brightness_in_lumen,);
                    println!("    - Minimum: {} lm", device.minimum_brightness_in_lumen);
                    println!("    - Maximum: {} lm", device.maximum_brightness_in_lumen);
                    println!("  - Temperature: {} K", device.temperature_in_kelvin);
                    println!("    - Minimum: {} K", device.minimum_temperature_in_kelvin);
                    println!("    - Maximum: {} K", device.maximum_temperature_in_kelvin);
                }

                if litra_devices.len() < 1 {
                    println!("No devices found");
                }
            }
        }
        Commands::On { serial_number } => {
            let devices = get_connected_devices(api, serial_number.as_deref());

            if devices.len() == 0 {
                println!("Device not found");
                std::process::exit(exitcode::DATAERR);
            }

            let device = &devices[0];

            turn_on(&device.device_handle, &device.device_type);
        }
        Commands::Off { serial_number } => {
            let devices = get_connected_devices(api, serial_number.as_deref());

            if devices.len() == 0 {
                println!("Device not found");
                std::process::exit(exitcode::DATAERR);
            }

            let device = &devices[0];

            turn_off(&device.device_handle, &device.device_type);
        }
        Commands::Toggle { serial_number } => {
            let devices = get_connected_devices(api, serial_number.as_deref());

            if devices.len() == 0 {
                println!("Device not found");
                std::process::exit(exitcode::DATAERR);
            }

            let device = &devices[0];

            if device.is_on {
                turn_off(&device.device_handle, &device.device_type);
            } else {
                turn_on(&device.device_handle, &device.device_type);
            }
        }
        Commands::Brightness {
            serial_number,
            value,
            percentage,
        } => {
            let devices = get_connected_devices(api, serial_number.as_deref());

            if devices.len() == 0 {
                println!("Device not found");
                std::process::exit(exitcode::DATAERR);
            }

            let device = &devices[0];

            match (value, percentage) {
                (Some(_), None) => {
                    let brightness_in_lumen = value.unwrap();

                    if brightness_in_lumen < device.minimum_brightness_in_lumen
                        || brightness_in_lumen > device.maximum_brightness_in_lumen
                    {
                        println!(
                            "Brightness must be set to a value between {} lm and {} lm",
                            device.minimum_brightness_in_lumen, device.maximum_brightness_in_lumen
                        );
                        std::process::exit(exitcode::DATAERR);
                    }

                    set_brightness_in_lumen(
                        &device.device_handle,
                        &device.device_type,
                        brightness_in_lumen,
                    );
                }
                (None, Some(_)) => {
                    let brightness_in_lumen = percentage_within_range(
                        percentage.unwrap().into(),
                        device.minimum_brightness_in_lumen.into(),
                        device.maximum_brightness_in_lumen.into(),
                    );

                    set_brightness_in_lumen(
                        &device.device_handle,
                        &device.device_type,
                        brightness_in_lumen.try_into().unwrap(),
                    );
                }
                _ => unreachable!(),
            }
        }
        Commands::Temperature {
            serial_number,
            value,
        } => {
            let devices = get_connected_devices(api, serial_number.as_deref());

            if devices.len() == 0 {
                println!("Device not found");
                std::process::exit(exitcode::DATAERR);
            }

            let device = &devices[0];

            let allowed_temperatures_in_kelvin = multiples_within_range(
                100,
                device.minimum_temperature_in_kelvin,
                device.maximum_temperature_in_kelvin,
            );

            if !allowed_temperatures_in_kelvin.contains(&value) {
                println!(
                    "Temperature must be set to a multiple of 100 between {} K and {} K",
                    device.minimum_temperature_in_kelvin, device.maximum_temperature_in_kelvin
                );
                std::process::exit(exitcode::DATAERR);
            }

            set_temperature_in_kelvin(&device.device_handle, &device.device_type, *value);
        }
    };
}
