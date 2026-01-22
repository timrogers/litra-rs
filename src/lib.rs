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
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Litra context.
///
/// This can be used to list available devices.
pub struct Litra {
    hidapi: HidApi,
    sorted_device_paths: Vec<String>,
    device_path_indices: HashMap<String, usize>,
}

impl fmt::Debug for Litra {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Litra")
            .field("sorted_device_paths", &self.sorted_device_paths)
            .finish_non_exhaustive()
    }
}

impl Litra {
    /// Initialize a new Litra context.
    pub fn new() -> DeviceResult<Self> {
        let hidapi = HidApi::new()?;
        #[cfg(target_os = "macos")]
        hidapi.set_open_exclusive(false);

        // Build the initial sorted device paths cache
        let (sorted_device_paths, device_path_indices) = Self::build_device_cache(&hidapi);

        Ok(Litra {
            hidapi,
            sorted_device_paths,
            device_path_indices,
        })
    }

    /// Helper function to build a sorted list of device paths and their index map from the HidApi
    fn build_device_cache(hidapi: &HidApi) -> (Vec<String>, HashMap<String, usize>) {
        let mut paths: Vec<String> = hidapi
            .device_list()
            .filter_map(|device_info| {
                if Device::try_from(device_info).is_ok() {
                    Some(device_info.path().to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();
        paths.sort();

        let indices: HashMap<String, usize> = paths
            .iter()
            .enumerate()
            .map(|(idx, path)| (path.clone(), idx))
            .collect();

        (paths, indices)
    }

    /// Returns an [`Iterator`] of cached connected devices supported by this library. To refresh the list of connected devices, use [`Litra::refresh_connected_devices`].
    pub fn get_connected_devices(&self) -> impl Iterator<Item = Device<'_>> {
        let devices: Vec<Device<'_>> = self
            .hidapi
            .device_list()
            .filter_map(|device_info| Device::try_from(device_info).ok())
            .collect();

        // Create a vector of (device, sort_key) pairs to avoid calling device_path()
        // multiple times during sorting. The sort key uses cached indices for O(1) lookup.
        let mut devices_with_keys: Vec<(Device<'_>, usize)> = devices
            .into_iter()
            .map(|device| {
                let path = device.device_path();
                let sort_key = self
                    .device_path_indices
                    .get(&path)
                    .copied()
                    .unwrap_or(usize::MAX);
                (device, sort_key)
            })
            .collect();

        devices_with_keys.sort_by_key(|(_, key)| *key);
        devices_with_keys.into_iter().map(|(device, _)| device)
    }

    /// Refreshes the list of connected devices, returned by [`Litra::get_connected_devices`].
    pub fn refresh_connected_devices(&mut self) -> DeviceResult<()> {
        self.hidapi.refresh_devices()?;

        // Rebuild the sorted device paths cache after refresh
        let (sorted_device_paths, device_path_indices) = Self::build_device_cache(&self.hidapi);
        self.sorted_device_paths = sorted_device_paths;
        self.device_path_indices = device_path_indices;

        Ok(())
    }

    /// Retrieve the underlying hidapi context.
    #[must_use]
    pub fn hidapi(&self) -> &HidApi {
        &self.hidapi
    }
}

/// The model of the device.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub enum DeviceType {
    /// Logitech [Litra Glow][glow] streaming light with TrueSoft.
    ///
    /// [glow]: https://www.logitech.com/products/lighting/litra-glow.html
    #[serde(rename = "glow")]
    LitraGlow,
    /// Logitech [Litra Beam][beam] LED streaming key light with TrueSoft.
    ///
    /// [beam]: https://www.logitechg.com/products/cameras-lighting/litra-beam-streaming-light.html
    #[serde(rename = "beam")]
    LitraBeam,
    /// Logitech [Litra Beam LX][beamlx] dual-sided RGB streaming key light.
    ///
    /// [beamlx]: https://www.logitechg.com/products/cameras-lighting/litra-beam-lx-led-light.html
    #[serde(rename = "beam_lx")]
    LitraBeamLX,
}

impl DeviceType {
    /// Returns true if this device type has a colorful back side (only Litra Beam LX).
    #[must_use]
    pub fn has_back_side(&self) -> bool {
        *self == DeviceType::LitraBeamLX
    }
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

impl std::str::FromStr for DeviceType {
    type Err = DeviceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_lower = s.to_lowercase().replace(" ", "");
        match s_lower.as_str() {
            "glow" => Ok(DeviceType::LitraGlow),
            "beam" => Ok(DeviceType::LitraBeam),
            "beam_lx" => Ok(DeviceType::LitraBeamLX),
            _ => Err(DeviceError::UnsupportedDeviceType),
        }
    }
}

/// A device-related error.
#[derive(Debug)]
pub enum DeviceError {
    /// Tried to use a device that is not supported.
    Unsupported,
    /// Tried to set an invalid brightness value.
    InvalidBrightness(u16),
    /// Tried to set an invalid temperature value.
    InvalidTemperature(u16),
    /// Tried to set an invalid percentage value.
    InvalidPercentage(u8),
    /// A [`hidapi`] operation failed.
    HidError(HidError),
    /// Tried to parse an unsupported device type.
    UnsupportedDeviceType,
    /// Tried to set an invalid color zone.
    InvalidZone(u8),
    /// Tried to set an invalid color value.
    InvalidColor(String),
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::Unsupported => write!(f, "Device is not supported"),
            DeviceError::InvalidBrightness(value) => {
                write!(f, "Brightness {} lm is not supported", value)
            }
            DeviceError::InvalidTemperature(value) => {
                write!(f, "Temperature {} K is not supported", value)
            }
            DeviceError::HidError(error) => write!(f, "HID error occurred: {}", error),
            DeviceError::UnsupportedDeviceType => write!(f, "Unsupported device type"),
            DeviceError::InvalidZone(zone_id) => write!(
                f,
                "Back color zone {} is not valid. Only zones 1-7 are allowed.",
                zone_id
            ),
            DeviceError::InvalidColor(str) => write!(
                f,
                "Back color {} is not valid. Only hexadecimal colors are allowed.",
                str
            ),
            DeviceError::InvalidPercentage(value) => {
                write!(
                    f,
                    "Percentage {}% is not valid. Only values between 0 and 100 are allowed.",
                    value
                )
            }
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

    /// Returns the device path, which is a unique identifier for the device.
    #[must_use]
    pub fn device_path(&self) -> String {
        self.device_info.path().to_string_lossy().to_string()
    }

    /// Opens the device and returns a [`DeviceHandle`] that can be used for getting and setting the
    /// device status. On macOS, this will open the device in non-exclusive mode.
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

    /// The [`HidDevice`] for the device.
    #[must_use]
    pub fn hid_device(&self) -> &HidDevice {
        &self.hid_device
    }

    /// Returns the serial number of the device.
    ///
    /// This may return None if the device doesn't provide a serial number.
    pub fn serial_number(&self) -> DeviceResult<Option<String>> {
        match self.hid_device.get_device_info() {
            Ok(device_info) => {
                if let Some(serial) = device_info.serial_number() {
                    if !serial.is_empty() {
                        return Ok(Some(String::from(serial)));
                    }
                }

                Ok(None)
            }
            Err(error) => Err(DeviceError::HidError(error)),
        }
    }

    /// Returns the unique device path.
    ///
    /// This is a stable identifier that can be used to target a specific device,
    /// even when the device doesn't provide a serial number.
    pub fn device_path(&self) -> DeviceResult<String> {
        match self.hid_device.get_device_info() {
            Ok(device_info) => Ok(device_info.path().to_string_lossy().to_string()),
            Err(error) => Err(DeviceError::HidError(error)),
        }
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

        Ok(u16::from(response_buffer[..response][4]) * 256
            + u16::from(response_buffer[..response][5]))
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
        if temperature_in_kelvin < self.minimum_temperature_in_kelvin()
            || temperature_in_kelvin > self.maximum_temperature_in_kelvin()
            || !temperature_in_kelvin.is_multiple_of(100)
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

    /// Sets the color of one or more of the zones on the colorful back side of the Litra Beam LX. Only Litra Beam LX devices are supported.
    pub fn set_back_color(&self, zone_id: u8, red: u8, green: u8, blue: u8) -> DeviceResult<()> {
        if self.device_type != DeviceType::LitraBeamLX {
            return Err(DeviceError::UnsupportedDeviceType);
        }

        // The device is divided into 7 sections
        if zone_id == 0 || zone_id > 7 {
            return Err(DeviceError::InvalidZone(zone_id));
        }

        // The device seems to freak out if these values are 0, prevent it
        let message = generate_set_back_color_bytes(zone_id, red.max(1), green.max(1), blue.max(1));

        self.hid_device.write(&message)?;
        self.hid_device
            .write(&[0x11, 0xff, 0x0C, 0x7B, 0, 0, 1, 0, 0])?;
        Ok(())
    }

    /// Sets the brightness of the colorful back side of the Litra Beam LX to a percentage value. Only Litra Beam LX devices are supported.
    pub fn set_back_brightness_percentage(&self, brightness: u8) -> DeviceResult<()> {
        if self.device_type != DeviceType::LitraBeamLX {
            return Err(DeviceError::UnsupportedDeviceType);
        }
        if brightness == 0 || brightness > 100 {
            return Err(DeviceError::InvalidPercentage(brightness));
        }

        let message = generate_set_back_brightness_percentage_bytes(brightness);

        self.hid_device.write(&message)?;
        Ok(())
    }

    /// Sets the power status of the colorful back side of the Litra Beam LX. Only Litra Beam LX devices are supported.
    /// Turns the device on if `true` is passed and turns it off on `false`.
    pub fn set_back_on(&self, on: bool) -> DeviceResult<()> {
        if self.device_type != DeviceType::LitraBeamLX {
            return Err(DeviceError::UnsupportedDeviceType);
        }
        let message = generate_set_back_on_bytes(on);

        self.hid_device.write(&message)?;
        Ok(())
    }

    /// Queries the current power status of the colorful back side of the Litra Beam LX. Returns `true` if the back light is currently on. Only Litra Beam LX devices are supported.
    pub fn is_back_on(&self) -> DeviceResult<bool> {
        if self.device_type != DeviceType::LitraBeamLX {
            return Err(DeviceError::UnsupportedDeviceType);
        }
        let message = generate_get_back_on_bytes();

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        Ok(response_buffer[..response][4] == 1)
    }

    /// Queries the brightness of the colorful back side of the Litra Beam LX as a percentage. Only Litra Beam LX devices are supported.
    pub fn back_brightness_percentage(&self) -> DeviceResult<u8> {
        if self.device_type != DeviceType::LitraBeamLX {
            return Err(DeviceError::UnsupportedDeviceType);
        }
        let message = generate_get_back_brightness_percentage_bytes();

        self.hid_device.write(&message)?;

        let mut response_buffer = [0x00; 20];
        let response = self.hid_device.read(&mut response_buffer[..])?;

        // The brightness is returned as a 16-bit value but represents a percentage
        let brightness = u16::from(response_buffer[..response][4]) * 256
            + u16::from(response_buffer[..response][5]);
        Ok(brightness as u8)
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

fn generate_set_back_color_bytes(zone_id: u8, red: u8, green: u8, blue: u8) -> [u8; 20] {
    [
        0x11, 0xff, 0x0C, 0x1B, zone_id, red, green, blue, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00,
        0x00, 0x00, 0xFF, 0x00, 0x00, 0x00,
    ]
}

fn generate_set_back_brightness_percentage_bytes(brightness: u8) -> [u8; 20] {
    [
        0x11, 0xff, 0x0a, 0x2b, 0x00, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

fn generate_set_back_on_bytes(on: bool) -> [u8; 20] {
    [
        0x11,
        0xff,
        0x0a,
        0x4b,
        if on { 1 } else { 0 },
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ]
}

fn generate_get_back_on_bytes() -> [u8; 20] {
    [
        0x11, 0xff, 0x0a, 0x3b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]
}

fn generate_get_back_brightness_percentage_bytes() -> [u8; 20] {
    [
        0x11, 0xff, 0x0a, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ]
}
