// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! hexaTune embedded adapter — typed commands, dispatch helpers, heapless wrappers.

#![no_std]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod command;
pub mod dispatch;
pub mod error;

pub use command::HexaCommand;
pub use error::HexaError;
