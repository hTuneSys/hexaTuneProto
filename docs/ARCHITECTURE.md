# Architecture

## Overview

hexaTuneProto is a Rust workspace providing a single source of truth for the
AT-over-SysEx-over-USB-MIDI protocol used between a Flutter mobile application
and Rust embedded firmware (RP2040 / RP2350).

## Design Goals

- **Single protocol source** — one Rust codebase defines the wire format
- **no_std core** — the protocol crate runs on bare-metal with zero allocations
- **Dual target** — the same logic ships as a crates.io library (embedded) and
  a C dynamic library (Flutter FFI)
- **Caller-owned buffers** — all encode/decode functions write into slices
  provided by the caller; no heap allocation in the core

## Crate Hierarchy

```
hexa-tune-proto            (core, no_std, platform agnostic)
├── hexa-tune-proto-embedded   (no_std adapter, typed commands, heapless helpers)
└── hexa-tune-proto-ffi        (std, cdylib/staticlib, extern "C" API)
```

### hexa-tune-proto (core)

The protocol kernel. Depends only on `core`. Provides:

| Module      | Purpose                                       |
|-------------|-----------------------------------------------|
| `at`        | AT command model, parse / encode              |
| `sysex`     | SysEx frame (`F0..F7`) / unframe              |
| `usb_midi`  | USB MIDI 4-byte packetize / depacketize       |
| `stream`    | Streaming state machine decoder               |
| `codec`     | Full pipeline helpers (AT → SysEx → USB ↔)    |
| `error`     | `ProtoError` enum (`#[repr(u8)]`, Copy)       |

### hexa-tune-proto-embedded

Adapter for embedded firmware. Adds:

- `HexaCommand` — typed enum (`Version`, `SetRgb`, `Freq`, …) with
  `TryFrom<&AtMessage>` conversion
- `HexaError` — domain error enum wrapping `ProtoError`
- `dispatch::resolve()` — command resolution helper
- heapless convenience wrappers (via `heapless` dependency)

### hexa-tune-proto-ffi

C-ABI adapter for Flutter / desktop usage. Exports `extern "C"` functions:

- `htp_at_encode`, `htp_at_parse`
- `htp_sysex_frame`, `htp_sysex_unframe`
- `htp_usb_packetize`, `htp_usb_depacketize`
- `htp_encode_to_packets`

Generates `hexa_tune_proto.h` via cbindgen at build time.

## Data Flow

```
Flutter / Desktop                 Embedded Firmware
─────────────────                 ─────────────────
AT command string                 ← USB MIDI packets
    │                                     ↑
    ├─ at::encode()                       │
    ├─ sysex::frame()              stream::push_packet()
    ├─ usb_midi::packetize()              │
    │                              sysex::unframe()
    └─── USB ──────────────────►          │
                                   at::parse()
                                          │
                                   HexaCommand::try_from()
                                          │
                                   dispatch → handler
```

## Key Design Decisions

1. **Buffer strategy** — byte slice in/out (`&[u8]` / `&mut [u8]`). No
   heapless or alloc in the core crate.

2. **ID type** — `u32` numeric. Parse is strict; non-numeric IDs yield
   `Error::InvalidId`. Default is `0`.

3. **Error model** — `ProtoError` is `#[repr(u8)]` with 10 variants.
   `code()` / `from_code()` enable wire-format transport.

4. **Streaming parser** — 3-state machine (Idle / Receiving / Discarding).
   Returns `Result<Option<usize>, ProtoError>`. On buffer overflow the
   decoder enters Discarding state until F7, then returns to Idle.

5. **Feature flags** — default is no_std with zero dependencies. Optional
   features: `std` (Display, Error impl), `defmt` (embedded debug logging).

6. **FFI convention** — manual `extern "C"` + cbindgen. Return values: ≥ 0
   indicates success (often byte count), < 0 indicates `-(error_code)`.

See the project README for an overview of all design decisions.
