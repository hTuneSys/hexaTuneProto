// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! hexaTune FFI adapter — C ABI functions for Flutter/Dart integration.
//!
//! All functions use caller-owned buffers (pointer + length pattern).
//! Return values: 0 = success, negative = error code.

mod c_api;

pub use c_api::*;
