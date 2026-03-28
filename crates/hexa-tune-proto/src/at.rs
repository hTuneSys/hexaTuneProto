// SPDX-FileCopyrightText: 2026 hexaTune LLC
// SPDX-License-Identifier: MIT

//! AT command parse and encode.
//!
//! ## Wire Format
//!
//! | Format | Description |
//! |---|---|
//! | `AT+NAME?` | Query |
//! | `AT+NAME=id` | Set without params |
//! | `AT+NAME=id#P1#P2...` | Set with params |
//! | `AT+NAME=id#P1#...` | Response (same format as set) |

use crate::error::ProtoError;

/// AT command operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtOp {
    /// Set command (`AT+NAME=id#P1#P2...`)
    Set,
    /// Query command (`AT+NAME?`)
    Query,
    /// Response from device (same wire format as Set)
    Response,
}

/// Zero-copy parameter iterator over `#`-separated values.
#[derive(Debug, Clone)]
pub struct Params<'a> {
    remainder: Option<&'a [u8]>,
}

impl<'a> Params<'a> {
    /// Creates a new parameter iterator from a byte slice.
    /// The slice should contain `#`-separated parameter values.
    pub fn new(data: &'a [u8]) -> Self {
        if data.is_empty() {
            Self { remainder: None }
        } else {
            Self {
                remainder: Some(data),
            }
        }
    }

    /// Creates an empty parameter iterator.
    pub fn empty() -> Self {
        Self { remainder: None }
    }

    /// Returns `true` if there are no parameters.
    pub fn is_empty(&self) -> bool {
        self.remainder.is_none()
    }
}

impl<'a> Iterator for Params<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.remainder?;
        if let Some(pos) = data.iter().position(|&b| b == b'#') {
            let param = &data[..pos];
            self.remainder = if pos + 1 < data.len() {
                Some(&data[pos + 1..])
            } else {
                None
            };
            Some(param)
        } else {
            self.remainder = None;
            Some(data)
        }
    }
}

/// Parsed AT message with zero-copy references into the input buffer.
#[derive(Debug, Clone)]
pub struct AtMessage<'a> {
    /// Command tracking ID (parsed from decimal string, defaults to 0).
    pub id: u32,
    /// Command name (e.g., `VERSION`, `FREQ`).
    pub name: &'a [u8],
    /// Operation type.
    pub op: AtOp,
    /// Parameter iterator (zero-copy, `#`-separated).
    pub params: Params<'a>,
}

/// Parses an AT command string from a byte slice.
///
/// Expects input in one of these formats:
/// - `AT+NAME?` → Query
/// - `AT+NAME=id` → Set without params
/// - `AT+NAME=id#P1#P2` → Set with params
///
/// The `op` field is set to [`AtOp::Set`] for non-query commands.
/// Use [`AtOp::Response`] in adapter layers when the message direction is known.
pub fn parse(input: &[u8]) -> Result<AtMessage<'_>, ProtoError> {
    let input = trim_bytes(input);
    if input.is_empty() {
        return Err(ProtoError::EmptyInput);
    }
    if input.len() < 4 || &input[..3] != b"AT+" {
        return Err(ProtoError::InvalidCommand);
    }
    let cmd = &input[3..];

    if cmd.is_empty() {
        return Err(ProtoError::EmptyName);
    }

    // Query: AT+NAME?
    if cmd.last() == Some(&b'?') {
        let name = &cmd[..cmd.len() - 1];
        if name.is_empty() {
            return Err(ProtoError::EmptyName);
        }
        return Ok(AtMessage {
            id: 0,
            name,
            op: AtOp::Query,
            params: Params::empty(),
        });
    }

    // Set: AT+NAME=id or AT+NAME=id#P1#P2
    if let Some(eq_pos) = cmd.iter().position(|&b| b == b'=') {
        let name = &cmd[..eq_pos];
        if name.is_empty() {
            return Err(ProtoError::EmptyName);
        }
        let after_eq = &cmd[eq_pos + 1..];
        if after_eq.is_empty() {
            return Ok(AtMessage {
                id: 0,
                name,
                op: AtOp::Set,
                params: Params::empty(),
            });
        }

        // Split id from params at first '#'
        let (id_bytes, param_bytes) =
            if let Some(hash_pos) = after_eq.iter().position(|&b| b == b'#') {
                (&after_eq[..hash_pos], &after_eq[hash_pos + 1..])
            } else {
                (after_eq, &[] as &[u8])
            };

        let id = parse_u32(id_bytes)?;
        let params = if param_bytes.is_empty() {
            Params::empty()
        } else {
            Params::new(param_bytes)
        };

        return Ok(AtMessage {
            id,
            name,
            op: AtOp::Set,
            params,
        });
    }

    // Bare command: AT+NAME (no = or ?)
    Ok(AtMessage {
        id: 0,
        name: cmd,
        op: AtOp::Set,
        params: Params::empty(),
    })
}

/// Encodes an AT message into the output buffer.
///
/// Returns the number of bytes written.
///
/// # Formats
/// - Query: `AT+NAME?`
/// - Set without params: `AT+NAME=id`
/// - Set with params: `AT+NAME=id#P1#P2...`
pub fn encode(
    name: &[u8],
    id: u32,
    op: AtOp,
    params: &[&[u8]],
    out: &mut [u8],
) -> Result<usize, ProtoError> {
    let mut pos = 0;

    // "AT+"
    write_bytes(out, &mut pos, b"AT+")?;
    // NAME
    write_bytes(out, &mut pos, name)?;

    match op {
        AtOp::Query => {
            write_byte(out, &mut pos, b'?')?;
        }
        AtOp::Set | AtOp::Response => {
            write_byte(out, &mut pos, b'=')?;
            // id
            let id_len = write_u32(out, &mut pos, id)?;
            if id_len == 0 {
                return Err(ProtoError::BufferTooSmall);
            }
            // params
            for p in params {
                write_byte(out, &mut pos, b'#')?;
                write_bytes(out, &mut pos, p)?;
            }
        }
    }

    Ok(pos)
}

// -- Helper functions --

fn parse_u32(bytes: &[u8]) -> Result<u32, ProtoError> {
    if bytes.is_empty() {
        return Ok(0);
    }
    let mut result: u32 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return Err(ProtoError::InvalidId);
        }
        result = result
            .checked_mul(10)
            .and_then(|r| r.checked_add((b - b'0') as u32))
            .ok_or(ProtoError::InvalidId)?;
    }
    Ok(result)
}

fn write_byte(out: &mut [u8], pos: &mut usize, byte: u8) -> Result<(), ProtoError> {
    if *pos >= out.len() {
        return Err(ProtoError::BufferTooSmall);
    }
    out[*pos] = byte;
    *pos += 1;
    Ok(())
}

fn write_bytes(out: &mut [u8], pos: &mut usize, bytes: &[u8]) -> Result<(), ProtoError> {
    if *pos + bytes.len() > out.len() {
        return Err(ProtoError::BufferTooSmall);
    }
    out[*pos..*pos + bytes.len()].copy_from_slice(bytes);
    *pos += bytes.len();
    Ok(())
}

fn write_u32(out: &mut [u8], pos: &mut usize, value: u32) -> Result<usize, ProtoError> {
    // Max u32 is 4294967295 (10 digits)
    let mut buf = [0u8; 10];
    let mut n = value;
    let mut i = buf.len();

    if n == 0 {
        write_byte(out, pos, b'0')?;
        return Ok(1);
    }

    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }

    let digits = &buf[i..];
    write_bytes(out, pos, digits)?;
    Ok(digits.len())
}

fn trim_bytes(input: &[u8]) -> &[u8] {
    let start = input.iter().position(|b| !b.is_ascii_whitespace());
    let end = input.iter().rposition(|b| !b.is_ascii_whitespace());
    match (start, end) {
        (Some(s), Some(e)) => &input[s..=e],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn parse_query() {
        let msg = parse(b"AT+VERSION?").unwrap();
        assert_eq!(msg.name, b"VERSION");
        assert_eq!(msg.op, AtOp::Query);
        assert_eq!(msg.id, 0);
        assert!(msg.params.is_empty());
    }

    #[test]
    fn parse_set_no_params() {
        let msg = parse(b"AT+RESET=1").unwrap();
        assert_eq!(msg.name, b"RESET");
        assert_eq!(msg.op, AtOp::Set);
        assert_eq!(msg.id, 1);
        assert!(msg.params.is_empty());
    }

    #[test]
    fn parse_set_with_params() {
        let msg = parse(b"AT+FREQ=1#440#1000#1").unwrap();
        assert_eq!(msg.name, b"FREQ");
        assert_eq!(msg.op, AtOp::Set);
        assert_eq!(msg.id, 1);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"440" as &[u8], b"1000", b"1"]);
    }

    #[test]
    fn parse_set_rgb() {
        let msg = parse(b"AT+SETRGB=5#255#128#0").unwrap();
        assert_eq!(msg.name, b"SETRGB");
        assert_eq!(msg.id, 5);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"255" as &[u8], b"128", b"0"]);
    }

    #[test]
    fn parse_trims_whitespace() {
        let msg = parse(b"  AT+VERSION?  \n").unwrap();
        assert_eq!(msg.name, b"VERSION");
        assert_eq!(msg.op, AtOp::Query);
    }

    #[test]
    fn parse_error_response() {
        let msg = parse(b"AT+ERROR=1#5").unwrap();
        assert_eq!(msg.name, b"ERROR");
        assert_eq!(msg.id, 1);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"5" as &[u8]]);
    }

    #[test]
    fn parse_done_response() {
        let msg = parse(b"AT+DONE=42").unwrap();
        assert_eq!(msg.name, b"DONE");
        assert_eq!(msg.id, 42);
        assert!(msg.params.is_empty());
    }

    #[test]
    fn parse_completed_response() {
        let msg = parse(b"AT+OPERATION=3#5#PREPARE#COMPLETED").unwrap();
        assert_eq!(msg.name, b"OPERATION");
        assert_eq!(msg.id, 3);
        let params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(params, vec![b"5" as &[u8], b"PREPARE", b"COMPLETED"]);
    }

    #[test]
    fn parse_invalid_prefix() {
        assert!(parse(b"HELLO").is_err());
    }

    #[test]
    fn parse_invalid_id() {
        assert!(parse(b"AT+FREQ=abc#440").is_err());
    }

    #[test]
    fn parse_empty_input() {
        assert!(parse(b"").is_err());
        assert!(parse(b"   ").is_err());
    }

    #[test]
    fn encode_query() {
        let mut buf = [0u8; 64];
        let n = encode(b"VERSION", 0, AtOp::Query, &[], &mut buf).unwrap();
        assert_eq!(&buf[..n], b"AT+VERSION?");
    }

    #[test]
    fn encode_set_no_params() {
        let mut buf = [0u8; 64];
        let n = encode(b"RESET", 1, AtOp::Set, &[], &mut buf).unwrap();
        assert_eq!(&buf[..n], b"AT+RESET=1");
    }

    #[test]
    fn encode_set_with_params() {
        let mut buf = [0u8; 64];
        let n = encode(b"FREQ", 1, AtOp::Set, &[b"440", b"1000", b"1"], &mut buf).unwrap();
        assert_eq!(&buf[..n], b"AT+FREQ=1#440#1000#1");
    }

    #[test]
    fn encode_buffer_too_small() {
        let mut buf = [0u8; 4];
        let result = encode(b"VERSION", 0, AtOp::Query, &[], &mut buf);
        assert_eq!(result, Err(ProtoError::BufferTooSmall));
    }

    #[test]
    fn roundtrip_query() {
        let mut buf = [0u8; 64];
        let n = encode(b"VERSION", 0, AtOp::Query, &[], &mut buf).unwrap();
        let msg = parse(&buf[..n]).unwrap();
        assert_eq!(msg.name, b"VERSION");
        assert_eq!(msg.op, AtOp::Query);
    }

    #[test]
    fn roundtrip_set_with_params() {
        let mut buf = [0u8; 64];
        let params: &[&[u8]] = &[b"440", b"1000", b"1"];
        let n = encode(b"FREQ", 1, AtOp::Set, params, &mut buf).unwrap();
        let msg = parse(&buf[..n]).unwrap();
        assert_eq!(msg.name, b"FREQ");
        assert_eq!(msg.id, 1);
        let parsed_params: Vec<&[u8]> = msg.params.collect();
        assert_eq!(parsed_params, params);
    }

    #[test]
    fn params_iterator_empty() {
        let p = Params::empty();
        assert!(p.is_empty());
        assert_eq!(p.count(), 0);
    }

    #[test]
    fn params_iterator_single() {
        let p = Params::new(b"hello");
        let items: Vec<&[u8]> = p.collect();
        assert_eq!(items, vec![b"hello" as &[u8]]);
    }

    #[test]
    fn params_iterator_multiple() {
        let p = Params::new(b"a#b#c");
        let items: Vec<&[u8]> = p.collect();
        assert_eq!(items, vec![b"a" as &[u8], b"b", b"c"]);
    }
}
