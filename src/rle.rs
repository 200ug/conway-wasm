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
    pub _name: &'static str, // just for in-code docs
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
        _name: "pulsar",
        rle: "2b3o3b3o2b$13b$o4bobo4bo$o4bobo4bo$o4bobo4bo$2b3o3b3o2b$13b$2b3o3b3o2b$o4bobo4bo$o4bobo4bo$o4bobo4bo$13b$2b3o3b3o!",
        weight: 4,
        easter: false,
    },
    PatternEntry {
        _name: "gosper-glider-gun",
        rle: "24bo11b$22bobo11b$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o14b$2o8bo3bob2o4bobo11b$10bo5bo7bo11b$11bo3bo20b$12b2o!",
        weight: 1,
        easter: true,
    },
];

pub fn normal_patterns() -> Vec<&'static PatternEntry> {
    PATTERNS.iter().filter(|p| !p.easter).collect()
}

pub fn easter_patterns() -> Vec<&'static PatternEntry> {
    PATTERNS.iter().filter(|p| p.easter).collect()
}

/// pick a random normal pattern using weights.
/// 'r' should be a value in range [0, 1).
pub fn pick_weighted(r: f32) -> &'static PatternEntry {
    let normals = normal_patterns();
    let total: u32 = normals.iter().map(|p| p.weight).sum();
    let mut threshold = r * total as f32;

    for p in &normals {
        threshold -= p.weight as f32;
        if threshold <= 0.0 {
            return p;
        }
    }

    normals.last().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_patterns_parse() {
        for entry in PATTERNS {
            let s = Stamp::from_rle(entry.rle);
            assert!(s.width > 0 && s.height > 0, "failed to parse: {}", entry._name);
        }
    }
}

