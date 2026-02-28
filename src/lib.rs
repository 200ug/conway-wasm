mod rle;

use rle::{Stamp, easter_patterns, pick_weighted};
use wasm_bindgen::prelude::*;

const BG_FILL_CHANCE: f32 = 0.07; // kept low so stamped patterns stand out
const CELLS_PER_PATTERN: u32 = 3500; // approx. number of cells per one stamped pattern
const EASTER_CHANCE: f32 = 0.02; // probability of spawning an easter egg pattern per grid gen.
const STAMP_MARGIN: u32 = 8; // min. clear margin (in cells) around each stamped pattern

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
    Dimming = 2,
    Fading = 3,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    scratch: Vec<Cell>,
    visited: Vec<bool>,
    pixels: Vec<u8>,
}

struct Rng(u64); // xorshift64 pseudo-rng

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 1 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut s = self.0;
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        self.0 = s;
        s
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() as u32 as f32) / (u32::MAX as f32)
    }

    /// uniform u32 in [0, bound)
    fn next_u32(&mut self, bound: u32) -> u32 {
        (self.next_f32() * bound as f32) as u32 % bound
    }

    /// uniform u8 in [0, bound)
    fn next_u8(&mut self, bound: u8) -> u8 {
        (self.next_f32() * bound as f32) as u8 % bound
    }
}

#[wasm_bindgen]
extern "C" {
    // access to Math.Random() for seed generation
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

impl Universe {
    fn live_neighbor_count(&self, x: u32, y: u32) -> u8 {
        let mut count = 0u8;
        let w = self.width;
        let h = self.height;

        for dy in [h - 1, 0, 1] {
            for dx in [w - 1, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = (x + dx) % w;
                let ny = (y + dy) % h;

                if self.cells[(ny * w + nx) as usize] == Cell::Alive {
                    count += 1;
                }
            }
        }

        count
    }

    /// stamp a parsed pattern onto the grid at (ox, oy). wraps around the edges.
    /// if 'clear_magin' is true, dead cells are written in a margin aroudn the 
    /// stamp to give it breathing room.
    fn stamp(&mut self, stamp: &Stamp, ox: u32, oy: u32, clear_margin: bool) {
        let w = self.width;
        let h = self.height;

        if clear_margin {
            let m = STAMP_MARGIN;
            for dy in 0..(stamp.height + 2 * m) {
                for dx in 0..(stamp.width + 2 * m) {
                    let gx = (ox + w - m + dx) % w;
                    let gy = (oy + h - m + dy) % h;
                    self.cells[(gy * w + gx) as usize] = Cell::Dead;
                }
            }
        }

        for sy in 0..stamp.height {
            for sx in 0..stamp.width {
                if stamp.cells[(sy * stamp.width + sx) as usize] {
                    let gx = (ox + sx) % w;
                    let gy = (oy + sy) % h;
                    self.cells[(gy * w + gx) as usize] = Cell::Alive;
                }
            }
        }
    }

    fn mark_visited(&mut self) {
        for i in 0..self.cells.len() {
            if self.cells[i] == Cell::Alive {
                self.visited[i] = true;
            }
        }
    }
}

#[wasm_bindgen]
impl Universe {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let seed = (random() * u64::MAX as f64) as u64;
        let mut rng = Rng::new(seed);

        // sparse random bg
        let cells: Vec<Cell> = (0..size)
            .map(|_| {
                if rng.next_f32() < BG_FILL_CHANCE { Cell::Alive } else { Cell::Dead }
            })
            .collect();

        let scratch = vec![Cell::Dead; size];
        let visited = vec![false; size];
        let mut uni = Universe { width, height, cells, scratch, visited, pixels: Vec::new() };

        // roll for an easter egg
        if rng.next_f32() < EASTER_CHANCE {
            console_log!("hit an easter egg!");
            let eggs = easter_patterns();
            
            if !eggs.is_empty() {
                let idx = rng.next_u32(eggs.len() as u32) as usize;
                let base_stamp = Stamp::from_rle(eggs[idx].rle);
                let stamp = base_stamp.transform(rng.next_u8(8));

                let ox = rng.next_u32(width);
                let oy = rng.next_u32(height);

                uni.stamp(&stamp, ox, oy, true);
            }

            uni.mark_visited();

            return uni;
        }

        // stamp known patterns at random positions
        let num_patterns = (width * height) / CELLS_PER_PATTERN;
        console_log!("stamping {} patterns into universe", num_patterns);
        for _ in 0..num_patterns {
            let Some(entry) = pick_weighted(rng.next_f32()) else { break };
            let base_stamp = Stamp::from_rle(entry.rle);
            let stamp = base_stamp.transform(rng.next_u8(8));

            let ox = rng.next_u32(width);
            let oy = rng.next_u32(height);

            uni.stamp(&stamp, ox, oy, true);
        }

        uni.mark_visited();

        uni
    }

    pub fn tick(&mut self) {
        let w = self.width;
        let h = self.height;

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let cell = self.cells[idx];
                let n = self.live_neighbor_count(x, y);

                self.scratch[idx] = match cell {
                    Cell::Alive => {
                        if n == 2 || n == 3 { Cell::Alive } else { Cell::Dead }
                    }
                    _ => { // fading or dead
                        if n == 3 { Cell::Alive } else { Cell::Dead }
                    }
                };
            }
        }

        std::mem::swap(&mut self.cells, &mut self.scratch);
        self.mark_visited();
    }

    pub fn render(&mut self, cw: u32, ch: u32, cell_size: u32, c_alive: u32, c_visited: u32) -> *const u8 {
        let px_count = (cw * ch * 4) as usize;
        self.pixels.resize(px_count, 0);
        self.pixels.fill(0);
        
        let gap = cell_size - 1;

        for cy in 0..self.height {
            let cell_row = (cy * self.width) as usize;
            let py_base = cy * cell_size;

            for cx in 0..self.width {
                let cell_idx = cell_row + cx as usize;
                let state = self.cells[cell_idx];

                let rgba = if state == Cell::Alive {
                    c_alive
                } else if self.visited[cell_idx] {
                    c_visited
                } else { 
                    // state: dead and never visited
                    continue;
                };

                let r = ((rgba >> 24) & 0xFF) as u8;
                let g = ((rgba >> 16) & 0xFF) as u8;
                let b = ((rgba >> 8) & 0xFF) as u8;
                let a = (rgba & 0xFF) as u8;

                let px_base = cx * cell_size;

                for dy in 0..gap {
                    let row_start = ((py_base + dy) * cw + px_base) as usize * 4;

                    for dx in 0..gap {
                        let i = row_start + dx as usize * 4;
                        self.pixels[i] = r;
                        self.pixels[i + 1] = g;
                        self.pixels[i + 2] = b;
                        self.pixels[i + 3] = a;
                    }
                }
            }
        }

        self.pixels.as_ptr()
    }

    // raw array pointer for js so it can read directly from wasm memory
    pub fn cells_ptr(&self) -> *const Cell {
        self.cells.as_ptr()
    }
}

