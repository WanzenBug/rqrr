use ::Point;
use DeQRResult;
use DeQRError;

#[derive(Copy, Clone)]
pub struct Code {
    pub corners: [Point; 4],
    pub size: i32,
    pub cell_bitmap: [u8; 3917],
}
/* This structure holds the decoded QR-code data */
#[derive(Copy, Clone)]
pub struct quirc_data {
    pub version: i32,
    pub ecc_level: i32,
    pub mask: i32,
    pub data_type: i32,
    pub payload: [u8; 8896],
    pub payload_len: i32,
    pub eci: uint32_t,
}
/* ***********************************************************************
 * Decoder algorithm
 */
#[derive(Copy, Clone)]
pub struct datastream {
    pub raw: [u8; 8896],
    pub data_bits: i32,
    pub ptr: i32,
    pub data: [u8; 8896],
}
/* ***********************************************************************
 * QR-code version information database
 */
#[derive(Copy, Clone)]
pub struct ReedSolomonParams {
    pub bs: i32,
    pub dw: i32,
    pub ns: i32,
}

#[derive(Copy, Clone)]
pub struct VersionInfo {
    pub data_bytes: i32,
    pub apat: [i32; 7],
    pub ecc: [ReedSolomonParams; 4],
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
/* ***********************************************************************
 * Galois fields
 */
#[derive(Copy, Clone)]
pub struct GaloisField {
    pub p: usize,
    pub log: &'static [u8],
    pub exp: &'static [u8],
}
/* Decode a QR-code, returning the payload data. */
#[no_mangle]
pub unsafe extern "C" fn quirc_decode(
    mut code: *const quirc_code,
    mut data: *mut quirc_data,
) -> quirc_decode_error_t {
    let mut err: quirc_decode_error_t = QUIRC_SUCCESS;
    let mut ds: datastream = datastream {
        raw: [0; 8896],
        data_bits: 0,
        ptr: 0,
        data: [0; 8896],
    };
    if 0 != ((*code).size - 17i32) % 4i32 {
        return QUIRC_ERROR_INVALID_GRID_SIZE;
    } else {
        memset(
            data as *mut libc::c_void,
            0i32,
            ::std::mem::size_of::<quirc_data>() as u64,
        );
        memset(
            &mut ds as *mut datastream as *mut libc::c_void,
            0i32,
            ::std::mem::size_of::<datastream>() as u64,
        );
        (*data).version = ((*code).size - 17i32) / 4i32;
        if (*data).version < 1i32 || (*data).version > 40i32 {
            return QUIRC_ERROR_INVALID_VERSION;
        } else {
            /* Read format information -- try both locations */
            err = read_format(code, data, 0i32);
            if 0 != err as u64 {
                err = read_format(code, data, 1i32)
            }
            if 0 != err as u64 {
                return err;
            } else {
                read_data(code, data, &mut ds);
                err = codestream_ecc(data, &mut ds);
                if 0 != err as u64 {
                    return err;
                } else {
                    err = decode_payload(data, &mut ds);
                    if 0 != err as u64 {
                        return err;
                    } else {
                        return QUIRC_SUCCESS;
                    }
                }
            }
        }
    };
}

unsafe extern "C" fn decode_payload(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) -> quirc_decode_error_t {
    while bits_remaining(ds) >= 4i32 {
        let mut err: quirc_decode_error_t = QUIRC_SUCCESS;
        let mut type_0: i32 = take_bits(ds, 4i32);
        match type_0 {
            1 => err = decode_numeric(data, ds),
            2 => err = decode_alpha(data, ds),
            4 => err = decode_byte(data, ds),
            8 => err = decode_kanji(data, ds),
            7 => err = decode_eci(data, ds),
            _ => {
                break;
            }
        }
        if 0 != err as u64 {
            return err;
        } else {
            if !(0 == type_0 & type_0 - 1i32 && type_0 > (*data).data_type) {
                continue;
            }
            (*data).data_type = type_0
        }
    }
    /* Add nul terminator to all payloads */
    if (*data).payload_len as u64
        >= ::std::mem::size_of::<[u8; 8896]>() as u64
        {
            (*data).payload_len -= 1
        }
    (*data).payload[(*data).payload_len as usize] = 0_u8;
    return QUIRC_SUCCESS;
}

unsafe extern "C" fn take_bits(mut ds: *mut datastream, mut len: i32) -> i32 {
    let mut ret: i32 = 0i32;
    while 0 != len && (*ds).ptr < (*ds).data_bits {
        let mut b: u8 = (*ds).data[((*ds).ptr >> 3i32) as usize];
        let mut bitpos: i32 = (*ds).ptr & 7i32;
        ret <<= 1i32;
        if 0 != (b as i32) << bitpos & 0x80i32 {
            ret |= 1i32
        }
        (*ds).ptr += 1;
        len -= 1
    }
    return ret;
}

unsafe extern "C" fn decode_eci(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) -> quirc_decode_error_t {
    if bits_remaining(ds) < 8i32 {
        return QUIRC_ERROR_DATA_UNDERFLOW;
    } else {
        (*data).eci = take_bits(ds, 8i32) as uint32_t;
        if (*data).eci & 0xc0i32 as u32 == 0x80i32 as u32 {
            if bits_remaining(ds) < 8i32 {
                return QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                (*data).eci = (*data).eci << 8i32 | take_bits(ds, 8i32) as u32
            }
        } else if (*data).eci & 0xe0i32 as u32 == 0xc0i32 as u32 {
            if bits_remaining(ds) < 16i32 {
                return QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                (*data).eci = (*data).eci << 16i32 | take_bits(ds, 16i32) as u32
            }
        }
        return QUIRC_SUCCESS;
    };
}

unsafe extern "C" fn bits_remaining(mut ds: *const datastream) -> i32 {
    return (*ds).data_bits - (*ds).ptr;
}

unsafe extern "C" fn decode_kanji(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) -> quirc_decode_error_t {
    let mut bits: i32 = 12i32;
    let mut count: i32 = 0;
    let mut i: i32 = 0;
    if (*data).version < 10i32 {
        bits = 8i32
    } else if (*data).version < 27i32 {
        bits = 10i32
    }
    count = take_bits(ds, bits);
    if (*data).payload_len + count * 2i32 + 1i32 > 8896i32 {
        return QUIRC_ERROR_DATA_OVERFLOW;
    } else if bits_remaining(ds) < count * 13i32 {
        return QUIRC_ERROR_DATA_UNDERFLOW;
    } else {
        i = 0i32;
        while i < count {
            let mut d: i32 = take_bits(ds, 13i32);
            let mut msB: i32 = d / 0xc0i32;
            let mut lsB: i32 = d % 0xc0i32;
            let mut intermediate: i32 = msB << 8i32 | lsB;
            let mut sjw: uint16_t = 0;
            if intermediate + 0x8140i32 <= 0x9ffci32 {
                /* bytes are in the range 0x8140 to 0x9FFC */
                sjw = (intermediate + 0x8140i32) as uint16_t
            } else {
                sjw = (intermediate + 0xc140i32) as uint16_t
            }
            let fresh0 = (*data).payload_len;
            (*data).payload_len = (*data).payload_len + 1;
            (*data).payload[fresh0 as usize] = (sjw as i32 >> 8i32) as u8;
            let fresh1 = (*data).payload_len;
            (*data).payload_len = (*data).payload_len + 1;
            (*data).payload[fresh1 as usize] = (sjw as i32 & 0xffi32) as u8;
            i += 1
        }
        return QUIRC_SUCCESS;
    };
}

unsafe extern "C" fn decode_byte(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) -> quirc_decode_error_t {
    let mut bits: i32 = 16i32;
    let mut count: i32 = 0;
    let mut i: i32 = 0;
    if (*data).version < 10i32 {
        bits = 8i32
    }
    count = take_bits(ds, bits);
    if (*data).payload_len + count + 1i32 > 8896i32 {
        return QUIRC_ERROR_DATA_OVERFLOW;
    } else if bits_remaining(ds) < count * 8i32 {
        return QUIRC_ERROR_DATA_UNDERFLOW;
    } else {
        i = 0i32;
        while i < count {
            let fresh2 = (*data).payload_len;
            (*data).payload_len = (*data).payload_len + 1;
            (*data).payload[fresh2 as usize] = take_bits(ds, 8i32) as u8;
            i += 1
        }
        return QUIRC_SUCCESS;
    };
}

unsafe extern "C" fn decode_alpha(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) -> quirc_decode_error_t {
    let mut bits: i32 = 13i32;
    let mut count: i32 = 0;
    if (*data).version < 10i32 {
        bits = 9i32
    } else if (*data).version < 27i32 {
        bits = 11i32
    }
    count = take_bits(ds, bits);
    if (*data).payload_len + count + 1i32 > 8896i32 {
        return QUIRC_ERROR_DATA_OVERFLOW;
    } else {
        while count >= 2i32 {
            if alpha_tuple(data, ds, 11i32, 2i32) < 0i32 {
                return QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                count -= 2i32
            }
        }
        if 0 != count {
            if alpha_tuple(data, ds, 6i32, 1i32) < 0i32 {
                return QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                count -= 1
            }
        }
        return QUIRC_SUCCESS;
    };
}

unsafe extern "C" fn alpha_tuple(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
    mut bits: i32,
    mut digits: i32,
) -> i32 {
    let mut tuple: i32 = 0;
    let mut i: i32 = 0;
    if bits_remaining(ds) < bits {
        return -1i32;
    } else {
        tuple = take_bits(ds, bits);
        i = 0i32;
        while i < digits {
            static mut alpha_map: *const u8 = unsafe {
                b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:\x00" as *const u8
                    as *const u8
            };
            (*data).payload[((*data).payload_len + digits - i - 1i32) as usize] =
                *alpha_map.offset((tuple % 45i32) as isize) as u8;
            tuple /= 45i32;
            i += 1
        }
        (*data).payload_len += digits;
        return 0i32;
    };
}

unsafe extern "C" fn decode_numeric(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) -> quirc_decode_error_t {
    let mut bits: i32 = 14i32;
    let mut count: i32 = 0;
    if (*data).version < 10i32 {
        bits = 10i32
    } else if (*data).version < 27i32 {
        bits = 12i32
    }
    count = take_bits(ds, bits);
    if (*data).payload_len + count + 1i32 > 8896i32 {
        return QUIRC_ERROR_DATA_OVERFLOW;
    } else {
        while count >= 3i32 {
            if numeric_tuple(data, ds, 10i32, 3i32) < 0i32 {
                return QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                count -= 3i32
            }
        }
        if count >= 2i32 {
            if numeric_tuple(data, ds, 7i32, 2i32) < 0i32 {
                return QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                count -= 2i32
            }
        }
        if 0 != count {
            if numeric_tuple(data, ds, 4i32, 1i32) < 0i32 {
                return QUIRC_ERROR_DATA_UNDERFLOW;
            } else {
                count -= 1
            }
        }
        return QUIRC_SUCCESS;
    };
}

unsafe extern "C" fn numeric_tuple(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
    mut bits: i32,
    mut digits: i32,
) -> i32 {
    let mut tuple: i32 = 0;
    let mut i: i32 = 0;
    if bits_remaining(ds) < bits {
        return -1i32;
    } else {
        tuple = take_bits(ds, bits);
        i = digits - 1i32;
        while i >= 0i32 {
            (*data).payload[((*data).payload_len + i) as usize] =
                (tuple % 10i32 + '0' as i32) as u8;
            tuple /= 10i32;
            i -= 1
        }
        (*data).payload_len += digits;
        return 0i32;
    };
}

unsafe extern "C" fn codestream_ecc(
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) -> quirc_decode_error_t {
    let mut ver: *const VersionInfo =
        &quirc_version_db[(*data).version as usize] as *const VersionInfo;
    let mut sb_ecc: *const ReedSolomonParams =
        &(*ver).ecc[(*data).ecc_level as usize] as *const ReedSolomonParams;
    let mut lb_ecc: ReedSolomonParams = ReedSolomonParams {
        bs: 0,
        dw: 0,
        ns: 0,
    };
    let lb_count: i32 =
        ((*ver).data_bytes - (*sb_ecc).bs * (*sb_ecc).ns) / ((*sb_ecc).bs + 1i32);
    let bc: i32 = lb_count + (*sb_ecc).ns;
    let ecc_offset: i32 = (*sb_ecc).dw * bc + lb_count;
    let mut dst_offset: i32 = 0i32;
    let mut i: i32 = 0;
    memcpy(
        &mut lb_ecc as *mut ReedSolomonParams as *mut libc::c_void,
        sb_ecc as *const libc::c_void,
        ::std::mem::size_of::<ReedSolomonParams>() as u64,
    );
    lb_ecc.dw += 1;
    lb_ecc.bs += 1;
    i = 0i32;
    while i < bc {
        let mut dst: *mut u8 = (*ds).data.as_mut_ptr().offset(dst_offset as isize);
        let mut ecc: *const ReedSolomonParams = if i < (*sb_ecc).ns {
            sb_ecc
        } else {
            &mut lb_ecc
        };
        let num_ec: i32 = (*ecc).bs - (*ecc).dw;
        let mut err: quirc_decode_error_t = QUIRC_SUCCESS;
        let mut j: i32 = 0;
        j = 0i32;
        while j < (*ecc).dw {
            *dst.offset(j as isize) = (*ds).raw[(j * bc + i) as usize];
            j += 1
        }
        j = 0i32;
        while j < num_ec {
            *dst.offset(((*ecc).dw + j) as isize) = (*ds).raw[(ecc_offset + j * bc + i) as usize];
            j += 1
        }
        err = correct_block(dst, ecc);
        if 0 != err as u64 {
            return err;
        } else {
            dst_offset += (*ecc).dw;
            i += 1
        }
    }
    (*ds).data_bits = dst_offset * 8i32;
    return QUIRC_SUCCESS;
}

unsafe extern "C" fn correct_block(
    mut data: *mut u8,
    mut ecc: *const ReedSolomonParams,
) -> quirc_decode_error_t {
    let mut npar: i32 = (*ecc).bs - (*ecc).dw;
    let mut s: [u8; 64] = [0; 64];
    let mut sigma: [u8; 64] = [0; 64];
    let mut sigma_deriv: [u8; 64] = [0; 64];
    let mut omega: [u8; 64] = [0; 64];
    let mut i: i32 = 0;
    /* Compute syndrome vector */
    if 0 == block_syndromes(data, (*ecc).bs, npar, s.as_mut_ptr()) {
        return QUIRC_SUCCESS;
    } else {
        berlekamp_massey(s.as_mut_ptr(), npar, &gf256, sigma.as_mut_ptr());
        /* Compute derivative of sigma */
        memset(
            sigma_deriv.as_mut_ptr() as *mut libc::c_void,
            0i32,
            64i32 as u64,
        );
        i = 0i32;
        while i + 1i32 < 64i32 {
            sigma_deriv[i as usize] = sigma[(i + 1i32) as usize];
            i += 2i32
        }
        /* Compute error evaluator polynomial */
        eloc_poly(
            omega.as_mut_ptr(),
            s.as_mut_ptr(),
            sigma.as_mut_ptr(),
            npar - 1i32,
        );
        /* Find error locations and magnitudes */
        i = 0i32;
        while i < (*ecc).bs {
            let mut xinv: u8 = gf256_exp[(255i32 - i) as usize];
            if 0 == poly_eval(sigma.as_mut_ptr(), xinv, &gf256) {
                let mut sd_x: u8 = poly_eval(sigma_deriv.as_mut_ptr(), xinv, &gf256);
                let mut omega_x: u8 = poly_eval(omega.as_mut_ptr(), xinv, &gf256);
                let mut error: u8 =
                    gf256_exp[((255i32 - gf256_log[sd_x as usize] as i32
                        + gf256_log[omega_x as usize] as i32)
                        % 255i32) as usize];
                let ref mut fresh3 = *data.offset(((*ecc).bs - i - 1i32) as isize);
                *fresh3 = (*fresh3 as i32 ^ error as i32) as u8
            }
            i += 1
        }
        if 0 != block_syndromes(data, (*ecc).bs, npar, s.as_mut_ptr()) {
            return QUIRC_ERROR_DATA_ECC;
        } else {
            return QUIRC_SUCCESS;
        }
    };
}
/* ***********************************************************************
 * Code stream error correction
 *
 * Generator polynomial for GF(2^8) is x^8 + x^4 + x^3 + x^2 + 1
 */
unsafe extern "C" fn block_syndromes(
    mut data: *const u8,
    mut bs: i32,
    mut npar: i32,
    mut s: *mut u8,
) -> i32 {
    let mut nonzero: i32 = 0i32;
    let mut i: i32 = 0;
    memset(s as *mut libc::c_void, 0i32, 64i32 as u64);
    i = 0i32;
    while i < npar {
        let mut j: i32 = 0;
        j = 0i32;
        while j < bs {
            let mut c: u8 = *data.offset((bs - j - 1i32) as isize);
            if !(0 == c) {
                let ref mut fresh4 = *s.offset(i as isize);
                *fresh4 = (*fresh4 as i32
                    ^ gf256_exp[((gf256_log[c as usize] as i32 + i * j) % 255i32) as usize]
                    as i32) as u8
            }
            j += 1
        }
        if 0 != *s.offset(i as isize) {
            nonzero = 1i32
        }
        i += 1
    }
    return nonzero;
}

static mut gf256_log: [u8; 256] = {
    [
        0_u8,
        0xff_u8,
        0x1_u8,
        0x19_u8,
        0x2_u8,
        0x32_u8,
        0x1a_u8,
        0xc6_u8,
        0x3_u8,
        0xdf_u8,
        0x33_u8,
        0xee_u8,
        0x1b_u8,
        0x68_u8,
        0xc7_u8,
        0x4b_u8,
        0x4_u8,
        0x64_u8,
        0xe0_u8,
        0xe_u8,
        0x34_u8,
        0x8d_u8,
        0xef_u8,
        0x81_u8,
        0x1c_u8,
        0xc1_u8,
        0x69_u8,
        0xf8_u8,
        0xc8_u8,
        0x8_u8,
        0x4c_u8,
        0x71_u8,
        0x5_u8,
        0x8a_u8,
        0x65_u8,
        0x2f_u8,
        0xe1_u8,
        0x24_u8,
        0xf_u8,
        0x21_u8,
        0x35_u8,
        0x93_u8,
        0x8e_u8,
        0xda_u8,
        0xf0_u8,
        0x12_u8,
        0x82_u8,
        0x45_u8,
        0x1d_u8,
        0xb5_u8,
        0xc2_u8,
        0x7d_u8,
        0x6a_u8,
        0x27_u8,
        0xf9_u8,
        0xb9_u8,
        0xc9_u8,
        0x9a_u8,
        0x9_u8,
        0x78_u8,
        0x4d_u8,
        0xe4_u8,
        0x72_u8,
        0xa6_u8,
        0x6_u8,
        0xbf_u8,
        0x8b_u8,
        0x62_u8,
        0x66_u8,
        0xdd_u8,
        0x30_u8,
        0xfd_u8,
        0xe2_u8,
        0x98_u8,
        0x25_u8,
        0xb3_u8,
        0x10_u8,
        0x91_u8,
        0x22_u8,
        0x88_u8,
        0x36_u8,
        0xd0_u8,
        0x94_u8,
        0xce_u8,
        0x8f_u8,
        0x96_u8,
        0xdb_u8,
        0xbd_u8,
        0xf1_u8,
        0xd2_u8,
        0x13_u8,
        0x5c_u8,
        0x83_u8,
        0x38_u8,
        0x46_u8,
        0x40_u8,
        0x1e_u8,
        0x42_u8,
        0xb6_u8,
        0xa3_u8,
        0xc3_u8,
        0x48_u8,
        0x7e_u8,
        0x6e_u8,
        0x6b_u8,
        0x3a_u8,
        0x28_u8,
        0x54_u8,
        0xfa_u8,
        0x85_u8,
        0xba_u8,
        0x3d_u8,
        0xca_u8,
        0x5e_u8,
        0x9b_u8,
        0x9f_u8,
        0xa_u8,
        0x15_u8,
        0x79_u8,
        0x2b_u8,
        0x4e_u8,
        0xd4_u8,
        0xe5_u8,
        0xac_u8,
        0x73_u8,
        0xf3_u8,
        0xa7_u8,
        0x57_u8,
        0x7_u8,
        0x70_u8,
        0xc0_u8,
        0xf7_u8,
        0x8c_u8,
        0x80_u8,
        0x63_u8,
        0xd_u8,
        0x67_u8,
        0x4a_u8,
        0xde_u8,
        0xed_u8,
        0x31_u8,
        0xc5_u8,
        0xfe_u8,
        0x18_u8,
        0xe3_u8,
        0xa5_u8,
        0x99_u8,
        0x77_u8,
        0x26_u8,
        0xb8_u8,
        0xb4_u8,
        0x7c_u8,
        0x11_u8,
        0x44_u8,
        0x92_u8,
        0xd9_u8,
        0x23_u8,
        0x20_u8,
        0x89_u8,
        0x2e_u8,
        0x37_u8,
        0x3f_u8,
        0xd1_u8,
        0x5b_u8,
        0x95_u8,
        0xbc_u8,
        0xcf_u8,
        0xcd_u8,
        0x90_u8,
        0x87_u8,
        0x97_u8,
        0xb2_u8,
        0xdc_u8,
        0xfc_u8,
        0xbe_u8,
        0x61_u8,
        0xf2_u8,
        0x56_u8,
        0xd3_u8,
        0xab_u8,
        0x14_u8,
        0x2a_u8,
        0x5d_u8,
        0x9e_u8,
        0x84_u8,
        0x3c_u8,
        0x39_u8,
        0x53_u8,
        0x47_u8,
        0x6d_u8,
        0x41_u8,
        0xa2_u8,
        0x1f_u8,
        0x2d_u8,
        0x43_u8,
        0xd8_u8,
        0xb7_u8,
        0x7b_u8,
        0xa4_u8,
        0x76_u8,
        0xc4_u8,
        0x17_u8,
        0x49_u8,
        0xec_u8,
        0x7f_u8,
        0xc_u8,
        0x6f_u8,
        0xf6_u8,
        0x6c_u8,
        0xa1_u8,
        0x3b_u8,
        0x52_u8,
        0x29_u8,
        0x9d_u8,
        0x55_u8,
        0xaa_u8,
        0xfb_u8,
        0x60_u8,
        0x86_u8,
        0xb1_u8,
        0xbb_u8,
        0xcc_u8,
        0x3e_u8,
        0x5a_u8,
        0xcb_u8,
        0x59_u8,
        0x5f_u8,
        0xb0_u8,
        0x9c_u8,
        0xa9_u8,
        0xa0_u8,
        0x51_u8,
        0xb_u8,
        0xf5_u8,
        0x16_u8,
        0xeb_u8,
        0x7a_u8,
        0x75_u8,
        0x2c_u8,
        0xd7_u8,
        0x4f_u8,
        0xae_u8,
        0xd5_u8,
        0xe9_u8,
        0xe6_u8,
        0xe7_u8,
        0xad_u8,
        0xe8_u8,
        0x74_u8,
        0xd6_u8,
        0xf4_u8,
        0xea_u8,
        0xa8_u8,
        0x50_u8,
        0x58_u8,
        0xaf_u8,
    ]
};
static mut gf256_exp: [u8; 256] = {
    [
        0x1_u8,
        0x2_u8,
        0x4_u8,
        0x8_u8,
        0x10_u8,
        0x20_u8,
        0x40_u8,
        0x80_u8,
        0x1d_u8,
        0x3a_u8,
        0x74_u8,
        0xe8_u8,
        0xcd_u8,
        0x87_u8,
        0x13_u8,
        0x26_u8,
        0x4c_u8,
        0x98_u8,
        0x2d_u8,
        0x5a_u8,
        0xb4_u8,
        0x75_u8,
        0xea_u8,
        0xc9_u8,
        0x8f_u8,
        0x3_u8,
        0x6_u8,
        0xc_u8,
        0x18_u8,
        0x30_u8,
        0x60_u8,
        0xc0_u8,
        0x9d_u8,
        0x27_u8,
        0x4e_u8,
        0x9c_u8,
        0x25_u8,
        0x4a_u8,
        0x94_u8,
        0x35_u8,
        0x6a_u8,
        0xd4_u8,
        0xb5_u8,
        0x77_u8,
        0xee_u8,
        0xc1_u8,
        0x9f_u8,
        0x23_u8,
        0x46_u8,
        0x8c_u8,
        0x5_u8,
        0xa_u8,
        0x14_u8,
        0x28_u8,
        0x50_u8,
        0xa0_u8,
        0x5d_u8,
        0xba_u8,
        0x69_u8,
        0xd2_u8,
        0xb9_u8,
        0x6f_u8,
        0xde_u8,
        0xa1_u8,
        0x5f_u8,
        0xbe_u8,
        0x61_u8,
        0xc2_u8,
        0x99_u8,
        0x2f_u8,
        0x5e_u8,
        0xbc_u8,
        0x65_u8,
        0xca_u8,
        0x89_u8,
        0xf_u8,
        0x1e_u8,
        0x3c_u8,
        0x78_u8,
        0xf0_u8,
        0xfd_u8,
        0xe7_u8,
        0xd3_u8,
        0xbb_u8,
        0x6b_u8,
        0xd6_u8,
        0xb1_u8,
        0x7f_u8,
        0xfe_u8,
        0xe1_u8,
        0xdf_u8,
        0xa3_u8,
        0x5b_u8,
        0xb6_u8,
        0x71_u8,
        0xe2_u8,
        0xd9_u8,
        0xaf_u8,
        0x43_u8,
        0x86_u8,
        0x11_u8,
        0x22_u8,
        0x44_u8,
        0x88_u8,
        0xd_u8,
        0x1a_u8,
        0x34_u8,
        0x68_u8,
        0xd0_u8,
        0xbd_u8,
        0x67_u8,
        0xce_u8,
        0x81_u8,
        0x1f_u8,
        0x3e_u8,
        0x7c_u8,
        0xf8_u8,
        0xed_u8,
        0xc7_u8,
        0x93_u8,
        0x3b_u8,
        0x76_u8,
        0xec_u8,
        0xc5_u8,
        0x97_u8,
        0x33_u8,
        0x66_u8,
        0xcc_u8,
        0x85_u8,
        0x17_u8,
        0x2e_u8,
        0x5c_u8,
        0xb8_u8,
        0x6d_u8,
        0xda_u8,
        0xa9_u8,
        0x4f_u8,
        0x9e_u8,
        0x21_u8,
        0x42_u8,
        0x84_u8,
        0x15_u8,
        0x2a_u8,
        0x54_u8,
        0xa8_u8,
        0x4d_u8,
        0x9a_u8,
        0x29_u8,
        0x52_u8,
        0xa4_u8,
        0x55_u8,
        0xaa_u8,
        0x49_u8,
        0x92_u8,
        0x39_u8,
        0x72_u8,
        0xe4_u8,
        0xd5_u8,
        0xb7_u8,
        0x73_u8,
        0xe6_u8,
        0xd1_u8,
        0xbf_u8,
        0x63_u8,
        0xc6_u8,
        0x91_u8,
        0x3f_u8,
        0x7e_u8,
        0xfc_u8,
        0xe5_u8,
        0xd7_u8,
        0xb3_u8,
        0x7b_u8,
        0xf6_u8,
        0xf1_u8,
        0xff_u8,
        0xe3_u8,
        0xdb_u8,
        0xab_u8,
        0x4b_u8,
        0x96_u8,
        0x31_u8,
        0x62_u8,
        0xc4_u8,
        0x95_u8,
        0x37_u8,
        0x6e_u8,
        0xdc_u8,
        0xa5_u8,
        0x57_u8,
        0xae_u8,
        0x41_u8,
        0x82_u8,
        0x19_u8,
        0x32_u8,
        0x64_u8,
        0xc8_u8,
        0x8d_u8,
        0x7_u8,
        0xe_u8,
        0x1c_u8,
        0x38_u8,
        0x70_u8,
        0xe0_u8,
        0xdd_u8,
        0xa7_u8,
        0x53_u8,
        0xa6_u8,
        0x51_u8,
        0xa2_u8,
        0x59_u8,
        0xb2_u8,
        0x79_u8,
        0xf2_u8,
        0xf9_u8,
        0xef_u8,
        0xc3_u8,
        0x9b_u8,
        0x2b_u8,
        0x56_u8,
        0xac_u8,
        0x45_u8,
        0x8a_u8,
        0x9_u8,
        0x12_u8,
        0x24_u8,
        0x48_u8,
        0x90_u8,
        0x3d_u8,
        0x7a_u8,
        0xf4_u8,
        0xf5_u8,
        0xf7_u8,
        0xf3_u8,
        0xfb_u8,
        0xeb_u8,
        0xcb_u8,
        0x8b_u8,
        0xb_u8,
        0x16_u8,
        0x2c_u8,
        0x58_u8,
        0xb0_u8,
        0x7d_u8,
        0xfa_u8,
        0xe9_u8,
        0xcf_u8,
        0x83_u8,
        0x1b_u8,
        0x36_u8,
        0x6c_u8,
        0xd8_u8,
        0xad_u8,
        0x47_u8,
        0x8e_u8,
        0x1_u8,
    ]
};
static gf256: GaloisField = GaloisField {
    p: 255,
    log: &gf256_log,
    exp: &gf256_exp,
};

fn poly_eval(
    mut s: &[u8],
    mut x: u8,
    mut gf: &GaloisField,
) -> u8 {
    let mut i: i32 = 0;
    let mut sum: u8 = 0_u8;
    let mut log_x: u8 = gf.log[x];

    if 0 == x {
        return s[0];
    } else {
        i = 0i32;
        while i < 64i32 {
            let mut c: u8 = *s.offset(i as isize);
            if !(0 == c) {
                sum = (sum as i32
                    ^ *(*gf).exp.offset(
                    ((*(*gf).log.offset(c as isize) as i32 + log_x as i32 * i)
                        % (*gf).p) as isize,
                ) as i32) as u8
            }
            i += 1
        }
        return sum;
    };
}

unsafe extern "C" fn eloc_poly(
    mut omega: *mut u8,
    mut s: *const u8,
    mut sigma: *const u8,
    mut npar: i32,
) -> () {
    let mut i: i32 = 0;
    memset(omega as *mut libc::c_void, 0i32, 64i32 as u64);
    i = 0i32;
    while i < npar {
        let a: u8 = *sigma.offset(i as isize);
        let log_a: u8 = gf256_log[a as usize];
        let mut j: i32 = 0;
        if !(0 == a) {
            j = 0i32;
            while j + 1i32 < 64i32 {
                let b: u8 = *s.offset((j + 1i32) as isize);
                if i + j >= npar {
                    break;
                }
                if !(0 == b) {
                    let ref mut fresh5 = *omega.offset((i + j) as isize);
                    *fresh5 = (*fresh5 as i32
                        ^ gf256_exp[((log_a as i32 + gf256_log[b as usize] as i32)
                        % 255i32) as usize]
                        as i32) as u8
                }
                j += 1
            }
        }
        i += 1
    }
}
/* ***********************************************************************
 * Berlekamp-Massey algorithm for finding error locator polynomials.
 */
fn berlekamp_massey(
    mut s: &[u8; 64],
    mut N: usize,
    mut gf: &GaloisField,
    mut sigma: &mut [u8],
) -> () {
    let mut T: [u8; 64] = [0; 64];
    let mut C: [u8; 64] = [0; 64];
    let mut B: [u8; 64] = [0; 64];
    let mut L: usize = 0;
    let mut m: usize = 1;
    let mut b: u8 = 1;
    B[0] = 1;
    C[0] = 1;

    for n in 0..N {
        let mut d = s[n];
        let mut mult = 0u8;
        let mut i = 1;

        // Calculate in GF(p):
        // d = s[n] + \Sum_{i=1}^{L} C[i] * s[n - i]
        for i in 1..L {
            if C[i] != 0 && s[n - i] != 0 {

                d ^= gf.exp[(gf.log[C[i]] + gf.log[s[n - i]]) % gf.p];
            }
        }
        // No underflow here: log(x) <= p in GF(p), so p - log(x) >= 0
        // Pre-calculate d * b^-1 in GF(p)
        mult = gf.exp[((gf.p - gf.log[b] + gf.log[d]) % gf.p)];
        if 0 == d {
            m += 1
        } else if L * 2i32 <= n {
            T.copy_from_slice(&C);
            poly_add(&mut C, &B, mult, m, gf);
            B.copy_from_slice(&T);
            L = n + 1 - L;
            b = d;
            m = 1
        } else {
            poly_add(&mut C, &B, mult, m, gf);
            m += 1
        }
    }
    sigma.copy_from_slice(&C);
}
/* ***********************************************************************
 * Polynomial operations
 */
fn poly_add(
    dst: &mut [u8],
    src: &[u8],
    c: u8,
    shift: usize,
    gf: &GaloisField,
) -> () {
    let mut i: i32 = 0;
    let log_c: i32 = gf.log[c] as i32;
    if 0 == c {
        return;
    } else {
        i = 0i32;
        while i < 64i32 {
            let mut p: i32 = i + shift;
            let mut v: u8 = src[i];
            if !(p < 0i32 || p >= 64i32) {
                if 0 != v {
                    let new_val = (dst[p] as i32 ^ gf.exp[((gf.log.offset(v as isize) as i32 + log_c) % (*gf).p)]) as u8;
                    dst[p] = new_val;
                }
            }
            i += 1
        }
    };
}

unsafe extern "C" fn read_data(
    mut code: *const quirc_code,
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
) -> () {
    let mut y: i32 = (*code).size - 1i32;
    let mut x: i32 = (*code).size - 1i32;
    let mut dir: i32 = -1i32;
    while x > 0i32 {
        if x == 6i32 {
            x -= 1
        }
        if 0 == reserved_cell((*data).version, y, x) {
            read_bit(code, data, ds, y, x);
        }
        if 0 == reserved_cell((*data).version, y, x - 1i32) {
            read_bit(code, data, ds, y, x - 1i32);
        }
        y += dir;
        if !(y < 0i32 || y >= (*code).size) {
            continue;
        }
        dir = -dir;
        x -= 2i32;
        y += dir
    }
}

unsafe extern "C" fn read_bit(
    mut code: *const quirc_code,
    mut data: *mut quirc_data,
    mut ds: *mut datastream,
    mut i: i32,
    mut j: i32,
) -> () {
    let mut bitpos: i32 = (*ds).data_bits & 7i32;
    let mut bytepos: i32 = (*ds).data_bits >> 3i32;
    let mut v: i32 = grid_bit(code, j, i);
    if 0 != mask_bit((*data).mask, i, j) {
        v ^= 1i32
    }
    if 0 != v {
        (*ds).raw[bytepos as usize] =
            ((*ds).raw[bytepos as usize] as i32 | 0x80i32 >> bitpos) as u8
    }
    (*ds).data_bits += 1;
}

unsafe extern "C" fn grid_bit(
    mut code: *const quirc_code,
    mut x: i32,
    mut y: i32,
) -> i32 {
    let mut p: i32 = y * (*code).size + x;
    return (*code).cell_bitmap[(p >> 3i32) as usize] as i32 >> (p & 7i32) & 1i32;
}

fn mask_bit(
    mask: i32,
    i: i32,
    j: i32,
) -> i32 {
    match mask {
        0 => return (0 == (i + j) % 2i32) as i32,
        1 => return (0 == i % 2i32) as i32,
        2 => return (0 == j % 3i32) as i32,
        3 => return (0 == (i + j) % 3i32) as i32,
        4 => return (0 == (i / 2i32 + j / 3i32) % 2i32) as i32,
        5 => return (0 == i * j % 2i32 + i * j % 3i32) as i32,
        6 => return (0 == (i * j % 2i32 + i * j % 3i32) % 2i32) as i32,
        7 => return (0 == (i * j % 3i32 + (i + j) % 2i32) % 2i32) as i32,
        _ => return 0i32,
    };
}

unsafe extern "C" fn reserved_cell(
    mut version: i32,
    mut i: i32,
    mut j: i32,
) -> i32 {
    let mut ver: *const VersionInfo =
        &quirc_version_db[version as usize] as *const VersionInfo;
    let mut size: i32 = version * 4i32 + 17i32;
    let mut ai: i32 = -1i32;
    let mut aj: i32 = -1i32;
    let mut a: i32 = 0;
    /* Finder + format: top left */
    if i < 9i32 && j < 9i32 {
        return 1i32;
    } else if i + 8i32 >= size && j < 9i32 {
        return 1i32;
    } else if i < 9i32 && j + 8i32 >= size {
        return 1i32;
    } else if i == 6i32 || j == 6i32 {
        return 1i32;
    } else {
        /* Exclude version info, if it exists. Version info sits adjacent to
         * the top-right and bottom-left finders in three rows, bounded by
         * the timing pattern.
         */
        if version >= 7i32 {
            if i < 6i32 && j + 11i32 >= size {
                return 1i32;
            } else if i + 11i32 >= size && j < 6i32 {
                return 1i32;
            }
        }
        /* Exclude alignment patterns */
        a = 0i32;
        while a < 7i32 && 0 != (*ver).apat[a as usize] {
            let mut p: i32 = (*ver).apat[a as usize];
            if abs(p - i) < 3i32 {
                ai = a
            }
            if abs(p - j) < 3i32 {
                aj = a
            }
            a += 1
        }
        if ai >= 0i32 && aj >= 0i32 {
            a -= 1;
            if ai > 0i32 && ai < a {
                return 1i32;
            } else if aj > 0i32 && aj < a {
                return 1i32;
            } else if aj == a && ai == a {
                return 1i32;
            }
        }
        return 0i32;
    };
}

unsafe extern "C" fn read_format(
    mut code: *const quirc_code,
    mut data: *mut quirc_data,
    mut which: i32,
) -> quirc_decode_error_t {
    let mut i: i32 = 0;
    let mut format: uint16_t = 0i32 as uint16_t;
    let mut fdata: uint16_t = 0;
    let mut err: quirc_decode_error_t = QUIRC_SUCCESS;
    if 0 != which {
        i = 0i32;
        while i < 7i32 {
            format = ((format as i32) << 1i32
                | grid_bit(code, 8i32, (*code).size - 1i32 - i)) as uint16_t;
            i += 1
        }
        i = 0i32;
        while i < 8i32 {
            format = ((format as i32) << 1i32
                | grid_bit(code, (*code).size - 8i32 + i, 8i32)) as uint16_t;
            i += 1
        }
    } else {
        static mut xs: [i32; 15] = unsafe {
            [
                8i32, 8i32, 8i32, 8i32, 8i32, 8i32, 8i32, 8i32, 7i32, 5i32, 4i32, 3i32, 2i32, 1i32,
                0i32,
            ]
        };
        static mut ys: [i32; 15] = unsafe {
            [
                0i32, 1i32, 2i32, 3i32, 4i32, 5i32, 7i32, 8i32, 8i32, 8i32, 8i32, 8i32, 8i32, 8i32,
                8i32,
            ]
        };
        i = 14i32;
        while i >= 0i32 {
            format = ((format as i32) << 1i32
                | grid_bit(code, xs[i as usize], ys[i as usize])) as uint16_t;
            i -= 1
        }
    }
    format = (format as i32 ^ 0x5412i32) as uint16_t;
    err = correct_format(&mut format);
    if 0 != err as u64 {
        return err;
    } else {
        fdata = (format as i32 >> 10i32) as uint16_t;
        (*data).ecc_level = fdata as i32 >> 3i32;
        (*data).mask = fdata as i32 & 7i32;
        return QUIRC_SUCCESS;
    };
}

fn correct_format(mut word: u16) -> DeQRResult<u16> {
    let mut s: [u8; 64] = [0; 64];
    let mut sigma: [u8; 64] = [0; 64];

    // TODO: This needs to somehow halfway modify s

    /* Evaluate U (received codeword) at each of alpha_1 .. alpha_6
     * to get S_1 .. S_6 (but we index them from 0).
     */
    if let Err(_) = format_syndromes(word) {
        berlekamp_massey(&mut s, 6, &gf16, &mut sigma);

        /* Now, find the roots of the polynomial */
        for i in 0..15 {
            if poly_eval(&sigma, gf16_exp[15 - i], &gf16) == 0 {
                word ^= (1 << i);
            }
        }

        // Double check syndromes
        format_syndromes(word)?;
    }
    Ok(word)

}
/* ***********************************************************************
 * Format value error correction
 *
 * Generator polynomial for GF(2^4) is x^4 + x + 1
 */
fn format_syndromes(u: u16) -> DeQRResult<[u8; 64]> {
    let mut result: [u8; 64] = [0; 64];

    for i in 0..6 {
        for j in 0..15usize {
            if u & (1 << j) != 0 {
                result[i] ^= gf16_exp[((i + 1) * j % 15)];
            }
        }
        if result[i] != 0 {
            Err(DeQRError::FORMAT_ECC)?
        }
    }
    return Ok(result);
}


static gf16: GaloisField =
    GaloisField {
        p: 15,
        log: &gf16_log,
        exp: &gf16_exp,
    };
static gf16_exp: [u8; 16] =
    [
        0x1_u8,
        0x2_u8,
        0x4_u8,
        0x8_u8,
        0x3_u8,
        0x6_u8,
        0xc_u8,
        0xb_u8,
        0x5_u8,
        0xa_u8,
        0x7_u8,
        0xe_u8,
        0xf_u8,
        0xd_u8,
        0x9_u8,
        0x1_u8,
    ];
static gf16_log: [u8; 16] =
    [
        0_u8,
        0xf_u8,
        0x1_u8,
        0x4_u8,
        0x2_u8,
        0x8_u8,
        0x5_u8,
        0xa_u8,
        0x3_u8,
        0xe_u8,
        0x9_u8,
        0x7_u8,
        0x6_u8,
        0xd_u8,
        0xb_u8,
        0xc_u8,
    ];
