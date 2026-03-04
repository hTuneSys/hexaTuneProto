// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! Golden fixture tests — validates parse/encode against JSONL test vectors.

use hexa_tune_proto::at::{self, AtOp};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct TestVector {
    at: String,
    id: u32,
    name: String,
    op: String,
    params: Vec<String>,
}

fn load_fixtures() -> Vec<TestVector> {
    let data = fs::read_to_string("tests/fixtures/commands.jsonl").unwrap();
    data.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn parse_all_fixtures() {
    for tv in load_fixtures() {
        let msg = at::parse(tv.at.as_bytes())
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {:?}", tv.at, e));

        assert_eq!(msg.id, tv.id, "ID mismatch for '{}'", tv.at);
        assert_eq!(
            msg.name,
            tv.name.as_bytes(),
            "Name mismatch for '{}'",
            tv.at
        );

        let expected_op = match tv.op.as_str() {
            "Query" => AtOp::Query,
            "Set" => AtOp::Set,
            "Response" => AtOp::Response,
            other => panic!("Unknown op '{}' in fixture", other),
        };
        assert_eq!(msg.op, expected_op, "Op mismatch for '{}'", tv.at);

        let parsed_params: Vec<&[u8]> = msg.params.collect();
        let expected_params: Vec<&[u8]> = tv.params.iter().map(|s| s.as_bytes()).collect();
        assert_eq!(
            parsed_params, expected_params,
            "Params mismatch for '{}'",
            tv.at
        );
    }
}

#[test]
fn encode_all_fixtures() {
    for tv in load_fixtures() {
        let op = match tv.op.as_str() {
            "Query" => AtOp::Query,
            "Set" => AtOp::Set,
            "Response" => AtOp::Response,
            other => panic!("Unknown op '{}'", other),
        };

        let params_bytes: Vec<Vec<u8>> = tv.params.iter().map(|s| s.as_bytes().to_vec()).collect();
        let params_refs: Vec<&[u8]> = params_bytes.iter().map(|v| v.as_slice()).collect();

        let mut buf = [0u8; 256];
        let n = at::encode(tv.name.as_bytes(), tv.id, op, &params_refs, &mut buf)
            .unwrap_or_else(|e| panic!("Failed to encode '{}': {:?}", tv.at, e));

        assert_eq!(
            &buf[..n],
            tv.at.as_bytes(),
            "Encode mismatch for '{}'",
            tv.at
        );
    }
}

#[test]
fn full_pipeline_roundtrip_all_fixtures() {
    for tv in load_fixtures() {
        let op = match tv.op.as_str() {
            "Query" => AtOp::Query,
            "Set" => AtOp::Set,
            "Response" => AtOp::Response,
            other => panic!("Unknown op '{}'", other),
        };

        let params_bytes: Vec<Vec<u8>> = tv.params.iter().map(|s| s.as_bytes().to_vec()).collect();
        let params_refs: Vec<&[u8]> = params_bytes.iter().map(|v| v.as_slice()).collect();

        // Encode AT → SysEx → USB MIDI packets
        let mut at_buf = [0u8; 256];
        let mut sysex_buf = [0u8; 512];
        let mut packets = [[0u8; 4]; 64];

        let np = hexa_tune_proto::codec::encode_to_packets(
            tv.name.as_bytes(),
            tv.id,
            op,
            &params_refs,
            &mut at_buf,
            &mut sysex_buf,
            &mut packets,
        )
        .unwrap_or_else(|e| panic!("Pipeline encode failed for '{}': {:?}", tv.at, e));

        // Decode USB MIDI packets → SysEx → AT
        let mut decode_buf = [0u8; 512];
        let msg = hexa_tune_proto::codec::decode_from_packets(&packets[..np], &mut decode_buf)
            .unwrap_or_else(|e| panic!("Pipeline decode failed for '{}': {:?}", tv.at, e));

        assert_eq!(msg.id, tv.id, "Roundtrip ID mismatch for '{}'", tv.at);
        assert_eq!(
            msg.name,
            tv.name.as_bytes(),
            "Roundtrip name mismatch for '{}'",
            tv.at
        );

        let parsed_params: Vec<&[u8]> = msg.params.collect();
        let expected_params: Vec<&[u8]> = tv.params.iter().map(|s| s.as_bytes()).collect();
        assert_eq!(
            parsed_params, expected_params,
            "Roundtrip params mismatch for '{}'",
            tv.at
        );
    }
}
