# hexaTuneProto

Unified Rust protocol library for the **hexaTune** AT-over-SysEx-over-USB-MIDI
communication stack. One codebase, two targets: embedded firmware (`no_std`)
and Flutter mobile app (FFI).

> **hexaTune LLC** — MIT License

## Quick Start

```bash
# Prerequisites: Rust toolchain, just (task runner)
cargo install just    # if not already installed

just build            # build workspace (dev)
just test             # run all tests
just ci               # full CI pipeline (lint + build + test + no_std check)
```

## Workspace Structure

```
hexaTuneProto/
├── crates/
│   ├── hexa-tune-proto/              # Core protocol — no_std, zero dependencies
│   │   └── src/
│   │       ├── at.rs                 # AT command parse / encode
│   │       ├── sysex.rs              # SysEx frame / unframe
│   │       ├── usb_midi.rs           # USB MIDI 4-byte packetize / depacketize
│   │       ├── stream.rs             # Streaming state machine decoder
│   │       ├── codec.rs              # Full pipeline helpers
│   │       └── error.rs              # ProtoError enum (#[repr(u8)])
│   │
│   ├── hexa-tune-proto-embedded/     # Embedded adapter — typed commands, dispatch
│   │   └── src/
│   │       ├── command.rs            # HexaCommand enum + TryFrom
│   │       ├── dispatch.rs           # Command resolution helper
│   │       └── error.rs              # HexaError (domain errors)
│   │
│   └── hexa-tune-proto-ffi/          # FFI adapter — cdylib for Flutter
│       ├── src/c_api.rs              # extern "C" functions (htp_*)
│       ├── include/                  # Generated C header
│       └── cbindgen.toml             # Header generation config
│
├── scripts/                          # Shell scripts for all tasks
├── docs/                             # Detailed documentation
├── justfile                          # Task runner (delegates to scripts/)

```

## Crates

| Crate | Target | Description |
|-------|--------|-------------|
| [`hexa-tune-proto`](crates/hexa-tune-proto/README.md) | `no_std` | Core protocol: AT, SysEx, USB MIDI, streaming decoder |
| [`hexa-tune-proto-embedded`](crates/hexa-tune-proto-embedded/README.md) | `no_std` | Typed command enums, dispatch helpers, heapless wrappers |
| [`hexa-tune-proto-ffi`](crates/hexa-tune-proto-ffi/README.md) | `std` | C-ABI functions for Flutter / desktop FFI integration |

## Protocol Stack

```
┌─────────────────────────┐
│   AT Command Layer      │  AT+FREQ=5#440#1000
├─────────────────────────┤
│   SysEx Framing         │  F0 … payload … F7
├─────────────────────────┤
│   USB MIDI Packets      │  [CIN, b1, b2, b3] × N
└─────────────────────────┘
```

Data flows through these layers in both directions — the same functions are
used on both the Flutter (via FFI) and embedded (via crate dependency) sides.

## Commands (just)

All `just` commands delegate to shell scripts in `scripts/`:

| Command | Description |
|---------|-------------|
| `just build [dev\|release\|ffi]` | Build workspace or specific target |
| `just test [all\|proto\|embedded\|ffi]` | Run tests for workspace or crate |
| `just lint [all\|clippy\|fmt\|fix]` | Run clippy / check format / auto-fix |
| `just check-nostd` | Verify no_std compilation |
| `just gen-header` | Generate C header via cbindgen |
| `just ci` | Full pipeline: lint → build → test → no_std |
| `just clean` | Remove build artifacts |

## Documentation

### Technical

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | Crate hierarchy, design decisions, data flow |
| [Protocol](docs/PROTOCOL.md) | AT command format, SysEx framing, USB MIDI packets |
| [FFI Guide](docs/FFI.md) | C API reference, Dart integration examples |
| [Embedded Guide](docs/EMBEDDED.md) | Firmware integration, StreamDecoder usage, memory budget |

### Process

| Document | Description |
|----------|-------------|
| [Branch Strategy](docs/BRANCH_STRATEGY.md) | Branching model, merge rules, protection |
| [Commit Strategy](docs/COMMIT_STRATEGY.md) | Conventional commit format and types |
| [PR Strategy](docs/PR_STRATEGY.md) | Pull request conventions and workflow |
| [Labelling Strategy](docs/LABELLING_STRATEGY.md) | Issue and PR label taxonomy |
| [Contributing](docs/CONTRIBUTING.md) | How to contribute to the project |
| [Code of Conduct](docs/CODE_OF_CONDUCT.md) | Community behavior standards |
| [Security](docs/SECURITY.md) | Vulnerability reporting policy |
| [Support](docs/SUPPORT.md) | How to get help |
| [Community](docs/COMMUNITY.md) | Community guidelines |
| [Contact](docs/CONTACT.md) | Communication channels |

## Features

The core crate uses feature flags for optional functionality:

| Feature | Default | Description |
|---------|---------|-------------|
| *(none)* | ✅ | `no_std`, zero dependencies — bare-metal ready |
| `std` | — | Adds `Display` and `std::error::Error` impls |
| `defmt` | — | Adds `defmt::Format` for embedded debug logging |

## Project Files

| File | Description |
|------|-------------|
| [AGENTS.md](AGENTS.md) | AI agent rules and project conventions |
| [LICENSE](LICENSE) | MIT License |

## License

MIT — hexaTune LLC