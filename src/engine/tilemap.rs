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
use crate::util;
use std::io::Read;
use std::path::PathBuf;

pub const TILE_SIZE: i32 = 64;
pub const TILE_SIZE_F: f32 = TILE_SIZE as f32;

const FLAG_SOLID: u8 = 1;
const FLAG_LADDER: u8 = 2;

#[derive(Default)]
pub struct TileMap {
    pub width: i32,
    pub height: i32,
    tiles: Vec<u8>,
    tile_flags: Vec<u8>,
    atlas_coords: Vec<(f32, f32, f32, f32, u32, u32, i32, i32)>,
    pub objects: Vec<(String, i32, i32)>,
    pub player_start_x: i32,
    pub player_start_y: i32,
}

impl TileMap {
    pub fn new(path: &PathBuf) -> TileMap {
        // Format
        //    magic [u8; 4]  "TMAP"
        //    width: u32
        //    height: u32
        //    num_tiles: u32
        //    tile_locs: [(f32, f32, f32, f32), num_tiles]
        //    map: [u8; width * height]

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
        println!("Loading tilemap {}x{}", width, height);

        reader.read_exact(&mut buf).unwrap();
        let player_start_x = i32::from_le_bytes(buf);
        reader.read_exact(&mut buf).unwrap();
        let player_start_y = i32::from_le_bytes(buf);

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

            atlas_coords.push((
                left,
                top,
                right,
                bottom,
                TILE_SIZE as u32,
                TILE_SIZE as u32,
                0,
                0,
            ));
        }

        let mut tile_flags = vec![0; num_tiles as usize];
        reader.read_exact(&mut tile_flags).unwrap();

        // Read tile data
        let mut tiles = vec![0; (width * height) as usize];
        reader.read_exact(&mut tiles).unwrap();

        // Read object locations.
        reader.read_exact(&mut buf).unwrap();
        let num_objects = u32::from_le_bytes(buf) as usize;
        let mut objects: Vec<(String, i32, i32)> = Vec::new();
        for _ in 0..num_objects {
            let mut name_buf = [0u8; 32];
            reader.read_exact(&mut name_buf).unwrap();
            let pos = name_buf.iter().position(|&x| x == 0).unwrap();
            let name = String::from_utf8_lossy(&name_buf[..pos]).to_string();

            reader.read_exact(&mut buf).unwrap();
            let x = i32::from_le_bytes(buf);
            reader.read_exact(&mut buf).unwrap();
            let y = i32::from_le_bytes(buf);

            objects.push((name, x, y));
        }

        TileMap {
            tiles,
            tile_flags,
            atlas_coords,
            width,
            height,
            objects,
            player_start_x,
            player_start_y,
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

    pub fn draw(&self, context: &mut gfx::RenderContext, visible_rect: &util::Rect<i32>) {
        let left_tile = visible_rect.left / TILE_SIZE;
        let right_tile = std::cmp::min(
            (visible_rect.right() + TILE_SIZE - 1) / TILE_SIZE,
            self.width,
        );
        let top_tile = visible_rect.top / TILE_SIZE;
        let bottom_tile = std::cmp::min(
            (visible_rect.bottom() + TILE_SIZE - 1) / TILE_SIZE,
            self.height,
        );

        for y in top_tile..bottom_tile {
            for x in left_tile..right_tile {
                let tile = self.tiles[(y * self.width + x) as usize];
                if tile != 0 {
                    context.draw_image(
                        (TILE_SIZE * x, TILE_SIZE * y),
                        &self.atlas_coords[tile as usize - 1],
                        0.0,
                        false,
                    );
                }
            }
        }
    }
}
