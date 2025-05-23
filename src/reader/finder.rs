use crate::metadata::Color;

use super::{
    binarize::{BinaryImage, Pixel, Region},
    utils::{
        accumulate::{AllCornerFinder, FirstCornerFinder},
        geometry::{Homography, Point, Slope},
    },
};

// Finder line
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct DatumLine {
    left: u32,
    stone: u32,
    right: u32,
    y: u32,
}

// Finder type
//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Finder {
    pub id: usize,
    pub h: Homography,
    pub corners: [Point; 4],
    pub center: Point,
}

impl Finder {
    #[inline]
    pub fn map(&self, x: f64, y: f64) -> Point {
        self.h.map(x, y)
    }

    #[inline]
    pub fn unmap(&self, p: &Point) -> (f64, f64) {
        self.h.unmap(p)
    }

    pub fn rotate(&mut self, pt: &Point, m: &Slope) {
        let (top_left, _) = self
            .corners
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| (c.y - pt.y) * m.dx - (c.x - pt.x) * m.dy)
            .expect("Corners cannot be empty");
        self.corners.rotate_left(top_left);

        self.h =
            Homography::create(&self.corners, 7.0, 7.0).expect("rotating homography cant fail");
    }
}

// Line scanner to detect finder line
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct LineScanner {
    buffer: [u32; 6],    // Run length of each transition
    prev: Option<Color>, // Last observed color
    transitions: u32,    // Count of color changes
    pos: u32,            // Current position
    y: u32,
}

impl LineScanner {
    pub fn new() -> Self {
        Self { buffer: [0; 6], prev: None, transitions: 0, pos: 0, y: 0 }
    }

    pub fn reset(&mut self, y: u32) {
        self.buffer[5] = 0;
        self.prev = None;
        self.transitions = 0;
        self.pos = 0;
        self.y = y;
    }

    pub fn advance(&mut self, px: Option<Color>) -> Option<DatumLine> {
        self.pos += 1;

        if self.prev == px {
            self.buffer[5] += 1;
            return None;
        }

        self.buffer.rotate_left(1);
        self.buffer[5] = 1;
        self.prev = px;
        self.transitions += 1;

        if self.is_finder_line() {
            Some(DatumLine {
                left: self.pos - self.buffer[..5].iter().sum::<u32>(),
                stone: self.pos - self.buffer[2..5].iter().sum::<u32>(),
                right: self.pos - self.buffer[4],
                y: self.y,
            })
        } else {
            None
        }
    }

    fn is_finder_line(&self) -> bool {
        let white = Color::White;
        if !(self.prev == Some(white) && self.transitions >= 5) {
            return false;
        }

        let avg = self.buffer[..5].iter().sum::<u32>() / 7;
        let tol = avg * 3 / 4;

        let ratio: [u32; 5] = [1, 1, 3, 1, 1];
        for (i, r) in ratio.iter().enumerate() {
            if self.buffer[i] < r * avg - tol || self.buffer[i] > r * avg + tol {
                return false;
            }
        }

        true
    }
}

// Locate finders
//------------------------------------------------------------------------------

pub fn locate_finders(img: &mut BinaryImage) -> Vec<Finder> {
    let mut finders = Vec::with_capacity(100);
    let w = img.w;
    let h = img.h;
    let mut scanner = LineScanner::new();

    for y in 0..h {
        for x in 0..w {
            let color = Color::from(img.get(x, y));
            let datum = match scanner.advance(Some(color)) {
                Some(d) => d,
                None => continue,
            };

            if !is_finder(img, &datum) {
                continue;
            }

            if let Some(f) = construct_finder(img, &datum, finders.len()) {
                finders.push(f);
            }
        }
        scanner.reset(y + 1);
    }

    finders
}

// Sweeps stone and ring regions from datum line and validates finder if:
// Stone area is roughly 37.5% of ring area
// Stone and ring areas arent connected
// Left and right points of row lying inside the ring are connected
fn is_finder(img: &mut BinaryImage, datum: &DatumLine) -> bool {
    let (l, r, s, y) = (datum.left, datum.right, datum.stone, datum.y);
    let ring = img.get_region((r, y));
    let stone = img.get_region((s, y));

    if img.get(l, y) != img.get(r, y) {
        return false;
    }

    if let (
        Some(Region { src: r_src, area: r_area, .. }),
        Some(Region { src: s_src, area: s_area, .. }),
    ) = (ring, stone)
    {
        let ratio = s_area * 100 / r_area;
        let r_color = img.get(r_src.0, r_src.1);
        let s_color = img.get(s_src.0, s_src.1);
        r_color != s_color && (20 < ratio && ratio < 50)
    } else {
        false
    }
}

fn construct_finder(img: &mut BinaryImage, datum: &DatumLine, id: usize) -> Option<Finder> {
    let (_left, right, y) = (datum.left, datum.right, datum.y);
    let color = Color::from(img.get(right, y));
    let refr_pt = Point { x: right as i32, y: y as i32 };

    // Locating first corner
    let fcf = FirstCornerFinder::new(refr_pt);
    let to = Pixel::Temporary(color);
    let fcf = img.fill_and_accumulate((right, y), to, fcf);

    // Locating rest of the corners
    let to = Pixel::Candidate(color);
    let acf = AllCornerFinder::new(refr_pt, fcf.corner);
    let acf = img.fill_and_accumulate((right, y), to, acf);

    // Setting up homographic projection
    let h = Homography::create(&acf.corners, 7.0, 7.0)?;
    let corners = acf.corners;
    let center = h.map(3.5, 3.5);

    Some(Finder { id, h, corners, center })
}

#[cfg(test)]
mod finder_highlight {
    use image::RgbImage;

    use crate::reader::utils::{
        geometry::{BresenhamLine, X, Y},
        Highlight,
    };

    use super::Finder;

    impl Highlight for Finder {
        fn highlight(&self, img: &mut RgbImage) {
            for (i, crn) in self.corners.iter().enumerate() {
                let next = self.corners[(i + 1) % 4];
                let dx = (next.x - crn.x).abs();
                let dy = (next.y - crn.y).abs();
                if dx > dy {
                    let line = BresenhamLine::<X>::new(crn, &next);
                    for pt in line {
                        pt.highlight(img);
                    }
                } else {
                    let line = BresenhamLine::<Y>::new(crn, &next);
                    for pt in line {
                        pt.highlight(img);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod finder_tests {

    use crate::{
        reader::{binarize::BinaryImage, utils::geometry::Point},
        ECLevel, MaskPattern, Palette, QRBuilder, Version,
    };

    use super::locate_finders;

    #[test]
    fn test_locate_finder() {
        let data = "Hello, world!🌎";
        let ver = Version::Normal(4);
        let ecl = ECLevel::L;
        let mask = MaskPattern::new(1);
        let pal = Palette::Mono;

        let qr = QRBuilder::new(data.as_bytes())
            .version(ver)
            .ec_level(ecl)
            .palette(pal)
            .mask(mask)
            .build()
            .unwrap();
        let img = qr.to_image(10);

        let corners = [
            [[40, 40], [109, 40], [109, 109], [40, 109]],
            [[300, 109], [300, 40], [369, 109], [369, 40]],
            [[40, 369], [40, 300], [109, 300], [109, 369]],
        ];
        let centers = [[75, 75], [335, 75], [75, 335]];
        let mut img = BinaryImage::prepare(img);
        let finders = locate_finders(&mut img);
        for (i, f) in finders.iter().enumerate() {
            for crn in corners[i] {
                let pt = Point { x: crn[0], y: crn[1] };
                assert!(f.corners.contains(&pt), "Finder corners don't match");
            }
            let cent_pt = Point { x: centers[i][0], y: centers[i][1] };
            assert_eq!(f.center, cent_pt, "Finder center doesn't match");
        }
    }
}

// Groups finders in 3, which form potential symbols
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical,
    None,
}

impl Orientation {
    pub fn is_none(&self) -> bool {
        matches!(self, Orientation::None)
    }
}

pub struct FinderGroup {
    pub finders: [Finder; 3],
    pub score: f64,
}

pub fn group_finders(finders: &[Finder]) -> Vec<FinderGroup> {
    let mut groups: Vec<FinderGroup> = Vec::new();
    let len = finders.len();

    for (i1, f1) in finders.iter().enumerate() {
        // Indices of best horizontal and vertical neighbor
        let mut ih: Option<usize> = None;
        let mut iv: Option<usize> = None;
        // Equidistance score of 2 finders from datum finder. Lower is better
        let mut best_score = 2.5;

        for (i2, f2) in finders.iter().enumerate() {
            if i2 == i1 {
                continue;
            }

            let (o2, d2) = get_relative_position(f1, f2);
            if o2.is_none() {
                continue;
            }

            for (i3, f3) in finders.iter().enumerate() {
                if i3 == i2 || i3 == i1 {
                    continue;
                }

                let (o3, d3) = get_relative_position(f1, f3);

                match (o2, o3) {
                    (Orientation::Horizontal, Orientation::Vertical) => {
                        let score = (1.0f64 - d2 / d3).abs();
                        if score < best_score {
                            (ih, iv) = (Some(i2), Some(i3));
                            best_score = score;
                        }
                    }
                    (Orientation::Vertical, Orientation::Horizontal) => {
                        let score = (1.0f64 - d2 / d3).abs();
                        if score < best_score {
                            (ih, iv) = (Some(i3), Some(i2));
                            best_score = score;
                        }
                    }
                    _ => (),
                }
            }
        }

        if let (Some(ih), Some(iv)) = (ih, iv) {
            let finders = [finders[iv].clone(), f1.clone(), finders[ih].clone()];
            let score = best_score;
            groups.push(FinderGroup { finders, score });
        }
    }

    groups.sort_unstable_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

    groups
}

// Returns orientation of 2 finders and distance between their centers
fn get_relative_position(f1: &Finder, f2: &Finder) -> (Orientation, f64) {
    let (mut x, mut y) = f1.h.unmap(&f2.center);
    x = (x - 3.5f64).abs();
    y = (y - 3.5f64).abs();

    if y < 0.2f64 * x {
        (Orientation::Horizontal, x)
    } else if x < 0.2f64 * y {
        (Orientation::Vertical, y)
    } else {
        (Orientation::None, 0.0)
    }
}

#[cfg(test)]
mod group_finders_tests {

    use crate::{reader::binarize::BinaryImage, ECLevel, MaskPattern, Palette, QRBuilder, Version};

    use super::{group_finders, locate_finders};

    #[test]
    fn test_group_finder() {
        let data = "Hello, world!🌎";
        let ver = Version::Normal(4);
        let ecl = ECLevel::L;
        let mask = MaskPattern::new(1);
        let pal = Palette::Mono;

        let qr = QRBuilder::new(data.as_bytes())
            .version(ver)
            .ec_level(ecl)
            .palette(pal)
            .mask(mask)
            .build()
            .unwrap();
        let img = qr.to_image(10);

        let centers = [(75, 75), (335, 75), (75, 335)];

        let mut img = BinaryImage::prepare(img);
        let finders = locate_finders(&mut img);
        let group = group_finders(&finders);
        assert_eq!(group.len(), 1, "No group found");
        for f in group[0].finders.iter() {
            let c = (f.center.x, f.center.y);
            assert!(centers.contains(&c))
        }
    }
}
