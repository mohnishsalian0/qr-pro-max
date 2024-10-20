use std::cmp::PartialOrd;
use std::fmt::{Debug, Display, Error, Formatter};
use std::ops::{Deref, Not};

use crate::mask::MaskingPattern;

// Error
//------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum QRError {
    EmptyData,
    DataTooLong,
    CapacityOverflow,
    InvalidVersion,
    InvalidECLevel,
    InvalidPalette,
    InvalidColor,
    InvalidChar,
    InvalidMaskingPattern,
}

impl Display for QRError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let msg = match *self {
            Self::EmptyData => "empty data",
            Self::DataTooLong => "data too long",
            Self::CapacityOverflow => "capacity overflow",
            Self::InvalidVersion => "invalid version",
            Self::InvalidECLevel => "invalid error correction level",
            Self::InvalidPalette => "invalid color palette",
            Self::InvalidColor => "invalid color",
            Self::InvalidChar => "invalid character",
            Self::InvalidMaskingPattern => "invalid masking pattern",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for QRError {}

pub type QRResult<T> = Result<T, QRError>;

// Version
//------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Version {
    Micro(usize),
    Normal(usize),
}

impl Deref for Version {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Micro(v) => v,
            Self::Normal(v) => v,
        }
    }
}

impl Version {
    pub const fn get_width(self) -> usize {
        debug_assert!(
            matches!(self, Self::Micro(1..=4) | Self::Normal(1..=40)),
            "Invalid version"
        );
        match self {
            Self::Micro(v) => v * 2 + 9,
            Self::Normal(v) => v * 4 + 17,
        }
    }

    pub fn get_alignment_pattern(self) -> &'static [i16] {
        debug_assert!(
            matches!(self, Self::Micro(1..=4) | Self::Normal(1..=40)),
            "Invalid version"
        );
        match self {
            Self::Micro(_) => &[],
            Self::Normal(v) => ALIGNMENT_PATTERN_POSITIONS[v - 1],
        }
    }

    pub fn get_version_info(self) -> u32 {
        debug_assert!(matches!(self, Self::Normal(7..=40)), "Invalid version");
        match self {
            Self::Normal(v) => VERSION_INFOS[v - 7],
            _ => unreachable!(),
        }
    }

    pub fn get_mode_len(self) -> usize {
        match self {
            Version::Micro(v) => v - 1,
            Version::Normal(_) => 4,
        }
    }

    pub fn get_bit_capacity(self, ec_level: ECLevel) -> usize {
        match self {
            Version::Micro(v) => VERSION_BIT_CAPACITY[39 + v][ec_level as usize],
            Version::Normal(v) => VERSION_BIT_CAPACITY[v - 1][ec_level as usize],
        }
    }
}

#[cfg(test)]
mod version_tests {
    use super::Version::*;

    #[test]
    #[should_panic(expected = "Invalid version")]
    fn test_width_invalid_micro_version_low() {
        let invalid_version = Micro(0);
        invalid_version.get_alignment_pattern();
    }

    #[test]
    #[should_panic(expected = "Invalid version")]
    fn test_width_invalid_micro_version_high() {
        let invalid_version = Micro(5);
        invalid_version.get_alignment_pattern();
    }

    #[test]
    #[should_panic(expected = "Invalid version")]
    fn test_width_invalid_normal_version_low() {
        let invalid_version = Normal(0);
        invalid_version.get_alignment_pattern();
    }

    #[test]
    #[should_panic(expected = "Invalid version")]
    fn test_width_invalid_normal_version_high() {
        let invalid_version = Normal(41);
        invalid_version.get_alignment_pattern();
    }

    #[test]
    #[should_panic(expected = "Invalid version")]
    fn test_version_info_invalid_version_low() {
        let invalid_version = Normal(0);
        invalid_version.get_alignment_pattern();
    }

    #[test]
    #[should_panic(expected = "Invalid version")]
    fn test_version_info_invalid_version_high() {
        let invalid_version = Normal(41);
        invalid_version.get_alignment_pattern();
    }
}

// Error correction level
//------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum ECLevel {
    L = 0,
    M = 1,
    Q = 2,
    H = 3,
}

// Palette
//------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Palette {
    Monochrome,
    Polychrome(u8),
}

impl Deref for Palette {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Monochrome => &1,
            Self::Polychrome(p) => p,
        }
    }
}

impl Palette {
    pub fn get_palette_info(self) -> u32 {
        debug_assert!(0 < *self && *self < 17, "Invalid palette");

        match self {
            Self::Monochrome => 1,
            Self::Polychrome(p) => PALETTE_INFOS[p as usize],
        }
    }
}

// Color
//------------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Color {
    Light,
    Dark,
    Hue(u32),
}

// TODO: Figure out how to handle hue
impl Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
            Self::Hue(h) => Self::Hue(!h),
        }
    }
}

// TODO: Figure out how to handle hue
impl Color {
    pub fn select<T: Debug>(&self, light: T, dark: T, hue: T) -> T {
        match self {
            Self::Light => light,
            Self::Dark => dark,
            Self::Hue(_) => hue,
        }
    }
}

// Format information
//------------------------------------------------------------------------------

pub fn get_format_info(version: Version, ec_level: ECLevel, mask_pattern: MaskingPattern) -> u32 {
    match version {
        Version::Micro(_) => todo!(),
        Version::Normal(_) => {
            let format_data = ((ec_level as usize) ^ 1) << 3 | (*mask_pattern as usize);
            FORMAT_INFOS_QR[format_data]
        }
    }
}

// Global constants
//------------------------------------------------------------------------------

static ALIGNMENT_PATTERN_POSITIONS: [&[i16]; 40] = [
    &[],
    &[6, 18],
    &[6, 22],
    &[6, 26],
    &[6, 30],
    &[6, 34],
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
];

static VERSION_INFOS: [u32; 34] = [
    0x07c94, 0x085bc, 0x09a99, 0x0a4d3, 0x0bbf6, 0x0c762, 0x0d847, 0x0e60d, 0x0f928, 0x10b78,
    0x1145d, 0x12a17, 0x13532, 0x149a6, 0x15683, 0x168c9, 0x177ec, 0x18ec4, 0x191e1, 0x1afab,
    0x1b08e, 0x1cc1a, 0x1d33f, 0x1ed75, 0x1f250, 0x209d5, 0x216f0, 0x228ba, 0x2379f, 0x24b0b,
    0x2542e, 0x26a64, 0x27541, 0x28c69,
];

static FORMAT_INFOS_QR: [u32; 32] = [
    0x5412, 0x5125, 0x5e7c, 0x5b4b, 0x45f9, 0x40ce, 0x4f97, 0x4aa0, 0x77c4, 0x72f3, 0x7daa, 0x789d,
    0x662f, 0x6318, 0x6c41, 0x6976, 0x1689, 0x13be, 0x1ce7, 0x19d0, 0x0762, 0x0255, 0x0d0c, 0x083b,
    0x355f, 0x3068, 0x3f31, 0x3a06, 0x24b4, 0x2183, 0x2eda, 0x2bed,
];

// TODO: Fill out palette info
static PALETTE_INFOS: [u32; 12] = [0xFFF; 12];

// Bit capacity per error level per version
static VERSION_BIT_CAPACITY: [[usize; 4]; 44] = [
    [152, 128, 104, 72],
    [272, 224, 176, 128],
    [440, 352, 272, 208],
    [640, 512, 384, 288],
    [864, 688, 496, 368],
    [1088, 864, 608, 480],
    [1248, 992, 704, 528],
    [1552, 1232, 880, 688],
    [1856, 1456, 1056, 800],
    [2192, 1728, 1232, 976],
    [2592, 2032, 1440, 1120],
    [2960, 2320, 1648, 1264],
    [3424, 2672, 1952, 1440],
    [3688, 2920, 2088, 1576],
    [4184, 3320, 2360, 1784],
    [4712, 3624, 2600, 2024],
    [5176, 4056, 2936, 2264],
    [5768, 4504, 3176, 2504],
    [6360, 5016, 3560, 2728],
    [6888, 5352, 3880, 3080],
    [7456, 5712, 4096, 3248],
    [8048, 6256, 4544, 3536],
    [8752, 6880, 4912, 3712],
    [9392, 7312, 5312, 4112],
    [10208, 8000, 5744, 4304],
    [10960, 8496, 6032, 4768],
    [11744, 9024, 6464, 5024],
    [12248, 9544, 6968, 5288],
    [13048, 10136, 7288, 5608],
    [13880, 10984, 7880, 5960],
    [14744, 11640, 8264, 6344],
    [15640, 12328, 8920, 6760],
    [16568, 13048, 9368, 7208],
    [17528, 13800, 9848, 7688],
    [18448, 14496, 10288, 7888],
    [19472, 15312, 10832, 8432],
    [20528, 15936, 11408, 8768],
    [21616, 16816, 12016, 9136],
    [22496, 17728, 12656, 9776],
    [23648, 18672, 13328, 10208],
    // Micro versions
    [20, 0, 0, 0],
    [40, 32, 0, 0],
    [84, 68, 0, 0],
    [128, 112, 80, 0],
];
