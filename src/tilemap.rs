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

pub struct TileMap {
    tiles: Vec<u8>,
    width: i32,
    height: i32,
}

impl TileMap {
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
        let mut buf = [0; 4];
        reader.read_exact(&mut buf).unwrap();
        let width = i32::from_le_bytes(buf);
        reader.read_exact(&mut buf).unwrap();
        let height = i32::from_le_bytes(buf);

        // Read tile data
        let mut tiles = vec![0; (width * height) as usize];
        reader.read_exact(&mut tiles).unwrap();

        TileMap {
            tiles,
            width,
            height,
        }
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width * TILE_SIZE || y >= self.height * TILE_SIZE {
            return true;
        }

        self.tiles[((y / TILE_SIZE) * self.width + (x / TILE_SIZE)) as usize] != 0
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
                        &gfx::TILE_BRICK,
                        0.0,
                        (0, 0),
                        false,
                    );
                }
            }
        }
    }
}
