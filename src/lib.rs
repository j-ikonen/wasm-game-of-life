mod utils;

use fixedbitset::FixedBitSet;
use js_sys::Math;
use wasm_bindgen::prelude::*;

extern crate web_sys;
use web_sys::console;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )*).into());
    };
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: [FixedBitSet;2],
    now: usize,
    d_alive: Vec<u32>,
    d_dead: Vec<u32>,
}

impl Universe {
    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 { self.height - 1 } else { row - 1 };

        let south = if row == self.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[self.now][nw] as u8;

        let n = self.get_index(north, column);
        count += self.cells[self.now][n] as u8;

        let ne = self.get_index(north, east);
        count += self.cells[self.now][ne] as u8;

        let w = self.get_index(row, west);
        count += self.cells[self.now][w] as u8;

        let e = self.get_index(row, east);
        count += self.cells[self.now][e] as u8;

        let sw = self.get_index(south, west);
        count += self.cells[self.now][sw] as u8;

        let s = self.get_index(south, column);
        count += self.cells[self.now][s] as u8;

        let se = self.get_index(south, east);
        count += self.cells[self.now][se] as u8;

        count

        // for delta_row in [self.height - 1, 0, 1].iter().cloned() {
        //     for delta_col in [self.width - 1, 0, 1].iter().cloned() {
        //         if delta_col == 0 && delta_row == 0 {
        //             continue;
        //         }
        //         let neighbor_row = (row + delta_row) % self.height;
        //         let neighbor_col = (col + delta_col) % self.width;
        //         let idx = self.get_index(neighbor_row, neighbor_col);
        //         count += self.cells[idx] as u8;
        //     }
        // }
        // count
    }

    
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[self.now].set(idx, true);
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");

        self.d_alive.clear();
        self.d_dead.clear();

        let next = self.now ^ 1;

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[self.now][idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let value = match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise,
                };
                self.cells[next].set(idx, value);
                if value != cell {
                    if value { self.d_alive.push(idx as u32); }
                    else { self.d_dead.push(idx as u32); }
                }
            }
        }
        self.now = next;
        self.d_alive.push(0);
        self.d_alive.push(0);
        self.d_dead.push(0);
        self.d_dead.push(0);
        // self.cells = next;
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width: u32 = 128;
        let height = 128;
        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, i % 2 == 0 || i % 7 == 0);
        }

        Universe {
            width: width,
            height: height,
            cells: [cells.clone(), cells],
            now: 0,
            d_alive: Vec::with_capacity(size),
            d_dead: Vec::with_capacity(size),
        }
    }

    pub fn new_spaceship() -> Universe {
        utils::set_panic_hook();
        let width: u32 = 64;
        let height = 64;
        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        let pos = (10, 10);
        let ss = [
            (0, 1),
            (0, 4),
            (1, 0),
            (2, 0),
            (2, 4),
            (3, 0),
            (3, 1),
            (3, 2),
            (3, 3),
        ];

        for (row, col) in ss.iter() {
            let idx = ((row + pos.0) * width + col + pos.1) as usize;
            cells.set(idx, true);
        }

        Universe {
            width: width,
            height: height,
            cells: [cells.clone(), cells],
            now: 0,    
            d_alive: Vec::with_capacity(size),
            d_dead: Vec::with_capacity(size),
        
        }
    }

    pub fn new_random() -> Universe {
        utils::set_panic_hook();
        let width = 128;
        let height = 128;
        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            if Math::random() < 0.5 {
                cells.set(i, true);
            } else {
                cells.set(i, false);
            }
        }
        // panic!("Testing panic in Universe::new_random()");
        Universe {
            width: width,
            height: height,
            cells: [cells.clone(), cells],
            now: 0,
            d_alive: Vec::with_capacity(size),
            d_dead: Vec::with_capacity(size),
        }
    }

    pub fn reset_dead(&mut self) {
        self.cells[self.now].clear();
    }

    pub fn reset_random(&mut self) {
        let size = (self.width * self.height) as usize;
        for i in 0..size {
            if Math::random() < 0.5 {
                self.cells[self.now].set(i, true);
            } else {
                self.cells[self.now].set(i, false);
            }
        }
    }

    pub fn insert_glider(&mut self, row: u32, col: u32) {
        let glider_se = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
        let glider_size = (3, 3);
        if row + glider_size.0 > self.height || col + glider_size.1 > self.width {
            log!("Cannot insert glider on borders.");
        } else {
            for (r, c) in glider_se {
                let idx = self.get_index(row + r, col + c);
                self.cells[self.now].set(idx, true);
            }
        }
    }

    pub fn insert_pulsar(&mut self, row: u32, col: u32) {
        let pulsar = [
            // Horizontal lines
            (1, 3),
            (1, 4),
            (1, 5),
            (1, 9),
            (1, 10),
            (1, 11),
            (6, 3),
            (6, 4),
            (6, 5),
            (6, 9),
            (6, 10),
            (6, 11),
            (8, 3),
            (8, 4),
            (8, 5),
            (8, 9),
            (8, 10),
            (8, 11),
            (13, 3),
            (13, 4),
            (13, 5),
            (13, 9),
            (13, 10),
            (13, 11),
            // Vertical lines
            (3, 1),
            (4, 1),
            (5, 1),
            (9, 1),
            (10, 1),
            (11, 1),
            (3, 6),
            (4, 6),
            (5, 6),
            (9, 6),
            (10, 6),
            (11, 6),
            (3, 8),
            (4, 8),
            (5, 8),
            (9, 8),
            (10, 8),
            (11, 8),
            (3, 13),
            (4, 13),
            (5, 13),
            (9, 13),
            (10, 13),
            (11, 13),
        ];
        let size = (15, 15);
        if row + size.0 > self.height || col + size.1 > self.width {
            log!("Cannot insert pulsar on borders.");
        } else {
            for (r, c) in pulsar {
                let idx = self.get_index(row + r, col + c);
                self.cells[self.now].set(idx, true);
            }
        }
    }
    // pub fn render(&self) -> String {
    // self.to_string()
    // }

    /// Set the width of the universe. Resets all cells to dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells[self.now].clear();
    }

    /// Set the height of the universe. Resets all cells to dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells[self.now].clear();
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn cells(&self) -> *const u32 {
        self.cells[self.now].as_slice().as_ptr()
    }
    pub fn delta_a(&self) -> *const u32 {
        self.d_alive.as_slice().as_ptr()
    }
    pub fn delta_d(&self) -> *const u32 {
        self.d_dead.as_slice().as_ptr()
    }

    pub fn toggle_cell(&mut self, row: u32, col: u32) {
        let idx = self.get_index(row, col);
        self.cells[self.now].toggle(idx);
    }

    pub fn is_alive(&self, idx: u32) -> bool {
        self.cells[self.now][idx as usize]
    }
}

// use std::fmt;
// impl fmt::Display for Universe {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         for line in self.cells.as_slice().chunks(self.width as usize) {
//             for &cell in line {
//                 let symbol = if cell == false { '◻' } else { '◼' };
//                 write!(f, "{}", symbol)?;
//             }
//             write!(f, "\n")?;
//         }

//         Ok(())
//     }
// }
