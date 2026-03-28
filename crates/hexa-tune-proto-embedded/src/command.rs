// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! Typed hexaTune command enum with conversion from generic `AtMessage`.

use hexa_tune_proto::at::{AtMessage, AtOp};

use crate::error::HexaError;

/// Typed hexaTune command enum.
///
/// Provides compile-time type safety for the hexaTune command set.
/// Convert from generic `AtMessage` using `TryFrom`.
#[derive(Debug, Clone)]
pub enum HexaCommand {
    /// Query firmware version (`AT+VERSION?`)
    VersionQuery,
    /// Version response (`AT+VERSION=0#x.y.z`)
    Version {
        /// Version string bytes
        version: [u8; 16],
        /// Length of version string
        version_len: usize,
    },
    /// Set RGB LED color (`AT+SETRGB=id#R#G#B`)
    SetRgb {
        /// Command tracking ID
        id: u32,
        /// Red component (0-255)
        r: u8,
        /// Green component (0-255)
        g: u8,
        /// Blue component (0-255)
        b: u8,
    },
    /// Reset device (`AT+RESET=id`)
    Reset {
        /// Command tracking ID
        id: u32,
    },
    /// Enter firmware update mode (`AT+FWUPDATE=id`)
    FwUpdate {
        /// Command tracking ID
        id: u32,
    },
    /// Set frequency output (`AT+FREQ=id#freq#timeMs#isOneShot`)
    Freq {
        /// Command tracking ID
        id: u32,
        /// Frequency in Hz
        freq: u32,
        /// Duration in milliseconds
        time_ms: u32,
        /// One-shot mode (true = play once, false = repeat)
        is_one_shot: bool,
    },
    /// Operation command (`AT+OPERATION=id#repeatCount#PREPARE` or `AT+OPERATION=id#GENERATE`)
    Operation {
        /// Command tracking ID
        id: u32,
        /// Operation sub-command
        sub: OperationSub,
        /// Repeat count (only used with PREPARE, defaults to 0)
        repeat_count: u8,
    },
    /// Operation status query (`AT+OPERATION?`)
    OperationQuery,
}

/// Operation sub-command variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationSub {
    /// Prepare for generation
    Prepare,
    /// Start generation
    Generate,
}

impl<'a> TryFrom<&AtMessage<'a>> for HexaCommand {
    type Error = HexaError;

    fn try_from(msg: &AtMessage<'a>) -> Result<Self, Self::Error> {
        match (msg.name, msg.op) {
            (b"VERSION", AtOp::Query) => Ok(HexaCommand::VersionQuery),
            (b"VERSION", AtOp::Set | AtOp::Response) => {
                let mut params = msg.params.clone();
                let ver_bytes = params.next().unwrap_or(b"");
                let mut version = [0u8; 16];
                let len = ver_bytes.len().min(16);
                version[..len].copy_from_slice(&ver_bytes[..len]);
                Ok(HexaCommand::Version {
                    version,
                    version_len: len,
                })
            }
            (b"SETRGB", AtOp::Set) => {
                let mut params = msg.params.clone();
                let r = parse_param_u8(params.next())?;
                let g = parse_param_u8(params.next())?;
                let b = parse_param_u8(params.next())?;
                Ok(HexaCommand::SetRgb {
                    id: msg.id,
                    r,
                    g,
                    b,
                })
            }
            (b"RESET", AtOp::Set) => Ok(HexaCommand::Reset { id: msg.id }),
            (b"FWUPDATE", AtOp::Set) => Ok(HexaCommand::FwUpdate { id: msg.id }),
            (b"FREQ", AtOp::Set) => {
                let mut params = msg.params.clone();
                let freq = parse_param_u32(params.next())?;
                let time_ms = parse_param_u32(params.next())?;
                let is_one_shot = parse_param_bool(params.next())?;
                Ok(HexaCommand::Freq {
                    id: msg.id,
                    freq,
                    time_ms,
                    is_one_shot,
                })
            }
            (b"OPERATION", AtOp::Query) => Ok(HexaCommand::OperationQuery),
            (b"OPERATION", AtOp::Set) => {
                let mut params = msg.params.clone();
                let first = params.next().ok_or(HexaError::MissingParam)?;
                let (repeat_count, sub_bytes) = if first == b"PREPARE" || first == b"GENERATE" {
                    (0u8, first)
                } else {
                    let rc = parse_param_u8(Some(first))?;
                    let sub = params.next().ok_or(HexaError::MissingParam)?;
                    (rc, sub)
                };
                let sub = match sub_bytes {
                    b"PREPARE" => OperationSub::Prepare,
                    b"GENERATE" => OperationSub::Generate,
                    _ => return Err(HexaError::InvalidParam),
                };
                Ok(HexaCommand::Operation {
                    id: msg.id,
                    sub,
                    repeat_count,
                })
            }
            _ => Err(HexaError::UnknownCommand),
        }
    }
}

fn parse_param_u8(param: Option<&[u8]>) -> Result<u8, HexaError> {
    let bytes = param.ok_or(HexaError::MissingParam)?;
    let mut val: u16 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return Err(HexaError::InvalidParam);
        }
        val = val * 10 + (b - b'0') as u16;
        if val > 255 {
            return Err(HexaError::InvalidParam);
        }
    }
    Ok(val as u8)
}

fn parse_param_u32(param: Option<&[u8]>) -> Result<u32, HexaError> {
    let bytes = param.ok_or(HexaError::MissingParam)?;
    let mut val: u32 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return Err(HexaError::InvalidParam);
        }
        val = val
            .checked_mul(10)
            .and_then(|v| v.checked_add((b - b'0') as u32))
            .ok_or(HexaError::InvalidParam)?;
    }
    Ok(val)
}

fn parse_param_bool(param: Option<&[u8]>) -> Result<bool, HexaError> {
    let bytes = param.ok_or(HexaError::MissingParam)?;
    match bytes {
        b"0" => Ok(false),
        b"1" => Ok(true),
        _ => Err(HexaError::InvalidParam),
    }
}
