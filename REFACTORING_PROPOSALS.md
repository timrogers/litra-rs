# Code Duplication Refactoring Proposals

This document identifies areas of code duplication in the litra-rs codebase and proposes concrete refactoring strategies to reduce duplication while maintaining code clarity and maintainability.

## Executive Summary

The codebase has several significant areas of duplication across three main files:
1. **lib.rs**: HID command generation functions with near-identical structure
2. **main.rs**: CLI command definitions with repeated argument patterns and handler functions
3. **mcp.rs**: Parameter structs with duplicated device identification fields

All proposed refactorings maintain backward compatibility and follow Rust best practices.

---

## 1. HID Command Generation Functions (lib.rs)

### Current Problem

Lines 514-673 contain six functions that generate HID commands with nearly identical structure:
- `generate_is_on_bytes()`
- `generate_get_brightness_in_lumen_bytes()`
- `generate_get_temperature_in_kelvin_bytes()`
- `generate_set_on_bytes()`
- `generate_set_brightness_in_lumen_bytes()`
- `generate_set_temperature_in_kelvin_bytes()`

Each function follows the same pattern:
```rust
[0x11, 0xff, prefix, command, data_bytes..., 0x00, 0x00, ...]
```
Where:
- Bytes 0-1 are always `[0x11, 0xff]`
- Byte 2 is the prefix: `0x04` for Glow/Beam, `0x06` for BeamLX
- Byte 3 is the command byte (varies by operation)
- Bytes 4+ contain operation-specific data, padded with zeros to reach 20 bytes

### Refactoring Proposal

**Strategy**: Create a generic command builder function that encapsulates the common pattern.

```rust
/// Generate a HID command with the appropriate prefix for the device type
fn generate_command_bytes(
    device_type: &DeviceType, 
    command: u8, 
    data: &[u8]
) -> [u8; 20] {
    let prefix = match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => 0x04,
        DeviceType::LitraBeamLX => 0x06,
    };
    
    let mut result = [0x00; 20];
    result[0] = 0x11;
    result[1] = 0xff;
    result[2] = prefix;
    result[3] = command;
    
    // Copy data bytes, ensuring we don't exceed array bounds
    let data_len = data.len().min(16); // Max 16 data bytes (20 - 4 header bytes)
    result[4..4 + data_len].copy_from_slice(&data[..data_len]);
    
    result
}
```

Then simplify each function:

```rust
fn generate_is_on_bytes(device_type: &DeviceType) -> [u8; 20] {
    generate_command_bytes(device_type, 0x01, &[])
}

fn generate_get_brightness_in_lumen_bytes(device_type: &DeviceType) -> [u8; 20] {
    generate_command_bytes(device_type, 0x31, &[])
}

fn generate_get_temperature_in_kelvin_bytes(device_type: &DeviceType) -> [u8; 20] {
    generate_command_bytes(device_type, 0x81, &[])
}

fn generate_set_on_bytes(device_type: &DeviceType, on: bool) -> [u8; 20] {
    let on_byte = if on { 0x01 } else { 0x00 };
    generate_command_bytes(device_type, 0x1c, &[on_byte])
}

fn generate_set_brightness_in_lumen_bytes(
    device_type: &DeviceType,
    brightness_in_lumen: u16,
) -> [u8; 20] {
    let brightness_bytes = brightness_in_lumen.to_be_bytes();
    generate_command_bytes(device_type, 0x4c, &brightness_bytes)
}

fn generate_set_temperature_in_kelvin_bytes(
    device_type: &DeviceType,
    temperature_in_kelvin: u16,
) -> [u8; 20] {
    let temperature_bytes = temperature_in_kelvin.to_be_bytes();
    generate_command_bytes(device_type, 0x9c, &temperature_bytes)
}
```

**Benefits**:
- Reduces ~160 lines to ~50 lines
- Single source of truth for command structure
- Easier to modify command format if protocol changes
- Clear separation between command structure and command-specific data
- Maintains same function signatures for backward compatibility

**Impact**: Low risk - purely internal refactoring with no API changes

---

## 2. CLI Command Arguments (main.rs)

### Current Problem

Lines 104-501 define 16 CLI commands, with 13 of them repeating the same device identification arguments:
```rust
#[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, 
       conflicts_with_all = ["device_path", "device_type"])]
serial_number: Option<String>,

#[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP,
       conflicts_with_all = ["serial_number", "device_type"])]
device_path: Option<String>,

#[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, 
       value_parser = DeviceTypeValueParser, 
       conflicts_with_all = ["serial_number", "device_path"])]
device_type: Option<DeviceType>,
```

This pattern appears in: `On`, `Off`, `Toggle`, `Brightness`, `BrightnessUp`, `BrightnessDown`, `Temperature`, `TemperatureUp`, `TemperatureDown`, and slightly modified in `BackOn`, `BackOff`, `BackToggle`, `BackBrightness`, etc.

### Refactoring Proposal

**Strategy**: Use Clap's `#[command(flatten)]` feature to extract common arguments into shared structs.

```rust
/// Device identification arguments for targeting specific devices
#[derive(Debug, Parser)]
struct DeviceTargetArgs {
    #[clap(
        long, 
        short,
        help = SERIAL_NUMBER_ARGUMENT_HELP,
        conflicts_with_all = ["device_path", "device_type"]
    )]
    serial_number: Option<String>,
    
    #[clap(
        long,
        short('p'),
        help = DEVICE_PATH_ARGUMENT_HELP,
        conflicts_with_all = ["serial_number", "device_type"]
    )]
    device_path: Option<String>,
    
    #[clap(
        long,
        short('t'),
        help = DEVICE_TYPE_ARGUMENT_HELP,
        value_parser = DeviceTypeValueParser,
        conflicts_with_all = ["serial_number", "device_path"]
    )]
    device_type: Option<DeviceType>,
}

/// Device identification for back-light commands (no device_type since only BeamLX)
#[derive(Debug, Parser)]
struct BackDeviceTargetArgs {
    #[clap(
        long,
        short,
        help = SERIAL_NUMBER_ARGUMENT_HELP,
        conflicts_with = "device_path"
    )]
    serial_number: Option<String>,
    
    #[clap(
        long,
        short('p'),
        help = DEVICE_PATH_ARGUMENT_HELP,
        conflicts_with = "serial_number"
    )]
    device_path: Option<String>,
}
```

Then use them in commands:

```rust
enum Commands {
    /// Turn your Logitech Litra device on
    On {
        #[command(flatten)]
        target: DeviceTargetArgs,
    },
    
    /// Turn your Logitech Litra device off
    Off {
        #[command(flatten)]
        target: DeviceTargetArgs,
    },
    
    /// Toggle your Logitech Litra device on or off
    Toggle {
        #[command(flatten)]
        target: DeviceTargetArgs,
    },
    
    /// Sets the brightness of your Logitech Litra device
    #[clap(group = ArgGroup::new("brightness").required(true).multiple(false))]
    Brightness {
        #[command(flatten)]
        target: DeviceTargetArgs,
        
        #[clap(long, short, help = "The brightness to set, measured in lumens", 
               group = "brightness")]
        value: Option<u16>,
        
        #[clap(long, short('b'), 
               help = "The brightness to set, as a percentage of the maximum brightness",
               group = "brightness",
               value_parser = clap::value_parser!(u8).range(1..=100))]
        percentage: Option<u8>,
    },
    
    // ... similar for other commands
    
    /// Turn on the colorful backlight on your Logitech Litra Beam LX device
    BackOn {
        #[command(flatten)]
        target: BackDeviceTargetArgs,
    },
    
    // ... similar for other back-light commands
}
```

Then update command handlers to use the nested struct:

```rust
Commands::On { target } => {
    handle_on_command(
        target.serial_number.as_deref(),
        target.device_path.as_deref(), 
        target.device_type.as_ref()
    )
}
```

**Benefits**:
- Reduces ~300 lines of repetitive argument definitions
- Single source of truth for device targeting arguments
- Easier to add new device identification methods in the future
- Improves consistency across commands
- Type-safe grouping of related arguments

**Impact**: Low risk - CLI interface remains identical to users

---

## 3. MCP Parameter Structs (mcp.rs)

### Current Problem

Lines 36-100 define 7 parameter structs that all repeat the same device identification fields:

```rust
pub struct LitraToolParams {
    pub serial_number: Option<String>,
    pub device_path: Option<String>,
    pub device_type: Option<String>,
}

pub struct LitraBrightnessParams {
    pub serial_number: Option<String>,
    pub device_path: Option<String>,
    pub device_type: Option<String>,
    pub value: Option<u16>,
    pub percentage: Option<u8>,
}

// ... 5 more similar structs
```

### Refactoring Proposal

**Strategy**: Create base trait or use composition with shared device targeting struct.

#### Option A: Composition (Recommended)

```rust
/// Common device identification parameters used across MCP tools
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct DeviceTargetParams {
    /// The serial number of the device to target (optional)
    pub serial_number: Option<String>,
    /// The device path to target (optional)
    pub device_path: Option<String>,
    /// The device type to target (optional)
    pub device_type: Option<String>,
}

/// Back-light device targeting (only BeamLX, no device_type needed)
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct BackDeviceTargetParams {
    /// The serial number of the device to target (optional)
    pub serial_number: Option<String>,
    /// The device path to target (optional)
    pub device_path: Option<String>,
}

// Then use composition:
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraToolParams {
    #[serde(flatten)]
    pub target: DeviceTargetParams,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    #[serde(flatten)]
    pub target: DeviceTargetParams,
    
    /// The brightness value to set in lumens
    pub value: Option<u16>,
    /// The brightness as a percentage of maximum brightness
    pub percentage: Option<u8>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBackBrightnessParams {
    #[serde(flatten)]
    pub target: BackDeviceTargetParams,
    
    /// The brightness as a percentage (1-100)
    pub percentage: u8,
}

// ... similar for other parameter structs
```

Then update handlers to access nested fields:

```rust
async fn litra_on(
    &self,
    Parameters(params): Parameters<LitraToolParams>,
) -> Result<CallToolResult, McpError> {
    match handle_on_command(
        params.target.serial_number.as_deref(),
        params.target.device_path.as_deref(),
        parse_device_type(params.target.device_type.as_ref()).as_ref(),
    ) {
        // ... rest of handler
    }
}
```

**Note**: The `#[serde(flatten)]` attribute ensures the JSON schema remains identical to the current implementation, so the MCP API is unchanged.

#### Option B: Helper Methods on Each Struct

If flattening doesn't work well with JSON Schema generation, add helper methods:

```rust
trait DeviceTarget {
    fn serial_number(&self) -> Option<&str>;
    fn device_path(&self) -> Option<&str>;
    fn device_type_str(&self) -> Option<&str>;
}

impl DeviceTarget for LitraToolParams {
    fn serial_number(&self) -> Option<&str> { self.serial_number.as_deref() }
    fn device_path(&self) -> Option<&str> { self.device_path.as_deref() }
    fn device_type_str(&self) -> Option<&str> { self.device_type.as_deref() }
}

// Implement for all param structs...

// Then create a generic handler:
fn call_with_device_target<P: DeviceTarget>(
    params: &P,
    handler: impl Fn(Option<&str>, Option<&str>, Option<&DeviceType>) -> CliResult
) -> Result<CallToolResult, McpError> {
    match handler(
        params.serial_number(),
        params.device_path(),
        parse_device_type(params.device_type_str().map(|s| &s.to_string()).as_ref()).as_ref()
    ) {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text("Success")])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}
```

**Benefits**:
- Reduces field duplication across 7 structs
- Single source of truth for device targeting parameters
- Easier to add new targeting options
- Maintains MCP API compatibility with proper use of `#[serde(flatten)]`

**Impact**: Low risk - Option A with `#[serde(flatten)]` maintains exact API compatibility

---

## 4. Handler Function Patterns (main.rs)

### Current Problem

Several groups of handler functions follow near-identical patterns:

#### A. Toggle Operations (lines 874-896, 1163-1186)

Both `handle_toggle_command()` and `handle_back_toggle_command()` have identical structure:
```rust
fn handle_toggle_command(...) -> CliResult {
    let context = Litra::new()?;
    let devices = get_all_supported_devices(&context, ...)?;
    if devices.is_empty() { return Err(CliError::DeviceNotFound); }
    
    for device_handle in devices {
        if let Ok(is_on) = device_handle.is_on() {
            let _ = device_handle.set_on(!is_on);
        }
    }
    Ok(())
}
```

#### B. Back Brightness Adjustments (lines 1188-1242)

Both `handle_back_brightness_up_command()` and `handle_back_brightness_down_command()` follow the same pattern with only the calculation differing.

### Refactoring Proposal

**Strategy**: Create generic helper functions using closures for the operation-specific logic.

#### A. Generic Toggle Helper

```rust
/// Generic toggle operation for any boolean state
fn with_toggle<FGet, FSet>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    getter: FGet,
    setter: FSet,
) -> CliResult
where
    FGet: Fn(&DeviceHandle) -> DeviceResult<bool>,
    FSet: Fn(&DeviceHandle, bool) -> DeviceResult<()>,
{
    let context = Litra::new()?;
    let devices = get_all_supported_devices(&context, serial_number, device_path, device_type)?;
    
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }
    
    for device_handle in devices {
        if let Ok(is_on) = getter(&device_handle) {
            let _ = setter(&device_handle, !is_on);
        }
    }
    Ok(())
}

// Then simplify handlers:
fn handle_toggle_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
) -> CliResult {
    with_toggle(
        serial_number,
        device_path,
        device_type,
        DeviceHandle::is_on,
        DeviceHandle::set_on,
    )
}

fn handle_back_toggle_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
) -> CliResult {
    with_toggle(
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
        DeviceHandle::is_back_on,
        DeviceHandle::set_back_on,
    )
}
```

#### B. Generic Back Brightness Adjustment Helper

```rust
/// Generic brightness adjustment for back light
fn with_back_brightness_adjustment<F>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    adjustment: F,
) -> CliResult
where
    F: Fn(u8) -> u8,
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
        if let Ok(current_brightness) = device_handle.back_brightness_percentage() {
            let new_brightness = adjustment(current_brightness);
            let _ = device_handle.set_back_brightness_percentage(new_brightness);
        }
    }
    Ok(())
}

// Then simplify handlers:
fn handle_back_brightness_up_command(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    percentage: u8,
) -> CliResult {
    with_back_brightness_adjustment(
        serial_number,
        device_path,
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
        |current| current.saturating_sub(percentage).max(1),
    )
}
```

**Benefits**:
- Reduces duplication in toggle operations (~50 lines saved)
- Reduces duplication in brightness adjustments (~40 lines saved)
- Centralizes device iteration and error handling logic
- Makes operation-specific logic more visible
- Easier to add similar operations in the future

**Impact**: Low risk - purely internal refactoring

---

## 5. MCP Tool Handler Pattern (mcp.rs)

### Current Problem

Lines 123-337 contain 10 nearly identical async tool handlers with the same structure:

```rust
async fn litra_on(...) -> Result<CallToolResult, McpError> {
    match handle_on_command(...) {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text("Success message")])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}
```

### Refactoring Proposal

**Strategy**: Create a helper function that wraps CLI handlers for MCP.

```rust
/// Convert a CLI result to an MCP CallToolResult
fn cli_to_mcp_result(
    result: CliResult,
    success_message: &str,
) -> Result<CallToolResult, McpError> {
    match result {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(success_message)])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}

// Then simplify handlers:
async fn litra_on(
    &self,
    Parameters(params): Parameters<LitraToolParams>,
) -> Result<CallToolResult, McpError> {
    cli_to_mcp_result(
        handle_on_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
        ),
        "Device(s) turned on successfully",
    )
}
```

**Benefits**:
- Reduces ~60 lines of duplicated error handling
- Consistent success/error message formatting
- Single place to modify error conversion logic
- Easier to add logging or metrics in the future

**Impact**: Low risk - purely internal refactoring

---

## Implementation Priority

Based on impact and effort, recommended implementation order:

### Phase 1: Quick Wins (Low effort, immediate benefit)
1. **HID Command Generation (Proposal #1)**: ~1 hour, reduces 160 lines to 50
2. **MCP Tool Handler Pattern (Proposal #5)**: ~30 minutes, reduces 60 lines

### Phase 2: Structural Improvements (Medium effort, high benefit)
3. **Handler Function Patterns (Proposal #4)**: ~2 hours, saves ~90 lines, improves maintainability
4. **MCP Parameter Structs (Proposal #3)**: ~2 hours, DRY principles, maintains API compatibility

### Phase 3: CLI Improvements (Higher effort, good long-term benefit)
5. **CLI Command Arguments (Proposal #2)**: ~3 hours, saves ~300 lines, requires thorough testing

---

## Testing Strategy

For each refactoring:

1. **Unit Tests**: Verify that refactored functions produce identical output to original functions
2. **Integration Tests**: Ensure CLI commands and MCP tools work identically
3. **Comparison Testing**: Run before/after versions with same inputs, compare outputs
4. **Linting**: Ensure `cargo clippy` passes with no new warnings
5. **Build**: Ensure `cargo build --release` succeeds

---

## Risks and Mitigations

### Risk: Breaking backward compatibility
**Mitigation**: All proposals maintain public API compatibility. Changes are internal only.

### Risk: Introducing subtle bugs
**Mitigation**: Extensive testing, gradual rollout by priority phases

### Risk: Reducing code clarity
**Mitigation**: Each proposal improves clarity by reducing duplication and making patterns explicit

### Risk: Over-abstraction
**Mitigation**: All abstractions are motivated by concrete duplication (3+ instances), not speculative

---

## Conclusion

These refactorings would reduce the codebase by approximately 600-700 lines while improving maintainability, consistency, and extensibility. All proposals maintain backward compatibility and follow Rust idioms.

The modular nature of these proposals allows them to be implemented independently, reducing risk and allowing for incremental improvement.
