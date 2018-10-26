use ::Point;
use DeQRError;
use DeQRResult;
use version_db::{RSParameters, VersionDataBase};
use galois::{GF16, GF256, GaloisField};

const MAX_PAYLOAD_SIZE: usize = 8896;

#[derive(Clone)]
pub struct Code {
    pub corners: [Point; 4],
    pub size: usize,
    pub cell_bitmap: [u8; 3917],
}

pub enum DataType {
    
}

/* This structure holds the decoded QR-code data */
pub struct Data {
    pub version: usize,
    pub ecc_level: u16,
    pub mask: u16,
    pub data_type: usize,
    pub payload: [u8; MAX_PAYLOAD_SIZE],
    pub payload_len: usize,
    pub eci: u32,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            version: 0,
            ecc_level: 0,
            mask: 0,
            data_type: 0,
            payload: [0; MAX_PAYLOAD_SIZE],
            payload_len: 0,
            eci: 0,
        }
    }
}
/* ***********************************************************************
 * Decoder algorithm
 */
#[derive(Copy, Clone)]
pub struct DataStream {
    pub raw: [u8; MAX_PAYLOAD_SIZE],
    pub data_bits: usize,
    pub ptr: usize,
    pub data: [u8; MAX_PAYLOAD_SIZE],
}

/* quirc -- QR-code recognition library
 * Copyright (C) 2010-2012 Daniel Beer <dlbeer@gmail.com>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */


impl Code {
    pub fn decode(&self) -> DeQRResult<Data> {
        let mut ds = DataStream {
            raw: [0; MAX_PAYLOAD_SIZE],
            data_bits: 0,
            ptr: 0,
            data: [0; MAX_PAYLOAD_SIZE],
        };
        if (self.size - 17) % 4 != 0 {
            return Err(DeQRError::INVALID_GRID_SIZE);
        }
        let mut data: Data = Default::default();
        data.version = (self.size - 17) / 4;
        if data.version == 0 || data.version > 40 {
            return Err(DeQRError::INVALID_VERSION);
        }

        read_format(self, &mut data)?;
        read_data(self, &mut data, &mut ds);
        codestream_ecc(&mut data, &mut ds)?;
        decode_payload(&mut data, &mut ds)?;
        Ok(data)
    }
}

fn decode_payload(
    data: &mut Data,
    ds: &mut DataStream,
) -> DeQRResult<()> {
    while bits_remaining(ds) >= 4 {
        let mut ty = take_bits(ds, 4);
        match ty {
            1 => decode_numeric(data, ds),
            2 => decode_alpha(data, ds),
            4 => decode_byte(data, ds),
            8 => decode_kanji(data, ds),
            7 => decode_eci(data, ds),
            _ => {
                break;
            }
        }?;

        if (0 != (ty & ty - 1)) || (ty <= data.data_type) {
            continue;
        }
        data.data_type = ty
    }

    /* Add nul terminator to all payloads */
    if data.payload_len >= MAX_PAYLOAD_SIZE {
        data.payload_len -= 1;
    }
    let end = ::std::cmp::min(data.payload_len - 1, MAX_PAYLOAD_SIZE - 1);
    data.payload[end] = 0;
    Ok(())
}

fn take_bits(ds: &mut DataStream, len: usize) -> usize {
    let mut ret = 0;
    let max_len = ::std::cmp::min(ds.data_bits - ds.ptr, len);
    for _ in 0..max_len {
        let b = ds.data[ds.ptr >> 3];
        let bitpos = ds.ptr & 7;
        ret <<= 1;
        if 0 != (b << bitpos) & 0x80 {
            ret |= 1
        }
        ds.ptr += 1;
    }
    ret
}

fn decode_eci(
    data: &mut Data,
    ds: &mut DataStream,
) -> DeQRResult<()> {
    if bits_remaining(ds) < 8 {
        Err(DeQRError::DATA_UNDERFLOW)?
    }

    data.eci = take_bits(ds, 8) as u32;
    if data.eci & 0xc0 == 0x80 {
        if bits_remaining(ds) < 8 {
            Err(DeQRError::DATA_UNDERFLOW)?
        }
        data.eci = (data.eci << 8) | (take_bits(ds, 8) as u32)
    } else if data.eci & 0xe0 == 0xc0 {
        if bits_remaining(ds) < 16 {
            Err(DeQRError::DATA_UNDERFLOW)?
        }

        data.eci = (data.eci << 16) | (take_bits(ds, 16) as u32)
    }
    Ok(())
}

fn bits_remaining(ds: &DataStream) -> usize {
    assert!(ds.data_bits > ds.ptr);
    ds.data_bits - ds.ptr
}

fn decode_kanji(
    data: &mut Data,
    ds: &mut DataStream,
) -> DeQRResult<()> {
    let bits = match data.version {
        0...9 => 8,
        10...26 => 10,
        _ => 12,
    };

    let count = take_bits(ds, bits);
    if data.payload_len + count * 2 + 1 > MAX_PAYLOAD_SIZE {
        Err(DeQRError::DATA_OVERFLOW)?
    }
    if bits_remaining(ds) < count * 13 {
        Err(DeQRError::DATA_UNDERFLOW)?
    }

    for _ in 0..count {
        let d = take_bits(ds, 13);
        let msB = d / 0xc0;
        let lsB = d % 0xc0;
        let intermediate = msB << 8 | lsB;
        let sjw = if intermediate + 0x8140 <= 0x9ffc {
            /* bytes are in the range 0x8140 to 0x9FFC */
            (intermediate + 0x8140) as u16
        } else {
            (intermediate + 0xc140) as u16
        };
        let idx = data.payload_len;
        data.payload[idx..(idx + 2)].copy_from_slice(&[(sjw >> 8) as u8, (sjw & 0xff) as u8]);
        data.payload_len += 2;
    }
    Ok(())
}

fn decode_byte(
    data: &mut Data,
    ds: &mut DataStream,
) -> DeQRResult<()> {
    let bits = match data.version {
        0...9 => 8,
        _ => 16
    };

    let count = take_bits(ds, bits);
    if data.payload_len + count + 1 > MAX_PAYLOAD_SIZE {
        Err(DeQRError::DATA_OVERFLOW)?
    }
    if bits_remaining(ds) < count * 8 {
        return Err(DeQRError::DATA_UNDERFLOW)?;
    }

    for _ in 0..count {
        let idx = data.payload_len;
        data.payload[idx] = take_bits(ds, 8) as u8;
        data.payload_len += 1;
    }
    Ok(())
}

fn decode_alpha(
    data: &mut Data,
    ds: &mut DataStream,
) -> DeQRResult<()> {
    let bits = match data.version {
        0...9 => 9,
        10...26 => 11,
        _ => 13,
    };
    let mut count = take_bits(ds, bits);
    if (*data).payload_len + count + 1 > MAX_PAYLOAD_SIZE {
        Err(DeQRError::DATA_OVERFLOW)?
    }

    while count >= 2 {
        alpha_tuple(data, ds, 11, 2).map_err(|_| DeQRError::DATA_UNDERFLOW)?;
        count -= 2;
    }

    if count == 1 {
        alpha_tuple(data, ds, 6, 1).map_err(|_| DeQRError::DATA_UNDERFLOW)?;
    }

    Ok(())
}

fn alpha_tuple(
    data: &mut Data,
    ds: &mut DataStream,
    bits: usize,
    digits: usize,
) -> Result<(), ()> {
    if bits_remaining(ds) < bits {
        Err(())
    } else {
        let mut tuple = take_bits(ds, bits);
        for i in 0..digits {
            const alpha_map: &[u8; 46] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:\x00";
            data.payload[data.payload_len + digits - i - 1] = alpha_map[tuple % 45];
            tuple /= 45;
        }
        data.payload_len += digits;
        Ok(())
    }
}

fn decode_numeric(
    data: &mut Data,
    ds: &mut DataStream,
) -> DeQRResult<()> {
    let bits = match data.version {
        0...9 => 10,
        10...26 => 12,
        _ => 14,
    };

    let mut count = take_bits(ds, bits);
    if data.payload_len + count + 1 > MAX_PAYLOAD_SIZE {
        Err(DeQRError::DATA_OVERFLOW)?;
    }

    while count >= 3 {
        numeric_tuple(data, ds, 10, 3).map_err(|_| DeQRError::DATA_UNDERFLOW)?;
        count -= 3;
    }

    if count == 2 {
        numeric_tuple(data, ds, 7, 2).map_err(|_| DeQRError::DATA_UNDERFLOW)?;
        count -= 2;
    }
    if count == 1 {
        numeric_tuple(data, ds, 4, 1).map_err(|_| DeQRError::DATA_UNDERFLOW)?;
    }

    Ok(())
}

fn numeric_tuple(
    data: &mut Data,
    ds: &mut DataStream,
    bits: usize,
    digits: usize,
) -> Result<(), ()> {
    if bits_remaining(ds) < bits {
        Err(())
    } else {
        let mut tuple = take_bits(ds, bits);
        for i in (0..digits).rev() {
            data.payload[data.payload_len + i] = (tuple % 10) as u8 + b'0';
            tuple /= 10;
        }
        data.payload_len += digits;
        Ok(())
    }
}

fn codestream_ecc(
    data: &mut Data,
    ds: &mut DataStream,
) -> DeQRResult<()> {
    let ver = &VersionDataBase[data.version as usize];
    let sb_ecc = &ver.ecc[data.ecc_level as usize];
    let lb_ecc = RSParameters {
        bs: sb_ecc.bs + 1,
        dw: sb_ecc.dw + 1,
        ns: sb_ecc.ns,
    };

    let lb_count = (ver.data_bytes - sb_ecc.bs * sb_ecc.ns) / (sb_ecc.bs + 1);
    let bc = lb_count + sb_ecc.ns;
    let ecc_offset = sb_ecc.dw * bc + lb_count;

    let mut dst_offset = 0;
    for i in 0..bc {
        let ecc = if i < sb_ecc.ns {
            sb_ecc
        } else {
            &lb_ecc
        };
        let dst = &mut ds.data[dst_offset..(dst_offset + ecc.bs)];
        let num_ec = ecc.bs - ecc.dw;
        for j in 0..ecc.dw {
            dst[j] = ds.raw[j * bc + i];
        }
        for j in 0..num_ec {
            dst[ecc.dw + j] = ds.raw[ecc_offset + j * bc + i];
        }
        correct_block(dst, ecc)?;

        dst_offset += ecc.dw;
    }

    ds.data_bits = dst_offset * 8;
    Ok(())
}

fn correct_block(
    data: &mut [u8],
    ecc: &RSParameters,
) -> DeQRResult<()> {
    assert!(ecc.bs > ecc.dw);

    let mut npar = ecc.bs - ecc.dw;
    let mut sigma_deriv = [GF256::Zero; 64];

    // Calculate syndromes. If all 0 there is nothing to do.
    let s = match block_syndromes(data, ecc.bs, npar) {
        Ok(_) => return Ok(()),
        Err(s) => s,
    };

    let sigma = berlekamp_massey(&s, npar);
    /* Compute derivative of sigma */
    for i in (1..64).step_by(2) {
        sigma_deriv[i - 1] = sigma[i];
    }

    /* Compute error evaluator polynomial */
    let mut omega = eloc_poly(
        &s,
        &sigma,
        npar - 1,
    );

    /* Find error locations and magnitudes */
    for i in 0..ecc.bs {
        let xinv = GF256::pow(255 - i);
        if poly_eval(&sigma, xinv) == GF256::Zero {
            let sd_x = poly_eval(&sigma_deriv, xinv);
            let omega_x = poly_eval(&omega, xinv);
            let error = omega_x / sd_x;
            data[ecc.bs - i - 1] = (GF256(data[ecc.bs - i - 1]) + error).0;
        }
    }

    match block_syndromes(data, ecc.bs, npar) {
        Ok(_) => Ok(()),
        Err(_) => Err(DeQRError::DATA_ECC),
    }
}
/* ***********************************************************************
 * Code stream error correction
 *
 * Generator polynomial for GF(2^8) is x^8 + x^4 + x^3 + x^2 + 1
 */
fn block_syndromes(
    data: &[u8],
    bs: usize,
    npar: usize,
) -> Result<[GF256; 64], [GF256; 64]> {
    let mut nonzero: bool = false;
    let mut s = [GF256::Zero; 64];

    for i in 0..npar {
        for j in 0..bs {
            let c = GF256(data[bs - j - 1]);
            s[i] += c * GF256::pow(i * j);
        }
        if s[i] != GF256::Zero {
            nonzero = true;
        }
    }
    if nonzero {
        Err(s)
    } else {
        Ok(s)
    }
}


fn poly_eval<G>(
    s: &[G; 64],
    x: G,
) -> G where G: GaloisField {
    let mut sum = G::Zero;
    let mut x_pow = G::One;

    for i in 0..64 {
        sum += s[i] * x_pow;
        x_pow *= x;
    }
    sum
}

fn eloc_poly(
    s: &[GF256; 64],
    sigma: &[GF256; 64],
    npar: usize,
) -> [GF256; 64] {
    let mut omega = [GF256::Zero; 64];
    for i in 0..npar {
        let a = sigma[i];
        for j in 0..(npar - i) {
            let b = s[j + 1];
            omega[i + j] += a * b;
        }
    }
    omega
}
/* ***********************************************************************
 * Berlekamp-Massey algorithm for finding error locator polynomials.
 */
fn berlekamp_massey<G>(
    s: &[G; 64],
    N: usize,
) -> [G; 64] where G: GaloisField {
    let mut T: [G; 64] = [G::Zero; 64];
    let mut C: [G; 64] = [G::Zero; 64];
    let mut B: [G; 64] = [G::Zero; 64];
    let mut L: usize = 0;
    let mut m: usize = 1;
    let mut b = G::One;
    B[0] = G::One;
    C[0] = G::One;

    for n in 0..N {
        let mut d = s[n];

        // Calculate in GF(p):
        // d = s[n] + \Sum_{i=1}^{L} C[i] * s[n - i]
        for i in 1..=L {
            d += C[i] * s[n - i];
        }
        // Pre-calculate d * b^-1 in GF(p)
        let mult = d / b;

        if d == G::Zero {
            m += 1
        } else if L * 2 <= n {
            T.copy_from_slice(&C);
            poly_add(&mut C, &B, mult, m);
            B.copy_from_slice(&T);
            L = n + 1 - L;
            b = d;
            m = 1
        } else {
            poly_add(&mut C, &B, mult, m);
            m += 1
        }
    }
    C
}
/* ***********************************************************************
 * Polynomial operations
 */
fn poly_add<G>(
    dst: &mut [G; 64],
    src: &[G; 64],
    c: G,
    shift: usize,
) -> () where G: GaloisField {
    if c == G::Zero {
        return;
    }

    for i in 0..64 {
        let p = i + shift;
        if p >= 64 {
            break;
        }
        let v = src[i];
        dst[p] += v * c;
    }
}

fn read_data(
    code: &Code,
    data: &mut Data,
    ds: &mut DataStream,
) -> () {
    assert!(code.size > 0);

    let mut y = code.size - 1;
    let mut x = code.size - 1;
    let mut neg_dir = true;

    while x > 0 {
        if x == 6 {
            x -= 1;
        }
        if !reserved_cell(data.version, y, x) {
            read_bit(code, data, ds, y, x);
        }
        if !reserved_cell((*data).version, y, x - 1) {
            read_bit(code, data, ds, y, x - 1);
        }

        let (new_y, new_neg_dir) = match (y, neg_dir) {
            (0, true) => {
                x = x.saturating_sub(2);
                (0, false)
            }
            (y, false) if y == code.size - 1 => {
                x = x.saturating_sub(2);
                (code.size - 1, true)
            }
            (y, true) => (y - 1, true),
            (y, false) => (y + 1, false),
        };

        y = new_y;
        neg_dir = new_neg_dir;
    }
}

fn read_bit(
    code: &Code,
    data: &Data,
    ds: &mut DataStream,
    i: usize,
    j: usize,
) -> () {
    let bitpos = (ds.data_bits & 7) as u8;
    let bytepos = ds.data_bits >> 3;
    let mut v = grid_bit(code, j, i);
    if mask_bit(data.mask, i, j) {
        v ^= 1
    }
    if v != 0 {
        ds.raw[bytepos as usize] |= 0x80_u8 >> bitpos;
    }
    ds.data_bits += 1;
}

fn grid_bit(
    mut code: &Code,
    x: usize,
    y: usize,
) -> u8 {
    let p = y * code.size + x;
    (code.cell_bitmap[p >> 3] >> (p & 7)) & 1
}

fn mask_bit(
    mask: u16,
    i: usize,
    j: usize,
) -> bool {
    match mask {
        0 => 0 == (i + j) % 2,
        1 => 0 == i % 2,
        2 => 0 == j % 3,
        3 => 0 == (i + j) % 3,
        4 => 0 == ((i / 2) + (j / 3)) % 2,
        5 => 0 == ((i * j) % 2 + (i * j)) % 3,
        6 => 0 == ((i * j) % 2 + (i * j) % 3) % 2,
        7 => 0 == ((i * j) % 3 + (i + j) % 2) % 2,
        _ => false,
    }
}

fn reserved_cell(
    version: usize,
    i: usize,
    j: usize,
) -> bool {
    let ver = &VersionDataBase[version as usize];
    let size = version * 4 + 17;

    /* Finder + format: top left */
    if i < 9 && j < 9 {
        return true;
    }

    /* Finder + format: bottom left */
    if i + 8 >= size && j < 9 {
        return true;
    }

    /* Finder + format: top right */
    if i < 9 && j + 8 >= size {
        return true;
    }

    /* Exclude timing patterns */
    if i == 6 || j == 6 {
        return true;
    }

    /* Exclude version info, if it exists. Version info sits adjacent to
     * the top-right and bottom-left finders in three rows, bounded by
     * the timing pattern.
     */
    if version >= 7 {
        if i < 6 && j + 11 >= size {
            return true;
        } else if i + 11 >= size && j < 6 {
            return true;
        }
    }

    /* Exclude alignment patterns */
    let mut ai = None;
    let mut aj = None;

    fn abs_diff(x: usize, y: usize) -> usize {
        if x < y {
            y - x
        } else {
            x - y
        }
    }

    let mut len = 0;
    for (a, &pattern) in ver.apat.iter().take_while(|&&x| x != 0).enumerate() {
        len = a;
        if abs_diff(pattern, i) < 3 {
            ai = Some(a)
        }
        if abs_diff(pattern, j) < 3 {
            aj = Some(a)
        }
    }

    match (ai, aj) {
        (Some(x), Some(y)) if x == len && y == len => true,
        (Some(x), Some(_)) if 0 < x && x < len => true,
        (Some(_), Some(x)) if 0 < x && x < len => true,
        _ => false,
    }
}

fn correct_format(mut word: u16) -> DeQRResult<u16> {
    /* Evaluate U (received codeword) at each of alpha_1 .. alpha_6
     * to get S_1 .. S_6 (but we index them from 0).
     */
    if let Err(mut s) = format_syndromes(word) {
        let sigma = berlekamp_massey(&mut s, 6);

        /* Now, find the roots of the polynomial */
        for i in 0..15 {
            if poly_eval(&sigma, GF16::pow(15 - i)) == GF16::Zero {
                word ^= 1 << i;
            }
        }

        // Double check syndromes
        format_syndromes(word)
            .map_err(|_| DeQRError::FORMAT_ECC)?;
    }
    Ok(word)
}

fn read_format(
    code: &Code,
    data: &mut Data,
) -> DeQRResult<()> {
    let mut format = 0;

    // Try first location
    const xs: [usize; 15] = [
        8, 8, 8, 8, 8, 8, 8, 8, 7, 5, 4, 3, 2, 1, 0
    ];
    const ys: [usize; 15] = [
        0, 1, 2, 3, 4, 5, 7, 8, 8, 8, 8, 8, 8, 8, 8
    ];
    for i in (0..15).rev() {
        format = (format << 1) | grid_bit(code, xs[i], ys[i]) as u16;
    }
    format ^= 0x5412;


// Check format, try other location if needed
    let verified_format = correct_format(format).or_else(|_| {
        let mut format = 0;
        for i in 0..7 {
            format = (format << 1) | grid_bit(code, 8, (code).size - 1 - i) as u16;
        }
        for i in 0..8 {
            format = (format << 1) | grid_bit(code, (code).size - 8 + i, 8) as u16;
        }
        format ^= 0x5412;
        correct_format(format)
    })?;

    let fdata = verified_format >> 10;
    data.ecc_level = fdata >> 3;
    data.mask = fdata & 7;
    Ok(())
}
/* ***********************************************************************
 * Format value error correction
 *
 * Generator polynomial for GF(2^4) is x^4 + x + 1
 */
fn format_syndromes(u: u16) -> Result<[GF16; 64], [GF16; 64]> {
    let mut result = [GF16(0); 64];
    let mut nonzero = false;

    for i in 0..6 {
        for j in 0..15 {
            if u & (1 << j) != 0 {
                result[i] += GF16::pow((i + 1) * j);
            }
        }
        if result[i].0 != 0 {
            nonzero = true;
        }
    }

    if nonzero {
        Err(result)
    } else {
        Ok(result)
    }
}
