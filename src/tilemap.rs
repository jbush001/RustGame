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

const TILE_SIZE: i32 = 64;

pub struct TileMap {
    tiles: Vec<u8>,
    width: i32,
    height: i32,
}

impl TileMap {
    pub fn new() -> TileMap {
        let width: i32 = 64;
        let height: i32 = 9;

        let mut map = TileMap {
            tiles: vec![0; (width * height) as usize],
            width,
            height,
        };

        for x in 0..width {
            map.tiles[(6 * map.width + x) as usize] = 1;
        }

        map.tiles[(5 * map.width + 5) as usize] = 1;
        map
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width * TILE_SIZE || y >= self.height * TILE_SIZE {
            return true;
        }

        self.tiles[((y / TILE_SIZE) * self.width + (x / TILE_SIZE)) as usize] != 0
    }

    pub fn draw(&mut self, context: &mut gfx::RenderContext, visible_rect: &(i32, i32, i32, i32)) {
        let (left, top, right, bottom) = *visible_rect;

        let left_tile = left as i32 / TILE_SIZE;
        let right_tile = (left + right as i32 + TILE_SIZE - 1) / TILE_SIZE;
        let top_tile = top as i32 / TILE_SIZE;
        let bottom_tile = (top + bottom as i32 + TILE_SIZE - 1) / TILE_SIZE;

        for y in top_tile..bottom_tile {
            for x in left_tile..right_tile {
                let tile = self.tiles[(y * self.width + x) as usize];
                if tile != 0 {
                    context.draw_image(
                        (TILE_SIZE * x as i32, TILE_SIZE * y as i32),
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
