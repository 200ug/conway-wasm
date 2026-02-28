/*
    rle format reference: https://conwaylife.com/wiki/Run_Length_Encoded

    - 'b'   dead cell
    - 'o'   alive cell
    - '$'   end of row
    - '!'   pattern terminator

    - digits before any of the above = repeat count
*/

#[derive(Clone)]
pub struct Stamp {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<bool>, // row-major, len = w * h
}

impl Stamp {
    pub fn from_rle(rle: &str) -> Self {
        let body: String = rle
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('x'))
            .collect();
        
        let mut rows: Vec<Vec<bool>> = Vec::new();
        let mut cur_row: Vec<bool> = Vec::new();
        let mut run: u32 = 0;

        for ch in body.chars() {
            match ch {
                '0'..='9' => {
                    run = run * 10 + ch.to_digit(10).unwrap();
                }
                'b' => {
                    let count = if run == 0 { 1 } else { run };
                    for _ in 0..count {
                        cur_row.push(false);
                    }
                    run = 0;
                }
                'o' => {
                    let count = if run == 0 { 1 } else { run };
                    for _ in 0..count {
                        cur_row.push(true);
                    }
                    run = 0;
                }
                '$' => {
                    let count = if run == 0 { 1 } else { run };
                    rows.push(std::mem::take(&mut cur_row));
                    for _ in 1..count {
                        rows.push(Vec::new());
                    }
                    run = 0;
                }
                '!' => break,
                _ => {} // whitespace, etc.
            }
        }

        // flush last row if not terminated by '$' before '!' (usually not)
        if !cur_row.is_empty() {
            rows.push(cur_row);
        }

        if rows.is_empty() {
            return Self { width: 0, height: 0, cells: Vec::new() }
        }

        let width = rows.iter().map(|r| r.len()).max().unwrap_or(0) as u32;
        let height = rows.len() as u32;

        // pad rows to width and flatten to 1d
        let mut cells = Vec::with_capacity((width * height) as usize);
        for row in &rows {
            cells.extend_from_slice(row);
            cells.resize(cells.len() + (width as usize - row.len()), false);
        }

        Self { width, height, cells }
    }

    /// return new stamp rotated 90 degrees clockwise
    pub fn rotate_cw(&self) -> Self {
        let nw = self.height;
        let nh = self.width;
        let mut cells = vec![false; (nw * nh) as usize];

        for y in 0..self.height {
            for x in 0..self.width {
                let src = (y * self.width + x) as usize;
                // (x, y) -> (heigh - 1 - y, x)
                let dst = (x * nw + (self.height - 1 - y)) as usize;
                cells[dst] = self.cells[src];
            }
        }

        Self { width: nw, height: nh, cells }
    }

    /// return new stamp mirrored horizontally
    pub fn mirror_h(&self) -> Self {
        let mut cells = self.cells.clone();

        for y in 0..self.height {
            let start = (y * self.width) as usize;
            let end = start + self.width as usize;
            cells[start..end].reverse();
        }

        Self { width: self.width, height: self.height, cells }
    }

    /// return new stamp randomly transformed (rotation + optional mirror).
    /// 'r' should be a value in range [0, 8).
    pub fn transform(&self, r: u8) -> Self {
        let mirrored = if r >= 4 { self.mirror_h() } else { self.clone() };
        let rotations = r % 4;
        let mut s = mirrored;

        for _ in 0..rotations {
            s = s.rotate_cw();
        }

        s
    }
}

pub struct PatternEntry {
    pub _name: &'static str, // just for documenting tests
    pub rle: &'static str,
    pub weight: u32, // relative spawn weight
    pub easter: bool, // pattern limited to easter eggs only
}

pub static PATTERNS: &[PatternEntry] = &[
    PatternEntry {
        _name: "glider",
        rle: "bo$2bo$3o!",
        weight: 10,
        easter: false,
    },
    PatternEntry {
        _name: "10-cell infinite growth",
        rle: "6bob$4bob2o$4bobob$4bo3b$2bo5b$obo!",
        weight: 2,
        easter: false,
    },
    PatternEntry {
        _name: "gourmet",
        rle: "10b2o$10bo$2bob2ob2obo3b2o$2b2obobobo4bo$8bo8bo$16b2o2$16b2o$o10bo3bob
o$3o7b3o3bo$3bo6bobo4b3o$2bobo14bo$2b2o2$2b2o$2bo8bo$5bo4bobobob2o$4b
2o3bob2ob2obo$9bo$8b2o!",
        weight: 2,
        easter: false,
    },
    PatternEntry {
        _name: "infinite hotel glider",
        rle: "13b2o551b$13b2o551b6$13b3o550b2$13bobo550b$12b5o549b$11b2o3b2o548b$11b
2o3b2o548b4$2bo6bo556b$b2o6b2o555b$3o6b3o554b$b2o6b2o555b$2bo6bo556b
165$301bo264b2$297bo7bo260b$291b3o2bo8bo260b$291b3o2bo3b2o4bo259b$289b
o2b2o2bo6bobo260b$289bobo10b2obo260b$289b3o274b$280b2o18bo265b$280b2o
284b$316bo249b$314b2ob2o247b2$314b2o250b$314bo251b$316bo3bo245b$272b2o
38bo3bo249b$272b2o292b$306bo259b$305bobo10bo247b$305bobo7bobo248b$304b
o2bo6bo251b$304bobo8b2o249b$303bo11bobo248b$264b2o37b2o10bobo248b$264b
2o38bo9bob2o248b$313b2ob2o9bo238b$316b3o247b$323bo7bo234b$306bo15bo8bo
234b$272b3o30bobo14bo3b2o4bo233b$305bo16bo6bobo234b$270bo34bo22b2obo
234b$269b2o3bo30bo260b$270bobo53bo22b2o215b$273b3o28bo261b$273bo29b3ob
2o38bo3bo214b$301b2ob3o39bo219b$273b2o28bo41bo7bo212b$272b2o75bo4bo
211b$273bo70bo4bobobo212b$272b2o32bo37bo8bo212b$273b2o30bob2o35bob2o4b
o213b$283bo20b2obob2o33b3ob2obo214b$276bo3bo2bobo2bo19bo36b2ob3o215b$
276b5ob2o4bo21bob2o4b2o12b2o232b$277bobo4bo3bo25bob2o8b2o4b2o232b$311b
4obob3o5b2o238b$284b3o28bo250b$285bo280b2$370bo195b$368b2obo194b$324b
2o41bo198b$318b2o4b2o40bo2bo3bo192b$318b2o45bo9bo190b$240b2o56b3o64b2o
2b2o4bo190b$239b2o124b2o199b$241bo21b2o31bo70bo5bo192b$262bobo30b2o3bo
64b3o4bo193b$252b2o7bo13bobo18bobo66b2o4bo3b2o189b$252b2o7bo2bo6b2o2bo
2bo20b3o14b2o48b4o196b$261bo8bobo5b2o19bo10b2o4b2o39bobo13bo3bo188b$
262bobo3bo3bo3bo3b2o4b2o22b2o45b3o12bo193b$263b2o2b2ob3o5b2o6b2o11b2o
57b2o11bo7bo186b$270b2o3bo2bo19b2o75bo4bo185b$275bobo21bo70bo4bobobo
186b$298b2o56bo13bo8bo186b$299b2o55bobo11bob2o4bo187b$261bo94b3o11b3ob
2obo188b$261bobo38b2o51bo15b2ob3o189b$238bo22b2o39b2o21bobo27b5o206b$
236bobo85b2ob2o12bo224b$227bo7bobo86b2obo12bobo223b$226b2o6bo2bo11b2o
72bo17bo224b$225b2o4b2o2bobo11b2o73bo18bo222b$215b2o7b3o4b2o3bobo15bo
69bo18b2o221b$215b2o8b2o4b2o5bo14bo87bo2bo221b$226b2o25b3o85b3o222b$
227bo107bo6bo223b$239bo94b3o229b$234b4o2bo9b4o26b4o84b2o196b$236bob3o
8bo3bo25bo3bo50b3o29b2o2bo5bo189b$237bo15bo29bo5b3o39b2ob2o30bo2b3o4bo
189b$234bo14bo2bo26bo2bo6bo43bo32bo4bo4bo189b$237bo52bo76b5o194b$234bo
134b2o195b2$227bo338b$227b2o337b$226bobo337b$370b2o194b$214b2o27bo126b
2o194b$213bobo26b2o322b$203b2o7bo6b2o4bo16bobo321b$203b2o7bo2bo2bo2bo
2bobo26bo312b$212bo6b3obob2o25bobo311b$213bobo6b2ob2o14bo8b2o3bo9b2o
84bobo212b$214b2o7bob2o14b4o5b2o3bo9b2o83b2ob2o211b$224bobo15b4o4b2o3b
o94b2obo8b2o202b$225bo16bo2bo6bobo94bo12b2o202b$242b4o7bo72b2o22bo215b
$241b4o80bobo22bo215b$241bo82b3o239b$324b2o240b$324b2o240b$325bobo238b
$326bo27b2o210b$354b2o210b2$323b2o3b2o236b$323b2o3b2o236b2$319b3o3b3o
238b$319bo5b3o238b$203b2o115bo5bo239b$203b2o113bo247b$318b2o246b$317bo
bo246b3$208bo114b3o240b$208bo114b3o240b$195b2o12bo112bo3bo239b$195b2o
8bob2o101b2o254b$204b2ob2o102b2o8b2o3b2o238b$205bobo102bo255b5$187b2o
114bo262b$187b2o114b2o261b$302bobo261b2$290b2o31b2o241b$289bobo31b2o
241b$188b2o89b2o7bo6b2o4bo264b$187b5o87b2o7bo2bo2bo2bo2bobo263b$182bo
4bo4bo32bo62bo6b3obob2o263b$182bo4b3o2bo30b2ob2o61bobo6b2ob2o10b2o251b
$182bo5bo2b2o29b3o65b2o7bob2o10b2o251b$189b2o109bobo263b$222b3o76bo
264b$216bo6bo342b$215b3o348b$214bo2bo348b$214b2o18bo331b$215bo18bo331b
$217bo17bo330b$216bobo12bob2o331b$217bo12b2ob2o331b$199b5o27bobo21b2o
309b$182b3ob2o15bo51b2o309b$181bob2ob3o11b3o363b$180bo4b2obo11bobo55b
2o306b$179bo8bo13bo56b2o305b$179bobobo4bo70bo306b$178bo4bo75b2o305b$
179bo7bo11b2o57b2o306b$186bo12b3o45b2o317b$181bo3bo13bobo39b2o4b2o10bo
306b$189b4o48b2o14b3o306b$182b2o3bo4b2o66bobo303b$186bo4b3o64bo3b2o83b
o218b$185bo5bo70bo85bo217b$192b2o152b3o217b$183bo4b2o2b2o64b3o305b$
183bo9bo45b2o325b$185bo3bo2bo40b2o4b2o325b$191bo41b2o331b$187bob2o375b
$188bo377b2$273bo292b$243bo28b3o291b$231b2o5b3obob4o318b$225b2o4b2o8b
2obo25bo3bo4bobo284b$225b2o12b2o4b2obo21bo4b2ob5o283b$208b3ob2o36bo19b
o2bobo2bo3bo283b$207bob2ob3o33b2obob2o20bo290b$206bo4b2obo35b2obo30b2o
280b$205bo8bo37bo32b2o279b$205bobobo4bo70bo280b$204bo4bo75b2o279b$205b
o7bo41bo28b2o280b$212bo39b3ob2o308b$207bo3bo38b2ob3o29bo280b$254bo28b
3o280b$208b2o22bo53bobo277b$253bo30bo3b2o276b$227bob2o22bo34bo277b$
227bobo6bo16bo312b$226bo4b2o3bo14bobo30b3o279b$227bo8bo15bo313b$227bo
7bo330b$240b3o323b$231bo9b2ob2o320b$241b2obo9bo38b2o271b$241bobo10b2o
37b2o271b$241bobo11bo310b$242b2o8bobo311b$244bo6bo2bo311b$241bobo7bobo
312b$240bo10bobo312b$252bo313b$285b2o279b$242bo3bo38b2o279b$238bo3bo
323b$244bo321b$243b2o321b2$240b2ob2o321b$242bo323b$277b2o287b$258bo18b
2o287b$267b3o296b$253bob2o10bobo296b$253bobo6bo2b2o2bo296b$252bo4b2o3b
o2b3o298b$253bo8bo2b3o298b$253bo7bo304b2$257bo308b144$558bo4bo2b$556b
2ob4ob2o$552bo5bo4bo2b$551bo14b$551bo14b3$549b2o3b2o10b$550b5o11b$551b
3o12b$552bo13b8$552b2o12b$552b2o!",
        weight: 10,
        easter: true,
    },
];

/// pick a random pattern (normal by default, easter if 'easter') using weights.
/// 'r' should be a value in range [0, 1).
pub fn pick_weighted(r: f32, easter: bool) -> Option<&'static PatternEntry> {
    let pool: Vec<_> = PATTERNS.iter().filter(|p| p.easter == easter).collect();

    if pool.is_empty() {
        return None;
    }

    let total: u32 = pool.iter().map(|p| p.weight).sum();
    let mut threshold = r * total as f32;

    for p in &pool {
        threshold -= p.weight as f32;
        if threshold <= 0.0 {
            return Some(p);
        }
    }

    Some(pool.last().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_glider() {
        let s = Stamp::from_rle("bo$2bo$3o!");
        assert_eq!(s.width, 3);
        assert_eq!(s.height, 3);
        /*
            .#.
            ..#
            ###
        */
        let expected = vec![
            false, true, false,
            false, false, true,
            true, true, true,
        ];
        assert_eq!(s.cells, expected);
    }

    #[test]
    fn parse_with_headers() {
        let rle = "\
            #N Glider\n\
            #O Richard K. Guy\n\
            x = 3, y = 3, rule = B3/S23\n\
            bo$2bo$3o!\n";
        let s = Stamp::from_rle(rle);
        assert_eq!(s.width, 3);
        assert_eq!(s.height, 3);
    }

    #[test]
    fn rotation_preserves_area() {
        let s = Stamp::from_rle("3o$o2b$o!");
        let r = s.rotate_cw();
        assert_eq!(r.width, s.height);
        assert_eq!(r.height, s.width);
        assert_eq!(r.cells.len(), s.cells.len());
    }

    #[test]
    fn all_patterns_parse() {
        for entry in PATTERNS {
            let s = Stamp::from_rle(entry.rle);
            assert!(s.width > 0 && s.height > 0, "failed to parse: {}", entry._name);
        }
    }
}

