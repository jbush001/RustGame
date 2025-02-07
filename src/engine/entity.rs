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
use std::any::Any;

pub const CONTROL_UP: u32 = 0x1;
pub const CONTROL_DOWN: u32 = 0x2;
pub const CONTROL_LEFT: u32 = 0x4;
pub const CONTROL_RIGHT: u32 = 0x8;
pub const CONTROL_FIRE: u32 = 0x10;
pub const CONTROL_JUMP: u32 = 0x20;

pub const COLL_MISSILE: u32 = 1;
pub const COLL_PLAYER: u32 = 2;

pub trait Entity: Any {
    fn update(
        &mut self,
        d_t: f32,
        new_entities: &mut Vec<Box<dyn Entity>>,
        buttons: u32,
        tile_map: &tilemap::TileMap,
    );
    fn draw(&mut self, context: &mut gfx::RenderContext);
    fn is_live(&self) -> bool;

    // Each bit in this represents a type of entity, which is used
    // in conjunction with get_collision_mask.
    fn get_collision_class(&self) -> u32;

    // Each 1 bit in this corresponds to a collision class that this entity
    // will 'accept' collisions with.
    fn get_collision_mask(&self) -> u32;

    // Return axis aligned bounding box of this object (left, top, width, height)
    // If two entities bounding boxes overlap they are considered to collide.
    fn get_bounding_box(&self) -> (f32, f32, f32, f32);
    fn collide(&mut self, other: &dyn Entity);

    fn as_any(&self) -> &dyn Any;
}

pub fn do_frame(
    entities: &mut Vec<Box<dyn Entity>>,
    d_t: f32,
    context: &mut gfx::RenderContext,
    buttons: u32,
    tilemap: &tilemap::TileMap,
    _visible_rect: &(i32, i32, i32, i32),
) {
    handle_collisions(entities);
    let mut new_entities: Vec<Box<dyn Entity>> = Vec::new();
    for entity in entities.iter_mut() {
        entity.update(d_t, &mut new_entities, buttons, tilemap);
    }

    entities.append(&mut new_entities);

    // XXX despawn things that are too far outsize visible rect
    entities.retain(|entity| entity.is_live());

    for entity in entities.iter_mut() {
        entity.draw(context);
    }
}

fn overlaps(a1: &(f32, f32, f32, f32), a2: &(f32, f32, f32, f32)) -> bool {
    let (x1, y1, w1, h1) = *a1;
    let (x2, y2, w2, h2) = *a2;

    x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
}

// Check for objects overlapping and call their collision handlers
// This is a brute force O(n^2) algorithm. While a broad phase step
// would reduce the computational complexity, for fairly small
// numbers of objects this is probably preferable.
fn handle_collisions(entities: &mut Vec<Box<dyn Entity>>) {
    for i in 0..entities.len() - 1 {
        let (arr1, arr2) = entities.split_at_mut(i + 1);
        let e1 = &mut arr1[i];
        let b1 = e1.get_bounding_box();
        for e2 in arr2.iter_mut() {
            let b2 = e2.get_bounding_box();
            if overlaps(&b1, &b2) {
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
