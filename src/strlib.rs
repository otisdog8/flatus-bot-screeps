use wasm_bindgen::prelude::*;
use web_sys::console;
use js_sys::JsString;
use std::convert::TryInto;

// CREDIT TO crazy10101#9079 (140580006542835713) on discord - khoover on GitHub
// https://github.com/khoover/screeps-starter-rust/tree/recast-poc

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (console::log_1(&format_args!($($t)*).to_string().into()))
}

#[wasm_bindgen(inline_js = "const fromCharCode = String.fromCharCode; export function utf16_to_jsstring(utf16) { let ret = ''; let i = 0; for (; i < utf16.length - 32; i += 32) { ret += fromCharCode(
    utf16[i+0], utf16[i+1], utf16[i+2], utf16[i+3], utf16[i+4], utf16[i+5], utf16[i+6], utf16[i+7], utf16[i+8], utf16[i+9],
    utf16[i+10], utf16[i+11], utf16[i+12], utf16[i+13], utf16[i+14], utf16[i+15], utf16[i+16], utf16[i+17], utf16[i+18], utf16[i+19],
    utf16[i+20], utf16[i+21], utf16[i+22], utf16[i+23], utf16[i+24], utf16[i+25], utf16[i+26], utf16[i+27], utf16[i+28], utf16[i+29],
    utf16[i+30], utf16[i+31]
); } if (i != utf16.length) ret += fromCharCode.apply(String, utf16.subarray(i, i+32)); return ret; }

export function jsstring_to_utf16(s, buf) {
    if (buf.length < s.length) throw new Error('Insufficient buffer space to store string.');
    for (let i = 0; i < buf.length; ++i) {
        buf[i] = s.charCodeAt(i);
    }
}")]
extern "C" {
    fn utf16_to_jsstring(utf16: &[u16]) -> JsString;

    #[wasm_bindgen(catch)]
    fn jsstring_to_utf16(s: &JsString, utf16: &mut [u16]) -> Result<(), JsValue>;
}

const LOW_FIFTEEN_BITS: u64 = 0x7FFF;
const LOW_FOUR_BITS: u64 = 0xF;
const INPUT_BLOCK_SIZE: usize = 120;
const OUTPUT_BLOCK_SIZE: usize = 64;

#[inline(always)]
fn to_u64(bytes: &[u8]) -> u64 {
    u64::from_ne_bytes(bytes[..8].try_into().unwrap())
}

#[inline(always)]
fn write_u64(num: u64, buf: &mut [u8]) {
    buf[..8].copy_from_slice(&u64::to_ne_bytes(num));
}

#[inline(always)]
fn write_byte_chunk(bytes: &[u8], extras: &mut u64, output_chunk: &mut [u16]) {
    let byte_u64 = to_u64(bytes);
    *extras = (*extras | (byte_u64 & LOW_FOUR_BITS)) << 4;

    output_chunk[0] = ((byte_u64 >> 4) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[1] = ((byte_u64 >> 19) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[2] = ((byte_u64 >> 34) & LOW_FIFTEEN_BITS) as u16;
    output_chunk[3] = ((byte_u64 >> 49) & LOW_FIFTEEN_BITS) as u16;
}

// borrowed parts from the base64 encoder
fn bytestring_to_utf15(bytes: &[u8]) -> Vec<u16> {
    if bytes.is_empty() { return Vec::new(); }

    // each block is 15 u64s, so 15*8=120 u8s
    let (encoded_len, has_remainder): (usize, bool) = {
        let len = bytes.len();
        let remainder = len % INPUT_BLOCK_SIZE;
        let num_blocks = len / INPUT_BLOCK_SIZE + (if remainder != 0 { 1 } else { 0 });
        (OUTPUT_BLOCK_SIZE*num_blocks + 1, remainder != 0)
    };
    let mut output: Vec<u16> = vec![0; encoded_len];
    if has_remainder { output[0] = 0x0031; } else { output[0] = 0x0030; }
    let mut output_index: usize = 1;
    let mut input_index: usize = 0;
    let last_fast_index = bytes.len().saturating_sub(120);
    
    if last_fast_index > 0 {
        while input_index <= last_fast_index {
            let bytes_chunk = &bytes[input_index..input_index + INPUT_BLOCK_SIZE];
            let output_chunk = &mut output[output_index..output_index + OUTPUT_BLOCK_SIZE];
            let mut extras: u64 = 0;

            for (bytes_index, output_chunk_index) in (0..INPUT_BLOCK_SIZE).step_by(8).zip((0..OUTPUT_BLOCK_SIZE-4).step_by(4)) {
                write_byte_chunk(&bytes_chunk[bytes_index..], &mut extras, &mut output_chunk[output_chunk_index..]);
            }
            output_chunk[60] = ((extras >> 4) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[61] = ((extras >> 19) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[62] = ((extras >> 34) & LOW_FIFTEEN_BITS) as u16;
            output_chunk[63] = ((extras >> 49) & LOW_FIFTEEN_BITS) as u16;

            input_index += INPUT_BLOCK_SIZE;
            output_index += OUTPUT_BLOCK_SIZE;
        }
    }

    if has_remainder {
        let mut bytes_chunk: [u8; INPUT_BLOCK_SIZE] = [0; INPUT_BLOCK_SIZE];
        let len = bytes.len();
        bytes_chunk[0] = (len - input_index) as u8;
        bytes_chunk[1..1 + len - input_index].copy_from_slice(&bytes[input_index..]);
        let bytes_chunk = bytes_chunk;
        let output_chunk = &mut output[output_index..output_index + OUTPUT_BLOCK_SIZE];
        let mut extras: u64 = 0;

        for (bytes_index, output_chunk_index) in (0..INPUT_BLOCK_SIZE).step_by(8).zip((0..OUTPUT_BLOCK_SIZE-4).step_by(4)) {
            write_byte_chunk(&bytes_chunk[bytes_index..], &mut extras, &mut output_chunk[output_chunk_index..]);
        }
        output_chunk[60] = ((extras >> 4) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[61] = ((extras >> 19) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[62] = ((extras >> 34) & LOW_FIFTEEN_BITS) as u16;
        output_chunk[63] = ((extras >> 49) & LOW_FIFTEEN_BITS) as u16;
    }

    output
}

#[wasm_bindgen]
pub fn bytestring_to_jsstring(bytes: &[u8]) -> JsString {
    let utf15 = bytestring_to_utf15(bytes);
    utf16_to_jsstring(utf15.as_slice())
}

#[inline(always)]
fn decode_block(input: &[u16], remainder: u64, output: &mut [u8]) {
    let mut accum = remainder;
    
    accum |= (input[0] as u64) << 4;
    accum |= (input[1] as u64) << 19;
    accum |= (input[2] as u64) << 34;
    accum |= (input[3] as u64) << 49;

    write_u64(accum, output);
}

#[inline(always)]
fn decode_extra(input: &[u16]) -> u64 {
    let mut accum: u64 = input[0] as u64;
    accum |= (input[1] as u64) << 15;
    accum |= (input[2] as u64) << 30;
    accum |= (input[3] as u64) << 45;
    accum
}

#[wasm_bindgen]
pub fn jsstring_to_bytestring(utf15: &JsString) -> Vec<u8> {
    if utf15.length() == 0 { return Vec::new(); }
    assert_eq!(utf15.length() % OUTPUT_BLOCK_SIZE as u32, 1);

    let mut utf15_vec: Vec<u16> = vec![0; utf15.length().try_into().unwrap()];
    jsstring_to_utf16(&utf15, utf15_vec.as_mut_slice()).unwrap();
    let first = utf15_vec[0];
    assert!(first == 0x0030 || first == 0x0031);
    let utf15_vec = &utf15_vec[1..];
    assert_eq!(utf15_vec.iter().max().unwrap() & 0x8000, 0);

    let has_remainder = first == 0x0031;
    let (fast_blocks, remainder_size): (usize, usize) = if has_remainder {
        let mut buf: [u8; 8] = [0; 8];
        let remainder_start = utf15_vec.len() - OUTPUT_BLOCK_SIZE;
        let extra = (utf15_vec[remainder_start + 63] >> 11) as u64 & LOW_FOUR_BITS;
        decode_block(&utf15_vec[remainder_start..], extra, &mut buf);
        (utf15_vec.len() / OUTPUT_BLOCK_SIZE - 1, buf[0] as usize)
    } else {
        (utf15_vec.len() / OUTPUT_BLOCK_SIZE, 0)
    };
    let mut output: Vec<u8> = vec![0; INPUT_BLOCK_SIZE * fast_blocks + remainder_size];
    let mut utf15_index: usize = 0;
    let mut output_index: usize = 0;
    for _ in 0 .. fast_blocks {
        let utf15_chunk = &utf15_vec[utf15_index..utf15_index + OUTPUT_BLOCK_SIZE];
        let output_chunk = &mut output[output_index..output_index + INPUT_BLOCK_SIZE];
        let extra = decode_extra(&utf15_chunk[60..]);

        decode_block(utf15_chunk, (extra >> 56) & LOW_FOUR_BITS, output_chunk);

        decode_block(&utf15_chunk[4..], (extra >> 52) & LOW_FOUR_BITS, &mut output_chunk[8..]);

        decode_block(&utf15_chunk[8..], (extra >> 48) & LOW_FOUR_BITS, &mut output_chunk[16..]);

        decode_block(&utf15_chunk[12..], (extra >> 44) & LOW_FOUR_BITS, &mut output_chunk[24..]);

        decode_block(&utf15_chunk[16..], (extra >> 40) & LOW_FOUR_BITS, &mut output_chunk[32..]);

        decode_block(&utf15_chunk[20..], (extra >> 36) & LOW_FOUR_BITS, &mut output_chunk[40..]);

        decode_block(&utf15_chunk[24..], (extra >> 32) & LOW_FOUR_BITS, &mut output_chunk[48..]);

        decode_block(&utf15_chunk[28..], (extra >> 28) & LOW_FOUR_BITS, &mut output_chunk[56..]);

        decode_block(&utf15_chunk[32..], (extra >> 24) & LOW_FOUR_BITS, &mut output_chunk[64..]);

        decode_block(&utf15_chunk[36..], (extra >> 20) & LOW_FOUR_BITS, &mut output_chunk[72..]);

        decode_block(&utf15_chunk[40..], (extra >> 16) & LOW_FOUR_BITS, &mut output_chunk[80..]);

        decode_block(&utf15_chunk[44..], (extra >> 12) & LOW_FOUR_BITS, &mut output_chunk[88..]);

        decode_block(&utf15_chunk[48..], (extra >> 8) & LOW_FOUR_BITS, &mut output_chunk[96..]);

        decode_block(&utf15_chunk[52..], (extra >> 4) & LOW_FOUR_BITS, &mut output_chunk[104..]);

        decode_block(&utf15_chunk[56..], extra & LOW_FOUR_BITS, &mut output_chunk[112..]);

        output_index += INPUT_BLOCK_SIZE;
        utf15_index += OUTPUT_BLOCK_SIZE;
    }
    if has_remainder {
        let utf15_chunk = &utf15_vec[utf15_index..];
        let output_chunk = &mut output[output_index..];
        let mut tmp: [u8; 8] = [0; 8];
        let extra = decode_extra(&utf15_chunk[60..]);

        decode_block(utf15_chunk, (extra >> 56) & LOW_FOUR_BITS, &mut tmp);
        output_chunk[..7.min(remainder_size)].copy_from_slice(&tmp[1..8.min(remainder_size+1)]);
        if remainder_size <= 7 { return output; }

        let output_chunk = &mut output_chunk[7..];
        let utf15_chunk = &utf15_chunk[4..];
        let semi_fast_blocks = remainder_size.saturating_sub(7) / 8;
        let annoying_bytes: usize = remainder_size.saturating_sub(7) % 8;
        if semi_fast_blocks > 0 {
            let mut extra_shift: u8 = 52;
            let mut inner_utf_index: usize = 0;
            let mut inner_output_index: usize = 0;

            for _ in 0 .. semi_fast_blocks {
                decode_block(&utf15_chunk[inner_utf_index..], (extra >> extra_shift) & LOW_FOUR_BITS, &mut output_chunk[inner_output_index..]);
                extra_shift -= 4;
                inner_utf_index += 4;
                inner_output_index += 8;
            }

            decode_block(&utf15_chunk[inner_utf_index..], (extra >> extra_shift) & LOW_FOUR_BITS, &mut tmp);
            output_chunk[inner_output_index..inner_output_index + annoying_bytes].copy_from_slice(&tmp[..annoying_bytes]);
        } else if annoying_bytes > 0 {
            decode_block(utf15_chunk, (extra >> 52) & LOW_FOUR_BITS, &mut tmp);
            output_chunk[..annoying_bytes].copy_from_slice(&tmp[..annoying_bytes]);
        }
    }

    output
}