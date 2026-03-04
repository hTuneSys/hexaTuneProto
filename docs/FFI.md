# FFI Guide

## Overview

The `hexa-tune-proto-ffi` crate exports a C-compatible API that can be consumed
by any language with C FFI support — most notably Dart (Flutter) via `dart:ffi`.

## Build

```bash
just build ffi        # Release build of the cdylib
just gen-header       # Regenerate C header
```

Build artifacts:

| Platform | Library                          |
|----------|----------------------------------|
| Linux    | `target/release/libhexa_tune_proto_ffi.so` |
| macOS    | `target/release/libhexa_tune_proto_ffi.dylib` |
| Windows  | `target/release/hexa_tune_proto_ffi.dll` |
| iOS      | `target/release/libhexa_tune_proto_ffi.a` (staticlib) |
| Android  | Cross-compile per ABI            |

The generated C header is at:
`crates/hexa-tune-proto-ffi/include/hexa_tune_proto.h`

## Calling Convention

All functions follow these rules:

- **Caller-owned buffers** — the caller allocates input and output buffers
- **Return value** — `int32_t`:
  - `>= 0` → success (usually byte count or packet count written)
  - `< 0` → error: `-(ProtoError code)` (see [PROTOCOL.md](PROTOCOL.md))
- **No heap allocation** — Rust never allocates; all work happens in provided
  buffers
- **Thread safety** — all functions are stateless and re-entrant

## API Reference

### htp_at_encode

Encode an AT command into a byte buffer.

```c
int32_t htp_at_encode(
    const uint8_t *name_ptr, uint32_t name_len,
    uint8_t op,        // 0=Set, 1=Query, 2=Response
    uint32_t id,
    const uint8_t *params_ptr, uint32_t params_len,
    uint8_t *out_ptr, uint32_t out_cap
);
```

### htp_at_parse

Parse an AT command string into components.

```c
int32_t htp_at_parse(
    const uint8_t *input_ptr, uint32_t input_len,
    HtpAtParseResult *result
);
```

`HtpAtParseResult` contains offsets into the original input buffer for
zero-copy access to name, ID, and parameters.

### htp_sysex_frame

Wrap payload bytes in SysEx framing (F0 … F7).

```c
int32_t htp_sysex_frame(
    const uint8_t *payload_ptr, uint32_t payload_len,
    uint8_t *out_ptr, uint32_t out_cap
);
```

### htp_sysex_unframe

Extract payload from a SysEx message.

```c
int32_t htp_sysex_unframe(
    const uint8_t *sysex_ptr, uint32_t sysex_len,
    uint8_t *out_ptr, uint32_t out_cap
);
```

### htp_usb_packetize

Convert SysEx bytes into 4-byte USB MIDI packets.

```c
int32_t htp_usb_packetize(
    const uint8_t *sysex_ptr, uint32_t sysex_len,
    uint8_t *out_packets_ptr, uint32_t out_packets_cap
);
```

Output buffer must be a multiple of 4 bytes. Returns the number of packets
written.

### htp_usb_depacketize

Reassemble USB MIDI packets into SysEx bytes.

```c
int32_t htp_usb_depacketize(
    const uint8_t *packets_ptr, uint32_t packet_count,
    uint8_t *out_ptr, uint32_t out_cap
);
```

### htp_encode_to_packets

Full pipeline: AT string → SysEx → USB MIDI packets.

```c
int32_t htp_encode_to_packets(
    const uint8_t *at_ptr, uint32_t at_len,
    uint8_t *pkt_out_ptr, uint32_t pkt_out_cap,
    uint8_t *sysex_buf_ptr, uint32_t sysex_buf_cap
);
```

## Dart Integration Example

```dart
import 'dart:ffi';
import 'package:ffi/ffi.dart';

final lib = DynamicLibrary.open('libhexa_tune_proto_ffi.so');

typedef HtpSysexFrameNative = Int32 Function(
  Pointer<Uint8>, Uint32,
  Pointer<Uint8>, Uint32,
);
typedef HtpSysexFrameDart = int Function(
  Pointer<Uint8>, int,
  Pointer<Uint8>, int,
);

final htpSysexFrame = lib.lookupFunction<
    HtpSysexFrameNative, HtpSysexFrameDart>('htp_sysex_frame');

// Usage:
// Allocate buffers, call htpSysexFrame, read result
```

## Error Handling in FFI

```dart
final result = htpSysexFrame(payloadPtr, payloadLen, outPtr, outCap);
if (result < 0) {
  final errorCode = -result;
  // Map to ProtoError variant (see PROTOCOL.md error codes)
  throw Exception('Protocol error: code $errorCode');
}
final bytesWritten = result;
```
