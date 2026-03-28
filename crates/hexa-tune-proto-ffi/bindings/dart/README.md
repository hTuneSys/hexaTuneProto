# hexaTuneProto Dart FFI Bindings

Dart wrapper for the `hexa-tune-proto-ffi` native library.

## Dependency

This package requires the `ffi` package:

```yaml
dependencies:
  ffi: ^2.0.0
```

## Usage

```dart
import 'package:hexa_tune_proto_ffi/hexa_tune_proto_ffi.dart';

// Android — load shared library
final proto = HexaTuneProto('libhexa_tune_proto_ffi.so');

// iOS — static linking (symbols in process)
final proto = HexaTuneProto.open();

// Encode AT command → USB MIDI packets (full pipeline)
final packets = proto.encodeToPackets('FREQ', id: 5, params: ['440', '1000', '1']);

// Parse AT response
final result = proto.atParse(responseBytes);
print(result.name);   // e.g. "VERSION"
print(result.params);  // e.g. ["1.0.0"]

// Individual steps
final atBytes = proto.atEncode('SETRGB', id: 1, params: ['255', '0', '128']);
final sysex = proto.sysexFrame(atBytes);
final usbPackets = proto.usbPacketize(sysex);
```

## API

| Method | Description |
|--------|-------------|
| `atEncode(name, {id, op, params})` | Encode AT command string |
| `atParse(input)` | Parse AT command from bytes |
| `sysexFrame(payload)` | Frame payload into SysEx |
| `sysexUnframe(data)` | Extract payload from SysEx |
| `usbPacketize(sysex)` | Convert SysEx to USB MIDI packets |
| `usbDepacketize(packets)` | Reassemble SysEx from USB MIDI packets |
| `encodeToPackets(name, ...)` | Full pipeline: AT → SysEx → USB packets |

## Error Handling

All methods throw `HexaTuneProtoError` on failure. Error codes match
`ProtoError` variants from the Rust core crate (1–10).
