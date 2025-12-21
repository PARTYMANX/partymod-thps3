use std::fmt;

use crate::crc::crc32;

#[derive(Debug, Clone)]
pub enum BPSError {
    InputSize(usize, usize),
    OutputSize(usize, usize),
}

impl fmt::Display for BPSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BPSError::InputSize(expected, actual) => write!(f, "Bad input size: Expected: {}, Actual: {}", expected, actual),
            BPSError::OutputSize(expected, actual) => write!(f, "Bad input size: Expected: {}, Actual: {}", expected, actual),
        }
    }
}

static MAGIC_NUM: u32 = (b'B' as u32) << 24 | (b'P' as u32) << 16 | (b'S' as u32) << 8 | (b'1' as u32) << 0;

fn decode_number(buf: &[u8], offset: usize) -> (u64, usize) {
    let mut result = 0;
    let mut bit_offset = 0;
    let mut sz = 0;

    loop {
        let b = buf[offset + sz];
        sz += 1;

        result += ((b & 0x7f) as u64) << bit_offset;
        if b & 0x80 != 0 {
            break;
        }
        bit_offset += 7;
        result += 1 << bit_offset;
    }

    (result, sz)
}

pub fn bps_patch_unchecked(input: &[u8], patch: &[u8]) -> Result<Vec<u8>, BPSError> {
    let magic_num = u32::from_be_bytes(patch[0..4].try_into().unwrap());

    assert_eq!(magic_num, MAGIC_NUM);

    let mut current_offset = 4;

    let (input_expected_size, input_size_len) = decode_number(patch, current_offset);
    current_offset += input_size_len;

    if input_expected_size != input.len() as u64 {
        return Err(BPSError::InputSize(input_expected_size as usize, input.len()));
    }

    let (output_expected_size, output_size_len) = decode_number(patch, current_offset);
    current_offset += output_size_len;

    let mut output_buf = Vec::with_capacity(output_expected_size as usize);

    let (metadata_size, metadata_size_len) = decode_number(patch, current_offset);
    current_offset += metadata_size_len + metadata_size as usize;    // skip metadata

    println!("STARTING AT {}", current_offset);

    let mut input_offset_accumulator = 0;
    let mut output_offset_accumulator = 0;
    while current_offset < patch.len() - 12 {
        let (segment_head, segment_head_len) = decode_number(patch, current_offset);
        current_offset += segment_head_len;

        let segment_type = (segment_head & 0x03) as u8;
        let segment_len = (segment_head >> 2) + 1;
        match segment_type {
            0x00 => {
                let start = output_buf.len();
                let end = start + segment_len as usize;
                output_buf.extend_from_slice(&input[start..end]);

                println!("{:#04x}: {}..{} {:#010x}", segment_type, start, end - start, crc32(&output_buf));
            },
            0x01 => {
                let start = current_offset;
                let end = start + segment_len as usize;
                output_buf.extend_from_slice(&patch[start..end]);
                current_offset += segment_len as usize;

                println!("{:#04x}: {}..{} {:#010x}", segment_type, start, end - start, crc32(&output_buf));
            },
            0x02 => {
                let (offset, offset_len) = decode_number(patch, current_offset);
                current_offset += offset_len;

                let signed_offset = if offset & 0x01 != 0 {
                    -((offset >> 1) as i64)
                } else {
                    (offset >> 1) as i64
                };
                input_offset_accumulator += signed_offset;

                let start = input_offset_accumulator as usize;
                let end = start + segment_len as usize;
                output_buf.extend_from_slice(&input[start..end]);

                input_offset_accumulator += segment_len as i64;

                println!("{:#04x}: {}..{} {:#010x}", segment_type, start, end - start, crc32(&output_buf));
            },
            0x03 => {
                let (offset, offset_len) = decode_number(patch, current_offset);
                current_offset += offset_len;

                let signed_offset = if offset & 0x01 != 0 {
                    -((offset >> 1) as i64)
                } else {
                    (offset >> 1) as i64
                };
                output_offset_accumulator += signed_offset;

                let start = output_offset_accumulator as usize;
                let end = start + segment_len as usize;
                let intermediate_buffer = output_buf[start..end].to_vec();
                output_buf.extend_from_slice(&intermediate_buffer);

                output_offset_accumulator += segment_len as i64;

                println!("{:#04x}: {}..{} {:#010x}", segment_type, start, end - start, crc32(&output_buf));
            },
            _ => {
                unreachable!("Got segment type {}!", segment_type);
            }
        }
    }

    if output_buf.len() != output_expected_size as usize {
        return Err(BPSError::OutputSize(output_expected_size as usize, output_buf.len()));
    }

    Ok(output_buf)
}