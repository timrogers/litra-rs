#![warn(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use hidapi::{DeviceInfo, HidApi, HidDevice, HidResult};
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    LitraGlow,
    LitraBeam,
    LitraBeamLX,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::LitraGlow => write!(f, "Litra Glow"),
            DeviceType::LitraBeam => write!(f, "Litra Beam"),
            DeviceType::LitraBeamLX => write!(f, "Litra Beam LX"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceError {
    /// Tried to use a device that is not supported.
    Unsupported,
}

#[derive(Debug)]
pub struct Device<'a> {
    device_info: &'a DeviceInfo,
    device_type: DeviceType,
}

impl<'a> TryFrom<&'a DeviceInfo> for Device<'a> {
    type Error = DeviceError;

    fn try_from(device_info: &'a DeviceInfo) -> Result<Self, DeviceError> {
        if device_info.vendor_id() != VENDOR_ID || device_info.usage_page() != USAGE_PAGE {
            return Err(DeviceError::Unsupported);
        }
        device_type_from_product_id(device_info.product_id())
            .map(|device_type| Device {
                device_info,
                device_type,
            })
            .ok_or(DeviceError::Unsupported)
    }
}

impl Device<'_> {
    #[must_use]
    pub fn serial_number(&self) -> Option<&str> {
        self.device_info.serial_number()
    }

    #[must_use]
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    pub fn open(&self, api: &HidApi) -> HidResult<DeviceHandle> {
        self.device_info
            .open_device(api)
            .map(|hid_device| DeviceHandle {
                hid_device,
                device_type: self.device_type,
            })
    }
}

#[derive(Debug)]
pub struct DeviceHandle {
    hid_device: HidDevice,
    device_type: DeviceType,
}

impl DeviceHandle {
    #[must_use]
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    pub fn is_enabled(&self) -> HidResult<bool> {
        let message = generate_is_enabled_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        Ok(response_buffer[..response][4] == 1)
    }

    pub fn set_enabled(&self, enabled: bool) -> HidResult<()> {
        let message = generate_set_enabled_bytes(&self.device_type, enabled);

        self.hid_device.write(&message)?;
        Ok(())
    }

    pub fn brightness_in_lumen(&self) -> HidResult<u16> {
        let message = generate_get_brightness_in_lumen_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        Ok(response_buffer[..response][5].into())
    }

    pub fn set_brightness_in_lumen(&self, brightness_in_lumen: u16) -> HidResult<()> {
        let message =
            generate_set_brightness_in_lumen_bytes(&self.device_type, brightness_in_lumen);

        self.hid_device.write(&message)?;
        Ok(())
    }

    #[must_use]
    pub fn minimum_brightness_in_lumen(&self) -> u16 {
        match self.device_type {
            DeviceType::LitraGlow => 20,
            DeviceType::LitraBeam | DeviceType::LitraBeamLX => 30,
        }
    }

    #[must_use]
    pub fn maximum_brightness_in_lumen(&self) -> u16 {
        match self.device_type {
            DeviceType::LitraGlow => 250,
            DeviceType::LitraBeam | DeviceType::LitraBeamLX => 400,
        }
    }

    pub fn temperature_in_kelvin(&self) -> HidResult<u16> {
        let message = generate_get_temperature_in_kelvin_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;
        Ok(u16::from(response_buffer[..response][4]) * 256
            + u16::from(response_buffer[..response][5]))
    }

    pub fn set_temperature_in_kelvin(&self, temperature_in_kelvin: u16) -> HidResult<()> {
        let message =
            generate_set_temperature_in_kelvin_bytes(&self.device_type, temperature_in_kelvin);

        self.hid_device.write(&message)?;
        Ok(())
    }

    #[must_use]
    pub fn minimum_temperature_in_kelvin(&self) -> u16 {
        MINIMUM_TEMPERATURE_IN_KELVIN
    }

    #[must_use]
    pub fn maximum_temperature_in_kelvin(&self) -> u16 {
        MAXIMUM_TEMPERATURE_IN_KELVIN
    }
}

const VENDOR_ID: u16 = 0x046d;
const USAGE_PAGE: u16 = 0xff43;

fn device_type_from_product_id(product_id: u16) -> Option<DeviceType> {
    match product_id {
        0xc900 => DeviceType::LitraGlow.into(),
        0xc901 => DeviceType::LitraBeam.into(),
        0xb901 => DeviceType::LitraBeam.into(),
        0xc903 => DeviceType::LitraBeamLX.into(),
        _ => None,
    }
}

const MINIMUM_TEMPERATURE_IN_KELVIN: u16 = 2700;
const MAXIMUM_TEMPERATURE_IN_KELVIN: u16 = 6500;

pub fn get_connected_devices<'a>(
    api: &'a HidApi,
    serial_number: Option<&'a str>,
) -> impl Iterator<Item = Device<'a>> + 'a {
    api.device_list()
        .filter_map(|device_info| Device::try_from(device_info).ok())
        .filter(move |device| {
            serial_number.is_none()
                || serial_number.as_ref().is_some_and(|expected| {
                    device
                        .serial_number()
                        .is_some_and(|actual| &actual == expected)
                })
        })
}

fn generate_is_enabled_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    }
}

fn generate_get_brightness_in_lumen_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    }
}

fn generate_get_temperature_in_kelvin_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    }
}

fn generate_set_enabled_bytes(device_type: &DeviceType, enabled: bool) -> [u8; 20] {
    let enabled_byte = if enabled { 0x01 } else { 0x00 };
    match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => [
            0x11,
            0xff,
            0x04,
            0x1c,
            enabled_byte,
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
            0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11,
            0xff,
            0x06,
            0x1c,
            enabled_byte,
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
            0x00,
        ],
    }
}

fn generate_set_brightness_in_lumen_bytes(
    device_type: &DeviceType,
    brightness_in_lumen: u16,
) -> [u8; 20] {
    let brightness_bytes = brightness_in_lumen.to_be_bytes();

    match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => [
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

fn generate_set_temperature_in_kelvin_bytes(
    device_type: &DeviceType,
    temperature_in_kelvin: u16,
) -> [u8; 20] {
    let temperature_bytes = temperature_in_kelvin.to_be_bytes();

    match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => [
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
