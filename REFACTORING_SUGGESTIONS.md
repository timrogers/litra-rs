# Code Duplication Refactoring Suggestions

This document provides a comprehensive analysis of code duplication in the litra-rs codebase and suggests concrete refactoring approaches to reduce it.

## Executive Summary

The codebase contains several patterns of code duplication across three main files:
- **lib.rs**: HID command generation functions
- **mcp.rs**: Parameter struct definitions
- **main.rs**: Command handler functions

Total estimated lines of duplicated code: **~400 lines** that could be reduced to **~150 lines** (62.5% reduction).

---

## 1. HID Command Generation Functions (lib.rs)

### Current State

**Lines**: 514-673 (160 lines)

Six functions generate HID command byte arrays with very similar structure:
- `generate_is_on_bytes()`
- `generate_get_brightness_in_lumen_bytes()`
- `generate_get_temperature_in_kelvin_bytes()`
- `generate_set_on_bytes()`
- `generate_set_brightness_in_lumen_bytes()`
- `generate_set_temperature_in_kelvin_bytes()`

Each function:
1. Matches on `DeviceType` to determine prefix (0x04 for Glow/Beam, 0x06 for BeamLX)
2. Creates a 20-byte array with pattern: `[0x11, 0xff, prefix, command, data...]`
3. Fills remaining bytes with 0x00

### Duplication Analysis

**Duplicated pattern** (appears 6 times):
```rust
match device_type {
    DeviceType::LitraGlow | DeviceType::LitraBeam => [
        0x11, 0xff, 0x04, COMMAND_BYTE, /* ... */
    ],
    DeviceType::LitraBeamLX => [
        0x11, 0xff, 0x06, COMMAND_BYTE, /* ... */
    ],
}
```

### Recommended Refactoring

**Approach 1: Helper Function (Minimal Change)**

Create a helper function to build command arrays:

```rust
/// Builds a HID command byte array for the given device type and command.
/// 
/// # Arguments
/// * `device_type` - The type of device to generate the command for
/// * `command` - The command byte (e.g., 0x01 for is_on)
/// * `data` - Optional data bytes to include after the command byte
fn build_hid_command(device_type: &DeviceType, command: u8, data: &[u8]) -> [u8; 20] {
    let prefix = match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => 0x04,
        DeviceType::LitraBeamLX => 0x06,
    };
    
    let mut bytes = [0x00; 20];
    bytes[0] = 0x11;
    bytes[1] = 0xff;
    bytes[2] = prefix;
    bytes[3] = command;
    
    // Copy data bytes starting at position 4
    let data_len = data.len().min(16); // Max 16 bytes of data
    bytes[4..4 + data_len].copy_from_slice(&data[..data_len]);
    
    bytes
}

// Then simplify each function:
fn generate_is_on_bytes(device_type: &DeviceType) -> [u8; 20] {
    build_hid_command(device_type, 0x01, &[])
}

fn generate_get_brightness_in_lumen_bytes(device_type: &DeviceType) -> [u8; 20] {
    build_hid_command(device_type, 0x31, &[])
}

fn generate_set_on_bytes(device_type: &DeviceType, on: bool) -> [u8; 20] {
    build_hid_command(device_type, 0x1c, &[if on { 0x01 } else { 0x00 }])
}

fn generate_set_brightness_in_lumen_bytes(
    device_type: &DeviceType,
    brightness_in_lumen: u16,
) -> [u8; 20] {
    let brightness_bytes = brightness_in_lumen.to_be_bytes();
    build_hid_command(device_type, 0x4c, &brightness_bytes)
}
```

**Estimated Impact**:
- Reduces ~160 lines to ~80 lines (50% reduction)
- Improves maintainability: changes to command structure only need updating in one place
- Makes the command protocol more explicit and documented

**Approach 2: Builder Pattern (More Comprehensive)**

```rust
/// Builder for HID commands with a fluent interface
struct HidCommandBuilder {
    bytes: [u8; 20],
}

impl HidCommandBuilder {
    fn new(device_type: &DeviceType, command: u8) -> Self {
        let prefix = match device_type {
            DeviceType::LitraGlow | DeviceType::LitraBeam => 0x04,
            DeviceType::LitraBeamLX => 0x06,
        };
        
        let mut bytes = [0x00; 20];
        bytes[0] = 0x11;
        bytes[1] = 0xff;
        bytes[2] = prefix;
        bytes[3] = command;
        
        Self { bytes }
    }
    
    fn with_data(mut self, data: &[u8]) -> Self {
        let len = data.len().min(16);
        self.bytes[4..4 + len].copy_from_slice(&data[..len]);
        self
    }
    
    fn build(self) -> [u8; 20] {
        self.bytes
    }
}
```

---

## 2. MCP Parameter Structs (mcp.rs)

### Current State

**Lines**: 36-100 (65 lines)

Six parameter structs all share these fields:
- `LitraToolParams`
- `LitraBrightnessParams`
- `LitraTemperatureParams`
- `LitraBackToolParams`
- `LitraBackBrightnessParams`
- `LitraBackColorParams`

**Duplicated fields** (appear 6 times with same documentation):
```rust
/// The serial number of the device to target (optional - if not specified, all devices are targeted)
pub serial_number: Option<String>,
/// The device path to target (optional - useful for devices that don't show a serial number)
pub device_path: Option<String>,
```

### Recommended Refactoring

**Approach 1: Base Struct with Composition**

```rust
/// Common device targeting parameters used across MCP tools
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct DeviceTarget {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraToolParams {
    #[serde(flatten)]
    pub target: DeviceTarget,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    #[serde(flatten)]
    pub target: DeviceTarget,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
    /// The brightness value to set in lumens (use either this or percentage, not both)
    pub value: Option<u16>,
    /// The brightness as a percentage of maximum brightness (use either this or value, not both)
    pub percentage: Option<u8>,
}

// And so on...
```

**Approach 2: Macro-based Generation**

```rust
macro_rules! device_params {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $ty:ty
            ),*
        }
    ) => {
        $(#[$meta])*
        #[derive(serde::Deserialize, schemars::JsonSchema)]
        pub struct $name {
            /// The serial number of the device to target (optional - if not specified, all devices are targeted)
            pub serial_number: Option<String>,
            /// The device path to target (optional - useful for devices that don't show a serial number)
            pub device_path: Option<String>,
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )*
        }
    };
}

// Usage:
device_params! {
    LitraTemperatureParams {
        /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
        device_type: Option<String>,
        /// The temperature value in Kelvin (must be a multiple of 100 between 2700K and 6500K)
        value: u16
    }
}
```

**Estimated Impact**:
- Reduces ~65 lines to ~40 lines (38% reduction)
- Single source of truth for device targeting documentation
- Makes it easier to add new device targeting options (e.g., device index)

---

## 3. Toggle Command Logic (main.rs)

### Current State

**Lines**: 874-896, 1163-1186 (46 lines total)

Two nearly identical functions:
- `handle_toggle_command()` - toggles main light
- `handle_back_toggle_command()` - toggles back light

**Duplicated logic**:
```rust
// Get context to work with devices
let context = Litra::new()?;

// Get all matched devices
let devices = get_all_supported_devices(&context, serial_number, device_path, device_type)?;
if devices.is_empty() {
    return Err(CliError::DeviceNotFound);
}

// Toggle each device individually
for device_handle in devices {
    if let Ok(is_on) = device_handle.is_on() {  // or .is_back_on()
        let _ = device_handle.set_on(!is_on);    // or .set_back_on()
    }
}
Ok(())
```

### Recommended Refactoring

**Approach: Generic Toggle Helper**

```rust
/// Generic toggle helper that works with any getter/setter pair
fn handle_toggle<G, S>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    get_state: G,
    set_state: S,
) -> CliResult
where
    G: Fn(&DeviceHandle) -> DeviceResult<bool>,
    S: Fn(&DeviceHandle, bool) -> DeviceResult<()>,
{
    let context = Litra::new()?;
    let devices = get_all_supported_devices(&context, serial_number, device_path, device_type)?;
    
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    for device_handle in devices {
        if let Ok(is_on) = get_state(&device_handle) {
            let _ = set_state(&device_handle, !is_on);
        }
    }
    Ok(())
}

// Then simplify both functions:
fn handle_toggle_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
) -> CliResult {
    handle_toggle(
        serial_number,
        device_path,
        device_type,
        |handle| handle.is_on(),
        |handle, on| handle.set_on(on),
    )
}

fn handle_back_toggle_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
) -> CliResult {
    handle_toggle(
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
        |handle| handle.is_back_on(),
        |handle, on| handle.set_back_on(on),
    )
}
```

**Estimated Impact**:
- Reduces ~46 lines to ~30 lines (35% reduction)
- Makes pattern reusable for future toggle operations
- Improves type safety through generic constraints

---

## 4. Back Brightness Adjustment (main.rs)

### Current State

**Lines**: 1188-1243 (56 lines)

Two nearly identical functions:
- `handle_back_brightness_up_command()`
- `handle_back_brightness_down_command()`

**Duplicated pattern**:
```rust
let context = Litra::new()?;
let devices = get_all_supported_devices(&context, serial_number, device_path, Some(&DeviceType::LitraBeamLX))?;

if devices.is_empty() {
    return Err(CliError::DeviceNotFound);
}

for device_handle in devices {
    if let Ok(current_brightness) = device_handle.back_brightness_percentage() {
        let new_brightness = current_brightness.saturating_add(percentage).min(100);  // or saturating_sub
        let _ = device_handle.set_back_brightness_percentage(new_brightness);
    }
}
```

### Recommended Refactoring

**Approach 1: Direction Enum**

```rust
enum BrightnessAdjustment {
    Increase,
    Decrease,
}

fn handle_back_brightness_adjust(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
    direction: BrightnessAdjustment,
) -> CliResult {
    let context = Litra::new()?;
    let devices = get_all_supported_devices(
        &context,
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
    )?;
    
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    for device_handle in devices {
        if let Ok(current_brightness) = device_handle.back_brightness_percentage() {
            let new_brightness = match direction {
                BrightnessAdjustment::Increase => current_brightness.saturating_add(percentage).min(100),
                BrightnessAdjustment::Decrease => current_brightness.saturating_sub(percentage).max(1),
            };
            let _ = device_handle.set_back_brightness_percentage(new_brightness);
        }
    }
    Ok(())
}

fn handle_back_brightness_up_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    handle_back_brightness_adjust(serial_number, device_path, percentage, BrightnessAdjustment::Increase)
}

fn handle_back_brightness_down_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    handle_back_brightness_adjust(serial_number, device_path, percentage, BrightnessAdjustment::Decrease)
}
```

**Approach 2: Generic Adjustment Helper**

```rust
/// Generic helper for adjusting numeric device properties
fn handle_adjustment<G, S>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    amount: i16,  // Signed to allow both increase/decrease
    min: u8,
    max: u8,
    get_current: G,
    set_new: S,
) -> CliResult
where
    G: Fn(&DeviceHandle) -> DeviceResult<u8>,
    S: Fn(&DeviceHandle, u8) -> DeviceResult<()>,
{
    let context = Litra::new()?;
    let devices = get_all_supported_devices(&context, serial_number, device_path, device_type)?;
    
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }

    for device_handle in devices {
        if let Ok(current) = get_current(&device_handle) {
            let new_value = (current as i16 + amount).clamp(min as i16, max as i16) as u8;
            let _ = set_new(&device_handle, new_value);
        }
    }
    Ok(())
}
```

**Estimated Impact**:
- Reduces ~56 lines to ~35 lines (37% reduction)
- Pattern reusable for front brightness adjustment too
- Single place to change adjustment logic

---

## 5. CLI Argument Definitions (main.rs)

### Current State

Multiple command variants in the `Commands` enum repeat the same argument definitions:

```rust
#[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, conflicts_with_all = ["device_path", "device_type"])]
serial_number: Option<String>,
#[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP, conflicts_with_all = ["serial_number", "device_type"])]
device_path: Option<String>,
#[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
device_type: Option<DeviceType>,
```

This pattern appears in: `On`, `Off`, `Toggle`, `Brightness`, `BrightnessUp`, `BrightnessDown`, `Temperature`, `TemperatureUp`, `TemperatureDown` (9 times).

### Recommended Refactoring

**Note**: This is harder to refactor with clap's derive API, but here are two approaches:

**Approach 1: Struct Composition (requires clap flatten)**

```rust
#[derive(Debug, Parser)]
struct DeviceSelector {
    #[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, conflicts_with_all = ["device_path", "device_type"])]
    serial_number: Option<String>,
    #[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP, conflicts_with_all = ["serial_number", "device_type"])]
    device_path: Option<String>,
    #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
    device_type: Option<DeviceType>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    On {
        #[clap(flatten)]
        device: DeviceSelector,
    },
    Off {
        #[clap(flatten)]
        device: DeviceSelector,
    },
    // etc...
}
```

**Approach 2: Keep as-is with documentation**

Given clap's limitations, it might be best to keep the current structure but add a clear comment:

```rust
// Note: The following device selector arguments are intentionally duplicated across
// commands rather than using a shared struct, as this provides better CLI help
// messages and clearer documentation for each command. The pattern is:
//   - serial_number: Target by serial (mutually exclusive)
//   - device_path: Target by path (mutually exclusive)
//   - device_type: Target by type (mutually exclusive)
```

**Estimated Impact**:
- If using flatten: ~150 lines reduced to ~50 lines (67% reduction)
- Trade-off: May impact CLI help text clarity
- Recommendation: Only refactor if help text remains clear

---

## Implementation Priority

### High Priority (Recommended to implement)

1. **HID Command Generation (lib.rs)** 
   - **Impact**: High (50% reduction, 80 lines saved)
   - **Risk**: Low (internal implementation detail)
   - **Effort**: Small (2-3 hours)

2. **Toggle Command Logic (main.rs)**
   - **Impact**: Medium (35% reduction, 16 lines saved)
   - **Risk**: Low (well-tested pattern)
   - **Effort**: Small (1-2 hours)

3. **Back Brightness Adjustment (main.rs)**
   - **Impact**: Medium (37% reduction, 21 lines saved)
   - **Risk**: Low (simple refactor)
   - **Effort**: Small (1-2 hours)

### Medium Priority (Consider implementing)

4. **MCP Parameter Structs (mcp.rs)**
   - **Impact**: Medium (38% reduction, 25 lines saved)
   - **Risk**: Medium (affects public MCP API)
   - **Effort**: Medium (2-3 hours including testing)
   - **Note**: Ensure JSON schema generation still works correctly with `#[serde(flatten)]`

### Low Priority (Document but don't implement)

5. **CLI Argument Definitions (main.rs)**
   - **Impact**: High (67% reduction if implemented)
   - **Risk**: High (may reduce CLI UX quality)
   - **Effort**: Medium
   - **Recommendation**: Add documentation comment instead

---

## Testing Recommendations

For each refactoring:

1. **Unit Tests**: Ensure existing tests pass
2. **Integration Tests**: Test actual device commands work correctly
3. **CLI Tests**: Verify command-line interface behavior unchanged
4. **MCP Tests**: Verify MCP server responses match expected format

Specific test areas:
- HID command bytes match exactly (critical for hardware communication)
- Error handling preserved (device not found, invalid values, etc.)
- Edge cases (min/max values, saturation, etc.)

---

## Summary

### Total Impact

- **Lines reduced**: ~400 → ~150 (62.5% reduction)
- **Maintenance benefit**: Changes to common patterns now in single location
- **Code clarity**: Explicit abstractions make patterns more obvious
- **Type safety**: Generic helpers provide compile-time guarantees

### Next Steps

1. Review and approve this analysis
2. Implement high-priority refactorings first
3. Add tests for each refactoring
4. Review medium-priority items for API impact
5. Document low-priority items as known patterns

### Notes

- All refactorings maintain backward compatibility
- No public API changes (except optionally for MCP structs)
- All hardware communication protocols remain identical
- Existing tests should continue to pass
