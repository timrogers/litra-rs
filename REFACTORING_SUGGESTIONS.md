# Code Refactoring Suggestions for litra-rs

This document provides concrete suggestions for reducing code duplication in the litra-rs codebase. Each suggestion includes the problem, proposed solution, estimated impact, and implementation approach.

## Table of Contents
1. [HID Command Generation Functions](#1-hid-command-generation-functions)
2. [MCP Tool Parameter Structs](#2-mcp-tool-parameter-structs)
3. [CLI Command Device Filter Arguments](#3-cli-command-device-filter-arguments)
4. [Handler Function Patterns](#4-handler-function-patterns)
5. [Implementation Priority](#implementation-priority)

---

## 1. HID Command Generation Functions

### Problem
**Location:** `src/lib.rs`, lines 514-725

**Duplication:** 8 functions (`generate_is_on_bytes`, `generate_get_brightness_in_lumen_bytes`, `generate_get_temperature_in_kelvin_bytes`, `generate_set_on_bytes`, `generate_set_brightness_in_lumen_bytes`, `generate_set_temperature_in_kelvin_bytes`, etc.) follow nearly identical patterns:

```rust
fn generate_xxx_bytes(device_type: &DeviceType) -> [u8; 20] {
    match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => [
            0x11, 0xff, 0x04, COMMAND_CODE, ...data...
        ],
        DeviceType::LitraBeamLX => [
            0x11, 0xff, 0x06, COMMAND_CODE, ...data...
        ],
    }
}
```

**Only differences:**
- Byte 2: `0x04` for Glow/Beam, `0x06` for BeamLX (device prefix)
- Byte 3: Command code (0x01, 0x31, 0x81, 0x1c, 0x4c, 0x9c)
- Bytes 4-19: Payload data (mostly zeros or variable data)

### Proposed Solution

**Option A: Generic Command Builder Function**

Create a single function that builds HID commands:

```rust
/// Generates HID command bytes for Litra devices
/// 
/// # Arguments
/// * `device_type` - The device type (determines prefix byte)
/// * `command_code` - The command byte (0x01, 0x31, 0x81, etc.)
/// * `payload` - Optional payload bytes (up to 16 bytes)
fn generate_command_bytes(
    device_type: &DeviceType,
    command_code: u8,
    payload: &[u8],
) -> [u8; 20] {
    let mut bytes = [0u8; 20];
    bytes[0] = 0x11;
    bytes[1] = 0xff;
    
    // Device-specific prefix
    bytes[2] = match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => 0x04,
        DeviceType::LitraBeamLX => 0x06,
    };
    
    bytes[3] = command_code;
    
    // Copy payload data
    let payload_len = payload.len().min(16);
    bytes[4..4 + payload_len].copy_from_slice(&payload[..payload_len]);
    
    bytes
}

// Then refactor existing functions to use it:
fn generate_is_on_bytes(device_type: &DeviceType) -> [u8; 20] {
    generate_command_bytes(device_type, 0x01, &[])
}

fn generate_get_brightness_in_lumen_bytes(device_type: &DeviceType) -> [u8; 20] {
    generate_command_bytes(device_type, 0x31, &[])
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
```

**Option B: Command Enum with Builder Pattern**

```rust
enum CommandCode {
    IsOn = 0x01,
    GetBrightness = 0x31,
    GetTemperature = 0x81,
    SetOn = 0x1c,
    SetBrightness = 0x4c,
    SetTemperature = 0x9c,
}

struct CommandBuilder {
    device_type: DeviceType,
    command: CommandCode,
    payload: Vec<u8>,
}

impl CommandBuilder {
    fn new(device_type: DeviceType, command: CommandCode) -> Self {
        Self {
            device_type,
            command,
            payload: Vec::new(),
        }
    }
    
    fn with_payload(mut self, payload: &[u8]) -> Self {
        self.payload.extend_from_slice(payload);
        self
    }
    
    fn build(self) -> [u8; 20] {
        let mut bytes = [0u8; 20];
        bytes[0] = 0x11;
        bytes[1] = 0xff;
        bytes[2] = match self.device_type {
            DeviceType::LitraGlow | DeviceType::LitraBeam => 0x04,
            DeviceType::LitraBeamLX => 0x06,
        };
        bytes[3] = self.command as u8;
        
        let payload_len = self.payload.len().min(16);
        bytes[4..4 + payload_len].copy_from_slice(&self.payload[..payload_len]);
        
        bytes
    }
}
```

### Impact
- **Lines reduced:** ~150 lines (8 functions × ~20 lines each)
- **Maintainability:** Much easier to add new commands
- **Risk:** Low - existing tests should catch any regressions
- **Effort:** 2-3 hours

### Recommendation
**Use Option A** - It's simpler, more idiomatic for Rust, and preserves the existing function signatures as thin wrappers. Option B is over-engineered for this use case.

---

## 2. MCP Tool Parameter Structs

### Problem
**Location:** `src/mcp.rs`, lines 36-100

**Duplication:** 6 parameter structs share identical device selection fields:

```rust
pub struct LitraToolParams {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
}

// Repeated in:
// - LitraBrightnessParams (lines 47-58)
// - LitraTemperatureParams (lines 61-70)
// - LitraBackToolParams (lines 73-78) [minus device_type]
// - LitraBackBrightnessParams (lines 81-88) [minus device_type]
// - LitraBackColorParams (lines 91-100) [minus device_type]
```

### Proposed Solution

**Option A: Composition with Shared Struct (Recommended)**

```rust
/// Device selection parameters common to all MCP tools
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct DeviceSelector {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
    /// The device type to target: "litra_glow", "litra_beam", or "litra_beam_lx" (optional)
    pub device_type: Option<String>,
}

/// Device selection parameters for back light commands (no device type filtering)
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct BackDeviceSelector {
    /// The serial number of the device to target (optional - if not specified, all devices are targeted)
    pub serial_number: Option<String>,
    /// The device path to target (optional - useful for devices that don't show a serial number)
    pub device_path: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraToolParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: DeviceSelector,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: DeviceSelector,
    /// The brightness value to set in lumens (use either this or percentage, not both)
    pub value: Option<u16>,
    /// The brightness as a percentage of maximum brightness (use either this or value, not both)
    pub percentage: Option<u8>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBackToolParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: BackDeviceSelector,
}

// Usage in handlers becomes:
// params.device.serial_number instead of params.serial_number
```

**Option B: Macro-based Generation**

```rust
macro_rules! define_params {
    (
        $(#[$meta:meta])*
        struct $name:ident {
            device_type: $has_device_type:expr,
            $(
                $(#[$field_meta:meta])*
                $field:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        pub struct $name {
            /// The serial number of the device to target...
            pub serial_number: Option<String>,
            /// The device path to target...
            pub device_path: Option<String>,
            $(
                #[doc = concat!("The device type to target: \"litra_glow\", \"litra_beam\", or \"litra_beam_lx\" (optional)")]
                pub device_type: Option<String>,
            )?
            $(
                $(#[$field_meta])*
                pub $field: $field_type,
            )*
        }
    };
}

define_params! {
    #[derive(serde::Deserialize, schemars::JsonSchema)]
    struct LitraBrightnessParams {
        device_type: true,
        /// The brightness value to set in lumens
        value: Option<u16>,
        /// The brightness as a percentage
        percentage: Option<u8>,
    }
}
```

### Impact
- **Lines reduced:** ~60 lines (3 fields × 6 structs × ~3 lines each)
- **Maintainability:** Changes to device selection logic are centralized
- **Risk:** Low - JSON schema and serde behavior should remain identical with `flatten`
- **Effort:** 1-2 hours

### Recommendation
**Use Option A with `flatten`** - It's clean, maintainable, and properly supported by serde/schemars. The macro approach is harder to read and debug.

**Note:** You'll need to update all handler functions to access `params.device.serial_number` instead of `params.serial_number`, but this is straightforward.

---

## 3. CLI Command Device Filter Arguments

### Problem
**Location:** `src/main.rs`, lines 104-501

**Duplication:** 10+ command variants repeat identical device filter arguments:

```rust
On {
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
},
```

This pattern repeats in: `On`, `Off`, `Toggle`, `Brightness`, `BrightnessUp`, `BrightnessDown`, `Temperature`, `TemperatureUp`, `TemperatureDown`, and more.

### Proposed Solution

**Option A: Flatten Approach (Recommended)**

```rust
/// Device filter arguments shared across CLI commands
#[derive(clap::Args)]
struct DeviceFilterArgs {
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

/// Device filter for back light commands (no device type)
#[derive(clap::Args)]
struct BackDeviceFilterArgs {
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

// Then use with flatten:
#[derive(Subcommand)]
enum Commands {
    /// Turn your Logitech Litra device on...
    On {
        #[clap(flatten)]
        device: DeviceFilterArgs,
    },
    
    /// Turn your Logitech Litra device off...
    Off {
        #[clap(flatten)]
        device: DeviceFilterArgs,
    },
    
    /// Toggle your Logitech Litra device...
    Toggle {
        #[clap(flatten)]
        device: DeviceFilterArgs,
    },
    
    /// Set brightness...
    #[clap(group = ArgGroup::new("brightness").required(true).multiple(false))]
    Brightness {
        #[clap(flatten)]
        device: DeviceFilterArgs,
        
        #[clap(long, short, help = "...", group = "brightness")]
        value: Option<u16>,
        
        #[clap(long, short('b'), help = "...", group = "brightness")]
        percentage: Option<u8>,
    },
    
    // Back light commands use BackDeviceFilterArgs
    BackOn {
        #[clap(flatten)]
        device: BackDeviceFilterArgs,
    },
}
```

**Option B: Macro-based Generation**

```rust
macro_rules! device_filter_fields {
    ($with_type:expr) => {
        #[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, ...)]
        serial_number: Option<String>,
        #[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP, ...)]
        device_path: Option<String>,
        $(
            #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, ...)]
            device_type: Option<DeviceType>,
        )?
    };
}

// Used in commands:
On {
    device_filter_fields!(true),
},
```

### Impact
- **Lines reduced:** ~180 lines (3 arguments × 10 commands × ~6 lines each)
- **Maintainability:** Much easier to change device filtering behavior
- **Risk:** Low - clap's flatten is well-tested
- **Effort:** 2-3 hours

### Recommendation
**Use Option A with `flatten`** - This is the idiomatic clap approach and is officially supported. You'll need to update handler functions to access `device.serial_number` instead of `serial_number`, but it's cleaner long-term.

---

## 4. Handler Function Patterns

### Problem
**Location:** `src/main.rs`, lines 854-1242

**Duplication:** Handler functions follow repetitive patterns:

1. **On/Off/Toggle pattern** (3 groups):
   - Main light: `handle_on_command`, `handle_off_command`, `handle_toggle_command`
   - Back light: `handle_back_on_command`, `handle_back_off_command`, `handle_back_toggle_command`
   
2. **Brightness adjustment pattern** (2 groups):
   - Main light: `handle_brightness_up_command`, `handle_brightness_down_command`
   - Back light: `handle_back_brightness_up_command`, `handle_back_brightness_down_command`

### Proposed Solution

**Option A: Generic Handler Functions with Closures**

The codebase already has some helper functions (`with_toggle`, `with_brightness_setting`). Extend this pattern:

```rust
/// Generic on/off handler
fn handle_on_off_command<F>(
    serial_number: Option<String>,
    device_path: Option<String>,
    device_type: Option<DeviceType>,
    action: F,
) -> Result<()>
where
    F: Fn(&Device) -> Result<()>,
{
    with_device(serial_number, device_path, device_type, |device| {
        action(device)?;
        Ok(())
    })
}

// Refactor existing handlers:
fn handle_on_command(
    serial_number: Option<String>,
    device_path: Option<String>,
    device_type: Option<DeviceType>,
) -> Result<()> {
    handle_on_off_command(serial_number, device_path, device_type, |device| {
        device.set_on(true)
    })
}

fn handle_off_command(
    serial_number: Option<String>,
    device_path: Option<String>,
    device_type: Option<DeviceType>,
) -> Result<()> {
    handle_on_off_command(serial_number, device_path, device_type, |device| {
        device.set_on(false)
    })
}

/// Generic toggle handler
fn handle_toggle_command<FGet, FSet>(
    serial_number: Option<String>,
    device_path: Option<String>,
    device_type: Option<DeviceType>,
    getter: FGet,
    setter: FSet,
) -> Result<()>
where
    FGet: Fn(&Device) -> Result<bool>,
    FSet: Fn(&Device, bool) -> Result<()>,
{
    with_device(serial_number, device_path, device_type, |device| {
        let current_state = getter(device)?;
        setter(device, !current_state).ok(); // Ignore errors as per existing logic
        Ok(())
    })
}

// Usage:
fn handle_main_toggle_command(...) -> Result<()> {
    handle_toggle_command(
        serial_number,
        device_path,
        device_type,
        |d| d.is_on(),
        |d, state| d.set_on(state),
    )
}

fn handle_back_toggle_command(...) -> Result<()> {
    handle_toggle_command(
        serial_number,
        device_path,
        None, // Back commands don't filter by device type
        |d| d.is_back_light_on(),
        |d, state| d.set_back_light_on(state),
    )
}
```

**Option B: Trait-based Approach**

```rust
trait DeviceAction {
    fn execute(&self, device: &Device) -> Result<()>;
}

struct OnAction;
impl DeviceAction for OnAction {
    fn execute(&self, device: &Device) -> Result<()> {
        device.set_on(true)
    }
}

struct OffAction;
impl DeviceAction for OffAction {
    fn execute(&self, device: &Device) -> Result<()> {
        device.set_on(false)
    }
}

fn handle_action<A: DeviceAction>(
    action: A,
    serial_number: Option<String>,
    device_path: Option<String>,
    device_type: Option<DeviceType>,
) -> Result<()> {
    with_device(serial_number, device_path, device_type, |device| {
        action.execute(device)
    })
}
```

### Impact
- **Lines reduced:** ~100 lines across duplicated handlers
- **Maintainability:** Adding new similar commands is much easier
- **Risk:** Low - just refactoring existing logic
- **Effort:** 3-4 hours

### Recommendation
**Use Option A** - The closure-based approach is more idiomatic for Rust and doesn't require defining new types. The trait approach is overkill for this use case.

**Note:** The codebase already has `with_toggle` helper (line 874) - this can be renamed and extended to support both main and back light toggles.

---

## 5. Additional Opportunities

### A. Brightness Adjustment Logic

**Location:** `src/main.rs`, lines 957-1034 and 1188-1242

Both `handle_brightness_up_command` and `handle_brightness_down_command` (and their back light variants) have similar logic:

```rust
fn handle_brightness_adjustment<F>(
    serial_number: Option<String>,
    device_path: Option<String>,
    device_type: Option<DeviceType>,
    adjustment: F,
) -> Result<()>
where
    F: Fn(u16, &Device) -> Result<u16>,
{
    with_device(serial_number, device_path, device_type, |device| {
        let current = device.get_brightness_in_lumen()?;
        let new_brightness = adjustment(current, device)?;
        device.set_brightness_in_lumen(new_brightness)?;
        Ok(())
    })
}

// Usage:
fn handle_brightness_up_command(..., value: Option<u16>, percentage: Option<u8>) {
    handle_brightness_adjustment(serial_number, device_path, device_type, |current, device| {
        if let Some(val) = value {
            Ok(current + val)
        } else if let Some(pct) = percentage {
            let max = device.device_type().maximum_brightness_in_lumen();
            let adjustment = (max as f64 * (pct as f64 / 100.0)).round() as u16;
            Ok(current + adjustment)
        } else {
            unreachable!()
        }
    })
}
```

### B. Temperature Adjustment Logic

Similar pattern for temperature up/down commands (lines 1036-1120).

---

## Implementation Priority

Based on impact and effort, here's the recommended implementation order:

### Priority 1: High Impact, Low Effort
1. **HID Command Generation** (Section 1)
   - Reduces ~150 lines
   - 2-3 hours effort
   - Low risk

2. **MCP Parameter Structs** (Section 2)
   - Reduces ~60 lines
   - 1-2 hours effort
   - Low risk

### Priority 2: High Impact, Medium Effort
3. **CLI Command Arguments** (Section 3)
   - Reduces ~180 lines
   - 2-3 hours effort
   - Low risk but requires more updates

### Priority 3: Medium Impact, Medium Effort
4. **Handler Functions** (Section 4)
   - Reduces ~100 lines
   - 3-4 hours effort
   - Improves extensibility

5. **Brightness/Temperature Adjustment** (Section 5)
   - Reduces ~50 lines
   - 2 hours effort
   - Nice to have

### Total Estimated Impact
- **Lines of code reduced:** ~540 lines
- **Total effort:** 10-14 hours
- **Risk level:** Low to medium
- **Maintainability improvement:** High

---

## Testing Strategy

For each refactoring:

1. **Before refactoring:**
   ```bash
   cargo test
   cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
   cargo fmt --all -- --check
   ```

2. **After refactoring:**
   - Run the same checks
   - Manually test CLI commands: `./target/release/litra --help`
   - Test MCP server if modified
   - Verify HID command bytes haven't changed (add tests if needed)

3. **Add regression tests:**
   ```rust
   #[test]
   fn test_command_bytes_unchanged() {
       // Verify refactored functions produce identical bytes
       let old_bytes = [0x11, 0xff, 0x04, 0x01, ...];
       let new_bytes = generate_is_on_bytes(&DeviceType::LitraGlow);
       assert_eq!(old_bytes, new_bytes);
   }
   ```

---

## Conclusion

These refactoring suggestions will significantly reduce code duplication while maintaining functionality. The recommended approach is to tackle them in priority order, with comprehensive testing at each step.

**Key Benefits:**
- Reduced maintenance burden
- Easier to add new commands/features
- More consistent code structure
- Better testability

**Next Steps:**
1. Review and approve these suggestions
2. Implement Priority 1 items first
3. Test thoroughly
4. Continue with Priority 2 and 3 as time permits
