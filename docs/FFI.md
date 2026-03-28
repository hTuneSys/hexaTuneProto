# FFI Guide

## Overview

The `hexa-tune-proto-ffi` crate exports a C-compatible API that can be consumed
by any language with C FFI support — most notably Dart (Flutter) via `dart:ffi`.

## Build

```bash
just build ffi          # Host platform release build
just gen-header         # Regenerate C header
just build-android      # Cross-compile for Android (release)
just build-ios          # Cross-compile for iOS (release)
```

Build artifacts:

| Platform | Library | Command |
|----------|---------|---------|
| Linux    | `target/release/libhexa_tune_proto_ffi.so` | `just build ffi` |
| macOS    | `target/release/libhexa_tune_proto_ffi.dylib` | `just build ffi` |
| Windows  | `target/release/hexa_tune_proto_ffi.dll` | `just build ffi` |
| Android  | `target/android/jniLibs/{abi}/libhexa_tune_proto_ffi.so` | `just build-android` |
| iOS      | `target/ios/HexaTuneProto.xcframework` | `just build-ios` |

The generated C header is at:
`crates/hexa-tune-proto-ffi/include/hexa_tune_proto.h`

## Calling Convention

All functions follow these rules:

- **Caller-owned buffers** — the caller allocates input and output buffers
- **Return value** — `int32_t`:
  - `0` → success (byte/packet count written to `out_len`/`out_count` pointer)
  - `< 0` → error: `-(ProtoError code)` (see [PROTOCOL.md](PROTOCOL.md))
- **No heap allocation** — Rust never allocates; all work happens in provided
  buffers
- **Thread safety** — all functions are stateless and re-entrant

## API Reference

All function signatures come from the generated header (`hexa_tune_proto.h`).

### Structs

```c
typedef struct HtpSlice {
    const uint8_t *ptr;
    uintptr_t len;
} HtpSlice;

typedef struct HtpAtParseResult {
    uint32_t id;
    int32_t op;              // 0=Set, 1=Query, 2=Response
    uintptr_t name_offset;   // offset into input buffer
    uintptr_t name_len;
    uintptr_t param_count;   // max 8
    uintptr_t param_offsets[8];
    uintptr_t param_lens[8];
} HtpAtParseResult;
```

### htp_at_encode

Encode an AT command into a byte buffer.

```c
int32_t htp_at_encode(
    const uint8_t *name_ptr, uintptr_t name_len,
    uint32_t id, int32_t op,
    const HtpSlice *params_ptr, uintptr_t params_count,
    uint8_t *out_ptr, uintptr_t out_cap, uintptr_t *out_len
);
```

### htp_at_parse

Parse an AT command string into components.

```c
int32_t htp_at_parse(
    const uint8_t *input_ptr, uintptr_t input_len,
    HtpAtParseResult *result
);
```

`HtpAtParseResult` contains offsets into the original input buffer for
zero-copy access to name, ID, and parameters.

### htp_sysex_frame

Wrap payload bytes in SysEx framing (F0 … F7).

```c
int32_t htp_sysex_frame(
    const uint8_t *payload_ptr, uintptr_t payload_len,
    uint8_t *out_ptr, uintptr_t out_cap, uintptr_t *out_len
);
```

### htp_sysex_unframe

Extract payload from a SysEx message.

```c
int32_t htp_sysex_unframe(
    const uint8_t *data_ptr, uintptr_t data_len,
    uintptr_t *out_offset, uintptr_t *out_len
);
```

### htp_usb_packetize

Convert SysEx bytes into 4-byte USB MIDI packets.

```c
int32_t htp_usb_packetize(
    const uint8_t *sysex_ptr, uintptr_t sysex_len,
    uint8_t (*out_ptr)[4], uintptr_t out_cap, uintptr_t *out_count
);
```

### htp_usb_depacketize

Reassemble USB MIDI packets into SysEx bytes.

```c
int32_t htp_usb_depacketize(
    const uint8_t (*packets_ptr)[4], uintptr_t packet_count,
    uint8_t *out_ptr, uintptr_t out_cap, uintptr_t *out_len
);
```

### htp_encode_to_packets

Full pipeline: AT command → SysEx → USB MIDI packets.

```c
int32_t htp_encode_to_packets(
    const uint8_t *name_ptr, uintptr_t name_len,
    uint32_t id, int32_t op,
    const HtpSlice *params_ptr, uintptr_t params_count,
    uint8_t *at_buf_ptr, uintptr_t at_buf_cap,
    uint8_t *sysex_buf_ptr, uintptr_t sysex_buf_cap,
    uint8_t (*packets_out_ptr)[4], uintptr_t packets_out_cap,
    uintptr_t *packets_out_count
);
```

## Dart Bindings

Pre-built Dart FFI bindings are provided at:
`crates/hexa-tune-proto-ffi/bindings/dart/`

See [Dart bindings README](../crates/hexa-tune-proto-ffi/bindings/dart/README.md)
for usage.

```dart
import 'package:hexa_tune_proto_ffi/hexa_tune_proto_ffi.dart';

final proto = HexaTuneProto('libhexa_tune_proto_ffi.so');

// Full pipeline: AT → SysEx → USB packets
final packets = proto.encodeToPackets('FREQ', id: 5, params: ['440', '1000', '1']);

// Parse response
final result = proto.atParse(responseBytes);
print('${result.name} id=${result.id} params=${result.params}');
```

## Flutter Integration

### Prerequisites

- Rust toolchain with cross-compilation targets
- Android: [cargo-ndk](https://github.com/nickelc/cargo-ndk) + Android NDK
- iOS: macOS + Xcode

### Android Setup

1. Build the native libraries:

   ```bash
   just build-android release
   ```

2. Copy the output to your Flutter project:

   ```bash
   cp -r target/android/jniLibs/ \
       /path/to/flutter_project/android/app/src/main/jniLibs/
   ```

   Directory structure:
   ```
   android/app/src/main/jniLibs/
   ├── arm64-v8a/libhexa_tune_proto_ffi.so
   ├── armeabi-v7a/libhexa_tune_proto_ffi.so
   └── x86_64/libhexa_tune_proto_ffi.so
   ```

3. Load in Dart:
   ```dart
   final proto = HexaTuneProto('libhexa_tune_proto_ffi.so');
   ```

### iOS Setup

1. Build the XCFramework:

   ```bash
   just build-ios release
   ```

2. Add `target/ios/HexaTuneProto.xcframework` to your Xcode project:
   - Open Runner.xcworkspace
   - Go to Runner target → General → Frameworks, Libraries, and Embedded Content
   - Add the XCFramework

3. Add the header search path in Xcode Build Settings:
   - Header Search Paths → add path to `include/` directory

4. Load in Dart:
   ```dart
   final proto = HexaTuneProto.open(); // iOS uses static linking
   ```

### Automation

For CI/CD, chain the build and copy steps:

```bash
# Build all platforms
just build-android release
just build-ios release

# Copy to Flutter project
cp -r target/android/jniLibs/ $FLUTTER_PROJECT/android/app/src/main/jniLibs/
cp -r target/ios/HexaTuneProto.xcframework $FLUTTER_PROJECT/ios/
```

## Error Handling

All `htp_*` functions return negative values on error. The absolute value maps
to a `ProtoError` code:

| Code | Error | Description |
|------|-------|-------------|
| 1 | BufferTooSmall | Output buffer capacity insufficient |
| 2 | InvalidSysex | Malformed SysEx (missing F0/F7) |
| 3 | InvalidAtCommand | Unparseable AT string |
| 4 | InvalidId | Non-numeric ID field |
| 5 | PayloadNotUtf8 | Payload bytes are not valid UTF-8 |
| 6 | InvalidPacket | Malformed USB MIDI packet |
| 7 | Overflow | Internal buffer overflow |
| 8 | EmptyInput | Zero-length input |
| 9 | MissingPrefix | AT command missing `AT+` prefix |
| 10 | MissingOperator | AT command missing `=` or `?` operator |

In Dart, the `HexaTuneProto` wrapper throws `HexaTuneProtoError` with the
code and function name.
