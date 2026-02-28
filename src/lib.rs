use wasm_bindgen::prelude::*;

const CHANCE: f32 = 0.2;

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
    pixels: Vec<u8>,
}

struct Rng(u64); // xorshift64 pseudo-rng

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 1 } else { seed })
    }

    fn next_f32(&mut self) -> f32 {
        let mut s = self.0;
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        self.0 = s;
        // normalize to [0, 1]
        (s as u32 as f32) / (u32::MAX as f32)
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
}

#[wasm_bindgen]
impl Universe {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Universe {
        let size = (width * height) as usize;
        let seed = (random() * u64::MAX as f64) as u64;
        let mut rng = Rng::new(seed);

        let cells: Vec<Cell> = (0..size)
            .map(|_| {
                if rng.next_f32() < CHANCE { Cell::Alive } else { Cell::Dead }
            })
            .collect();

        let scratch = vec![Cell::Dead; size];

        Universe { width, height, cells, scratch, pixels: Vec::new() }
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
                        if n == 2 || n == 3 { Cell::Alive } else { Cell::Dimming }
                    }
                    Cell::Dimming => Cell::Fading,
                    _ => { // dimming or dead
                        if n == 3 { Cell::Alive } else { Cell::Dead }
                    }
                };
            }
        }

        std::mem::swap(&mut self.cells, &mut self.scratch);
    }

    pub fn render(&mut self, cw: u32, ch: u32, cell_size: u32, c_alive: u32, c_dimming: u32, c_fading: u32) -> *const u8 {
        let px_count = (cw * ch * 4) as usize;
        self.pixels.resize(px_count, 0);
        self.pixels.fill(0);
        
        let gap = cell_size - 1;

        for cy in 0..self.height {
            let cell_row = (cy * self.width) as usize;
            let py_base = cy * cell_size;

            for cx in 0..self.width {
                let state = self.cells[cell_row + cx as usize];
                if state == Cell::Dead {
                    continue;
                }

                let rgba = match state {
                    Cell::Alive => c_alive,
                    Cell::Dimming => c_dimming,
                    Cell::Fading => c_fading,
                    _ => continue,
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

