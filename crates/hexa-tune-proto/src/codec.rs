// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! Full encode/decode pipeline helpers.
//!
//! Combines AT, SysEx, and USB MIDI layers into convenient pipeline functions.

use crate::at::{self, AtMessage, AtOp};
use crate::error::ProtoError;
use crate::sysex;
use crate::usb_midi;

/// Encodes an AT command through the full pipeline: AT → SysEx → USB MIDI packets.
///
/// `at_buf` is a scratch buffer for the intermediate AT string.
/// `sysex_buf` is a scratch buffer for the intermediate SysEx frame.
/// `packets_out` receives the resulting USB MIDI packets.
///
/// Returns the number of USB MIDI packets written.
pub fn encode_to_packets(
    name: &[u8],
    id: u32,
    op: AtOp,
    params: &[&[u8]],
    at_buf: &mut [u8],
    sysex_buf: &mut [u8],
    packets_out: &mut [[u8; 4]],
) -> Result<usize, ProtoError> {
    let at_len = at::encode(name, id, op, params, at_buf)?;
    let sysex_len = sysex::frame(&at_buf[..at_len], sysex_buf)?;
    usb_midi::packetize(&sysex_buf[..sysex_len], packets_out)
}

/// Decodes USB MIDI packets through the full pipeline: USB MIDI → SysEx → AT message.
///
/// `sysex_buf` is a scratch buffer for the reassembled SysEx data.
///
/// Returns the parsed `AtMessage` with zero-copy references into `sysex_buf`.
pub fn decode_from_packets<'a>(
    packets: &[[u8; 4]],
    sysex_buf: &'a mut [u8],
) -> Result<AtMessage<'a>, ProtoError> {
    let sysex_len = usb_midi::depacketize(packets, sysex_buf)?;
    let payload = sysex::unframe(&sysex_buf[..sysex_len])?;

    // payload is a sub-slice of sysex_buf, so the lifetime is tied to sysex_buf
    at::parse(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn full_pipeline_query() {
        let mut at_buf = [0u8; 64];
        let mut sysex_buf = [0u8; 128];
        let mut packets = [[0u8; 4]; 32];

        let np = encode_to_packets(
            b"VERSION",
            0,
            AtOp::Query,
            &[],
            &mut at_buf,
            &mut sysex_buf,
            &mut packets,
        )
        .unwrap();

        let mut decode_buf = [0u8; 128];
        let msg = decode_from_packets(&packets[..np], &mut decode_buf).unwrap();
        assert_eq!(msg.name, b"VERSION");
        assert_eq!(msg.op, AtOp::Query);
        assert_eq!(msg.id, 0);
    }

    #[test]
    fn full_pipeline_set_with_params() {
        let mut at_buf = [0u8; 64];
        let mut sysex_buf = [0u8; 128];
        let mut packets = [[0u8; 4]; 32];

        let np = encode_to_packets(
            b"FREQ",
            1,
            AtOp::Set,
            &[b"440", b"1000", b"1"],
            &mut at_buf,
            &mut sysex_buf,
            &mut packets,
        )
        .unwrap();

        let mut decode_buf = [0u8; 128];
        let msg = decode_from_packets(&packets[..np], &mut decode_buf).unwrap();
        assert_eq!(msg.name, b"FREQ");
        assert_eq!(msg.op, AtOp::Set);
        assert_eq!(msg.id, 1);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"440" as &[u8], b"1000", b"1"]);
    }

    #[test]
    fn full_pipeline_error_response() {
        let mut at_buf = [0u8; 64];
        let mut sysex_buf = [0u8; 128];
        let mut packets = [[0u8; 4]; 32];

        let np = encode_to_packets(
            b"ERROR",
            42,
            AtOp::Response,
            &[b"5"],
            &mut at_buf,
            &mut sysex_buf,
            &mut packets,
        )
        .unwrap();

        let mut decode_buf = [0u8; 128];
        let msg = decode_from_packets(&packets[..np], &mut decode_buf).unwrap();
        assert_eq!(msg.name, b"ERROR");
        assert_eq!(msg.id, 42);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"5" as &[u8]]);
    }

    #[test]
    fn full_pipeline_setrgb() {
        let mut at_buf = [0u8; 64];
        let mut sysex_buf = [0u8; 128];
        let mut packets = [[0u8; 4]; 32];

        let np = encode_to_packets(
            b"SETRGB",
            5,
            AtOp::Set,
            &[b"255", b"128", b"0"],
            &mut at_buf,
            &mut sysex_buf,
            &mut packets,
        )
        .unwrap();

        let mut decode_buf = [0u8; 128];
        let msg = decode_from_packets(&packets[..np], &mut decode_buf).unwrap();
        assert_eq!(msg.name, b"SETRGB");
        assert_eq!(msg.id, 5);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"255" as &[u8], b"128", b"0"]);
    }

    #[test]
    fn full_pipeline_operation_completed() {
        let mut at_buf = [0u8; 64];
        let mut sysex_buf = [0u8; 128];
        let mut packets = [[0u8; 4]; 32];

        let np = encode_to_packets(
            b"OPERATION",
            3,
            AtOp::Response,
            &[b"5", b"PREPARE", b"COMPLETED"],
            &mut at_buf,
            &mut sysex_buf,
            &mut packets,
        )
        .unwrap();

        let mut decode_buf = [0u8; 128];
        let msg = decode_from_packets(&packets[..np], &mut decode_buf).unwrap();
        assert_eq!(msg.name, b"OPERATION");
        assert_eq!(msg.id, 3);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"5" as &[u8], b"PREPARE", b"COMPLETED"]);
    }

    #[test]
    fn full_pipeline_stop_immediately() {
        let mut at_buf = [0u8; 64];
        let mut sysex_buf = [0u8; 128];
        let mut packets = [[0u8; 4]; 32];

        let np = encode_to_packets(
            b"OPERATION",
            4,
            AtOp::Set,
            &[b"STOP", b"IMMEDIATELY"],
            &mut at_buf,
            &mut sysex_buf,
            &mut packets,
        )
        .unwrap();

        let mut decode_buf = [0u8; 128];
        let msg = decode_from_packets(&packets[..np], &mut decode_buf).unwrap();
        assert_eq!(msg.name, b"OPERATION");
        assert_eq!(msg.id, 4);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"STOP" as &[u8], b"IMMEDIATELY"]);
    }
}
