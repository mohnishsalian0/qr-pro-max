#[cfg(test)]
mod qr_proptests {

    use prop::string::string_regex;
    use proptest::prelude::*;

    use qr_pro_max::*;

    pub fn ec_level_strategy() -> BoxedStrategy<ECLevel> {
        prop_oneof![Just(ECLevel::L), Just(ECLevel::M), Just(ECLevel::Q), Just(ECLevel::H)].boxed()
    }

    pub fn palette_strategy() -> BoxedStrategy<Palette> {
        prop_oneof![Just(Palette::Mono), Just(Palette::Poly)].boxed()
    }

    pub fn qr_strategy(regex: String) -> impl Strategy<Value = (ECLevel, Palette, String)> {
        (ec_level_strategy(), palette_strategy()).prop_flat_map(move |(ecl, pal)| {
            let mut max_sz = match ecl {
                ECLevel::L => 2956,
                ECLevel::M => 2334,
                ECLevel::Q => 1656,
                ECLevel::H => 1276,
            };
            if matches!(pal, Palette::Poly) {
                max_sz *= 3;
            }
            let pattern = format!(r"{}{{1,{}}}", regex, max_sz);
            string_regex(&pattern).unwrap().prop_map(move |data| (ecl, pal, data))
        })
    }

    proptest! {
        #[test]
        fn proptest_numeric(params in qr_strategy("[0-9]".to_string())) {
            let (ecl, pal, data) = params;
            let qr = QRBuilder::new(data.as_bytes()).ec_level(ecl).palette(pal).build().unwrap();
            let ver = qr.version();
            let decoded = QRReader::read(&qr, ver).unwrap();
            prop_assert_eq!(data, decoded);
        }

        #[test]
        fn proptest_alphanumeric(params in qr_strategy(r"[0-9A-Z $%*+\-./:]".to_string())) {
            let (ecl, pal, data) = params;
            let qr = QRBuilder::new(data.as_bytes()).ec_level(ecl).palette(pal).build().unwrap();
            let ver = qr.version();
            let decoded = QRReader::read(&qr, ver).unwrap();
            prop_assert_eq!(data, decoded);
        }
    }
}

#[cfg(test)]
mod qr_tests {
    use std::path::Path;

    use qr_pro_max::{ECLevel, Palette, QRBuilder, QRReader};

    #[test]
    fn test_qr() {
        let data = "A0  aaa a 0   a00 a A AAaaAk?TyXn_Er\"i%u$<P&?(Aed*:{iZML$_OA0ne/6u{R VfEKw\\fJ-&M6mD+NM\"4z\\.$5?{1/iE+/.@x#{i%m'E$%M@=${.wgiTM&5U\"M;MHL|=KI/3X\\h{T'f<rJ]*jo7{YErEMlvR.,:_'oa&62JnAWHZd&'DK/R!:^*2BT`:ISW&HBuWm1'dT/=%N#6oLFi=fW:5$wr`{!1Pr*3:.9T{+J:2<w@'?U*2&%/NGn'vd=\\` ?XP8=T*D 9'rLpm(=-Td>*W3>\\%-}a=php:t`AW`?)/'?6*?u(-KG:K{)2l6*oBC?6Hm%?BJf(u=i+8<:H~u<=~0'YNNag<(tJ?E7>$b\\2*=?{i{oT8+p<\"+^//I\"n\"X:=H.e=4&~1'b|<\"|41aR-$hh/-Z*%`l\\`>*cp1wMO]6FJ{%S<,==_G(qH8&29\"`X~dj".to_string();
        let ecl = ECLevel::M;
        let pal = Palette::Poly;
        let qr = QRBuilder::new(data.as_bytes()).ec_level(ecl).palette(pal).build().unwrap();

        // let ver = qr.version();
        // let decoded = QRReader::read(&qr, ver).unwrap();
        // assert_eq!(data, decoded);

        let path = Path::new("assets/output.png");

        let image = qr.render_color(10);
        image.save(path).expect("Failed to save image");

        let img = image::open(path).unwrap().to_luma8();
        let mut img = rqrr::PreparedImage::prepare(img);
        let grids = img.detect_grids();
        assert_eq!(grids.len(), 1);
        let (meta, content) = grids[0].decode().unwrap();
        println!("Meta: {meta:?}");
        assert_eq!(data, content);
    }

    #[test]
    fn test_qr_1() {
        let data = "aA00000298542515764186657331886415260738886433510273480049978764987230758543376676257538587037428591120694472658573041637".to_string();
        let ecl = ECLevel::L;
        let pal = Palette::Mono;
        let qr = QRBuilder::new(data.as_bytes()).ec_level(ecl).palette(pal).build().unwrap();

        let ver = qr.version();
        let decoded = QRReader::read(&qr, ver).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_qr_2() {
        let data =
            "aA000000000000000000000000000000000000000000000000000000000000000000000".to_string();
        let ecl = ECLevel::L;
        let pal = Palette::Mono;
        let qr = QRBuilder::new(data.as_bytes()).ec_level(ecl).palette(pal).build().unwrap();

        let ver = qr.version();
        let decoded = QRReader::read(&qr, ver).unwrap();
        assert_eq!(data, decoded);
    }
}
