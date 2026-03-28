# AT-over-SysEx-over-USB-MIDI Protocol

## Overview

hexaTune devices communicate using AT commands transported inside MIDI SysEx
messages over USB MIDI. The protocol stack has three layers:

```
┌─────────────────────┐
│   AT Command Layer  │  human-readable text
├─────────────────────┤
│   SysEx Framing     │  0xF0 … 0xF7
├─────────────────────┤
│   USB MIDI Packets  │  4-byte packets with CIN codes
└─────────────────────┘
```

## AT Command Format

All commands follow the `AT+` prefix convention:

| Form                        | Example                          | Description         |
|-----------------------------|----------------------------------|---------------------|
| `AT+NAME?`                  | `AT+VERSION?`                    | Query               |
| `AT+NAME=id`                | `AT+RESET=1`                     | Set (no params)     |
| `AT+NAME=id#P1#P2…`         | `AT+FREQ=5#440#1000#1`             | Set with parameters |

### Response Format

| Form                        | Example                          | Description         |
|-----------------------------|----------------------------------|---------------------|
| `AT+NAME=id#…#COMPLETED`    | `AT+VERSION=0#1.2.3#COMPLETED`   | Success with data   |
| `AT+DONE=id`                | `AT+DONE=5`                      | Acknowledgement     |
| `AT+ERROR=id#code`          | `AT+ERROR=5#3`                   | Error (u8 code)     |

### ID Field

- Type: `u32` (decimal string on wire)
- Default: `0` (used for unsolicited or broadcast messages)
- Non-numeric values are rejected with `InvalidId` error

### Parameter Delimiter

Parameters are separated by `#`. No escape mechanism exists; parameters must
not contain `#`.

## Current Commands

| Command     | Direction | Parameters              | Description            |
|-------------|-----------|-------------------------|------------------------|
| `VERSION`   | Query     | —                       | Firmware version       |
| `SETRGB`    | Set       | `r#g#b`                 | Set LED color          |
| `RESET`     | Set       | —                       | Device reset           |
| `FWUPDATE`  | Set       | —                       | Enter update mode      |
| `FREQ`      | Set       | `frequency#duration#isOneShot` | Play frequency (Hz/ms/bool) |
| `OPERATION` | Set       | `[repeatCount#]PREPARE` or `GENERATE` | Operation control      |

## SysEx Framing

AT command bytes are wrapped in a standard MIDI SysEx envelope:

```
0xF0  <payload bytes>  0xF7
```

- `0xF0` — SysEx start
- `0xF7` — SysEx end (EOX)
- Payload is the raw AT command string bytes (UTF-8 / ASCII)

### API

```rust
// Frame payload into SysEx
sysex::frame(payload: &[u8], out: &mut [u8]) -> Result<usize, ProtoError>

// Extract payload from SysEx
sysex::unframe(sysex: &[u8]) -> Result<&[u8], ProtoError>
```

## USB MIDI Packetization

SysEx bytes are split into 4-byte USB MIDI packets. Each packet has:

```
[CIN, byte1, byte2, byte3]
```

### CIN Codes

| CIN  | Meaning                        | Data bytes used |
|------|--------------------------------|-----------------|
| 0x04 | SysEx start or continue        | 3               |
| 0x05 | SysEx end — 1 data byte        | 1               |
| 0x06 | SysEx end — 2 data bytes       | 2               |
| 0x07 | SysEx end — 3 data bytes       | 3               |

### Packetization Rules

1. Fill packets with 3 data bytes each using CIN `0x04`
2. Last packet uses CIN `0x05`, `0x06`, or `0x07` depending on remaining bytes
3. Unused trailing bytes in the last packet are zero-padded

### API

```rust
// SysEx bytes → USB MIDI packets
usb_midi::packetize(sysex: &[u8], out: &mut [[u8; 4]]) -> Result<usize, ProtoError>

// USB MIDI packets → SysEx bytes
usb_midi::depacketize(packets: &[[u8; 4]], count: usize, out: &mut [u8]) -> Result<usize, ProtoError>
```

## Streaming Decoder

For real-time USB MIDI reception, `StreamDecoder` processes packets one at a
time using a 3-state machine:

```
         ┌──────────┐
    ─────►   Idle    │
         └────┬─────┘
              │ F0 seen
         ┌────▼─────┐
    ─────► Receiving │──── F7 → Some(len) → Idle
         └────┬─────┘
              │ overflow
         ┌────▼──────┐
    ─────► Discarding │──── F7 → None → Idle
         └───────────┘
```

### API

```rust
let mut buf = [0u8; 256];
let mut dec = StreamDecoder::new(&mut buf);

// Push one 4-byte USB MIDI packet at a time
match dec.push_packet(packet) {
    Ok(Some(len)) => { /* complete message in buf[..len] */ }
    Ok(None)      => { /* need more packets */ }
    Err(e)        => { /* protocol error */ }
}
```

## Full Pipeline (codec)

The `codec` module provides convenience functions for the complete encode and
decode chain:

```rust
// AT string → SysEx → USB MIDI packets
codec::encode_to_packets(at_str: &[u8], pkt_out: &mut [[u8; 4]], sysex_buf: &mut [u8])
    -> Result<usize, ProtoError>

// USB MIDI packets → SysEx → AT payload bytes
codec::decode_from_packets(packets: &[[u8; 4]], count: usize, sysex_buf: &mut [u8], at_buf: &mut [u8])
    -> Result<usize, ProtoError>
```

## Error Codes

| Code | Variant          | Description                          |
|------|------------------|--------------------------------------|
| 1    | InvalidPrefix    | Missing `AT+` prefix                 |
| 2    | InvalidFormat    | Malformed AT command structure        |
| 3    | InvalidId        | Non-numeric ID field                  |
| 4    | BufferTooSmall   | Output buffer insufficient            |
| 5    | InvalidSysex     | Bad SysEx framing (missing F0/F7)     |
| 6    | InvalidPacket    | Unknown USB MIDI CIN code             |
| 7    | EmptyInput       | Zero-length input                     |
| 8    | Overflow         | Streaming buffer capacity exceeded    |
| 9    | Incomplete       | Truncated message                     |
| 10   | InvalidUtf8      | Payload not valid UTF-8               |
