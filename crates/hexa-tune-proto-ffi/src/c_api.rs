// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! C ABI exported functions for FFI consumers (Flutter/Dart).
//!
//! Naming convention: `htp_` prefix (hexaTuneProto).
//!
//! ## Error convention
//! - Return `>= 0` on success (usually the number of bytes/packets written).
//! - Return `< 0` on error: `-(error_code as i32)`.
//!
//! ## Buffer convention
//! - All buffers are caller-owned.
//! - `*_ptr` + `*_len` for input buffers.
//! - `*_ptr` + `*_cap` for output buffers, with `*_out_len` for actual written length.

use hexa_tune_proto::{at, codec, sysex, usb_midi};

/// Encodes an AT command string into the output buffer.
///
/// # Safety
/// All pointers must be valid for their respective lengths.
#[no_mangle]
pub unsafe extern "C" fn htp_at_encode(
    name_ptr: *const u8,
    name_len: usize,
    id: u32,
    op: i32, // 0=Set, 1=Query, 2=Response
    params_ptr: *const HtpSlice,
    params_count: usize,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
) -> i32 {
    let name = core::slice::from_raw_parts(name_ptr, name_len);
    let op = match op {
        0 => at::AtOp::Set,
        1 => at::AtOp::Query,
        2 => at::AtOp::Response,
        _ => return -1,
    };

    let param_slices: &[HtpSlice] = if params_count > 0 {
        core::slice::from_raw_parts(params_ptr, params_count)
    } else {
        &[]
    };

    // Convert HtpSlice array to &[&[u8]] — limited to 8 params
    let mut param_refs: [&[u8]; 8] = [&[]; 8];
    let count = params_count.min(8);
    for i in 0..count {
        param_refs[i] = core::slice::from_raw_parts(param_slices[i].ptr, param_slices[i].len);
    }

    let out = core::slice::from_raw_parts_mut(out_ptr, out_cap);
    match at::encode(name, id, op, &param_refs[..count], out) {
        Ok(n) => {
            *out_len = n;
            0
        }
        Err(e) => -(e.code() as i32),
    }
}

/// Parses an AT command from a byte buffer.
///
/// On success, populates the `HtpAtParseResult` struct with offsets into the input buffer.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn htp_at_parse(
    input_ptr: *const u8,
    input_len: usize,
    result: *mut HtpAtParseResult,
) -> i32 {
    let input = core::slice::from_raw_parts(input_ptr, input_len);
    match at::parse(input) {
        Ok(msg) => {
            let r = &mut *result;
            r.id = msg.id;
            r.op = match msg.op {
                at::AtOp::Set => 0,
                at::AtOp::Query => 1,
                at::AtOp::Response => 2,
            };
            // Name offset relative to input_ptr
            r.name_offset = msg.name.as_ptr().offset_from(input_ptr) as usize;
            r.name_len = msg.name.len();
            // Collect params
            r.param_count = 0;
            for param in msg.params {
                if r.param_count >= 8 {
                    break;
                }
                let idx = r.param_count;
                r.param_offsets[idx] = param.as_ptr().offset_from(input_ptr) as usize;
                r.param_lens[idx] = param.len();
                r.param_count += 1;
            }
            0
        }
        Err(e) => -(e.code() as i32),
    }
}

/// Frames payload bytes into a SysEx message.
///
/// # Safety
/// All pointers must be valid for their respective lengths.
#[no_mangle]
pub unsafe extern "C" fn htp_sysex_frame(
    payload_ptr: *const u8,
    payload_len: usize,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
) -> i32 {
    let payload = core::slice::from_raw_parts(payload_ptr, payload_len);
    let out = core::slice::from_raw_parts_mut(out_ptr, out_cap);
    match sysex::frame(payload, out) {
        Ok(n) => {
            *out_len = n;
            0
        }
        Err(e) => -(e.code() as i32),
    }
}

/// Extracts payload from a SysEx message.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn htp_sysex_unframe(
    data_ptr: *const u8,
    data_len: usize,
    out_offset: *mut usize,
    out_len: *mut usize,
) -> i32 {
    let data = core::slice::from_raw_parts(data_ptr, data_len);
    match sysex::unframe(data) {
        Ok(payload) => {
            *out_offset = 1; // payload starts after F0
            *out_len = payload.len();
            0
        }
        Err(e) => -(e.code() as i32),
    }
}

/// Converts SysEx bytes to USB MIDI 4-byte packets.
///
/// # Safety
/// All pointers must be valid for their respective lengths.
#[no_mangle]
pub unsafe extern "C" fn htp_usb_packetize(
    sysex_ptr: *const u8,
    sysex_len: usize,
    out_ptr: *mut [u8; 4],
    out_cap: usize,
    out_count: *mut usize,
) -> i32 {
    let sysex_data = core::slice::from_raw_parts(sysex_ptr, sysex_len);
    let out = core::slice::from_raw_parts_mut(out_ptr, out_cap);
    match usb_midi::packetize(sysex_data, out) {
        Ok(n) => {
            *out_count = n;
            0
        }
        Err(e) => -(e.code() as i32),
    }
}

/// Reassembles SysEx bytes from USB MIDI 4-byte packets.
///
/// # Safety
/// All pointers must be valid for their respective lengths.
#[no_mangle]
pub unsafe extern "C" fn htp_usb_depacketize(
    packets_ptr: *const [u8; 4],
    packet_count: usize,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
) -> i32 {
    let packets = core::slice::from_raw_parts(packets_ptr, packet_count);
    let out = core::slice::from_raw_parts_mut(out_ptr, out_cap);
    match usb_midi::depacketize(packets, out) {
        Ok(n) => {
            *out_len = n;
            0
        }
        Err(e) => -(e.code() as i32),
    }
}

/// Full pipeline: encode AT command → SysEx → USB MIDI packets.
///
/// # Safety
/// All pointers must be valid for their respective lengths.
#[no_mangle]
pub unsafe extern "C" fn htp_encode_to_packets(
    name_ptr: *const u8,
    name_len: usize,
    id: u32,
    op: i32,
    params_ptr: *const HtpSlice,
    params_count: usize,
    at_buf_ptr: *mut u8,
    at_buf_cap: usize,
    sysex_buf_ptr: *mut u8,
    sysex_buf_cap: usize,
    packets_out_ptr: *mut [u8; 4],
    packets_out_cap: usize,
    packets_out_count: *mut usize,
) -> i32 {
    let name = core::slice::from_raw_parts(name_ptr, name_len);
    let op = match op {
        0 => at::AtOp::Set,
        1 => at::AtOp::Query,
        2 => at::AtOp::Response,
        _ => return -1,
    };

    let param_slices: &[HtpSlice] = if params_count > 0 {
        core::slice::from_raw_parts(params_ptr, params_count)
    } else {
        &[]
    };

    let mut param_refs: [&[u8]; 8] = [&[]; 8];
    let count = params_count.min(8);
    for i in 0..count {
        param_refs[i] = core::slice::from_raw_parts(param_slices[i].ptr, param_slices[i].len);
    }

    let at_buf = core::slice::from_raw_parts_mut(at_buf_ptr, at_buf_cap);
    let sysex_buf = core::slice::from_raw_parts_mut(sysex_buf_ptr, sysex_buf_cap);
    let packets_out = core::slice::from_raw_parts_mut(packets_out_ptr, packets_out_cap);

    match codec::encode_to_packets(
        name,
        id,
        op,
        &param_refs[..count],
        at_buf,
        sysex_buf,
        packets_out,
    ) {
        Ok(n) => {
            *packets_out_count = n;
            0
        }
        Err(e) => -(e.code() as i32),
    }
}

/// A byte slice descriptor for FFI (pointer + length).
#[repr(C)]
pub struct HtpSlice {
    /// Pointer to the data.
    pub ptr: *const u8,
    /// Length of the data.
    pub len: usize,
}

/// Parse result returned by `htp_at_parse`.
#[repr(C)]
pub struct HtpAtParseResult {
    /// Parsed command ID.
    pub id: u32,
    /// Operation type: 0=Set, 1=Query, 2=Response.
    pub op: i32,
    /// Offset of command name within the input buffer.
    pub name_offset: usize,
    /// Length of command name.
    pub name_len: usize,
    /// Number of parameters (max 8).
    pub param_count: usize,
    /// Offsets of each parameter within the input buffer.
    pub param_offsets: [usize; 8],
    /// Lengths of each parameter.
    pub param_lens: [usize; 8],
}
