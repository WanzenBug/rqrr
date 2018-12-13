use std::ops::{Add, Mul, Div, Sub};
use std::ops::AddAssign;
use std::ops::MulAssign;
use std::ops::DivAssign;
use std::ops::SubAssign;

pub trait GaloisField: Add<Output=Self>
+ AddAssign
+ Mul<Output=Self>
+ MulAssign
+ Div<Output=Self>
+ DivAssign
+ Sub<Output=Self>
+ SubAssign
+ Sized
+ Copy
+ Eq
+ PartialEq {
    fn pow(x: usize) -> Self;
    const ZERO: Self;
    const ONE: Self;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GF16(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GF256(pub u8);

impl GaloisField for GF16 {
    fn pow(x: usize) -> Self {
        GF16(GF16_EXP[x % GF16_MUL_ORDER])
    }

    const ZERO: Self = GF16(0);
    const ONE: Self = GF16(1);
}

impl GaloisField for GF256 {
    fn pow(x: usize) -> Self {
        GF256(GF256_EXP[x % GF256_MUL_ORDER])
    }

    const ZERO: Self = GF256(0);
    const ONE: Self = GF256(1);
}

impl Add for GF16 {
    type Output = GF16;

    fn add(self, rhs: GF16) -> <Self as Add<GF16>>::Output {
        GF16(self.0 ^ rhs.0)
    }
}

impl Add for GF256 {
    type Output = GF256;

    fn add(self, rhs: GF256) -> <Self as Add<GF256>>::Output {
        GF256(self.0 ^ rhs.0)
    }
}

impl AddAssign for GF16 {
    fn add_assign(&mut self, rhs: GF16) {
        *self = *self + rhs;
    }
}

impl AddAssign for GF256 {
    fn add_assign(&mut self, rhs: GF256) {
        *self = *self + rhs;
    }
}

impl Sub for GF16 {
    type Output = GF16;

    fn sub(self, rhs: GF16) -> <Self as Sub<GF16>>::Output {
        self + rhs
    }
}

impl Sub for GF256 {
    type Output = GF256;

    fn sub(self, rhs: GF256) -> <Self as Sub<GF256>>::Output {
        self + rhs
    }
}

impl SubAssign for GF16 {
    fn sub_assign(&mut self, rhs: GF16) {
        *self = *self - rhs;
    }
}

impl SubAssign for GF256 {
    fn sub_assign(&mut self, rhs: GF256) {
        *self = *self - rhs;
    }
}

impl Mul for GF16 {
    type Output = GF16;

    fn mul(self, rhs: GF16) -> <Self as Mul<GF16>>::Output {
        if self == GF16::ZERO || rhs == GF16::ZERO {
            return GF16(0);
        }

        let self_log = GF16_LOG[self.0 as usize] as u16;
        let rhs_log = GF16_LOG[rhs.0 as usize] as u16;
        let x = ((self_log + rhs_log) % GF16_MUL_ORDER as u16) as usize;
        GF16(GF16_EXP[x])
    }
}

impl Mul for GF256 {
    type Output = GF256;

    fn mul(self, rhs: GF256) -> <Self as Mul<GF256>>::Output {
        if self == GF256::ZERO || rhs == GF256::ZERO {
            return GF256(0);
        }

        let self_log = GF256_LOG[self.0 as usize] as u16;
        let rhs_log = GF256_LOG[rhs.0 as usize] as u16;
        let x = ((self_log + rhs_log) % GF256_MUL_ORDER as u16) as usize;
        GF256(GF256_EXP[x])
    }
}

impl MulAssign for GF16 {
    fn mul_assign(&mut self, rhs: GF16) {
        *self = *self * rhs;
    }
}

impl MulAssign for GF256 {
    fn mul_assign(&mut self, rhs: GF256) {
        *self = *self * rhs;
    }
}

impl Div for GF16 {
    type Output = GF16;

    fn div(self, rhs: GF16) -> <Self as Div<GF16>>::Output {
        if rhs == GF16::ZERO {
            panic!("Divide by 0 in GF16");
        }

        let self_log = GF16_LOG[self.0 as usize] as u16;
        let rhs_log = GF16_LOG[rhs.0 as usize] as u16;
        let x = ((self_log + GF16_MUL_ORDER as u16 - rhs_log) % GF16_MUL_ORDER as u16) as usize;
        GF16(GF16_EXP[x])
    }
}

impl Div for GF256 {
    type Output = GF256;

    fn div(self, rhs: GF256) -> <Self as Div<GF256>>::Output {
        if rhs == GF256::ZERO {
            panic!("Divide by 0 in GF256");
        }

        let self_log = GF256_LOG[self.0 as usize] as u16;
        let rhs_log = GF256_LOG[rhs.0 as usize] as u16;
        let x = ((self_log + GF256_MUL_ORDER as u16 - rhs_log) % GF256_MUL_ORDER as u16) as usize;
        GF256(GF256_EXP[x])
    }
}

impl DivAssign for GF16 {
    fn div_assign(&mut self, rhs: GF16) {
        *self = *self / rhs;
    }
}

impl DivAssign for GF256 {
    fn div_assign(&mut self, rhs: GF256) {
        *self = *self / rhs;
    }
}


const GF16_MUL_ORDER: usize = 15;
const GF16_EXP: [u8; 16] =
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
const GF16_LOG: [u8; 16] =
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

const GF256_MUL_ORDER: usize = 255;
const GF256_LOG: [u8; 256] = {
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
const GF256_EXP: [u8; 256] = {
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
