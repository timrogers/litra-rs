//! Library to query and control your Logitech Litra lights.
//!
//! # Usage
//!
//! ```
//! use litra::Litra;
//!
//! let context = Litra::new().expect("Failed to initialize litra.");
//! for device in context.get_connected_devices() {
//!     println!("Device {:?}", device.device_type());
//!     if let Ok(handle) = device.open(&context) {
//!         println!("| - Is on: {}", handle.is_on()
//!             .map(|on| if on { "yes" } else { "no" })
//!             .unwrap_or("unknown"));
//!     }
//! }
//! ```

#![warn(unsafe_code)]
#![warn(missing_docs)]
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

use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};
use std::error::Error;
use std::fmt;

/// Litra context.
///
/// This can be used to list available devices.
pub struct Litra(HidApi);

impl fmt::Debug for Litra {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Litra").finish()
    }
}

impl Litra {
    /// Initialize a new Litra context.
    pub fn new() -> DeviceResult<Self> {
        Ok(HidApi::new().map(Litra)?)
    }

    /// Returns an [`Iterator`] of connected devices supported by this library.
    pub fn get_connected_devices(&self) -> impl Iterator<Item = Device<'_>> {
        self.0
            .device_list()
            .filter_map(|device_info| Device::try_from(device_info).ok())
    }

    /// Retrieve the underlying hidapi context.
    #[must_use]
    pub fn hidapi(&self) -> &HidApi {
        &self.0
    }
}

/// The model of the device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    /// Logitech [Litra Glow][glow] streaming light with TrueSoft.
    ///
    /// [glow]: https://www.logitech.com/products/lighting/litra-glow.html
    LitraGlow,
    /// Logitech [Litra Beam][beam] LED streaming key light with TrueSoft.
    ///
    /// [beam]: https://www.logitechg.com/products/cameras-lighting/litra-beam-streaming-light.html
    LitraBeam,
    /// Logitech [Litra Beam LX][beamlx] dual-sided RGB streaming key light.
    ///
    /// [beamlx]: https://www.logitechg.com/products/cameras-lighting/litra-beam-lx-led-light.html
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

/// A device-relatred error.
#[derive(Debug)]
pub enum DeviceError {
    /// Tried to use a device that is not supported.
    Unsupported,
    /// Tried to set an invalid brightness value.
    InvalidBrightness(u16),
    /// Tried to set an invalid temperature value.
    InvalidTemperature(u16),
    /// A [`hidapi`] operation failed.
    HidError(HidError),
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::Unsupported => write!(f, "Device is not supported."),
            DeviceError::InvalidBrightness(value) => {
                write!(f, "Brightness in Lumen '{}' is not supported.", value)
            }
            DeviceError::InvalidTemperature(value) => {
                write!(f, "Temperature in Kelvin '{}' is not supported.", value)
            }
            DeviceError::HidError(error) => write!(f, "HID error occurred: {}", error),
        }
    }
}

impl Error for DeviceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let DeviceError::HidError(error) = self {
            Some(error)
        } else {
            None
        }
    }
}

impl From<HidError> for DeviceError {
    fn from(error: HidError) -> Self {
        DeviceError::HidError(error)
    }
}

/// The [`Result`] of a Litra device operation.
pub type DeviceResult<T> = Result<T, DeviceError>;

/// A device that can be used.
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
    /// The model of the device.
    #[must_use]
    pub fn device_info(&self) -> &DeviceInfo {
        self.device_info
    }

    /// The model of the device.
    #[must_use]
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    /// Opens the device and returns a [`DeviceHandle`] that can be used for getting and setting the
    /// device status.
    pub fn open(&self, context: &Litra) -> DeviceResult<DeviceHandle> {
        let hid_device = self.device_info.open_device(context.hidapi())?;
        Ok(DeviceHandle {
            hid_device,
            device_type: self.device_type,
        })
    }
}

/// The handle of an opened device that can be used for getting and setting the device status.
#[derive(Debug)]
pub struct DeviceHandle {
    hid_device: HidDevice,
    device_type: DeviceType,
}

impl DeviceHandle {
    /// The model of the device.
    #[must_use]
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    /// Queries the current power status of the device. Returns `true` if the device is currently on.
    pub fn is_on(&self) -> DeviceResult<bool> {
        let message = generate_is_on_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        Ok(response_buffer[..response][4] == 1)
    }

    /// Sets the power status of the device. Turns the device on if `true` is passed and turns it
    /// of on `false`.
    pub fn set_on(&self, on: bool) -> DeviceResult<()> {
        let message = generate_set_on_bytes(&self.device_type, on);

        self.hid_device.write(&message)?;
        Ok(())
    }

    /// Queries the device's current brightness in Lumen.
    pub fn brightness_in_lumen(&self) -> DeviceResult<u16> {
        let message = generate_get_brightness_in_lumen_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        Ok(response_buffer[..response][5].into())
    }

    /// Sets the device's brightness in Lumen.
    pub fn set_brightness_in_lumen(&self, brightness_in_lumen: u16) -> DeviceResult<()> {
        if brightness_in_lumen < self.minimum_brightness_in_lumen()
            || brightness_in_lumen > self.maximum_brightness_in_lumen()
        {
            return Err(DeviceError::InvalidBrightness(brightness_in_lumen));
        }

        let message =
            generate_set_brightness_in_lumen_bytes(&self.device_type, brightness_in_lumen);

        self.hid_device.write(&message)?;
        Ok(())
    }

    /// Returns the minimum brightness supported by the device in Lumen.
    #[must_use]
    pub fn minimum_brightness_in_lumen(&self) -> u16 {
        match self.device_type {
            DeviceType::LitraGlow => 20,
            DeviceType::LitraBeam | DeviceType::LitraBeamLX => 30,
        }
    }

    /// Returns the maximum brightness supported by the device in Lumen.
    #[must_use]
    pub fn maximum_brightness_in_lumen(&self) -> u16 {
        match self.device_type {
            DeviceType::LitraGlow => 250,
            DeviceType::LitraBeam | DeviceType::LitraBeamLX => 400,
        }
    }

    /// Queries the device's current color temperature in Kelvin.
    pub fn temperature_in_kelvin(&self) -> DeviceResult<u16> {
        let message = generate_get_temperature_in_kelvin_bytes(&self.device_type);

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;
        Ok(u16::from(response_buffer[..response][4]) * 256
            + u16::from(response_buffer[..response][5]))
    }

    /// Sets the device's color temperature in Kelvin.
    pub fn set_temperature_in_kelvin(&self, temperature_in_kelvin: u16) -> DeviceResult<()> {
        if self.minimum_temperature_in_kelvin() < temperature_in_kelvin
            || temperature_in_kelvin > self.maximum_temperature_in_kelvin()
            || (temperature_in_kelvin % 100) != 0
        {
            return Err(DeviceError::InvalidTemperature(temperature_in_kelvin));
        }

        let message =
            generate_set_temperature_in_kelvin_bytes(&self.device_type, temperature_in_kelvin);

        self.hid_device.write(&message)?;
        Ok(())
    }

    /// Returns the minimum color temperature supported by the device in Kelvin.
    #[must_use]
    pub fn minimum_temperature_in_kelvin(&self) -> u16 {
        MINIMUM_TEMPERATURE_IN_KELVIN
    }

    /// Returns the maximum color temperature supported by the device in Kelvin.
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

fn generate_is_on_bytes(device_type: &DeviceType) -> [u8; 20] {
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

fn generate_set_on_bytes(device_type: &DeviceType, on: bool) -> [u8; 20] {
    let on_byte = if on { 0x01 } else { 0x00 };
    match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, 0x1c, on_byte, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, 0x1c, on_byte, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
