// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! Streaming USB MIDI SysEx decoder.
//!
//! Incrementally processes 4-byte USB MIDI packets and reassembles complete
//! SysEx messages. Designed for event-driven embedded and FFI use cases
//! where packets arrive one at a time.
//!
//! ## States
//! - **Idle** — Waiting for `0xF0` start byte
//! - **Receiving** — Accumulating SysEx data bytes
//! - **Discarding** — Buffer overflow occurred; skipping until `0xF7`

use crate::error::ProtoError;
use crate::usb_midi::{CIN_SYSEX_END_1, CIN_SYSEX_END_2, CIN_SYSEX_END_3, CIN_SYSEX_START};

/// Decoder state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Waiting for a SysEx start byte (`0xF0`).
    Idle,
    /// Accumulating SysEx payload bytes.
    Receiving,
    /// Buffer overflowed; discarding until `0xF7`.
    Discarding,
}

/// Streaming decoder that reassembles SysEx messages from USB MIDI packets.
///
/// The caller provides a mutable buffer; the decoder writes payload bytes into it.
/// When a complete message arrives, `push_packet` returns `Some(len)` and the
/// caller can read the payload from `buf[..len]` (without `0xF0`/`0xF7` markers).
pub struct StreamDecoder<'buf> {
    buf: &'buf mut [u8],
    pos: usize,
    state: StreamState,
}

impl<'buf> StreamDecoder<'buf> {
    /// Creates a new decoder backed by the given buffer.
    pub fn new(buf: &'buf mut [u8]) -> Self {
        Self {
            buf,
            pos: 0,
            state: StreamState::Idle,
        }
    }

    /// Returns the current decoder state.
    pub fn state(&self) -> StreamState {
        self.state
    }

    /// Resets the decoder to idle state.
    pub fn reset(&mut self) {
        self.pos = 0;
        self.state = StreamState::Idle;
    }

    /// Pushes a single 4-byte USB MIDI packet into the decoder.
    ///
    /// Returns:
    /// - `Ok(Some(len))` — A complete SysEx payload is ready in `buf[..len]`
    ///   (without `0xF0`/`0xF7` markers). The decoder resets to Idle automatically.
    /// - `Ok(None)` — Packet accepted, waiting for more data.
    /// - `Err(ProtoError::MalformedPacket)` — Unrecognized CIN code.
    /// - `Err(ProtoError::Overflow)` — Buffer overflow (transitions to Discarding).
    pub fn push_packet(&mut self, packet: [u8; 4]) -> Result<Option<usize>, ProtoError> {
        let cin = packet[0] & 0x0F;

        match self.state {
            StreamState::Discarding => {
                // In discarding state, skip all bytes until we see F7
                let has_end = match cin {
                    CIN_SYSEX_END_1 | CIN_SYSEX_END_2 | CIN_SYSEX_END_3 => true,
                    CIN_SYSEX_START => false,
                    _ => return Err(ProtoError::MalformedPacket),
                };
                if has_end {
                    self.reset();
                }
                Ok(None)
            }
            StreamState::Idle => {
                match cin {
                    CIN_SYSEX_START => {
                        self.pos = 0;
                        // Extract data bytes, skip 0xF0 start marker
                        let bytes = [packet[1], packet[2], packet[3]];
                        for &b in &bytes {
                            if b != 0 && b != 0xF0 {
                                self.push_byte(b)?;
                            }
                        }
                        self.state = StreamState::Receiving;
                        Ok(None)
                    }
                    CIN_SYSEX_END_1 | CIN_SYSEX_END_2 | CIN_SYSEX_END_3 => {
                        // End packet without a start — ignore
                        Ok(None)
                    }
                    _ => Err(ProtoError::MalformedPacket),
                }
            }
            StreamState::Receiving => {
                match cin {
                    CIN_SYSEX_START => {
                        // Continue packet — add data bytes
                        let bytes = [packet[1], packet[2], packet[3]];
                        for &b in &bytes {
                            if b != 0 {
                                self.push_byte(b)?;
                            }
                        }
                        Ok(None)
                    }
                    CIN_SYSEX_END_1 => {
                        // End with 1 byte — byte1 should be 0xF7 (don't add it)
                        let len = self.pos;
                        self.state = StreamState::Idle;
                        Ok(Some(len))
                    }
                    CIN_SYSEX_END_2 => {
                        // End with 2 bytes — byte1 is data, byte2 is 0xF7
                        if packet[1] != 0 && packet[1] != 0xF7 {
                            self.push_byte(packet[1])?;
                        }
                        let len = self.pos;
                        self.state = StreamState::Idle;
                        Ok(Some(len))
                    }
                    CIN_SYSEX_END_3 => {
                        // End with 3 bytes — byte1, byte2 are data, byte3 is 0xF7
                        for &b in &[packet[1], packet[2]] {
                            if b != 0 && b != 0xF7 {
                                self.push_byte(b)?;
                            }
                        }
                        let len = self.pos;
                        self.state = StreamState::Idle;
                        Ok(Some(len))
                    }
                    _ => Err(ProtoError::MalformedPacket),
                }
            }
        }
    }

    fn push_byte(&mut self, byte: u8) -> Result<(), ProtoError> {
        if self.pos >= self.buf.len() {
            self.state = StreamState::Discarding;
            self.pos = 0;
            return Err(ProtoError::Overflow);
        }
        self.buf[self.pos] = byte;
        self.pos += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysex;
    use crate::usb_midi;
    extern crate alloc;
    use alloc::vec::Vec;

    fn make_sysex_packets(payload: &[u8]) -> (Vec<[u8; 4]>, usize) {
        let mut sysex_buf = alloc::vec![0u8; payload.len() + 2];
        let sysex_len = sysex::frame(payload, &mut sysex_buf).unwrap();
        let mut packets = alloc::vec![[0u8; 4]; sysex_len.div_ceil(3) + 1];
        let np = usb_midi::packetize(&sysex_buf[..sysex_len], &mut packets).unwrap();
        (packets, np)
    }

    #[test]
    fn stream_basic() {
        let payload = b"AT+VERSION?";
        let (packets, np) = make_sysex_packets(payload);

        let mut buf = [0u8; 64];
        let mut dec = StreamDecoder::new(&mut buf);

        for packet in packets.iter().take(np - 1) {
            assert_eq!(dec.push_packet(*packet).unwrap(), None);
        }
        let len = dec.push_packet(packets[np - 1]).unwrap().unwrap();
        assert_eq!(&buf[..len], payload);
    }

    #[test]
    fn stream_freq_command() {
        let payload = b"AT+FREQ=1#440#1000";
        let (packets, np) = make_sysex_packets(payload);

        let mut buf = [0u8; 64];
        let mut dec = StreamDecoder::new(&mut buf);

        let mut result = None;
        for packet in packets.iter().take(np) {
            if let Some(len) = dec.push_packet(*packet).unwrap() {
                result = Some(len);
            }
        }
        let len = result.unwrap();
        assert_eq!(&buf[..len], payload);
    }

    #[test]
    fn stream_reuse_after_complete() {
        let payload1 = b"AT+VERSION?";
        let payload2 = b"AT+RESET=1";

        let (p1, np1) = make_sysex_packets(payload1);
        let (p2, np2) = make_sysex_packets(payload2);

        let mut buf = [0u8; 64];

        // First message
        {
            let mut dec = StreamDecoder::new(&mut buf);
            for packet in p1.iter().take(np1 - 1) {
                dec.push_packet(*packet).unwrap();
            }
            let len = dec.push_packet(p1[np1 - 1]).unwrap().unwrap();
            assert_eq!(len, payload1.len());
        }
        assert_eq!(&buf[..payload1.len()], payload1.as_slice());

        // Second message (reusing same buffer)
        {
            let mut dec = StreamDecoder::new(&mut buf);
            for packet in p2.iter().take(np2 - 1) {
                dec.push_packet(*packet).unwrap();
            }
            let len = dec.push_packet(p2[np2 - 1]).unwrap().unwrap();
            assert_eq!(len, payload2.len());
        }
        assert_eq!(&buf[..payload2.len()], payload2.as_slice());
    }

    #[test]
    fn stream_overflow_discards() {
        let payload = b"AT+FREQ=1#440#1000";
        let (packets, np) = make_sysex_packets(payload);

        // Buffer too small
        let mut buf = [0u8; 4];
        let mut dec = StreamDecoder::new(&mut buf);

        let mut overflow_seen = false;
        for packet in packets.iter().take(np) {
            if let Err(ProtoError::Overflow) = dec.push_packet(*packet) {
                overflow_seen = true;
            }
        }
        assert!(overflow_seen);
        assert_eq!(dec.state(), StreamState::Idle);
    }

    #[test]
    fn stream_recovers_after_overflow() {
        let long_payload = b"AT+FREQ=1#440#1000";
        let short_payload = b"AT+VERSION?";

        let (p_long, np_long) = make_sysex_packets(long_payload);
        let (p_short, np_short) = make_sysex_packets(short_payload);

        // Buffer too small for first message, fine for second
        let mut buf = [0u8; 12];
        let mut dec = StreamDecoder::new(&mut buf);

        // First message overflows
        for packet in p_long.iter().take(np_long) {
            let _ = dec.push_packet(*packet);
        }
        assert_eq!(dec.state(), StreamState::Idle);

        // Second message succeeds
        let mut result = None;
        for packet in p_short.iter().take(np_short) {
            if let Some(len) = dec.push_packet(*packet).unwrap() {
                result = Some(len);
            }
        }
        let len = result.unwrap();
        assert_eq!(&buf[..len], short_payload);
    }

    #[test]
    fn stream_idle_state_initially() {
        let mut buf = [0u8; 32];
        let dec = StreamDecoder::new(&mut buf);
        assert_eq!(dec.state(), StreamState::Idle);
    }
}
