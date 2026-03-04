# Embedded Integration Guide

## Overview

The `hexa-tune-proto-embedded` crate provides a `no_std` adapter layer on top
of `hexa-tune-proto` (the core protocol crate). It adds typed command enums,
domain-specific error types, and dispatch helpers suitable for Embassy-based
firmware.

## Dependency Setup

In your firmware's `Cargo.toml`:

```toml
[dependencies]
hexa-tune-proto = { version = "0.1", default-features = false }
hexa-tune-proto-embedded = { version = "0.1", default-features = false }

# Optional: enable defmt logging
# hexa-tune-proto = { version = "0.1", default-features = false, features = ["defmt"] }
```

## Receiving USB MIDI Data

Use `StreamDecoder` to accumulate USB MIDI packets into complete messages:

```rust
use hexa_tune_proto::stream::StreamDecoder;
use hexa_tune_proto::at;

let mut buf = [0u8; 256];
let mut decoder = StreamDecoder::new(&mut buf);

// In your USB MIDI receive handler:
fn on_usb_packet(packet: [u8; 4]) {
    match decoder.push_packet(packet) {
        Ok(Some(len)) => {
            // Complete SysEx payload in buf[..len]
            let payload = &buf[..len];
            // Payload is the raw SysEx content (without F0/F7)
            // which is the AT command string
            if let Ok(msg) = at::parse(payload) {
                handle_command(&msg);
            }
        }
        Ok(None) => { /* more packets needed */ }
        Err(_e) => { /* protocol error, decoder auto-recovers */ }
    }
}
```

## Typed Command Dispatch

Convert generic `AtMessage` into typed `HexaCommand`:

```rust
use hexa_tune_proto::at::{self, AtMessage};
use hexa_tune_proto_embedded::command::HexaCommand;

fn handle_command(msg: &AtMessage<'_>) {
    match HexaCommand::try_from(msg) {
        Ok(HexaCommand::Version) => {
            // Respond with firmware version
        }
        Ok(HexaCommand::SetRgb { r, g, b }) => {
            // Set LED color
        }
        Ok(HexaCommand::Freq { frequency, duration_ms }) => {
            // Play tone
        }
        Ok(HexaCommand::Operation { sub }) => {
            // Handle operation sub-command
        }
        Ok(HexaCommand::Reset) => {
            // Reset device
        }
        Ok(HexaCommand::FwUpdate) => {
            // Enter firmware update mode
        }
        Err(e) => {
            // Unknown command or parse error
        }
    }
}
```

## Building Responses

Use the core `at::encode()` to build response strings:

```rust
use hexa_tune_proto::at::{self, AtOp};

fn send_done(id: u32, tx_buf: &mut [u8]) -> usize {
    at::encode(b"DONE", AtOp::Response, id, &[], tx_buf).unwrap()
}

fn send_error(id: u32, code: u8, tx_buf: &mut [u8]) -> usize {
    let code_str: [u8; 3] = /* format code */;
    // Or use the codec pipeline for full encode-to-packets
    at::encode(b"ERROR", AtOp::Response, id, &[&code_str], tx_buf).unwrap()
}
```

## Full Encode Pipeline (Response → USB MIDI)

```rust
use hexa_tune_proto::codec;

let at_str = b"AT+DONE=5";
let mut sysex_buf = [0u8; 64];
let mut packets = [[0u8; 4]; 16];

let pkt_count = codec::encode_to_packets(
    at_str, &mut packets, &mut sysex_buf
).unwrap();

// Send packets[..pkt_count] over USB
for i in 0..pkt_count {
    usb_send(packets[i]);
}
```

## Error Handling

`HexaError` wraps `ProtoError` and adds domain-specific variants:

```rust
use hexa_tune_proto_embedded::error::HexaError;

match result {
    Err(HexaError::Proto(proto_err)) => {
        // Protocol-level error (parse, framing, etc.)
    }
    Err(HexaError::UnknownCommand) => {
        // Unrecognized AT command name
    }
    Err(HexaError::MissingParam) => {
        // Required parameter not provided
    }
    Err(HexaError::InvalidParam) => {
        // Parameter value out of range or wrong format
    }
    Ok(_) => {}
}
```

## Memory Budget

All protocol operations use caller-provided stack buffers. Typical sizes:

| Buffer          | Recommended Size | Notes                         |
|-----------------|------------------|-------------------------------|
| Stream decoder  | 256 bytes        | Max AT command length          |
| SysEx buffer    | 260 bytes        | AT + 2 bytes framing           |
| Packet buffer   | `[u8;4] × 90`   | ~260 / 3 rounded up           |
| AT encode       | 128 bytes        | Typical response               |

Total stack cost: ~1 KB for a complete receive + transmit path.
