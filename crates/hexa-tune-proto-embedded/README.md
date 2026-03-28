# hexa-tune-proto-embedded

Embedded adapter crate for the **hexaTune** protocol — typed commands, dispatch
helpers, and heapless wrappers on top of
[`hexa-tune-proto`](../hexa-tune-proto/README.md).

> `no_std` · heapless · typed command dispatch

## Overview

This crate bridges the generic protocol core (`hexa-tune-proto`) with embedded
firmware. It converts raw `AtMessage` values into strongly-typed `HexaCommand`
enums so handler code never does string parsing.

## Modules

| Module     | Purpose                                              |
|------------|------------------------------------------------------|
| `command`  | `HexaCommand` typed enum + `TryFrom<&AtMessage>`     |
| `dispatch` | `resolve()` helper — AtMessage → HexaCommand         |
| `error`    | `HexaError` — domain errors wrapping `ProtoError`    |

## Usage

```toml
[dependencies]
hexa-tune-proto = { version = "0.1", default-features = false }
hexa-tune-proto-embedded = { version = "0.1", default-features = false }
```

### Typed command dispatch

```rust
use hexa_tune_proto::at;
use hexa_tune_proto_embedded::command::HexaCommand;

let msg = at::parse(b"AT+FREQ=5#440#1000#1").unwrap();
match HexaCommand::try_from(&msg) {
    Ok(HexaCommand::Freq { frequency, duration_ms }) => {
        // frequency = 440, duration_ms = 1000
    }
    Ok(HexaCommand::SetRgb { r, g, b }) => { /* … */ }
    Ok(HexaCommand::Version) => { /* … */ }
    Err(e) => { /* unknown or malformed command */ }
    _ => {}
}
```

### Quick resolve

```rust
use hexa_tune_proto_embedded::dispatch;

let msg = at::parse(b"AT+RESET=1").unwrap();
let cmd = dispatch::resolve(&msg)?; // HexaCommand::Reset
```

## Commands

| Variant       | AT Format                        | Fields                  |
|---------------|----------------------------------|-------------------------|
| `Version`     | `AT+VERSION?`                    | —                       |
| `SetRgb`      | `AT+SETRGB=id#r#g#b`            | `r`, `g`, `b` (u8)     |
| `Reset`       | `AT+RESET=id`                    | —                       |
| `FwUpdate`    | `AT+FWUPDATE=id`                 | —                       |
| `Freq`        | `AT+FREQ=id#frequency#duration#isOneShot`  | `frequency`, `duration_ms` (u32), `is_one_shot` (bool) |
| `Operation`   | `AT+OPERATION=id#[repeatCount#]sub`        | `sub` (OperationSub), `repeat_count` (u8) |
| `Unknown`     | anything else                    | —                       |

## Error Types

`HexaError` wraps protocol errors and adds domain-specific variants:

| Variant          | Description                        |
|------------------|------------------------------------|
| `Proto(e)`       | Protocol-level error from core     |
| `UnknownCommand` | Unrecognized AT command name       |
| `MissingParam`   | Required parameter not provided    |
| `InvalidParam`   | Parameter value out of range       |

## Features

| Feature | Default | Description                               |
|---------|---------|-------------------------------------------|
| *(none)*| ✅      | `no_std` + heapless                        |
| `defmt` | —       | Enables defmt logging (propagates to core) |

## License

MIT — hexaTune LLC
