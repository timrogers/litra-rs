# Visual Refactoring Guide

This document provides visual representations of the refactoring patterns to help understand the transformations.

## 1. HID Command Generation Refactoring

### Before (Duplicated Pattern)
```
generate_is_on_bytes()
├─ match device_type
   ├─ Glow/Beam:  [0x11, 0xff, 0x04, 0x01, ...]
   └─ BeamLX:     [0x11, 0xff, 0x06, 0x01, ...]

generate_get_brightness_in_lumen_bytes()
├─ match device_type
   ├─ Glow/Beam:  [0x11, 0xff, 0x04, 0x31, ...]
   └─ BeamLX:     [0x11, 0xff, 0x06, 0x31, ...]

generate_get_temperature_in_kelvin_bytes()
├─ match device_type
   ├─ Glow/Beam:  [0x11, 0xff, 0x04, 0x81, ...]
   └─ BeamLX:     [0x11, 0xff, 0x06, 0x81, ...]

... 5 more functions with same pattern
```

### After (Consolidated Pattern)
```
generate_command_bytes(device_type, command_code, payload)
├─ Build base: [0x11, 0xff, prefix, command_code, ...]
├─ Select prefix based on device_type
│  ├─ Glow/Beam → 0x04
│  └─ BeamLX → 0x06
└─ Insert payload data

↓ Used by thin wrappers ↓

generate_is_on_bytes() → generate_command_bytes(type, 0x01, [])
generate_get_brightness_in_lumen_bytes() → generate_command_bytes(type, 0x31, [])
generate_get_temperature_in_kelvin_bytes() → generate_command_bytes(type, 0x81, [])
... all other functions become one-liners
```

**Reduction:** 8 functions × 20 lines = 160 lines → 1 function × 15 lines + 8 wrappers × 3 lines = ~40 lines
**Savings:** ~120 lines

---

## 2. MCP Parameter Structs Refactoring

### Before (Duplicated Fields)
```
LitraToolParams                    LitraBrightnessParams              LitraTemperatureParams
├─ serial_number: Option<String>   ├─ serial_number: Option<String>   ├─ serial_number: Option<String>
├─ device_path: Option<String>     ├─ device_path: Option<String>     ├─ device_path: Option<String>
└─ device_type: Option<String>     ├─ device_type: Option<String>     ├─ device_type: Option<String>
                                   ├─ value: Option<u16>              └─ value: u16
                                   └─ percentage: Option<u8>

... 3 more structs with same 3 fields
```

### After (Composition with Flatten)
```
DeviceSelector (shared)
├─ serial_number: Option<String>
├─ device_path: Option<String>
└─ device_type: Option<String>
        ↓
        ↓ #[serde(flatten)]
        ↓ #[schemars(flatten)]
        ↓
LitraToolParams          LitraBrightnessParams        LitraTemperatureParams
└─ device: DeviceSelector ├─ device: DeviceSelector    ├─ device: DeviceSelector
                          ├─ value: Option<u16>        └─ value: u16
                          └─ percentage: Option<u8>

... all other structs compose DeviceSelector
```

**Usage Update:**
```diff
- let sn = params.serial_number;
+ let sn = params.device.serial_number;
```

**Reduction:** 6 structs × 3 fields × 3 lines = 54 lines → 1 struct × 3 fields × 3 lines = 9 lines
**Savings:** ~45 lines + better maintainability

---

## 3. CLI Command Arguments Refactoring

### Before (Repeated Arguments)
```
Commands Enum
├─ On {
│   ├─ #[clap(long, short, ...)] serial_number
│   ├─ #[clap(long, short('p'), ...)] device_path
│   └─ #[clap(long, short('t'), ...)] device_type
├─ Off {
│   ├─ #[clap(long, short, ...)] serial_number
│   ├─ #[clap(long, short('p'), ...)] device_path
│   └─ #[clap(long, short('t'), ...)] device_type
├─ Toggle {
│   ├─ #[clap(long, short, ...)] serial_number
│   ├─ #[clap(long, short('p'), ...)] device_path
│   └─ #[clap(long, short('t'), ...)] device_type
... 10+ commands with same 3 arguments
```

### After (Flatten Approach)
```
DeviceFilterArgs (shared)
├─ #[clap(long, short, ...)] serial_number
├─ #[clap(long, short('p'), ...)] device_path
└─ #[clap(long, short('t'), ...)] device_type
        ↓
        ↓ #[clap(flatten)]
        ↓
Commands Enum
├─ On { device: DeviceFilterArgs }
├─ Off { device: DeviceFilterArgs }
├─ Toggle { device: DeviceFilterArgs }
├─ Brightness {
│   ├─ device: DeviceFilterArgs
│   ├─ value: Option<u16>
│   └─ percentage: Option<u8>
... all commands use DeviceFilterArgs
```

**Usage Update:**
```diff
- Commands::On { serial_number, device_path, device_type } => {
-     handle_on_command(serial_number, device_path, device_type)?;
+ Commands::On { device } => {
+     handle_on_command(device.serial_number, device.device_path, device.device_type)?;
```

**Reduction:** 10+ commands × 3 args × 6 lines = 180+ lines → 1 struct × 3 args × 6 lines = 18 lines
**Savings:** ~160 lines

---

## 4. Handler Function Patterns

### Before (Duplicated Logic)
```
Main Light Commands              Back Light Commands
├─ handle_on_command()           ├─ handle_back_on_command()
│   └─ with_device() → set_on(true)  └─ with_device() → set_back_light_on(true)
├─ handle_off_command()          ├─ handle_back_off_command()
│   └─ with_device() → set_on(false) └─ with_device() → set_back_light_on(false)
└─ handle_toggle_command()       └─ handle_back_toggle_command()
    ├─ with_device()                 ├─ with_device()
    ├─ get current state             ├─ get current back state
    └─ set opposite state            └─ set opposite back state

... similar patterns for brightness_up/down
```

### After (Generic Helpers)
```
Generic Handlers
├─ handle_toggle_command(getter, setter)
│   ├─ Takes closures for get/set operations
│   └─ Implements toggle logic once
│
└─ handle_brightness_adjustment(adjustment_fn)
    ├─ Takes closure for calculation
    └─ Implements get-calculate-set logic once
        ↓
        ↓ Used by specific handlers
        ↓
Main Light                       Back Light
├─ handle_on_command()           ├─ handle_back_on_command()
│   → (specialized impl)         │   → (specialized impl)
├─ handle_toggle_command()       ├─ handle_back_toggle_command()
│   → generic_toggle(            │   → generic_toggle(
│       |d| d.is_on(),          │       |d| d.is_back_light_on(),
│       |d, s| d.set_on(s)      │       |d, s| d.set_back_light_on(s)
│     )                          │     )
└─ handle_brightness_up()        └─ handle_back_brightness_up()
    → generic_adjust(...)            → generic_adjust(...)
```

**Reduction:** Multiple similar 30-line functions → Few generic 20-line helpers + thin wrappers
**Savings:** ~80 lines

---

## Combined Impact Visualization

```
Before Refactoring:
├─ lib.rs: 725 lines
│   ├─ HID functions: ~210 lines (8 × ~26 lines)
│   └─ Other code: 515 lines
├─ mcp.rs: ~564 lines
│   ├─ Parameter structs: ~90 lines (6 × ~15 lines)
│   └─ Other code: 474 lines
└─ main.rs: ~1910 lines
    ├─ CLI arguments: ~400 lines (10+ × ~36 lines)
    ├─ Handler functions: ~380 lines
    └─ Other code: 1130 lines

After Refactoring:
├─ lib.rs: ~575 lines (saved 150)
│   ├─ HID functions: ~60 lines (1 helper + 8 wrappers)
│   └─ Other code: 515 lines
├─ mcp.rs: ~504 lines (saved 60)
│   ├─ Parameter structs: ~30 lines (2 shared + 6 using flatten)
│   └─ Other code: 474 lines
└─ main.rs: ~1660 lines (saved 250)
    ├─ CLI arguments: ~220 lines (2 shared + 10+ using flatten)
    ├─ Handler functions: ~210 lines (generic helpers)
    └─ Other code: 1130 lines

Total: ~2500 lines → ~1960 lines
Reduction: 540 lines (21.6%)
```

---

## Dependency Graph After Refactoring

```
                    Core Abstractions
                    ┌────────────────┐
                    │ generate_      │
                    │ command_bytes()│
                    └────────┬───────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼────────┐    │    ┌─────────▼────────┐
     │ generate_is_on_ │    │    │ generate_set_    │
     │ bytes()         │    │    │ brightness_...() │
     └─────────────────┘    │    └──────────────────┘
                            ...
                    (8 thin wrappers)


         MCP & CLI Parameters
         ┌──────────────────┐
         │ DeviceSelector   │
         │ - serial_number  │
         │ - device_path    │
         │ - device_type    │
         └────────┬─────────┘
                  │
                  │ #[serde(flatten)]/#[clap(flatten)]
                  │
     ┌────────────┼────────────┬────────────┐
     │            │            │            │
┌────▼────┐  ┌───▼────┐  ┌────▼────┐  ┌───▼────┐
│ Litra   │  │ Litra  │  │ Commands│  │Commands│
│ Tool    │  │ Bright │  │ ::On    │  │::Off   │
│ Params  │  │ Params │  │         │  │        │
└─────────┘  └────────┘  └─────────┘  └────────┘
                    (6+ structs)


         Handler Functions
         ┌────────────────────┐
         │ handle_toggle_     │
         │ command(getter,    │
         │         setter)    │
         └─────────┬──────────┘
                   │
         ┌─────────┼─────────┐
         │                   │
    ┌────▼────┐         ┌────▼────────┐
    │ handle_ │         │ handle_back_│
    │ toggle  │         │ toggle      │
    └─────────┘         └─────────────┘
    (2 specific toggle implementations)
```

---

## Migration Path

```
Phase 1: HID Commands (2-3h)
[████████░░░░░░░░] 40% complete
└─ Impact: 150 lines reduced

Phase 2: MCP Params (1-2h)
[████████████░░░░] 60% complete
└─ Impact: 60 lines reduced

Phase 3: CLI Args (2-3h)
[████████████████] 95% complete
└─ Impact: 180 lines reduced

Phase 4: Handlers (3-4h)
[████████████████] 100% complete
└─ Impact: 100 lines reduced

Total Progress: ~540 lines reduced
Time Investment: 10-14 hours
Maintenance Savings: Ongoing
```

---

## Key Metrics

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Total LOC | ~2500 | ~1960 | -540 (-21.6%) |
| HID functions | 8 × 26 lines | 1 × 15 + 8 × 3 | -150 lines |
| MCP param fields | 6 × 15 lines | 2 × 9 + 6 × 5 | -60 lines |
| CLI arguments | 10+ × 36 lines | 2 × 18 + 10 × 8 | -180 lines |
| Handler patterns | ~380 lines | ~210 lines | -170 lines |
| Maintainability | Medium | High | ↑↑ |
| Extensibility | Medium | High | ↑↑ |
| Test coverage needs | High | Low | ↓ |

---

## Decision Tree: When to Use Each Pattern

```
Need to add a new HID command?
│
├─ Yes → Use generate_command_bytes() helper
│         ├─ Already have command code? → Create wrapper function
│         └─ New command type? → Add new command code constant
│
Need to add a new MCP tool?
│
├─ Has device selection? → Compose DeviceSelector with flatten
├─ Back light only? → Compose BackDeviceSelector with flatten
└─ No device selection? → Create standalone struct
│
Need to add a new CLI command?
│
├─ Has device filtering? → Use DeviceFilterArgs with flatten
├─ Back light command? → Use BackDeviceFilterArgs with flatten
└─ No device filtering? → Create standalone command
│
Need to add a new handler?
│
├─ Simple on/off? → Use existing pattern with with_device()
├─ Toggle operation? → Use handle_toggle_command() with closures
├─ Brightness adjust? → Use handle_brightness_adjustment() with closure
└─ New pattern? → Consider creating new generic helper
```

---

## Code Review Checklist

When implementing these refactorings:

- [ ] ✅ All existing tests pass
- [ ] ✅ `cargo clippy` shows no new warnings
- [ ] ✅ `cargo fmt` applied to all changed files
- [ ] ✅ HID command bytes match original output (add tests)
- [ ] ✅ CLI help text unchanged (`litra --help`)
- [ ] ✅ MCP JSON schema unchanged (or intentionally updated)
- [ ] ✅ Device filtering behavior identical
- [ ] ✅ Error messages preserved
- [ ] ✅ Performance characteristics unchanged
- [ ] ✅ Documentation updated (README, comments)
- [ ] ✅ No breaking API changes (or documented)

---

## Questions & Answers

**Q: Will this change the public API?**
A: No. External interfaces (CLI, MCP) remain identical. Only internal implementation changes.

**Q: Do I need to update tests?**
A: Existing tests should pass without changes. Add new tests for helper functions.

**Q: What if I only want to implement some refactorings?**
A: Phases are independent. Implement in priority order and stop whenever you want.

**Q: How do I verify HID bytes haven't changed?**
A: Add unit tests that compare output of old and new implementations.

**Q: What about backward compatibility?**
A: All changes are internal. No breaking changes to CLI, MCP, or library API.

**Q: Is this worth the effort?**
A: Yes! ~540 lines reduced + significantly easier maintenance and extensibility.
