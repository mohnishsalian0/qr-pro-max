// TODO: Remember to remove unused_variables & dead_code
#![allow(clippy::items_after_test_module, unused_variables, dead_code)]

use std::error::Error;

use builder::QRBuilder;
use types::{ECLevel, Version};

mod builder;
mod decode;
mod encode;
mod error_correction;
pub mod mask;
pub mod qr;
pub mod types;

fn main() -> Result<(), Box<dyn Error>> {
    let data = "Hello, world! 🌎";
    let version = Version::Normal(3);
    let ec_level = ECLevel::H;

    let qr = QRBuilder::new(data.as_bytes())
        .version(Version::Normal(3))
        .ec_level(ECLevel::H)
        .build()
        .unwrap()
        .render_as_string(1);
    println!("{qr}");

    // let path = "assets/test_image_1.png";
    // let img = image::open(path)?.to_luma8();
    // let mut img = rqrr::PreparedImage::prepare(img);
    // let grids = img.detect_grids();
    // assert_eq!(grids.len(), 1);
    // let (meta, content) = grids[0].decode().unwrap();
    // println!("{content}");

    Ok(())
}
