# hexa-tune-proto

Core protocol crate for the **hexaTune** AT-over-SysEx-over-USB-MIDI
communication stack.

> `no_std` · zero dependencies · caller-owned buffers

## Overview

This crate is the single source of truth for the hexaTune wire protocol. It
provides parse, encode, frame, packetize, and streaming decode functions that
work identically on bare-metal embedded targets and desktop/mobile hosts.

## Modules

| Module     | Purpose                                              |
|------------|------------------------------------------------------|
| `at`       | AT command model — parse and encode (`AT+NAME=id#…`) |
| `sysex`    | SysEx framing — wrap/unwrap with `0xF0` … `0xF7`    |
| `usb_midi` | USB MIDI 4-byte packet conversion (CIN codes)        |
| `stream`   | Streaming state machine decoder for incremental USB  |
| `codec`    | Full pipeline helpers (AT ↔ SysEx ↔ USB MIDI)        |
| `error`    | `ProtoError` enum — `#[repr(u8)]`, Copy, 10 variants|

## Usage

```toml
# Embedded (no_std)
[dependencies]
hexa-tune-proto = { version = "0.1", default-features = false }

# Desktop / tests (std)
[dependencies]
hexa-tune-proto = { version = "0.1", features = ["std"] }
```

### Parse an AT command

```rust
use hexa_tune_proto::at;

let msg = at::parse(b"AT+FREQ=5#440#1000#1").unwrap();
assert_eq!(msg.name, b"FREQ");
assert_eq!(msg.id, 5);
```

### Encode an AT command

```rust
use hexa_tune_proto::at::{self, AtOp};

let mut buf = [0u8; 64];
let len = at::encode(b"DONE", AtOp::Response, 5, &[], &mut buf).unwrap();
assert_eq!(&buf[..len], b"AT+DONE=5");
```

### Full pipeline (AT → SysEx → USB MIDI packets)

```rust
use hexa_tune_proto::codec;

let mut sysex_buf = [0u8; 128];
let mut packets = [[0u8; 4]; 32];
let count = codec::encode_to_packets(
    b"AT+VERSION?", &mut packets, &mut sysex_buf
).unwrap();
// packets[..count] ready to send over USB
```

### Streaming decoder (packet-by-packet)

```rust
use hexa_tune_proto::stream::StreamDecoder;

let mut buf = [0u8; 256];
let mut dec = StreamDecoder::new(&mut buf);

// Push USB MIDI packets one at a time
match dec.push_packet(packet) {
    Ok(Some(len)) => { /* complete message in buf[..len] */ }
    Ok(None)      => { /* need more packets */ }
    Err(e)        => { /* protocol error — decoder auto-recovers */ }
}
```

## Features

| Feature | Default | Description                                  |
|---------|---------|----------------------------------------------|
| *(none)*| ✅      | `no_std`, zero deps — bare-metal ready        |
| `std`   | —       | `Display` and `std::error::Error` impls       |
| `defmt` | —       | `defmt::Format` for embedded debug logging    |

## Error Codes

All errors are `ProtoError` with `#[repr(u8)]`:

| Code | Variant        | Description                        |
|------|----------------|------------------------------------|
| 1    | InvalidPrefix  | Missing `AT+` prefix               |
| 2    | InvalidFormat  | Malformed AT command structure      |
| 3    | InvalidId      | Non-numeric ID field                |
| 4    | BufferTooSmall | Output buffer insufficient          |
| 5    | InvalidSysex   | Bad SysEx framing (missing F0/F7)   |
| 6    | MalformedPacket| Unknown USB MIDI CIN code           |
| 7    | EmptyInput     | Zero-length input                   |
| 8    | Overflow       | Streaming buffer capacity exceeded  |
| 9    | Incomplete     | Truncated message                   |
| 10   | InvalidUtf8    | Payload not valid UTF-8             |

## License

MIT — hexaTune LLC
