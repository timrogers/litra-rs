use clap::{ArgGroup, Parser, Subcommand};
use hidapi::{HidApi, HidDevice};
use serde::Serialize;
use std::fmt;

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
            help = "The brightness to set, measured in lumens",
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
            help = "The temperature to set, measured in Kelvin. You can check the allowed values for your device with the `devices` command."
        )]
        value: u16,
    },
    /// List Logitech Litra devices connected to your computer
    Devices {
        #[clap(long, short, action, help = "Return the results in JSON format")]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
pub enum DeviceType {
    #[serde(rename = "Litra Glow")]
    LitraGlow,
    #[serde(rename = "Litra Beam")]
    LitraBeam,
    #[serde(rename = "Litra Beam LX")]
    LitraBeamLX,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceType::LitraGlow => write!(f, "Litra Glow"),
            DeviceType::LitraBeam => write!(f, "Litra Beam"),
            DeviceType::LitraBeamLX => write!(f, "Litra Beam LX"),
        }
    }
}

#[derive(Serialize, Debug)]
struct Device {
    serial_number: String,
    device_type: DeviceType,
    is_on: bool,
    brightness_in_lumen: u16,
    temperature_in_kelvin: u16,
    #[serde(skip_serializing)]
    device_handle: HidDevice,
    minimum_brightness_in_lumen: u16,
    maximum_brightness_in_lumen: u16,
    allowed_temperatures_in_kelvin: Vec<u16>,
}

const VENDOR_ID: u16 = 0x046d;
const PRODUCT_IDS: [u16; 4] = [0xc900, 0xc901, 0xb901, 0xc903];

fn get_device_type(product_id: u16) -> DeviceType {
    match product_id {
        0xc900 => DeviceType::LitraGlow,
        0xc901 => DeviceType::LitraBeam,
        0xb901 => DeviceType::LitraBeam,
        0xc903 => DeviceType::LitraBeamLX,
        _ => panic!("Unknown product ID"),
    }
}

fn get_minimum_brightness_in_lumen(device_type: &DeviceType) -> u16 {
    match device_type {
        DeviceType::LitraGlow => 20,
        DeviceType::LitraBeam => 30,
        DeviceType::LitraBeamLX => 30,
    }
}

fn get_maximum_brightness_in_lumen(device_type: &DeviceType) -> u16 {
    match device_type {
        DeviceType::LitraGlow => 250,
        DeviceType::LitraBeam => 400,
        DeviceType::LitraBeamLX => 400,
    }
}

fn multiples_within_range(multiples_of: u16, start_range: u16, end_range: u16) -> Vec<u16> {
    (start_range..=end_range)
        .filter(|n| n % multiples_of == 0)
        .collect()
}

fn get_allowed_temperatures_in_kelvin(_device_type: &DeviceType) -> Vec<u16> {
    return multiples_within_range(100, 2700, 6500);
}

fn get_connected_devices(api: HidApi, serial_number: Option<String>) -> Vec<Device> {
    let hid_devices = api.device_list();

    let mut litra_devices = Vec::new();

    for device in hid_devices {
        if device.vendor_id() == VENDOR_ID && PRODUCT_IDS.contains(&device.product_id()) {
            if let Some(serial_number) = &serial_number {
                if device.serial_number().unwrap_or("") != *serial_number {
                    continue;
                }
            }

            litra_devices.push(device);
        }
    }

    // On my macOS Sonoma device, every Litra device is returned twice for some reason
    litra_devices.dedup_by(|a, b| a.path() == b.path());

    return litra_devices
        .iter()
        .map(|device| {
            let device_type = get_device_type(device.product_id());
            let device_handle = api.open_path(device.path()).unwrap();
            let is_on = is_on(&device_handle, &device_type);
            let brightness_in_lumen = get_brightness_in_lumen(&device_handle, &device_type);
            let temperature_in_kelvin = get_temperature_in_kelvin(&device_handle, &device_type);
            let minimum_brightness_in_lumen = get_minimum_brightness_in_lumen(&device_type);
            let maximum_brightness_in_lumen = get_maximum_brightness_in_lumen(&device_type);
            let allowed_temperatures_in_kelvin = get_allowed_temperatures_in_kelvin(&device_type);

            Device {
                serial_number: device.serial_number().unwrap_or("").to_string(),
                device_type: device_type,
                is_on: is_on,
                brightness_in_lumen: brightness_in_lumen,
                temperature_in_kelvin: temperature_in_kelvin,
                device_handle: device_handle,
                minimum_brightness_in_lumen: minimum_brightness_in_lumen,
                maximum_brightness_in_lumen: maximum_brightness_in_lumen,
                allowed_temperatures_in_kelvin: allowed_temperatures_in_kelvin,
            }
        })
        .collect();
}

fn generate_is_on_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow => [
            0x11, 0xff, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    }
}

fn is_on(device_handle: &HidDevice, device_type: &DeviceType) -> bool {
    let message = generate_is_on_bytes(device_type);

    device_handle.write(&message).unwrap();

    let mut response_buffer = [0x00; 20];
    let response = device_handle.read(&mut response_buffer[..]).unwrap();

    return response_buffer[..response][4] == 1;
}

fn generate_get_brightness_in_lumen_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow => [
            0x11, 0xff, 0x04, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    }
}

fn get_brightness_in_lumen(device_handle: &HidDevice, device_type: &DeviceType) -> u16 {
    let message = generate_get_brightness_in_lumen_bytes(device_type);

    device_handle.write(&message).unwrap();

    let mut response_buffer = [0x00; 20];
    let response = device_handle.read(&mut response_buffer[..]).unwrap();

    return response_buffer[..response][5].into();
}

fn generate_get_temperature_in_kelvin_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow => [
            0x11, 0xff, 0x04, 0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    }
}

fn get_temperature_in_kelvin(device_handle: &HidDevice, device_type: &DeviceType) -> u16 {
    let message = generate_get_temperature_in_kelvin_bytes(device_type);

    device_handle.write(&message).unwrap();

    let mut response_buffer = [0x00; 20];
    let response = device_handle.read(&mut response_buffer[..]).unwrap();
    return (response_buffer[..response][4] as u16 * 256 + response_buffer[..response][5] as u16)
        .into();
}

fn generate_turn_on_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow => [
            0x11, 0xff, 0x04, 0x1c, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x1c, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x1c, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    }
}

fn turn_on(device_handle: &HidDevice, device_type: &DeviceType) {
    let message = generate_turn_on_bytes(device_type);

    device_handle.write(&message).unwrap();
}

fn generate_turn_off_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow => [
            0x11, 0xff, 0x04, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    }
}

fn turn_off(device_handle: &HidDevice, device_type: &DeviceType) {
    let message = generate_turn_off_bytes(device_type);

    device_handle.write(&message).unwrap();
}

fn get_is_on_text(is_on: bool) -> &'static str {
    if is_on {
        return "On";
    }

    return "Off";
}

fn get_is_on_emoji(is_on: bool) -> &'static str {
    if is_on {
        return "💡";
    }

    return "🌑";
}

fn integer_to_bytes(integer: u16) -> [u8; 2] {
    return [(integer / 256) as u8, (integer % 256) as u8];
}

fn generate_set_brightness_in_lumen_bytes(
    device_type: &DeviceType,
    brightness_in_lumen: u16,
) -> [u8; 20] {
    let brightness_bytes = integer_to_bytes(brightness_in_lumen);

    match device_type {
        DeviceType::LitraGlow => [
            0x11,
            0xff,
            0x04,
            0x4c,
            brightness_bytes[0],
            brightness_bytes[1],
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ],
        DeviceType::LitraBeam => [
            0x11,
            0xff,
            0x04,
            0x4c,
            brightness_bytes[0],
            brightness_bytes[1],
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11,
            0xff,
            0x06,
            0x4c,
            brightness_bytes[0],
            brightness_bytes[1],
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ],
    }
}

fn set_brightness_in_lumen(
    device_handle: &HidDevice,
    device_type: &DeviceType,
    brightness_in_lumen: u16,
) {
    let message = generate_set_brightness_in_lumen_bytes(device_type, brightness_in_lumen);

    device_handle.write(&message).unwrap();
}

fn generate_set_temperature_in_kelvin_bytes(
    device_type: &DeviceType,
    temperature_in_kelvin: u16,
) -> [u8; 20] {
    let temperature_bytes = integer_to_bytes(temperature_in_kelvin);

    match device_type {
        DeviceType::LitraGlow => [
            0x11,
            0xff,
            0x04,
            0x9c,
            temperature_bytes[0],
            temperature_bytes[1],
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ],
        DeviceType::LitraBeam => [
            0x11,
            0xff,
            0x04,
            0x9c,
            temperature_bytes[0],
            temperature_bytes[1],
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11,
            0xff,
            0x06,
            0x9c,
            temperature_bytes[0],
            temperature_bytes[1],
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ],
    }
}

fn set_temperature_in_kelvin(
    device_handle: &HidDevice,
    device_type: &DeviceType,
    temperature_in_kelvin: u16,
) {
    let message = generate_set_temperature_in_kelvin_bytes(device_type, temperature_in_kelvin);

    device_handle.write(&message).unwrap();
}

fn percentage_within_range(percentage: u32, start_range: u32, end_range: u32) -> u32 {
    let result = ((percentage - 1) as f64 / (100 - 1) as f64) * (end_range - start_range) as f64
        + start_range as f64;
    result.round() as u32
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

                    let comma_separated_values = device
                        .allowed_temperatures_in_kelvin
                        .iter()
                        .map(|x| x.to_string())
                        .map(|x| format!("{} K", x))
                        .collect::<Vec<String>>()
                        .join(", ");

                    println!("  - Temperature: {} K", device.temperature_in_kelvin);
                    println!("    - Allowed values: {}", comma_separated_values);
                }

                if litra_devices.len() < 1 {
                    println!("No devices found");
                }
            }
        }
        Commands::On { serial_number } => {
            let devices = get_connected_devices(api, serial_number.clone());

            if devices.len() == 0 {
                println!("Device not found");
                std::process::exit(exitcode::DATAERR);
            }

            let device = &devices[0];

            turn_on(&device.device_handle, &device.device_type);
        }
        Commands::Off { serial_number } => {
            let devices = get_connected_devices(api, serial_number.clone());

            if devices.len() == 0 {
                println!("Device not found");
                std::process::exit(exitcode::DATAERR);
            }

            let device = &devices[0];

            turn_off(&device.device_handle, &device.device_type);
        }
        Commands::Toggle { serial_number } => {
            let devices = get_connected_devices(api, serial_number.clone());

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
            let devices = get_connected_devices(api, serial_number.clone());

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
            let devices = get_connected_devices(api, serial_number.clone());

            if devices.len() == 0 {
                println!("Device not found");
                std::process::exit(exitcode::DATAERR);
            }

            let device = &devices[0];

            if !device.allowed_temperatures_in_kelvin.contains(&value) {
                let comma_separated_values = device
                    .allowed_temperatures_in_kelvin
                    .iter()
                    .map(|x| x.to_string())
                    .map(|x| format!("{} K", x))
                    .collect::<Vec<String>>()
                    .join(", ");

                println!(
                    "Temperature must be set to one of the following allowed values in kelvin (K): {}",
                    comma_separated_values
                );
                std::process::exit(exitcode::DATAERR);
            }

            set_temperature_in_kelvin(&device.device_handle, &device.device_type, *value);
        }
    };
}
