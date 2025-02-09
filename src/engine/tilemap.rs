//
// Copyright 2025 Jeff Bush
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use crate::gfx;
use std::io::Read;
use std::path::PathBuf;

const TILE_SIZE: i32 = 64;

const FLAG_SOLID: u8 = 1;
const FLAG_LADDER: u8 = 2;

pub struct TileMap {
    tiles: Vec<u8>,
    tile_flags: Vec<u8>,
    atlas_coords: Vec<(f32, f32, f32, f32, u32, u32)>,
    width: i32,
    height: i32,
}

impl TileMap {
    // Format
    //    magic [u8; 4]  "TMAP"
    //    width: u32
    //    height: u32
    //    num_tiles: u32
    //    tile_locs: [(f32, f32, f32, f32), num_tiles]
    //    map: [u8; width * height]
    pub fn new(path: &PathBuf) -> TileMap {
        let file = std::fs::File::open(path).unwrap();
        let mut reader = std::io::BufReader::new(file);

        // Check magic
        let mut magic = [0; 4];
        reader.read_exact(&mut magic).unwrap();
        if &magic != b"TMAP" {
            panic!("Invalid tilemap file");
        }

        // Read width and height
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf).unwrap();
        let width = i32::from_le_bytes(buf);
        reader.read_exact(&mut buf).unwrap();
        let height = i32::from_le_bytes(buf);

        reader.read_exact(&mut buf).unwrap();
        let num_tiles = i32::from_le_bytes(buf);
        let mut atlas_coords = Vec::new();
        for _ in 0..num_tiles {
            reader.read_exact(&mut buf).unwrap();
            let left = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf).unwrap();
            let top = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf).unwrap();
            let right = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf).unwrap();
            let bottom = f32::from_le_bytes(buf);

            atlas_coords.push((left, top, right, bottom, TILE_SIZE as u32, TILE_SIZE as u32));
        }

        let mut tile_flags = vec![0; num_tiles as usize];
        reader.read_exact(&mut tile_flags).unwrap();

        // Read tile data
        let mut tiles = vec![0; (width * height) as usize];
        reader.read_exact(&mut tiles).unwrap();

        TileMap {
            tiles,
            tile_flags,
            atlas_coords,
            width,
            height,
        }
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        (self.get_flags(x, y) & FLAG_SOLID) != 0
    }

    pub fn is_ladder(&self, x: i32, y: i32) -> bool {
        (self.get_flags(x, y) & FLAG_LADDER) != 0
    }

    pub fn get_flags(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width * TILE_SIZE || y >= self.height * TILE_SIZE {
            return 0;
        }

        let tile_num = self.tiles[((y / TILE_SIZE) * self.width + (x / TILE_SIZE)) as usize];
        if tile_num == 0 {
            return 0;
        }

        self.tile_flags[(tile_num - 1) as usize]
    }

    pub fn draw(&mut self, context: &mut gfx::RenderContext, visible_rect: &(i32, i32, i32, i32)) {
        let (left, top, right, bottom) = *visible_rect;

        let left_tile = left / TILE_SIZE;
        let right_tile = (left + right + TILE_SIZE - 1) / TILE_SIZE;
        let top_tile = top / TILE_SIZE;
        let bottom_tile = (top + bottom + TILE_SIZE - 1) / TILE_SIZE;

        for y in top_tile..bottom_tile {
            for x in left_tile..right_tile {
                let tile = self.tiles[(y * self.width + x) as usize];
                if tile != 0 {
                    context.draw_image(
                        (TILE_SIZE * x, TILE_SIZE * y),
                        &self.atlas_coords[tile as usize - 1],
                        0.0,
                        (0, 0),
                        false,
                    );
                }
            }
        }
    }
}
