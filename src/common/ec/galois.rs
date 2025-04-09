use std::{
    fmt,
    ops::{Add, AddAssign, Div, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct G(pub u8);

impl From<u8> for G {
    fn from(value: u8) -> Self {
        G(value)
    }
}

impl From<G> for u8 {
    fn from(g: G) -> Self {
        g.0
    }
}

impl fmt::Display for G {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for G {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

impl AddAssign for G {
    fn add_assign(&mut self, other: Self) {
        self.0 ^= other.0;
    }
}

impl Sub for G {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

impl SubAssign for G {
    fn sub_assign(&mut self, other: Self) {
        self.0 ^= other.0;
    }
}

impl Mul<Self> for G {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        if self.0 == 0 || rhs.0 == 0 {
            return Self(0);
        }

        let log_l = LOG_TABLE[self.0 as usize] as usize;
        let log_r = LOG_TABLE[rhs.0 as usize] as usize;

        let mut log_sum = log_l + log_r;
        if log_sum >= 255 {
            log_sum -= 255;
        }

        Self(EXP_TABLE[log_sum])
    }
}

impl MulAssign for G {
    fn mul_assign(&mut self, rhs: Self) {
        if self.0 == 0 || rhs.0 == 0 {
            self.0 = 0;
            return;
        }

        let log_l = LOG_TABLE[self.0 as usize] as usize;
        let log_r = LOG_TABLE[rhs.0 as usize] as usize;

        let mut log_sum = log_l + log_r;
        if log_sum >= 255 {
            log_sum -= 255;
        }

        self.0 = EXP_TABLE[log_sum];
    }
}

impl Div<Self> for G {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        assert!(rhs.0 != 0, "Division by zero in GF(256)");
        if self.0 == 0 {
            return Self(0);
        }
        let log_l = LOG_TABLE[self.0 as usize] as usize;
        let log_r = LOG_TABLE[rhs.0 as usize] as usize;
        let log_sum = if log_l < log_r { 255 + log_l - log_r } else { log_l - log_r };
        Self(EXP_TABLE[log_sum])
    }
}

impl G {
    pub fn gen_pow(p: usize) -> G {
        debug_assert!(p < 256, "Generator power must be less than 256: Power {p}");
        Self(EXP_TABLE[p])
    }
}

// Global constants
//------------------------------------------------------------------------------

static LOG_TABLE: &[u8] = b"\
\xff\x00\x01\x19\x02\x32\x1a\xc6\x03\xdf\x33\xee\x1b\x68\xc7\x4b\
\x04\x64\xe0\x0e\x34\x8d\xef\x81\x1c\xc1\x69\xf8\xc8\x08\x4c\x71\
\x05\x8a\x65\x2f\xe1\x24\x0f\x21\x35\x93\x8e\xda\xf0\x12\x82\x45\
\x1d\xb5\xc2\x7d\x6a\x27\xf9\xb9\xc9\x9a\x09\x78\x4d\xe4\x72\xa6\
\x06\xbf\x8b\x62\x66\xdd\x30\xfd\xe2\x98\x25\xb3\x10\x91\x22\x88\
\x36\xd0\x94\xce\x8f\x96\xdb\xbd\xf1\xd2\x13\x5c\x83\x38\x46\x40\
\x1e\x42\xb6\xa3\xc3\x48\x7e\x6e\x6b\x3a\x28\x54\xfa\x85\xba\x3d\
\xca\x5e\x9b\x9f\x0a\x15\x79\x2b\x4e\xd4\xe5\xac\x73\xf3\xa7\x57\
\x07\x70\xc0\xf7\x8c\x80\x63\x0d\x67\x4a\xde\xed\x31\xc5\xfe\x18\
\xe3\xa5\x99\x77\x26\xb8\xb4\x7c\x11\x44\x92\xd9\x23\x20\x89\x2e\
\x37\x3f\xd1\x5b\x95\xbc\xcf\xcd\x90\x87\x97\xb2\xdc\xfc\xbe\x61\
\xf2\x56\xd3\xab\x14\x2a\x5d\x9e\x84\x3c\x39\x53\x47\x6d\x41\xa2\
\x1f\x2d\x43\xd8\xb7\x7b\xa4\x76\xc4\x17\x49\xec\x7f\x0c\x6f\xf6\
\x6c\xa1\x3b\x52\x29\x9d\x55\xaa\xfb\x60\x86\xb1\xbb\xcc\x3e\x5a\
\xcb\x59\x5f\xb0\x9c\xa9\xa0\x51\x0b\xf5\x16\xeb\x7a\x75\x2c\xd7\
\x4f\xae\xd5\xe9\xe6\xe7\xad\xe8\x74\xd6\xf4\xea\xa8\x50\x58\xaf";

static EXP_TABLE: &[u8] = b"\
\x01\x02\x04\x08\x10\x20\x40\x80\x1d\x3a\x74\xe8\xcd\x87\x13\x26\
\x4c\x98\x2d\x5a\xb4\x75\xea\xc9\x8f\x03\x06\x0c\x18\x30\x60\xc0\
\x9d\x27\x4e\x9c\x25\x4a\x94\x35\x6a\xd4\xb5\x77\xee\xc1\x9f\x23\
\x46\x8c\x05\x0a\x14\x28\x50\xa0\x5d\xba\x69\xd2\xb9\x6f\xde\xa1\
\x5f\xbe\x61\xc2\x99\x2f\x5e\xbc\x65\xca\x89\x0f\x1e\x3c\x78\xf0\
\xfd\xe7\xd3\xbb\x6b\xd6\xb1\x7f\xfe\xe1\xdf\xa3\x5b\xb6\x71\xe2\
\xd9\xaf\x43\x86\x11\x22\x44\x88\x0d\x1a\x34\x68\xd0\xbd\x67\xce\
\x81\x1f\x3e\x7c\xf8\xed\xc7\x93\x3b\x76\xec\xc5\x97\x33\x66\xcc\
\x85\x17\x2e\x5c\xb8\x6d\xda\xa9\x4f\x9e\x21\x42\x84\x15\x2a\x54\
\xa8\x4d\x9a\x29\x52\xa4\x55\xaa\x49\x92\x39\x72\xe4\xd5\xb7\x73\
\xe6\xd1\xbf\x63\xc6\x91\x3f\x7e\xfc\xe5\xd7\xb3\x7b\xf6\xf1\xff\
\xe3\xdb\xab\x4b\x96\x31\x62\xc4\x95\x37\x6e\xdc\xa5\x57\xae\x41\
\x82\x19\x32\x64\xc8\x8d\x07\x0e\x1c\x38\x70\xe0\xdd\xa7\x53\xa6\
\x51\xa2\x59\xb2\x79\xf2\xf9\xef\xc3\x9b\x2b\x56\xac\x45\x8a\x09\
\x12\x24\x48\x90\x3d\x7a\xf4\xf5\xf7\xf3\xfb\xeb\xcb\x8b\x0b\x16\
\x2c\x58\xb0\x7d\xfa\xe9\xcf\x83\x1b\x36\x6c\xd8\xad\x47\x8e\x01";
