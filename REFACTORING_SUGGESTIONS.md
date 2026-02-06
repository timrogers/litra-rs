# Code Duplication Refactoring Suggestions

This document outlines several opportunities to refactor the codebase to reduce code duplication and improve maintainability.

## Executive Summary

Four major areas of code duplication have been identified:

1. **HID Command Generation** (lib.rs): 6 functions with nearly identical structure
2. **CLI Command Arguments** (main.rs): Device selector fields repeated across 13+ commands
3. **MCP Parameter Structs** (mcp.rs): Common fields repeated across 6 structs
4. **Back Light Command Handlers** (main.rs): Boilerplate code repeated in 3 handlers

Each section below provides detailed analysis, suggested refactoring approaches, and example implementations.

---

## 1. HID Command Generation Functions (lib.rs, lines 514-725)

### Current State

Six `generate_*_bytes` functions follow an identical pattern:
- `generate_is_on_bytes` (lines 514-525)
- `generate_get_brightness_in_lumen_bytes` (lines 527-538)
- `generate_get_temperature_in_kelvin_bytes` (lines 540-551)
- `generate_set_on_bytes` (lines 553-565)
- `generate_set_brightness_in_lumen_bytes` (lines 567-619)
- `generate_set_temperature_in_kelvin_bytes` (lines 621-673)

Each function:
1. Matches on `device_type` with two branches: `LitraGlow | LitraBeam` vs `LitraBeamLX`
2. Returns a 20-byte array with structure: `[0x11, 0xff, prefix, command, data...]`
3. Only differs in:
   - **Prefix byte**: `0x04` for Glow/Beam, `0x06` for BeamLX
   - **Command byte**: Unique per function (e.g., `0x01`, `0x31`, `0x81`, etc.)
   - **Data bytes**: Operation-specific payload

### Duplication Impact

- **~160 lines** of highly repetitive code
- Each device type change requires updates in 6+ locations
- Error-prone when adding new commands

### Refactoring Suggestion

**Option A: Single Helper Function**

Create a generic command builder:

```rust
fn build_command_bytes(device_type: &DeviceType, command: u8, data: &[u8]) -> [u8; 20] {
    let prefix = match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => 0x04,
        DeviceType::LitraBeamLX => 0x06,
    };
    
    let mut bytes = [0x00; 20];
    bytes[0] = 0x11;
    bytes[1] = 0xff;
    bytes[2] = prefix;
    bytes[3] = command;
    
    // Copy data bytes (up to 16 bytes)
    let data_len = data.len().min(16);
    bytes[4..4 + data_len].copy_from_slice(&data[..data_len]);
    
    bytes
}
```

Then simplify each function:

```rust
fn generate_is_on_bytes(device_type: &DeviceType) -> [u8; 20] {
    build_command_bytes(device_type, 0x01, &[])
}

fn generate_get_brightness_in_lumen_bytes(device_type: &DeviceType) -> [u8; 20] {
    build_command_bytes(device_type, 0x31, &[])
}

fn generate_set_on_bytes(device_type: &DeviceType, on: bool) -> [u8; 20] {
    build_command_bytes(device_type, 0x1c, &[if on { 0x01 } else { 0x00 }])
}

fn generate_set_brightness_in_lumen_bytes(
    device_type: &DeviceType,
    brightness_in_lumen: u16,
) -> [u8; 20] {
    let brightness_bytes = brightness_in_lumen.to_be_bytes();
    build_command_bytes(device_type, 0x4c, &brightness_bytes)
}

// Similar simplifications for other functions...
```

**Benefits:**
- Reduces ~160 lines to ~50 lines (~68% reduction)
- Single point of change for protocol modifications
- Clear separation of command structure vs. command data
- Easier to add new commands

**Option B: Command Constants + Builder Pattern**

Define command codes as constants and use a builder:

```rust
const CMD_IS_ON: u8 = 0x01;
const CMD_GET_BRIGHTNESS: u8 = 0x31;
const CMD_GET_TEMPERATURE: u8 = 0x81;
const CMD_SET_ON: u8 = 0x1c;
const CMD_SET_BRIGHTNESS: u8 = 0x4c;
const CMD_SET_TEMPERATURE: u8 = 0x9c;

struct CommandBuilder {
    device_type: DeviceType,
}

impl CommandBuilder {
    fn new(device_type: DeviceType) -> Self {
        Self { device_type }
    }
    
    fn build(&self, command: u8, data: &[u8]) -> [u8; 20] {
        // Same implementation as build_command_bytes
    }
}
```

**Benefits:**
- Named constants improve code readability
- Builder pattern encapsulates device type
- Fluent API possible for complex commands

### Recommendation

**Implement Option A** as it provides the best balance of simplicity and maintainability. Option B is more sophisticated but adds complexity that may not be needed for this use case.

### Migration Strategy

1. Add `build_command_bytes` function
2. Refactor one `generate_*_bytes` function at a time
3. Run tests after each change to ensure correctness
4. Update documentation to explain the protocol structure

---

## 2. CLI Command Arguments (main.rs, lines 104-501)

### Current State

The `Commands` enum has 13+ variants, each with repeated device selector fields:

```rust
On {
    #[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, conflicts_with_all = [...])]
    serial_number: Option<String>,
    #[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP, conflicts_with_all = [...])]
    device_path: Option<String>,
    #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = ..., conflicts_with_all = [...])]
    device_type: Option<DeviceType>,
},
Off { /* same 3 fields */ },
Toggle { /* same 3 fields */ },
// ... 10+ more variants with same fields
```

### Duplication Impact

- **~130 lines** of repeated field declarations
- Changes to device targeting require updates in 13+ locations
- Inconsistency risk if one variant is updated differently

### Refactoring Suggestion

**Use `#[clap(flatten)]` with a shared struct:**

```rust
/// Device selector arguments used across multiple commands
#[derive(Debug, Clone, clap::Args)]
#[clap(group(ArgGroup::new("device_selector").multiple(false)))]
struct DeviceSelector {
    /// Specify the device to target by its serial number
    #[clap(
        long,
        short,
        group = "device_selector"
    )]
    serial_number: Option<String>,
    
    /// Specify the device to target by its path (useful for devices that don't show a serial number)
    #[clap(
        long,
        short('p'),
        group = "device_selector"
    )]
    device_path: Option<String>,
    
    /// Specify the device to target by its type (`glow`, `beam` or `beam_lx`)
    #[clap(
        long,
        short('t'),
        value_parser = DeviceTypeValueParser,
        group = "device_selector"
    )]
    device_type: Option<DeviceType>,
}

impl DeviceSelector {
    fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }
    
    fn device_path(&self) -> Option<&str> {
        self.device_path.as_deref()
    }
    
    fn device_type(&self) -> Option<&DeviceType> {
        self.device_type.as_ref()
    }
}

/// For back light commands that only support BeamLX
#[derive(Debug, Clone, clap::Args)]
#[clap(group(ArgGroup::new("device_selector").multiple(false)))]
struct BackDeviceSelector {
    /// Specify the device to target by its serial number
    #[clap(
        long,
        short,
        group = "device_selector"
    )]
    serial_number: Option<String>,
    
    /// Specify the device to target by its path (useful for devices that don't show a serial number)
    #[clap(
        long,
        short('p'),
        group = "device_selector"
    )]
    device_path: Option<String>,
}

impl BackDeviceSelector {
    fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }
    
    fn device_path(&self) -> Option<&str> {
        self.device_path.as_deref()
    }
}

// Then simplify Commands enum:
#[derive(Debug, Subcommand)]
enum Commands {
    /// Turn your Logitech Litra device on
    On {
        #[clap(flatten)]
        device: DeviceSelector,
    },
    
    /// Turn your Logitech Litra device off
    Off {
        #[clap(flatten)]
        device: DeviceSelector,
    },
    
    /// Toggle your Logitech Litra device on or off
    Toggle {
        #[clap(flatten)]
        device: DeviceSelector,
    },
    
    /// Set the brightness of your Logitech Litra device
    #[clap(group = ArgGroup::new("brightness").required(true).multiple(false))]
    Brightness {
        #[clap(flatten)]
        device: DeviceSelector,
        
        #[clap(long, short, group = "brightness")]
        value: Option<u16>,
        
        #[clap(long, short('b'), group = "brightness", value_parser = clap::value_parser!(u8).range(1..=100))]
        percentage: Option<u8>,
    },
    
    // ... other variants simplified similarly
    
    /// Turn the back light on
    BackOn {
        #[clap(flatten)]
        device: BackDeviceSelector,
    },
    
    // ... other back commands
}
```

**Benefits:**
- Reduces ~130 lines to ~40 lines (~69% reduction)
- Single source of truth for device selector behavior
- Easier to add new selector options (e.g., device index)
- Helper methods on structs improve ergonomics
- Consistent conflict handling via ArgGroup

**Trade-offs:**
- Slightly more complex initial setup
- Command handling code needs minor updates to access nested fields

### Migration Strategy

1. Create `DeviceSelector` and `BackDeviceSelector` structs
2. Update one command variant at a time
3. Test CLI parsing after each change
4. Update command handlers to use `device.serial_number()` instead of direct field access
5. Update documentation and help text

---

## 3. MCP Parameter Structs (mcp.rs, lines 36-100)

### Current State

Six parameter structs repeat common device identification fields:

```rust
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraToolParams {
    pub serial_number: Option<String>,
    pub device_path: Option<String>,
    pub device_type: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    pub serial_number: Option<String>,
    pub device_path: Option<String>,
    pub device_type: Option<String>,
    pub value: Option<u16>,
    pub percentage: Option<u8>,
}

// ... 4 more structs with same pattern
```

### Duplication Impact

- **~60 lines** of repeated field declarations
- Changes to device targeting must be applied to 6 structs
- No shared validation or helper logic
- Documentation must be kept in sync across structs

### Refactoring Suggestion

**Option A: Struct Composition (Recommended)**

Use struct embedding to share common fields:

```rust
/// Common device identification fields used across MCP tools
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct DeviceIdentifier {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
}

impl DeviceIdentifier {
    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }
    
    pub fn device_path(&self) -> Option<&str> {
        self.device_path.as_deref()
    }
    
    pub fn device_type(&self) -> Option<DeviceType> {
        self.device_type.as_ref().and_then(|s| DeviceType::from_str(s).ok())
    }
}

/// For back light commands that only support BeamLX
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct BackDeviceIdentifier {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
}

impl BackDeviceIdentifier {
    pub fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_deref()
    }
    
    pub fn device_path(&self) -> Option<&str> {
        self.device_path.as_deref()
    }
}

// Flatten into existing structs
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraToolParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: DeviceIdentifier,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: DeviceIdentifier,
    
    /// The brightness value to set in lumens (use either this or percentage, not both)
    pub value: Option<u16>,
    /// The brightness as a percentage of maximum brightness (use either this or value, not both)
    pub percentage: Option<u8>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraTemperatureParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: DeviceIdentifier,
    
    /// The temperature value in Kelvin (must be a multiple of 100 between 2700K and 6500K)
    pub value: u16,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBackToolParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: BackDeviceIdentifier,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBackBrightnessParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: BackDeviceIdentifier,
    
    /// The brightness as a percentage (1-100)
    pub percentage: u8,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBackColorParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: BackDeviceIdentifier,
    
    /// The color in hex format (e.g., "#FF0000" for red)
    pub hex: String,
    /// The zone ID to target (1-7, optional - if not specified, all zones are targeted)
    pub zone_id: Option<u8>,
}
```

**Benefits:**
- Reduces ~60 lines to ~30 lines (50% reduction)
- Single source of documentation for common fields
- Helper methods provide cleaner API
- Easy to add validation or parsing logic
- Type-safe device type parsing moved to one place

**Updates to MCP tool handlers:**

```rust
async fn litra_on(
    &self,
    Parameters(params): Parameters<LitraToolParams>,
) -> Result<CallToolResult, McpError> {
    match handle_on_command(
        params.device.serial_number(),
        params.device.device_path(),
        params.device.device_type().as_ref(),
    ) {
        // ... rest unchanged
    }
}
```

**Option B: Macro-Based Generation**

Create a macro to generate parameter structs:

```rust
macro_rules! device_params {
    ($name:ident) => {
        #[derive(serde::Deserialize, schemars::JsonSchema)]
        pub struct $name {
            pub serial_number: Option<String>,
            pub device_path: Option<String>,
            pub device_type: Option<String>,
        }
    };
    ($name:ident, $($field:ident: $type:ty),+) => {
        #[derive(serde::Deserialize, schemars::JsonSchema)]
        pub struct $name {
            pub serial_number: Option<String>,
            pub device_path: Option<String>,
            pub device_type: Option<String>,
            $(pub $field: $type),+
        }
    };
}

device_params!(LitraToolParams);
device_params!(LitraBrightnessParams, value: Option<u16>, percentage: Option<u8>);
device_params!(LitraTemperatureParams, value: u16);
```

**Trade-offs:**
- Less boilerplate but harder to understand
- Can't easily add doc comments
- Debugging macro errors is challenging

### Recommendation

**Implement Option A (Struct Composition)** as it:
- Is idiomatic Rust
- Provides better type safety and encapsulation
- Supports documentation and helper methods
- Is easier to maintain and extend

### Migration Strategy

1. Create `DeviceIdentifier` and `BackDeviceIdentifier` structs
2. Update one parameter struct at a time
3. Update MCP tool handlers to access nested fields
4. Test with MCP clients
5. Update MCP documentation

---

## 4. Back Light Command Handlers (main.rs, lines 1163-1242)

### Current State

Three back light handlers have nearly identical boilerplate:

```rust
fn handle_back_toggle_command(serial_number: Option<&str>, device_path: Option<&str>) -> CliResult {
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
        if let Ok(is_on) = device_handle.is_back_on() {
            let _ = device_handle.set_back_on(!is_on);
        }
    }
    Ok(())
}

fn handle_back_brightness_up_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
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
            let new_brightness = current_brightness.saturating_add(percentage).min(100);
            let _ = device_handle.set_back_brightness_percentage(new_brightness);
        }
    }
    Ok(())
}

// handle_back_brightness_down_command has same structure
```

### Duplication Impact

- **~50 lines** of repeated boilerplate
- Each function has identical device setup and iteration logic
- Only the operation inside the loop differs

### Refactoring Suggestion

Create a generic helper function similar to `with_brightness_setting`:

```rust
/// Helper function to execute an operation on back light with state adjustment
fn with_back_brightness_adjustment<FGet, FSet>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    get_current: FGet,
    calculate_new: fn(u8) -> u8,
) -> CliResult
where
    FGet: Fn(&DeviceHandle) -> Result<u8, DeviceError>,
    FSet: Fn(&DeviceHandle, u8) -> Result<(), DeviceError>,
{
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
        if let Ok(current_value) = get_current(&device_handle) {
            let new_value = calculate_new(current_value);
            let _ = device_handle.set_back_brightness_percentage(new_value);
        }
    }
    
    Ok(())
}

// Simplified handler functions:
fn handle_back_brightness_up_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    with_back_brightness_adjustment(
        serial_number,
        device_path,
        |dh| dh.back_brightness_percentage(),
        |current| current.saturating_add(percentage).min(100),
    )
}

fn handle_back_brightness_down_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    with_back_brightness_adjustment(
        serial_number,
        device_path,
        |dh| dh.back_brightness_percentage(),
        |current| current.saturating_sub(percentage).max(1),
    )
}
```

**Alternative: Even More Generic Helper**

```rust
/// Execute a closure for each BeamLX device matching the selector
fn with_back_devices<F>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    operation: F,
) -> CliResult
where
    F: Fn(&DeviceHandle) -> Result<(), DeviceError>,
{
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
        let _ = operation(&device_handle);
    }
    
    Ok(())
}

// Then each handler becomes very simple:
fn handle_back_toggle_command(serial_number: Option<&str>, device_path: Option<&str>) -> CliResult {
    with_back_devices(serial_number, device_path, |device_handle| {
        if let Ok(is_on) = device_handle.is_back_on() {
            device_handle.set_back_on(!is_on)
        } else {
            Ok(())
        }
    })
}

fn handle_back_brightness_up_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    with_back_devices(serial_number, device_path, |device_handle| {
        device_handle.back_brightness_percentage()
            .and_then(|current| {
                let new_brightness = current.saturating_add(percentage).min(100);
                device_handle.set_back_brightness_percentage(new_brightness)
            })
            .or(Ok(()))
    })
}

fn handle_back_brightness_down_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    with_back_devices(serial_number, device_path, |device_handle| {
        device_handle.back_brightness_percentage()
            .and_then(|current| {
                let new_brightness = current.saturating_sub(percentage).max(1);
                device_handle.set_back_brightness_percentage(new_brightness)
            })
            .or(Ok(()))
    })
}
```

**Benefits:**
- Reduces ~50 lines to ~30 lines (40% reduction)
- Consistent error handling across all back light operations
- Easy to add new back light commands
- Matches the pattern already used for main light commands (`with_brightness_setting`)

### Recommendation

**Implement the "More Generic Helper" approach** (`with_back_devices`) as it:
- Provides maximum flexibility
- Is consistent with existing `with_device` pattern
- Keeps handlers focused on their specific logic
- Is easier to test and maintain

### Migration Strategy

1. Add `with_back_devices` helper function
2. Refactor `handle_back_toggle_command` first (simplest case)
3. Refactor brightness up/down commands
4. Run tests to ensure behavior is unchanged
5. Consider refactoring main light commands to use similar pattern if not already done

---

## Additional Opportunities

### 5. Temperature Command Handlers

Similar to brightness, temperature commands (`handle_temperature_up_command`, `handle_temperature_down_command`) follow a pattern that could benefit from a `with_temperature_adjustment` helper (similar to `with_brightness_setting` at lines 899-922).

### 6. Response Parsing in DeviceHandle

In `lib.rs`, response parsing follows a pattern:

```rust
let mut response_buffer = [0x00; 20];
let response = self.hid_device.read(&mut response_buffer[..])?;
Ok(u16::from(response_buffer[..response][4]) * 256 + u16::from(response_buffer[..response][5]))
```

This could be extracted into helper methods:
- `read_response() -> Result<[u8; 20], DeviceError>`
- `parse_u16_response(buffer: &[u8]) -> u16`
- `parse_bool_response(buffer: &[u8]) -> bool`

### 7. MCP Tool Handlers

MCP tool handlers (mcp.rs, lines 123-503) follow an identical pattern:

```rust
async fn litra_X(...) -> Result<CallToolResult, McpError> {
    match handle_X_command(...) {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text("...")])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}
```

This could be extracted into a macro or helper function to reduce the ~380 lines of similar code.

---

## Implementation Priority

Based on impact and complexity:

1. **HIGH PRIORITY** - HID Command Generation (lib.rs): Largest code reduction, foundational change
2. **HIGH PRIORITY** - CLI Command Arguments (main.rs): Improves user-facing API consistency
3. **MEDIUM PRIORITY** - MCP Parameter Structs (mcp.rs): Important for API consumers
4. **MEDIUM PRIORITY** - Back Light Command Handlers (main.rs): Matches existing patterns
5. **LOW PRIORITY** - Additional opportunities: Nice-to-have improvements

---

## Testing Recommendations

For each refactoring:

1. **Unit Tests**: Verify helper functions work correctly in isolation
2. **Integration Tests**: Ensure end-to-end functionality is preserved
3. **Regression Tests**: Run existing test suite before and after each change
4. **Manual Testing**: Test CLI commands and MCP tools with real devices if available

---

## Conclusion

These refactoring suggestions would:
- **Reduce codebase by ~400+ lines** (~20% reduction in main code files)
- **Improve maintainability** through single source of truth
- **Reduce bug risk** by eliminating copy-paste errors
- **Make future enhancements easier** with clear patterns

All suggestions maintain backward compatibility and can be implemented incrementally.
