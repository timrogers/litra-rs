# Code Duplication Refactoring Summary

Quick reference guide for reducing code duplication in litra-rs.

## Overview

Total potential lines reduced: **~540 lines** (from ~2500 total)
Estimated effort: **10-14 hours**
Risk level: **Low to Medium**

## Duplication Patterns Identified

| Pattern | Location | Lines | Priority | Effort |
|---------|----------|-------|----------|--------|
| HID command generation | `src/lib.rs:514-725` | ~150 | 🔴 High | 2-3h |
| MCP parameter structs | `src/mcp.rs:36-100` | ~60 | 🔴 High | 1-2h |
| CLI device filter args | `src/main.rs:104-501` | ~180 | 🟡 Medium | 2-3h |
| Handler functions | `src/main.rs:854-1242` | ~100 | 🟡 Medium | 3-4h |
| Brightness/temp adjust | `src/main.rs:957-1242` | ~50 | 🟢 Low | 2h |

## Quick Wins

### 1. HID Command Generation (Priority 1)

**Problem:** 8 functions with 95% identical code, only command byte differs

**Solution:**
```rust
fn generate_command_bytes(
    device_type: &DeviceType,
    command_code: u8,
    payload: &[u8],
) -> [u8; 20] {
    let mut bytes = [0u8; 20];
    bytes[0] = 0x11;
    bytes[1] = 0xff;
    bytes[2] = match device_type {
        DeviceType::LitraGlow | DeviceType::LitraBeam => 0x04,
        DeviceType::LitraBeamLX => 0x06,
    };
    bytes[3] = command_code;
    let len = payload.len().min(16);
    bytes[4..4+len].copy_from_slice(&payload[..len]);
    bytes
}

// Existing functions become one-liners:
fn generate_is_on_bytes(device_type: &DeviceType) -> [u8; 20] {
    generate_command_bytes(device_type, 0x01, &[])
}
```

**Impact:** ~150 lines reduced, easier to add new commands

### 2. MCP Parameter Structs (Priority 1)

**Problem:** 6 structs repeat 3 identical fields each

**Solution:**
```rust
#[derive(serde::Deserialize, schemars::JsonSchema)]
struct DeviceSelector {
    pub serial_number: Option<String>,
    pub device_path: Option<String>,
    pub device_type: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LitraBrightnessParams {
    #[serde(flatten)]
    #[schemars(flatten)]
    pub device: DeviceSelector,
    pub value: Option<u16>,
    pub percentage: Option<u8>,
}

// Update handlers: params.serial_number -> params.device.serial_number
```

**Impact:** ~60 lines reduced, centralized device selection logic

### 3. CLI Device Arguments (Priority 2)

**Problem:** 10+ commands repeat 3 device filter arguments

**Solution:**
```rust
#[derive(clap::Args)]
struct DeviceFilterArgs {
    #[clap(long, short, help = SERIAL_NUMBER_ARGUMENT_HELP, ...)]
    serial_number: Option<String>,
    #[clap(long, short('p'), help = DEVICE_PATH_ARGUMENT_HELP, ...)]
    device_path: Option<String>,
    #[clap(long, short('t'), help = DEVICE_TYPE_ARGUMENT_HELP, ...)]
    device_type: Option<DeviceType>,
}

#[derive(Subcommand)]
enum Commands {
    On {
        #[clap(flatten)]
        device: DeviceFilterArgs,
    },
    Off {
        #[clap(flatten)]
        device: DeviceFilterArgs,
    },
    // ...
}
```

**Impact:** ~180 lines reduced

## Implementation Checklist

- [ ] **Phase 1: HID Commands** (2-3 hours)
  - [ ] Add `generate_command_bytes()` helper
  - [ ] Refactor `generate_is_on_bytes()` and test
  - [ ] Refactor `generate_get_brightness_in_lumen_bytes()` and test
  - [ ] Refactor remaining 6 functions
  - [ ] Run `cargo test` and `cargo clippy`

- [ ] **Phase 2: MCP Params** (1-2 hours)
  - [ ] Create `DeviceSelector` and `BackDeviceSelector` structs
  - [ ] Refactor `LitraToolParams` with flatten
  - [ ] Refactor remaining 5 param structs
  - [ ] Update all MCP handlers to use `params.device.*`
  - [ ] Test MCP server functionality

- [ ] **Phase 3: CLI Arguments** (2-3 hours)
  - [ ] Create `DeviceFilterArgs` with clap::Args
  - [ ] Create `BackDeviceFilterArgs` for back light commands
  - [ ] Refactor `Commands` enum to use flatten
  - [ ] Update all command handlers to use `device.*`
  - [ ] Test CLI: `./target/release/litra --help`

- [ ] **Phase 4: Handler Functions** (3-4 hours)
  - [ ] Create generic `handle_toggle_command()` helper
  - [ ] Refactor main toggle and back toggle
  - [ ] Create generic brightness adjustment helper
  - [ ] Refactor brightness up/down handlers
  - [ ] Test all command functionality

## Testing Strategy

```bash
# Before each phase
cargo test
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
cargo fmt --all -- --check

# After each phase - same checks plus:
cargo build --release
./target/release/litra --help
./target/release/litra devices

# Verify HID bytes haven't changed (add test):
#[test]
fn test_command_bytes_unchanged() {
    assert_eq!(
        generate_is_on_bytes(&DeviceType::LitraGlow),
        [0x11, 0xff, 0x04, 0x01, 0x00, ...]
    );
}
```

## Key Principles

1. ✅ **Preserve behavior** - All refactoring must maintain identical functionality
2. ✅ **Test thoroughly** - Run full test suite after each change
3. ✅ **Incremental changes** - Commit after each successful phase
4. ✅ **Document changes** - Update comments and docs as needed
5. ✅ **Review carefully** - Use `cargo clippy` to catch issues

## Benefits

- **Maintainability:** Changes to common patterns require updates in one place
- **Extensibility:** Adding new commands/features is much easier
- **Readability:** Less duplication makes code easier to understand
- **Testing:** Shared code can be tested once instead of in many places
- **Bug fixes:** Fixes to shared code benefit all users automatically

## See Also

- Full detailed analysis: `REFACTORING_SUGGESTIONS.md`
- Project structure: `src/lib.rs`, `src/main.rs`, `src/mcp.rs`
