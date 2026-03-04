// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! Protocol error types.

/// Protocol-level errors.
///
/// Represented as `u8` for efficient FFI and wire format encoding.
/// Domain-specific errors (e.g., DdsBusy) belong in adapter crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProtoError {
    /// Input does not start with `AT+`
    InvalidCommand = 1,
    /// ID field is not a valid decimal number
    InvalidId = 2,
    /// Input contains invalid UTF-8 sequences
    InvalidUtf8 = 3,
    /// Too many parameters for the fixed-size buffer
    ParamCount = 4,
    /// Output buffer is too small for the encoded data
    BufferTooSmall = 5,
    /// SysEx data is missing `0xF0` start or `0xF7` end marker
    InvalidSysex = 6,
    /// USB MIDI packet has an unrecognized CIN code
    MalformedPacket = 7,
    /// Streaming decoder buffer overflow (message too large)
    Overflow = 8,
    /// AT command name is empty
    EmptyName = 9,
    /// Input is empty or whitespace-only
    EmptyInput = 10,
}

impl ProtoError {
    /// Returns the numeric error code.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// Reconstructs an error from a numeric code.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::InvalidCommand),
            2 => Some(Self::InvalidId),
            3 => Some(Self::InvalidUtf8),
            4 => Some(Self::ParamCount),
            5 => Some(Self::BufferTooSmall),
            6 => Some(Self::InvalidSysex),
            7 => Some(Self::MalformedPacket),
            8 => Some(Self::Overflow),
            9 => Some(Self::EmptyName),
            10 => Some(Self::EmptyInput),
            _ => None,
        }
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for ProtoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCommand => write!(f, "invalid command (missing AT+ prefix)"),
            Self::InvalidId => write!(f, "invalid id (not a decimal number)"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8"),
            Self::ParamCount => write!(f, "too many parameters"),
            Self::BufferTooSmall => write!(f, "buffer too small"),
            Self::InvalidSysex => write!(f, "invalid SysEx framing"),
            Self::MalformedPacket => write!(f, "malformed USB MIDI packet"),
            Self::Overflow => write!(f, "stream decoder overflow"),
            Self::EmptyName => write!(f, "empty command name"),
            Self::EmptyInput => write!(f, "empty input"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ProtoError {}

#[cfg(feature = "defmt")]
impl defmt::Format for ProtoError {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "ProtoError({})", self.code());
    }
}
