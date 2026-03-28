// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! SysEx framing and unframing.
//!
//! SysEx messages wrap a payload between `0xF0` (start) and `0xF7` (end) markers.

use crate::error::ProtoError;

/// SysEx start byte.
pub const SYSEX_START: u8 = 0xF0;
/// SysEx end byte.
pub const SYSEX_END: u8 = 0xF7;

/// Frames a payload into a SysEx message: `[0xF0, ...payload..., 0xF7]`.
///
/// Returns the number of bytes written to `out`.
pub fn frame(payload: &[u8], out: &mut [u8]) -> Result<usize, ProtoError> {
    let needed = payload.len() + 2; // F0 + payload + F7
    if out.len() < needed {
        return Err(ProtoError::BufferTooSmall);
    }
    out[0] = SYSEX_START;
    out[1..1 + payload.len()].copy_from_slice(payload);
    out[1 + payload.len()] = SYSEX_END;
    Ok(needed)
}

/// Extracts the payload from a SysEx message (strips `0xF0` and `0xF7`).
///
/// Returns a slice of the payload between the markers.
pub fn unframe(data: &[u8]) -> Result<&[u8], ProtoError> {
    if data.len() < 2 {
        return Err(ProtoError::InvalidSysex);
    }
    if data[0] != SYSEX_START || data[data.len() - 1] != SYSEX_END {
        return Err(ProtoError::InvalidSysex);
    }
    Ok(&data[1..data.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_basic() {
        let mut buf = [0u8; 32];
        let n = frame(b"AT+VERSION?", &mut buf).unwrap();
        assert_eq!(buf[0], SYSEX_START);
        assert_eq!(&buf[1..n - 1], b"AT+VERSION?");
        assert_eq!(buf[n - 1], SYSEX_END);
    }

    #[test]
    fn frame_empty_payload() {
        let mut buf = [0u8; 8];
        let n = frame(b"", &mut buf).unwrap();
        assert_eq!(n, 2);
        assert_eq!(&buf[..n], &[SYSEX_START, SYSEX_END]);
    }

    #[test]
    fn frame_buffer_too_small() {
        let mut buf = [0u8; 3];
        assert_eq!(frame(b"AB", &mut buf), Err(ProtoError::BufferTooSmall));
    }

    #[test]
    fn unframe_basic() {
        let data = [SYSEX_START, b'H', b'I', SYSEX_END];
        assert_eq!(unframe(&data).unwrap(), b"HI");
    }

    #[test]
    fn unframe_empty_payload() {
        let data = [SYSEX_START, SYSEX_END];
        assert_eq!(unframe(&data).unwrap(), b"");
    }

    #[test]
    fn unframe_invalid_start() {
        let data = [0x00, b'H', SYSEX_END];
        assert_eq!(unframe(&data), Err(ProtoError::InvalidSysex));
    }

    #[test]
    fn unframe_invalid_end() {
        let data = [SYSEX_START, b'H', 0x00];
        assert_eq!(unframe(&data), Err(ProtoError::InvalidSysex));
    }

    #[test]
    fn unframe_too_short() {
        assert_eq!(unframe(&[]), Err(ProtoError::InvalidSysex));
        assert_eq!(unframe(&[SYSEX_START]), Err(ProtoError::InvalidSysex));
    }

    #[test]
    fn roundtrip() {
        let payload = b"AT+FREQ=1#440#1000#1";
        let mut buf = [0u8; 64];
        let n = frame(payload, &mut buf).unwrap();
        let extracted = unframe(&buf[..n]).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn roundtrip_stop() {
        let payload = b"AT+OPERATION=5#STOP#GRACEFUL";
        let mut buf = [0u8; 64];
        let n = frame(payload, &mut buf).unwrap();
        let extracted = unframe(&buf[..n]).unwrap();
        assert_eq!(extracted, payload);
    }
}
