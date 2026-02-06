# Code Duplication Refactoring Proposal

This document identifies areas of code duplication in the litra-rs codebase and proposes ways to refactor them.

## 1. HID Command Generation Functions (lib.rs)

### Current State
Lines 514-725 contain multiple `generate_*_bytes` functions that follow a nearly identical pattern:
- `generate_is_on_bytes()`
- `generate_get_brightness_in_lumen_bytes()`
- `generate_get_temperature_in_kelvin_bytes()`
- `generate_set_on_bytes()`
- `generate_set_brightness_in_lumen_bytes()`
- `generate_set_temperature_in_kelvin_bytes()`

Each function creates a 20-byte array with the format `[0x11, 0xff, prefix, command, data...]` where:
- `prefix` is `0x04` for Glow/Beam or `0x06` for BeamLX
- Only the command byte (position 3) and sometimes data bytes differ

### Problem
- 6+ functions with ~90% identical code
- The only differences are the command byte and optional data parameters
- Adding support for new devices or commands requires duplicating the same pattern

### Proposed Solution
Create a generic HID command builder:

```rust
fn build_hid_command(device_type: &DeviceType, command: u8, data: &[u8]) -> [u8; 20] {
    let prefix = match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => 0x04,
        DeviceType::LitraBeamLX => 0x06,
    };
    
    let mut buffer = [0x00; 20];
    buffer[0] = 0x11;
    buffer[1] = 0xff;
    buffer[2] = prefix;
    buffer[3] = command;
    
    // Copy data bytes if provided (max 16 bytes)
    let data_len = data.len().min(16);
    buffer[4..4+data_len].copy_from_slice(&data[..data_len]);
    
    buffer
}

// Then simplify each function to:
fn generate_is_on_bytes(device_type: &DeviceType) -> [u8; 20] {
    build_hid_command(device_type, 0x01, &[])
}

fn generate_set_on_bytes(device_type: &DeviceType, on: bool) -> [u8; 20] {
    build_hid_command(device_type, 0x1c, &[if on { 0x01 } else { 0x00 }])
}

fn generate_set_brightness_in_lumen_bytes(device_type: &DeviceType, brightness: u16) -> [u8; 20] {
    let bytes = brightness.to_be_bytes();
    build_hid_command(device_type, 0x4c, &bytes)
}
```

### Benefits
- Reduces ~150 lines of code to ~20 lines
- Centralizes the HID command structure
- Makes it easier to add new commands
- Improves maintainability

---

## 2. CLI Command Argument Definitions (main.rs)

### Current State
Lines 104-501 define CLI subcommands with repeated argument patterns. Almost every command has:
```rust
#[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, conflicts_with_all = ["device_path", "device_type"])]
serial_number: Option<String>,

#[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP, conflicts_with_all = ["serial_number", "device_type"])]
device_path: Option<String>,

#[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
device_type: Option<DeviceType>,
```

This pattern is repeated in 13+ command variants.

### Problem
- ~45 lines of code repeated 13 times = ~585 lines
- Any change to filtering logic requires updating 13 places
- High risk of inconsistency

### Proposed Solution
Use Clap's `#[command(flatten)]` with a shared struct:

```rust
#[derive(Debug, Parser)]
struct DeviceFilter {
    #[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, conflicts_with_all = ["device_path", "device_type"])]
    serial_number: Option<String>,
    
    #[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP, conflicts_with_all = ["serial_number", "device_type"])]
    device_path: Option<String>,
    
    #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, value_parser = DeviceTypeValueParser, conflicts_with_all = ["serial_number", "device_path"])]
    device_type: Option<DeviceType>,
}

// Then in each command:
On {
    #[command(flatten)]
    filter: DeviceFilter,
},
```

For back commands that only use serial_number and device_path:

```rust
#[derive(Debug, Parser)]
struct BackDeviceFilter {
    #[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, conflicts_with = "device_path")]
    serial_number: Option<String>,
    
    #[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP, conflicts_with = "serial_number")]
    device_path: Option<String>,
}

BackOn {
    #[command(flatten)]
    filter: BackDeviceFilter,
},
```

### Benefits
- Reduces ~585 lines to ~30 lines
- Single source of truth for device filtering
- Changes to filtering logic propagate automatically
- Type safety ensures consistency

---

## 3. MCP Tool Parameter Structs (mcp.rs)

### Current State
Lines 36-100 define parameter structs with repeated fields:

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

// ... 5 more structs with the same 3 fields
```

### Problem
- Serial number, device path, and device type fields repeated in 7 structs
- ~21 duplicate field definitions with documentation

### Proposed Solution
Use struct composition:

```rust
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct DeviceIdentifier {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraToolParams {
    #[serde(flatten)]
    pub device: DeviceIdentifier,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    #[serde(flatten)]
    pub device: DeviceIdentifier,
    /// The brightness value to set in lumens (use either this or percentage, not both)
    pub value: Option<u16>,
    /// The brightness as a percentage of maximum brightness (use either this or value, not both)
    pub percentage: Option<u8>,
}
```

### Benefits
- Reduces ~63 lines to ~30 lines
- Single definition for device identification
- Changes propagate automatically
- Consistent field naming and documentation

---

## 4. MCP Tool Implementations (mcp.rs)

### Current State
Lines 115-337 contain 12 nearly identical tool implementations:

```rust
async fn litra_on(&self, Parameters(params): Parameters<LitraToolParams>) -> Result<CallToolResult, McpError> {
    match handle_on_command(
        params.serial_number.as_deref(),
        params.device_path.as_deref(),
        parse_device_type(params.device_type.as_ref()).as_ref(),
    ) {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text("Device(s) turned on successfully")])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}
```

This pattern repeats for: on, off, toggle, temperature, temperature_up, temperature_down, and 6 back-light commands.

### Problem
- Same match/result handling pattern repeated 12 times
- Error handling logic duplicated
- Any change to response format requires updating 12 places

### Proposed Solution
Create a helper macro or function:

```rust
// Helper function to wrap command handlers
fn wrap_command_result(result: CliResult, success_message: &str) -> Result<CallToolResult, McpError> {
    match result {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(success_message)])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
    }
}

// Then simplify each tool:
async fn litra_on(&self, Parameters(params): Parameters<LitraToolParams>) -> Result<CallToolResult, McpError> {
    wrap_command_result(
        handle_on_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
        ),
        "Device(s) turned on successfully"
    )
}
```

Or use a macro for more conciseness:

```rust
macro_rules! mcp_tool {
    ($handler:expr, $success:expr) => {
        match $handler {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text($success)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    };
}

async fn litra_on(&self, Parameters(params): Parameters<LitraToolParams>) -> Result<CallToolResult, McpError> {
    mcp_tool!(
        handle_on_command(
            params.serial_number.as_deref(),
            params.device_path.as_deref(),
            parse_device_type(params.device_type.as_ref()).as_ref(),
        ),
        "Device(s) turned on successfully"
    )
}
```

### Benefits
- Reduces repetitive boilerplate
- Centralizes error handling logic
- Makes tools easier to read
- Changes to response format happen in one place

---

## 5. Back Light Command Handlers (main.rs)

### Current State
Lines 1163-1242 contain three nearly identical handlers:
- `handle_back_toggle_command`
- `handle_back_brightness_up_command`
- `handle_back_brightness_down_command`

Each follows the pattern:
```rust
fn handle_back_X_command(serial_number: Option<&str>, device_path: Option<&str>, ...) -> CliResult {
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
        // Different logic here
    }
    Ok(())
}
```

### Problem
- Device enumeration logic repeated 3 times
- Could easily get out of sync

### Proposed Solution
Create a helper function similar to `with_device`:

```rust
fn with_back_devices<F>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    operation: F,
) -> CliResult
where
    F: Fn(&DeviceHandle) -> DeviceResult<()>,
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

// Then simplify each handler:
fn handle_back_toggle_command(serial_number: Option<&str>, device_path: Option<&str>) -> CliResult {
    with_back_devices(serial_number, device_path, |device_handle| {
        if let Ok(is_on) = device_handle.is_back_on() {
            device_handle.set_back_on(!is_on)
        } else {
            Ok(())
        }
    })
}

fn handle_back_brightness_up_command(serial_number: Option<&str>, device_path: Option<&str>, percentage: u8) -> CliResult {
    with_back_devices(serial_number, device_path, |device_handle| {
        let current_brightness = device_handle.back_brightness_percentage()?;
        let new_brightness = current_brightness.saturating_add(percentage).min(100);
        device_handle.set_back_brightness_percentage(new_brightness)
    })
}
```

### Benefits
- Removes ~60 lines of duplicate code
- Consistent device handling
- Easier to add new back-light commands

---

## 6. Toggle Command Pattern (main.rs)

### Current State
Lines 874-896 and 1163-1186 implement toggle logic with the same pattern:

```rust
fn handle_toggle_command(...) -> CliResult {
    let context = Litra::new()?;
    let devices = get_all_supported_devices(&context, ...)?;
    if devices.is_empty() {
        return Err(CliError::DeviceNotFound);
    }
    
    for device_handle in devices {
        if let Ok(is_on) = device_handle.is_on() {
            let _ = device_handle.set_on(!is_on);
        }
    }
    Ok(())
}

fn handle_back_toggle_command(...) -> CliResult {
    let context = Litra::new()?;
    let devices = get_all_supported_devices(&context, ...)?;
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
```

### Problem
- Same structure repeated twice
- Generic toggle pattern could be reused

### Proposed Solution
Create a generic toggle helper:

```rust
fn with_toggle<F, G>(
    serial_number: Option<&str>,
    device_path: Option<&str>,
    device_type: Option<&DeviceType>,
    get_state: F,
    set_state: G,
) -> CliResult
where
    F: Fn(&DeviceHandle) -> DeviceResult<bool>,
    G: Fn(&DeviceHandle, bool) -> DeviceResult<()>,
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

// Then simplify:
fn handle_toggle_command(serial_number: Option<&str>, device_path: Option<&str>, device_type: Option<&DeviceType>) -> CliResult {
    with_toggle(
        serial_number,
        device_path,
        device_type,
        |h| h.is_on(),
        |h, on| h.set_on(on),
    )
}

fn handle_back_toggle_command(serial_number: Option<&str>, device_path: Option<&str>) -> CliResult {
    with_toggle(
        serial_number,
        device_path,
        Some(&DeviceType::LitraBeamLX),
        |h| h.is_back_on(),
        |h, on| h.set_back_on(on),
    )
}
```

### Benefits
- Eliminates duplicate toggle logic
- More flexible for future toggle commands
- Clearer intent

---

## Summary

### Total Impact
- **lib.rs**: Reduce ~150 lines of HID command generation to ~20 lines
- **main.rs CLI args**: Reduce ~585 lines to ~30 lines
- **main.rs handlers**: Reduce ~60 lines of back-light handlers to ~30 lines
- **mcp.rs params**: Reduce ~63 lines to ~30 lines
- **mcp.rs tools**: Reduce boilerplate in 12 tool implementations

**Estimated total reduction**: ~800+ lines of code while improving maintainability and type safety.

### Priority Recommendations

1. **High Priority**: HID command generation (lib.rs #1) - Highest duplication, most error-prone
2. **High Priority**: CLI argument definitions (main.rs #2) - Large reduction, improves consistency
3. **Medium Priority**: MCP parameter structs (mcp.rs #3) - Good reduction, clean design
4. **Medium Priority**: MCP tool implementations (mcp.rs #4) - Reduces boilerplate
5. **Low Priority**: Back light handlers (main.rs #5) - Smaller gain but still valuable
6. **Low Priority**: Toggle pattern (main.rs #6) - Nice-to-have abstraction

### Testing Strategy

After each refactoring:
1. Run `cargo check --locked --workspace --all-features --all-targets`
2. Run `cargo test` to ensure functionality is preserved
3. Run `cargo clippy --locked --workspace --all-features --all-targets -- -D warnings`
4. Manually test affected CLI commands with actual devices if available
5. Verify MCP server still works correctly

### Notes

- All proposed changes are backward-compatible at the API level
- No changes to public library interface
- CLI interface remains identical
- MCP protocol remains unchanged
- Focus is purely on internal code organization
