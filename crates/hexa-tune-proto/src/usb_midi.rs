// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! USB MIDI 4-byte packet conversion.
//!
//! USB MIDI Class specification uses 4-byte packets:
//! `[CIN, byte1, byte2, byte3]`
//!
//! ## CIN Codes for SysEx
//! - `0x04` — SysEx start or continue (3 data bytes)
//! - `0x05` — SysEx end with 1 data byte
//! - `0x06` — SysEx end with 2 data bytes
//! - `0x07` — SysEx end with 3 data bytes

use crate::error::ProtoError;

/// CIN code: SysEx start or continue (3 data bytes).
pub const CIN_SYSEX_START: u8 = 0x04;
/// CIN code: SysEx end with 1 byte.
pub const CIN_SYSEX_END_1: u8 = 0x05;
/// CIN code: SysEx end with 2 bytes.
pub const CIN_SYSEX_END_2: u8 = 0x06;
/// CIN code: SysEx end with 3 bytes.
pub const CIN_SYSEX_END_3: u8 = 0x07;

/// Converts a SysEx byte stream into USB MIDI 4-byte packets.
///
/// `sysex` must be a complete SysEx message (starting with `0xF0`, ending with `0xF7`).
/// `out` is a buffer of 4-byte packet slots.
///
/// Returns the number of packets written.
pub fn packetize(sysex: &[u8], out: &mut [[u8; 4]]) -> Result<usize, ProtoError> {
    if sysex.len() < 2 {
        return Err(ProtoError::InvalidSysex);
    }

    let mut packet_idx = 0;
    let mut i = 0;

    while i < sysex.len() {
        let rem = sysex.len() - i;

        if packet_idx >= out.len() {
            return Err(ProtoError::BufferTooSmall);
        }

        if rem >= 3 {
            if rem == 3 && sysex[sysex.len() - 1] == 0xF7 {
                // End with 3 bytes
                out[packet_idx] = [CIN_SYSEX_END_3, sysex[i], sysex[i + 1], sysex[i + 2]];
            } else {
                // Start or continue
                out[packet_idx] = [CIN_SYSEX_START, sysex[i], sysex[i + 1], sysex[i + 2]];
            }
            i += 3;
        } else if rem == 2 {
            out[packet_idx] = [CIN_SYSEX_END_2, sysex[i], sysex[i + 1], 0x00];
            i += 2;
        } else {
            out[packet_idx] = [CIN_SYSEX_END_1, sysex[i], 0x00, 0x00];
            i += 1;
        }
        packet_idx += 1;
    }

    Ok(packet_idx)
}

/// Reconstructs a SysEx byte stream from USB MIDI 4-byte packets.
///
/// `packets` is a slice of 4-byte USB MIDI packets.
/// `out` is the output buffer for the reassembled SysEx bytes (including `0xF0` and `0xF7`).
///
/// Returns the number of bytes written.
pub fn depacketize(packets: &[[u8; 4]], out: &mut [u8]) -> Result<usize, ProtoError> {
    let mut pos = 0;

    for packet in packets {
        let cin = packet[0] & 0x0F;
        match cin {
            CIN_SYSEX_START => {
                push_if_nonzero(out, &mut pos, packet[1])?;
                push_if_nonzero(out, &mut pos, packet[2])?;
                push_if_nonzero(out, &mut pos, packet[3])?;
            }
            CIN_SYSEX_END_1 => {
                push_if_nonzero(out, &mut pos, packet[1])?;
            }
            CIN_SYSEX_END_2 => {
                push_if_nonzero(out, &mut pos, packet[1])?;
                push_if_nonzero(out, &mut pos, packet[2])?;
            }
            CIN_SYSEX_END_3 => {
                push_if_nonzero(out, &mut pos, packet[1])?;
                push_if_nonzero(out, &mut pos, packet[2])?;
                push_if_nonzero(out, &mut pos, packet[3])?;
            }
            _ => return Err(ProtoError::MalformedPacket),
        }
    }

    Ok(pos)
}

fn push_if_nonzero(out: &mut [u8], pos: &mut usize, byte: u8) -> Result<(), ProtoError> {
    if byte != 0 {
        if *pos >= out.len() {
            return Err(ProtoError::BufferTooSmall);
        }
        out[*pos] = byte;
        *pos += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysex::{SYSEX_END, SYSEX_START};

    #[test]
    fn packetize_short_message() {
        // F0 H I F7 = 4 bytes → 2 packets: [04,F0,H,I] [05,F7,00,00]
        let sysex = [SYSEX_START, b'H', b'I', SYSEX_END];
        let mut packets = [[0u8; 4]; 4];
        let n = packetize(&sysex, &mut packets).unwrap();
        // 4 bytes, rem=4 ≥ 3 but not last 3; first [04,F0,H,I], then rem=1 [05,F7,00,00]
        assert_eq!(n, 2);
        assert_eq!(packets[0], [CIN_SYSEX_START, SYSEX_START, b'H', b'I']);
        assert_eq!(packets[1], [CIN_SYSEX_END_1, SYSEX_END, 0x00, 0x00]);
    }

    #[test]
    fn packetize_exact_3_end() {
        // F0 A B C D E F7 = 7 bytes
        let sysex = [SYSEX_START, b'A', b'B', b'C', b'D', b'E', SYSEX_END];
        let mut packets = [[0u8; 4]; 4];
        let n = packetize(&sysex, &mut packets).unwrap();
        assert_eq!(n, 3);
        assert_eq!(packets[0], [CIN_SYSEX_START, SYSEX_START, b'A', b'B']);
        assert_eq!(packets[1], [CIN_SYSEX_START, b'C', b'D', b'E']);
        assert_eq!(packets[2], [CIN_SYSEX_END_1, SYSEX_END, 0x00, 0x00]);
    }

    #[test]
    fn packetize_end_with_3() {
        // F0 A B C D F7 = 6 bytes → last 3 = [D, F7] wait no...
        // Actually: [04, F0, A, B] [07, C, D, F7]
        let sysex = [SYSEX_START, b'A', b'B', b'C', b'D', SYSEX_END];
        let mut packets = [[0u8; 4]; 4];
        let n = packetize(&sysex, &mut packets).unwrap();
        assert_eq!(n, 2);
        assert_eq!(packets[0], [CIN_SYSEX_START, SYSEX_START, b'A', b'B']);
        assert_eq!(packets[1], [CIN_SYSEX_END_3, b'C', b'D', SYSEX_END]);
    }

    #[test]
    fn packetize_buffer_too_small() {
        let sysex = [SYSEX_START, b'A', b'B', b'C', b'D', SYSEX_END];
        let mut packets = [[0u8; 4]; 1]; // too small
        assert_eq!(
            packetize(&sysex, &mut packets),
            Err(ProtoError::BufferTooSmall)
        );
    }

    #[test]
    fn depacketize_basic() {
        let packets = [
            [CIN_SYSEX_START, SYSEX_START, b'H', b'I'],
            [CIN_SYSEX_END_1, SYSEX_END, 0x00, 0x00],
        ];
        let mut out = [0u8; 16];
        let n = depacketize(&packets, &mut out).unwrap();
        assert_eq!(&out[..n], &[SYSEX_START, b'H', b'I', SYSEX_END]);
    }

    #[test]
    fn depacketize_malformed_cin() {
        let packets = [[0x09, 0x00, 0x00, 0x00]];
        let mut out = [0u8; 16];
        assert_eq!(
            depacketize(&packets, &mut out),
            Err(ProtoError::MalformedPacket)
        );
    }

    #[test]
    fn roundtrip() {
        let sysex = [SYSEX_START, b'A', b'T', b'+', b'V', b'E', b'R', SYSEX_END];
        let mut packets = [[0u8; 4]; 8];
        let np = packetize(&sysex, &mut packets).unwrap();

        let mut restored = [0u8; 32];
        let nb = depacketize(&packets[..np], &mut restored).unwrap();
        assert_eq!(&restored[..nb], &sysex);
    }

    #[test]
    fn roundtrip_long_message() {
        let payload = b"AT+FREQ=1#440#1000#1";
        let mut sysex = [0u8; 64];
        sysex[0] = SYSEX_START;
        sysex[1..1 + payload.len()].copy_from_slice(payload);
        sysex[1 + payload.len()] = SYSEX_END;
        let sysex_len = payload.len() + 2;

        let mut packets = [[0u8; 4]; 16];
        let np = packetize(&sysex[..sysex_len], &mut packets).unwrap();

        let mut restored = [0u8; 64];
        let nb = depacketize(&packets[..np], &mut restored).unwrap();
        assert_eq!(&restored[..nb], &sysex[..sysex_len]);
    }
}
