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

pub const GRAVITY: f32 = 500.0;

const COLL_MISSILE: u32 = 1;
const COLL_PLAYER: u32 = 2;

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

    // Return axis aligned bounding box of this object (left, top, right, bottom)
    // If two entities bounding boxes overlap they are considered to collide.
    fn get_bounding_box(&self) -> (f32, f32, f32, f32);
    fn collide(&mut self, other: &dyn Entity);

    fn as_any(&self) -> &dyn Any;
}

pub struct Arrow {
    xpos: f32,
    ypos: f32,
    xvec: f32,
    yvec: f32,
    angle: f32,
    wobble: f32,
    collided: bool,
}

impl Arrow {
    pub fn new(xpos: f32, ypos: f32, angle: f32, velocity: f32) -> Arrow {
        Arrow {
            xpos,
            ypos,
            xvec: angle.cos() * velocity,
            yvec: angle.sin() * velocity,
            angle,
            wobble: 0.0,
            collided: false,
        }
    }
}

impl Entity for Arrow {
    fn update(
        &mut self,
        d_t: f32,
        _new_entities: &mut Vec<Box<dyn Entity>>,
        _buttons: u32,
        tile_map: &tilemap::TileMap,
    ) {
        if tile_map.is_solid(self.xpos as i32, self.ypos as i32) {
            self.collided = true;
        }

        self.xpos += self.xvec * d_t;
        self.ypos += self.yvec * d_t;
        self.angle = self.yvec.atan2(self.xvec);
        self.yvec += GRAVITY * d_t;
        self.wobble += d_t * 10.0;
    }

    fn draw(&mut self, context: &mut gfx::RenderContext) {
        context.draw_image(
            (self.xpos as i32, self.ypos as i32),
            &gfx::SPR_ARROW,
            self.angle + self.wobble.sin() * 0.1,
            (gfx::SPR_ARROW.4 as i32 / 2, gfx::SPR_ARROW.5 as i32 / 2),
            false,
        );
    }

    fn is_live(&self) -> bool {
        // XXX we don't check if this has gone out of scroll window.
        self.ypos < gfx::WINDOW_HEIGHT as f32 && self.xpos > 0.0 && !self.collided
    }

    fn get_bounding_box(&self) -> (f32, f32, f32, f32) {
        // We only track the tip of the arrow
        (
            self.xpos + self.angle.cos() * 14.0,
            self.ypos + self.angle.sin() * 14.0,
            4.0,
            4.0,
        )
    }

    fn get_collision_class(&self) -> u32 {
        COLL_MISSILE
    }

    fn get_collision_mask(&self) -> u32 {
        !COLL_MISSILE
    }

    fn collide(&mut self, _other: &dyn Entity) {
        self.collided = true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Player {
    angle: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    bow_drawn: bool,
    bow_draw_time: f32,
    facing_left: bool,
    run_frame: u32,
    is_running: bool,
    on_ground: bool,
    frame_time: f32,
    y_vec: f32,
    last_jump_button: bool,
    killed: bool,
}

const RUN_FRAME_DURATION: f32 = 0.1;

impl Player {
    pub fn new() -> Player {
        Player {
            angle: -std::f32::consts::PI / 4.0,
            pos_x: 20.0,
            pos_y: 30.0,
            bow_drawn: false,
            bow_draw_time: 0.0,
            facing_left: false,
            run_frame: 0,
            is_running: false,
            on_ground: false,
            frame_time: 0.0,
            y_vec: 0.0,
            last_jump_button: false,
            killed: false,
        }
    }
}

impl Entity for Player {
    fn update(
        &mut self,
        d_t: f32,
        new_entities: &mut Vec<Box<dyn Entity>>,
        buttons: u32,
        tile_map: &tilemap::TileMap,
    ) {
        if self.killed {
            return;
        }

        if buttons & CONTROL_FIRE == 0 {
            // Button not pressed
            if self.bow_drawn {
                // It was released
                let velocity = self.bow_draw_time.clamp(0.2, 0.4) * 3000.0;
                let arrow_angle = if self.facing_left {
                    std::f32::consts::PI - self.angle
                } else {
                    self.angle
                };
                new_entities.push(Box::new(Arrow::new(
                    self.pos_x,
                    self.pos_y,
                    arrow_angle,
                    velocity,
                )));

                self.bow_drawn = false;
            }
        } else {
            // Button pressed
            if self.bow_drawn {
                self.bow_draw_time += d_t;
            } else {
                self.bow_drawn = true;
                self.bow_draw_time = 0.0;
            }

            // Player can adjust angle when bow is drawn.
            if buttons & CONTROL_UP != 0 && self.angle > -std::f32::consts::PI / 2.0 {
                self.angle -= d_t * std::f32::consts::PI;
            }

            if buttons & CONTROL_DOWN != 0 && self.angle < std::f32::consts::PI / 2.0 {
                self.angle += d_t * std::f32::consts::PI;
            }
        }

        self.on_ground = tile_map.is_solid(self.pos_x as i32 - 12, self.pos_y as i32 + 48)
            || tile_map.is_solid(self.pos_x as i32 + 12, self.pos_y as i32 + 48);

        if self.on_ground {
            if buttons & CONTROL_JUMP != 0 && !self.last_jump_button {
                self.y_vec = -300.0;
            } else {
                self.y_vec = 0.0;

                // Ensure it is on the ground.
                self.pos_y = (self.pos_y / 64.0).floor() * 64.0 + (64.0 - 48.0);
            }
        } else {
            // In air
            self.is_running = false;
            self.y_vec += GRAVITY * d_t;
        }

        self.last_jump_button = buttons & CONTROL_JUMP != 0;
        self.pos_y += self.y_vec * d_t;

        // Movement
        if buttons & CONTROL_LEFT != 0
            && !tile_map.is_solid(self.pos_x as i32 - 16, self.pos_y as i32 + 45)
            && !tile_map.is_solid(self.pos_x as i32 - 16, self.pos_y as i32 - 15)
        {
            self.pos_x -= 150.0 * d_t;
            self.facing_left = true;
            self.is_running = self.on_ground;
        } else if buttons & CONTROL_RIGHT != 0
            && !tile_map.is_solid(self.pos_x as i32 + 16, self.pos_y as i32 + 45)
            && !tile_map.is_solid(self.pos_x as i32 + 16, self.pos_y as i32 - 15)
        {
            self.pos_x += 150.0 * d_t;
            self.facing_left = false;
            self.is_running = self.on_ground;
        } else {
            self.is_running = false;
            self.run_frame = 0;
            self.frame_time = 0.0;
        }

        if self.is_running {
            self.frame_time += d_t;
            if self.frame_time > RUN_FRAME_DURATION {
                self.frame_time -= RUN_FRAME_DURATION;
                self.run_frame += 1;
                if self.run_frame > 2 {
                    self.run_frame = 0;
                }
            }
        }
    }

    fn draw(&mut self, context: &mut gfx::RenderContext) {
        if self.killed {
            context.draw_image(
                (self.pos_x as i32, self.pos_y as i32),
                &gfx::SPR_PLAYER_DEAD,
                0.0,
                (33, 20),
                false,
            );
            return;
        }

        if !self.bow_drawn {
            // Draw bow on back
            context.draw_image(
                (self.pos_x as i32, self.pos_y as i32),
                &gfx::SPR_BOW_ON_BACK,
                0.0,
                (33, 20),
                self.facing_left,
            );
        }

        let body_image = if !self.on_ground {
            &gfx::SPR_PLAYER_BODY_JUMP
        } else if self.is_running {
            match self.run_frame {
                0 => &gfx::SPR_PLAYER_BODY_RUN1,
                1 => &gfx::SPR_PLAYER_BODY_RUN2,
                2 => &gfx::SPR_PLAYER_BODY_RUN3,
                _ => &gfx::SPR_PLAYER_BODY_RUN1,
            }
        } else {
            &gfx::SPR_PLAYER_BODY_IDLE
        };

        context.draw_image(
            (self.pos_x as i32, self.pos_y as i32),
            body_image,
            0.0,
            (33, 20),
            self.facing_left,
        );

        if self.bow_drawn {
            let angle = if self.facing_left {
                -self.angle
            } else {
                self.angle
            };
            context.draw_image(
                (self.pos_x as i32, self.pos_y as i32),
                &gfx::SPR_BOW_DRAWN,
                angle,
                (33, 20),
                self.facing_left,
            );
        } else {
            let arms_image = if self.is_running {
                match self.run_frame {
                    0 => &gfx::SPR_ARMS_RUN1,
                    1 => &gfx::SPR_ARMS_RUN2,
                    2 => &gfx::SPR_ARMS_RUN3,
                    _ => &gfx::SPR_ARMS_RUN1,
                }
            } else {
                &gfx::SPR_ARMS_IDLE
            };

            context.draw_image(
                (self.pos_x as i32, self.pos_y as i32),
                arms_image,
                0.0,
                (33, 20),
                self.facing_left,
            );
        }
    }

    fn is_live(&self) -> bool {
        true
    }

    fn get_bounding_box(&self) -> (f32, f32, f32, f32) {
        if self.killed {
            // This affects how subsequent arrows that hit the corpse are
            // displayed.
            (self.pos_x - 32.0, self.pos_y + 40.0, 64.0, 14.0)
        } else {
            // We only include the torso
            (self.pos_x - 5.0, self.pos_y - 5.0, 10.0, 15.0)
        }
    }

    fn get_collision_class(&self) -> u32 {
        COLL_PLAYER
    }

    fn get_collision_mask(&self) -> u32 {
        !COLL_PLAYER
    }

    fn collide(&mut self, _other: &(dyn Entity)) {
        // XXX check type
        self.killed = true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
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
