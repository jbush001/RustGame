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
use crate::tilemap;
use crate::util;
use std::any::Any;

pub const CONTROL_UP: u32 = 0x1;
pub const CONTROL_DOWN: u32 = 0x2;
pub const CONTROL_LEFT: u32 = 0x4;
pub const CONTROL_RIGHT: u32 = 0x8;
pub const CONTROL_FIRE: u32 = 0x10;
pub const CONTROL_JUMP: u32 = 0x20;

pub trait Entity: Any {
    fn update(
        &mut self,
        d_t: f32,
        new_entities: &mut Vec<Box<dyn Entity>>,
        buttons: u32,
        tile_map: &tilemap::TileMap,
        player_rect: &util::Rect<i32>,
    );
    fn draw(&self, context: &mut gfx::RenderContext);
    fn is_live(&self) -> bool;

    // Each bit in this represents a type of entity, which is used
    // in conjunction with get_collision_mask.
    fn get_collision_class(&self) -> u32;

    // Each 1 bit in this corresponds to a collision class that this entity
    // will 'accept' collisions with.
    fn get_collision_mask(&self) -> u32;

    // Return axis aligned bounding box of this object (left, top, width, height)
    // If two entities bounding boxes overlap they are considered to collide.
    fn get_bounding_box(&self) -> util::Rect<i32>;
    fn collide(&mut self, other: &dyn Entity);

    fn as_any(&self) -> &dyn Any;
}

// Check for objects overlapping and call their collision handlers
// This is a brute force O(n^2) algorithm. While a broad phase step
// would reduce the computational complexity, for fairly small
// numbers of objects this is probably preferable.
pub fn handle_collisions(entities: &mut Vec<Box<dyn Entity>>) {
    for i in 0..entities.len() - 1 {
        let (arr1, arr2) = entities.split_at_mut(i + 1);
        let e1 = &mut arr1[i];
        let box1 = e1.get_bounding_box();
        for e2 in arr2.iter_mut() {
            let box2 = e2.get_bounding_box();
            if box1.overlaps(&box2) {
                if (e1.get_collision_mask() & e2.get_collision_class()) != 0 {
                    e1.collide(e2.as_ref());
                }

                if (e2.get_collision_mask() & e1.get_collision_class()) != 0 {
                    e2.collide(e1.as_ref());
                }
            }
        }
    }
}
