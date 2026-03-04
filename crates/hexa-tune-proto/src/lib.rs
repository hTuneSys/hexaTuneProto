// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! # hexa-tune-proto
//!
//! AT-over-SysEx-over-USB-MIDI protocol core.
//!
//! This crate provides the canonical implementation of the hexaTune communication
//! protocol used between Flutter mobile apps and embedded firmware over USB MIDI.
//!
//! ## Architecture
//!
//! - **`at`** — AT command parse/encode (`AT+NAME=id#P1#P2`, `AT+NAME?`)
//! - **`sysex`** — SysEx framing/unframing (`0xF0 ... 0xF7`)
//! - **`usb_midi`** — USB MIDI 4-byte packet conversion (CIN codes)
//! - **`stream`** — Streaming decoder (state machine for incremental USB packets)
//! - **`codec`** — Full pipeline helpers (AT → SysEx → USB and back)
//!
//! ## no_std
//!
//! This crate is `no_std` by default with zero dependencies. Enable the `std`
//! feature for `Display` and `Error` trait implementations.

#![no_std]
#![deny(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
extern crate alloc;

pub mod at;
pub mod codec;
pub mod error;
pub mod stream;
pub mod sysex;
pub mod usb_midi;

pub use at::{AtMessage, AtOp, Params};
pub use error::ProtoError;
pub use stream::StreamDecoder;
