use hidapi::{HidApi, HidDevice};
use serde::Serialize;
use std::fmt;

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
pub struct Device {
    pub serial_number: String,
    pub device_type: DeviceType,
    pub is_on: bool,
    pub brightness_in_lumen: u16,
    pub temperature_in_kelvin: u16,
    #[serde(skip_serializing)]
    pub device_handle: HidDevice,
    pub minimum_brightness_in_lumen: u16,
    pub maximum_brightness_in_lumen: u16,
    pub minimum_temperature_in_kelvin: u16,
    pub maximum_temperature_in_kelvin: u16,
}

const VENDOR_ID: u16 = 0x046d;
const PRODUCT_IDS: [u16; 4] = [0xc900, 0xc901, 0xb901, 0xc903];
const USAGE_PAGE: u16 = 0xff43;

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

const MINIMUM_TEMPERATURE_IN_KELVIN: u16 = 2700;
const MAXIMUM_TEMPERATURE_IN_KELVIN: u16 = 6500;

pub fn get_connected_devices(api: HidApi, serial_number: Option<String>) -> Vec<Device> {
    let hid_devices = api.device_list();
    let litra_devices = hid_devices
        .into_iter()
        .filter(|device| {
            device.vendor_id() == VENDOR_ID
                && PRODUCT_IDS.contains(&device.product_id())
                && device.usage_page() == USAGE_PAGE
        })
        .filter(|device| {
            serial_number.is_none()
                || serial_number.as_ref().is_some_and(|expected| {
                    device
                        .serial_number()
                        .is_some_and(|actual| actual == expected)
                })
        });

    return litra_devices
        .filter_map(|device| match api.open_path(device.path()) {
            Ok(device_handle) => Some((device, device_handle)),
            Err(err) => {
                println!("Failed to open device {:?}: {:?}", device.path(), err);
                None
            }
        })
        .map(|(device, device_handle)| {
            let device_type = get_device_type(device.product_id());
            let is_on = is_on(&device_handle, &device_type);
            let brightness_in_lumen = get_brightness_in_lumen(&device_handle, &device_type);
            let temperature_in_kelvin = get_temperature_in_kelvin(&device_handle, &device_type);
            let minimum_brightness_in_lumen = get_minimum_brightness_in_lumen(&device_type);
            let maximum_brightness_in_lumen = get_maximum_brightness_in_lumen(&device_type);

            Device {
                serial_number: device.serial_number().unwrap_or("").to_string(),
                device_type: device_type,
                is_on: is_on,
                brightness_in_lumen: brightness_in_lumen,
                temperature_in_kelvin: temperature_in_kelvin,
                device_handle: device_handle,
                minimum_brightness_in_lumen: minimum_brightness_in_lumen,
                maximum_brightness_in_lumen: maximum_brightness_in_lumen,
                minimum_temperature_in_kelvin: MINIMUM_TEMPERATURE_IN_KELVIN,
                maximum_temperature_in_kelvin: MAXIMUM_TEMPERATURE_IN_KELVIN,
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

pub fn is_on(device_handle: &HidDevice, device_type: &DeviceType) -> bool {
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

pub fn get_brightness_in_lumen(device_handle: &HidDevice, device_type: &DeviceType) -> u16 {
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

pub fn get_temperature_in_kelvin(device_handle: &HidDevice, device_type: &DeviceType) -> u16 {
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

pub fn turn_on(device_handle: &HidDevice, device_type: &DeviceType) {
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

pub fn turn_off(device_handle: &HidDevice, device_type: &DeviceType) {
    let message = generate_turn_off_bytes(device_type);

    device_handle.write(&message).unwrap();
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

pub fn set_brightness_in_lumen(
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

pub fn set_temperature_in_kelvin(
    device_handle: &HidDevice,
    device_type: &DeviceType,
    temperature_in_kelvin: u16,
) {
    let message = generate_set_temperature_in_kelvin_bytes(device_type, temperature_in_kelvin);

    device_handle.write(&message).unwrap();
}
