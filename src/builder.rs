use std::ops::Deref;

use crate::{
    codec::{encode, encode_with_version},
    ec::{ecc, error_correction_capacity},
    error::{QRError, QRResult},
    mask::{apply_best_mask, MaskPattern},
    metadata::{ECLevel, Palette, Version},
    qr::QR,
};

pub struct QRBuilder<'a> {
    data: &'a [u8],
    version: Option<Version>,
    ec_level: ECLevel,
    palette: Palette,
    mask: Option<MaskPattern>,
}

impl<'a> QRBuilder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, version: None, ec_level: ECLevel::M, palette: Palette::Mono, mask: None }
    }

    pub fn data(&mut self, data: &'a [u8]) -> &mut Self {
        self.data = data;
        self
    }

    pub fn version(&mut self, version: Version) -> &mut Self {
        self.version = Some(version);
        self
    }

    pub fn unset_version(&mut self) -> &mut Self {
        self.version = None;
        self
    }

    pub fn ec_level(&mut self, ec_level: ECLevel) -> &mut Self {
        self.ec_level = ec_level;
        self
    }

    pub fn palette(&mut self, palette: Palette) -> &mut Self {
        self.palette = palette;
        self
    }

    pub fn mask(&mut self, mask: MaskPattern) -> &mut Self {
        self.mask = Some(mask);
        self
    }

    pub fn metadata(&self) -> String {
        match self.version {
            Some(v) => format!(
                "{{ Version: {:?}, Ec level: {:?}, Palette: {:?} }}",
                *v, self.ec_level, self.palette
            ),
            None => format!(
                "{{ Version: None, Ec level: {:?}, Palette: {:?} }}",
                self.ec_level, self.palette
            ),
        }
    }
}

#[cfg(test)]
mod qrbuilder_util_tests {
    use super::QRBuilder;
    use crate::metadata::{ECLevel, Palette, Version};

    #[test]
    fn test_metadata() {
        let data = "Hello, world!".as_bytes();
        let version = Version::Normal(1);
        let ec_level = ECLevel::L;
        let palette = Palette::Mono;
        let mut qr_builder = QRBuilder::new(data);
        qr_builder.version(version).ec_level(ec_level).palette(palette);
        assert_eq!(qr_builder.metadata(), "{ Version: 1, Ec level: L, Palette: Mono }");
        qr_builder.unset_version();
        assert_eq!(qr_builder.metadata(), "{ Version: None, Ec level: L, Palette: Mono }");
    }
}

impl<'a> QRBuilder<'a> {
    pub fn build(&self) -> QRResult<QR> {
        let data_len = self.data.len();

        println!("\nGenerating QR {}...", self.metadata());
        if self.data.is_empty() {
            return Err(QRError::EmptyData);
        }

        // Encode data optimally
        println!("Encoding data...");
        let (encoded_data, encoded_len, version) = match self.version {
            Some(v) => encode_with_version(self.data, self.ec_level, v, self.palette)?,
            None => encode(self.data, self.ec_level, self.palette)?,
        };

        let version_capacity = version.bit_capacity(self.ec_level, self.palette) >> 3;
        let err_corr_cap = error_correction_capacity(version, self.ec_level);

        // Compute error correction codewords
        println!("Computing ecc...");
        let (data_blocks, ecc_blocks) = ecc(&encoded_data, version, self.ec_level);

        // Interleave data and error correction codewords
        println!("Interleaving and chaining data & ecc...");
        let mut payload = Self::interleave(&data_blocks);
        payload.extend(Self::interleave(&ecc_blocks));

        // Construct QR
        println!("Constructing QR...");
        let mut qr = QR::new(version, self.ec_level, self.palette);

        println!("Drawing functional patterns...");
        qr.draw_all_function_patterns();

        println!("Drawing encoding region...");
        qr.draw_encoding_region(&payload);

        let mask = match self.mask {
            Some(m) => {
                println!("Apply mask {m:?}...");
                qr.mask(m);
                m
            }
            None => {
                println!("Finding & applying best mask...");
                apply_best_mask(&mut qr)
            }
        };

        println!("\x1b[1;32mQR generated successfully!\n \x1b[0m");

        let total_modules = version.width() * version.width();
        let dark_modules = qr.count_dark_modules();
        let light_modules = total_modules - dark_modules;

        println!("Report:");
        println!("{}", qr.metadata());
        println!("Data capacity: {}, Error Capacity: {}", version_capacity, err_corr_cap);
        println!(
            "Data size: {}, Encoded size: {}, Compression: {}%",
            data_len,
            encoded_len,
            encoded_len * 100 / data_len
        );
        println!(
            "Dark Cells: {}, Light Cells: {}, Balance: {}\n",
            dark_modules,
            light_modules,
            dark_modules * 100 / total_modules
        );

        Ok(qr)
    }

    pub fn interleave<T: Copy, V: Deref<Target = [T]>>(blocks: &[V]) -> Vec<T> {
        let max_block_size = blocks.iter().map(|b| b.len()).max().expect("Blocks is empty");
        let total_size = blocks.iter().map(|b| b.len()).sum::<usize>();
        let mut res = Vec::with_capacity(total_size);
        for i in 0..max_block_size {
            for b in blocks {
                if i < b.len() {
                    res.push(b[i]);
                }
            }
        }
        res
    }
}

#[cfg(test)]
mod builder_tests {
    use test_case::test_case;

    use crate::{
        builder::QRBuilder,
        metadata::{ECLevel, Version},
    };

    #[test]
    fn test_interleave() {
        let blocks = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9, 0]];
        let interleaved = QRBuilder::interleave(&blocks);
        let exp_interleaved = vec![1, 4, 7, 2, 5, 8, 3, 6, 9, 0];
        assert_eq!(interleaved, exp_interleaved);
    }

    #[test_case("Hello, world!🌎".to_string(), Version::Normal(1), ECLevel::L)]
    #[test_case("TEST".to_string(), Version::Normal(1), ECLevel::M)]
    #[test_case("12345".to_string(), Version::Normal(1), ECLevel::Q)]
    #[test_case("OK".to_string(), Version::Normal(1), ECLevel::H)]
    #[test_case("B3@j🎮#Z%8v🍣K!🔑3zC^8📖&r💾F9*🔐b6🌼".repeat(3).to_string(), Version::Normal(7), ECLevel::L)]
    #[test_case("A11111111111111".repeat(11).to_string(), Version::Normal(7), ECLevel::M)]
    #[test_case("aAAAAAA1111111111111AAAAAAa".repeat(3).to_string(), Version::Normal(7), ECLevel::Q)]
    #[test_case("1234567890".repeat(15).to_string(), Version::Normal(7), ECLevel::H)]
    #[test_case( "B3@j🎮#Z%8v🍣K!🔑3zC^8📖&r💾F9*🔐b6🌼".repeat(4).to_string(), Version::Normal(10), ECLevel::L)]
    #[test_case("A11111111111111".repeat(20).to_string(), Version::Normal(10), ECLevel::M)]
    #[test_case("aAAAAAAAAA1111111111111111AAAAAAAAAAa".repeat(4).to_string(), Version::Normal(10), ECLevel::Q)]
    #[test_case("1234567890".repeat(28).to_string(), Version::Normal(10), ECLevel::H)]
    #[test_case("B3@j🎮#Z%8v🍣K!🔑3zC^8📖&r💾F9*🔐b6🌼".repeat(22).to_string(), Version::Normal(27), ECLevel::L)]
    #[test_case("A111111111111111".repeat(100).to_string(), Version::Normal(27), ECLevel::M)]
    #[test_case("aAAAAAAAAA111111111111111111AAAAAAAAAAa".repeat(20).to_string(), Version::Normal(27), ECLevel::Q)]
    #[test_case("1234567890".repeat(145).to_string(), Version::Normal(27), ECLevel::H)]
    #[test_case("B3@j🎮#Z%8v🍣K!🔑3zC^8📖&r💾F9*🔐b6🌼".repeat(57).to_string(), Version::Normal(40), ECLevel::L)]
    #[test_case("A111111111111111".repeat(97).to_string(), Version::Normal(40), ECLevel::M)]
    #[test_case( "aAAAAAAAAA111111111111111111AAAAAAAAAAa".repeat(42).to_string(), Version::Normal(40), ECLevel::Q)]
    #[test_case("1234567890".repeat(305).to_string(), Version::Normal(40), ECLevel::H)]
    fn test_builder(data: String, version: Version, ec_level: ECLevel) {
        let qr = QRBuilder::new(data.as_bytes())
            .version(version)
            .ec_level(ec_level)
            .build()
            .unwrap()
            .render(10);

        let mut img = rqrr::PreparedImage::prepare(qr);
        let grids = img.detect_grids();
        assert_eq!(grids.len(), 1);
        let (meta, content) = grids[0].decode().unwrap();

        assert_eq!(*version, meta.version.0);
        assert_eq!(data, content);
    }

    #[test]
    #[should_panic]
    fn test_builder_data_overflow() {
        let data = "1234567890".repeat(306).to_string();

        QRBuilder::new(data.as_bytes())
            .version(Version::Normal(40))
            .ec_level(ECLevel::H)
            .build()
            .unwrap()
            .render(10);
    }
}
