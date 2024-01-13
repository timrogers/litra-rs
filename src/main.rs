use clap::{Parser, Subcommand};
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
    /// List Logitech Litra devices connected to your computer
    ListDevices {
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

            Device {
                serial_number: device.serial_number().unwrap_or("").to_string(),
                device_type: device_type,
                is_on: is_on,
                brightness_in_lumen: brightness_in_lumen,
                temperature_in_kelvin: temperature_in_kelvin,
                device_handle: device_handle,
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

fn main() {
    let args = Cli::parse();
    let api = hidapi::HidApi::new().unwrap();

    match &args.command {
        Commands::ListDevices { json } => {
            let litra_devices = get_connected_devices(api, None);

            if *json {
                println!("{}", serde_json::to_string(&litra_devices).unwrap());
            } else {
                for device in litra_devices {
                    println!(
                        "- {} ({}): {} {}",
                        device.device_type,
                        device.serial_number,
                        get_is_on_text(device.is_on),
                        get_is_on_emoji(device.is_on)
                    );

                    println!("  - Brightness: {} lm", device.brightness_in_lumen);
                    println!("  - Temperature: {} K", device.temperature_in_kelvin);
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
    };
}
