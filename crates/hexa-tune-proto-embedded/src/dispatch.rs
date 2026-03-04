// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! Dispatch helpers for routing parsed AT messages to typed handlers.

use hexa_tune_proto::at::AtMessage;

use crate::command::HexaCommand;
use crate::error::HexaError;

/// Converts a generic `AtMessage` into a typed `HexaCommand`.
///
/// This is a convenience wrapper around `HexaCommand::try_from`.
pub fn resolve(msg: &AtMessage<'_>) -> Result<HexaCommand, HexaError> {
    HexaCommand::try_from(msg)
}
