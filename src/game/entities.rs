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

use crate::assets;
use crate::util;
use engine::audio;
use engine::entity;
use engine::gfx;
use engine::tilemap;
use std::any::Any;

pub const GRAVITY: f32 = 1500.0;

// Collision classes
pub const COLL_MISSILE: u32 = 1;
pub const COLL_PLAYER: u32 = 2;
pub const COLL_OBJ: u32 = 4;

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

impl entity::Entity for Arrow {
    fn update(
        &mut self,
        d_t: f32,
        _new_entities: &mut Vec<Box<dyn entity::Entity>>,
        _buttons: u32,
        tile_map: &tilemap::TileMap,
    ) {
        if tile_map.is_solid(self.xpos as i32, self.ypos as i32) {
            self.collided = true;
        }

        self.xpos += self.xvec * d_t;
        self.ypos += self.yvec * d_t;
        self.angle = self.yvec.atan2(self.xvec);
        if self.yvec < 500.0 {
            self.yvec += GRAVITY * d_t;
        }

        self.wobble += d_t * 10.0;
    }

    fn draw(&self, context: &mut gfx::RenderContext) {
        context.draw_image(
            (self.xpos as i32, self.ypos as i32),
            &assets::SPR_ARROW,
            self.angle + self.wobble.sin() * 0.1,
            false,
        );
    }

    fn is_live(&self) -> bool {
        !self.collided
    }

    fn get_bounding_box(&self) -> util::Rect<i32> {
        // We only track the tip of the arrow
        util::Rect::<i32>::new(
            (self.xpos + self.angle.cos() * 14.0) as i32,
            (self.ypos + self.angle.sin() * 14.0) as i32,
            4,
            4,
        )
    }

    fn get_collision_class(&self) -> u32 {
        COLL_MISSILE
    }

    fn get_collision_mask(&self) -> u32 {
        !COLL_MISSILE
    }

    fn collide(&mut self, _other: &dyn entity::Entity) {
        self.collided = true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Player {
    angle: f32,

    // The origin for the player is in the center, right at the shoulder
    // level (since the bow pivots around this point).
    xpos: f32,
    ypos: f32,
    bow_drawn: bool,
    bow_draw_time: f32,
    facing_left: bool,
    run_frame: u32,
    is_running: bool,
    on_ground: bool,
    jump_counter: u32,
    frame_time: f32,
    yvec: f32,
    last_jump_button: bool,
    killed: bool,
    climbing: bool,
    ground_offset: i32, // Distance from origin to ground
}

const RUN_FRAME_DURATION: f32 = 0.1;
const MAX_JUMP_COUNTER: u32 = 5;

impl Player {
    pub fn new(xpos: f32, ypos: f32) -> Player {
        let ground_offset = assets::SPR_PLAYER_BODY_IDLE.5 as i32 - assets::SPR_PLAYER_BODY_IDLE.7;
        Player {
            angle: -std::f32::consts::PI / 4.0,
            xpos,
            ypos: ypos + 64.0 - ground_offset as f32,
            bow_drawn: false,
            bow_draw_time: 0.0,
            facing_left: false,
            run_frame: 0,
            is_running: false,
            on_ground: false,
            jump_counter: MAX_JUMP_COUNTER,
            frame_time: 0.0,
            yvec: 0.0,
            last_jump_button: false,
            killed: false,
            climbing: false,
            ground_offset,
        }
    }
}

impl entity::Entity for Player {
    fn update(
        &mut self,
        d_t: f32,
        new_entities: &mut Vec<Box<dyn entity::Entity>>,
        buttons: u32,
        tile_map: &tilemap::TileMap,
    ) {
        if self.killed {
            return;
        }

        let on_ladder = tile_map.is_ladder(self.xpos as i32, self.ypos as i32)
            || tile_map.is_ladder(self.xpos as i32, self.ypos as i32 + self.ground_offset);
        if self.climbing {
            if !on_ladder {
                self.climbing = false;
            }

            if buttons & entity::CONTROL_UP != 0
                && !tile_map.is_solid(self.xpos as i32, self.ypos as i32)
            {
                self.ypos -= 128.0 * d_t;
            } else if buttons & entity::CONTROL_DOWN != 0
                && !tile_map.is_solid(self.xpos as i32, self.ypos as i32 + self.ground_offset)
            {
                self.ypos += 128.0 * d_t;
            }

            if buttons & entity::CONTROL_LEFT != 0 {
                self.xpos -= 128.0 * d_t;
            } else if buttons & entity::CONTROL_RIGHT != 0 {
                self.xpos += 128.0 * d_t;
            }

            return;
        } else if on_ladder
            && (buttons & entity::CONTROL_UP != 0 || buttons & entity::CONTROL_DOWN != 0)
        {
            self.climbing = true;
            self.bow_drawn = false;
            return;
        }

        if buttons & entity::CONTROL_FIRE == 0 {
            // Button not pressed
            if self.bow_drawn {
                // It was released
                let velocity = self.bow_draw_time.clamp(0.2, 0.4) * 5000.0;
                let arrow_angle = if self.facing_left {
                    std::f32::consts::PI - self.angle
                } else {
                    self.angle
                };
                new_entities.push(Box::new(Arrow::new(
                    self.xpos,
                    self.ypos,
                    arrow_angle,
                    velocity,
                )));

                audio::play_effect(assets::SFX_ARROW);
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
            if buttons & entity::CONTROL_UP != 0 && self.angle > -std::f32::consts::PI / 2.0 {
                self.angle -= d_t * std::f32::consts::PI;
            }

            if buttons & entity::CONTROL_DOWN != 0 && self.angle < std::f32::consts::PI / 2.0 {
                self.angle += d_t * std::f32::consts::PI;
            }
        }

        self.on_ground = tile_map
            .is_solid(self.xpos as i32 - 12, self.ypos as i32 + self.ground_offset)
            || tile_map.is_solid(self.xpos as i32 + 12, self.ypos as i32 + self.ground_offset);

        if self.on_ground {
            if buttons & entity::CONTROL_JUMP != 0 && !self.last_jump_button {
                self.yvec = -100.0;
                self.jump_counter = 0;
            } else {
                self.yvec = 0.0;

                // Ensure it is on the ground.
                self.ypos = (self.ypos / tilemap::TILE_SIZE_F).floor() * tilemap::TILE_SIZE_F
                    + (tilemap::TILE_SIZE_F - self.ground_offset as f32);
            }
        } else if self.jump_counter < MAX_JUMP_COUNTER {
            // Size of jump is proportional to how long the button is held
            if buttons & entity::CONTROL_JUMP != 0 {
                self.yvec -= 5000.0 * d_t;
                self.jump_counter += 1;
            } else {
                // If you let off button, stops increasing jump height
                self.jump_counter = MAX_JUMP_COUNTER;
            }
        } else {
            // In air
            self.is_running = false;
            if self.yvec < 500.0 {
                self.yvec += GRAVITY * d_t;
            }
        }

        if self.yvec < 0.0 && tile_map.is_solid(self.xpos as i32, self.ypos as i32 - 20) {
            // Bumped head while jumping
            self.yvec = 0.0;
        }

        self.last_jump_button = buttons & entity::CONTROL_JUMP != 0;
        self.ypos += self.yvec * d_t;

        // Movement
        if buttons & entity::CONTROL_LEFT != 0
            && !tile_map.is_solid(
                self.xpos as i32 - 16,
                self.ypos as i32 + self.ground_offset - 3,
            )
            && !tile_map.is_solid(self.xpos as i32 - 16, self.ypos as i32 - 15)
        {
            self.xpos -= 150.0 * d_t;
            self.facing_left = true;
            self.is_running = self.on_ground;
        } else if buttons & entity::CONTROL_RIGHT != 0
            && !tile_map.is_solid(
                self.xpos as i32 + 16,
                self.ypos as i32 + self.ground_offset - 3,
            )
            && !tile_map.is_solid(self.xpos as i32 + 16, self.ypos as i32 - 15)
        {
            self.xpos += 150.0 * d_t;
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

    fn draw(&self, context: &mut gfx::RenderContext) {
        if self.killed {
            context.draw_image(
                (self.xpos as i32, self.ypos as i32),
                &assets::SPR_PLAYER_DEAD,
                0.0,
                false,
            );
            return;
        }

        if self.climbing {
            let sprite = if (((self.ypos + self.xpos) as i32) % 64) > 32 {
                &assets::SPR_PLAYER_CLIMB1
            } else {
                &assets::SPR_PLAYER_CLIMB2
            };
            context.draw_image((self.xpos as i32, self.ypos as i32), sprite, 0.0, false);
            return;
        }

        if !self.bow_drawn {
            // Draw bow on back
            context.draw_image(
                (self.xpos as i32, self.ypos as i32),
                &assets::SPR_PLAYER_BOW_ON_BACK,
                0.0,
                self.facing_left,
            );
        }

        let body_image = if !self.on_ground {
            &assets::SPR_PLAYER_BODY_JUMP
        } else if self.is_running {
            match self.run_frame {
                0 => &assets::SPR_PLAYER_BODY_RUN1,
                1 => &assets::SPR_PLAYER_BODY_RUN2,
                2 => &assets::SPR_PLAYER_BODY_RUN3,
                _ => &assets::SPR_PLAYER_BODY_RUN1,
            }
        } else {
            &assets::SPR_PLAYER_BODY_IDLE
        };

        context.draw_image(
            (self.xpos as i32, self.ypos as i32),
            body_image,
            0.0,
            self.facing_left,
        );

        if self.bow_drawn {
            let angle = if self.facing_left {
                -self.angle
            } else {
                self.angle
            };
            context.draw_image(
                (self.xpos as i32, self.ypos as i32),
                &assets::SPR_PLAYER_BOW_DRAWN,
                angle,
                self.facing_left,
            );
        } else {
            let arms_image = if self.is_running {
                match self.run_frame {
                    0 => &assets::SPR_PLAYER_ARMS_RUN1,
                    1 => &assets::SPR_PLAYER_ARMS_RUN2,
                    2 => &assets::SPR_PLAYER_ARMS_RUN3,
                    _ => &assets::SPR_PLAYER_ARMS_RUN1,
                }
            } else {
                &assets::SPR_PLAYER_ARMS_IDLE
            };

            context.draw_image(
                (self.xpos as i32, self.ypos as i32),
                arms_image,
                0.0,
                self.facing_left,
            );
        }
    }

    fn is_live(&self) -> bool {
        true
    }

    fn get_bounding_box(&self) -> util::Rect<i32> {
        if self.killed {
            // This affects how subsequent arrows that hit the corpse are
            // displayed.
            util::Rect::<i32>::new(self.xpos as i32 - 32, self.ypos as i32 + 40, 64, 14)
        } else {
            // We only include the torso
            util::Rect::<i32>::new(self.xpos as i32 - 5, self.ypos as i32 - 5, 10, 15)
        }
    }

    fn get_collision_class(&self) -> u32 {
        COLL_PLAYER
    }

    fn get_collision_mask(&self) -> u32 {
        COLL_MISSILE
    }

    fn collide(&mut self, _other: &(dyn entity::Entity)) {
        // XXX check type
        if !self.killed {
            self.killed = true;
            audio::play_effect(assets::SFX_DEATH);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Balloon {
    xpos: f32,
    ypos: f32,
    buoyancy: f32,
    popped: bool,
}

impl Balloon {
    pub fn new(x: f32, y: f32) -> Balloon {
        Balloon {
            xpos: x,
            ypos: y,
            buoyancy: 0.0,
            popped: false,
        }
    }
}

impl entity::Entity for Balloon {
    fn update(
        &mut self,
        d_t: f32,
        _new_entities: &mut Vec<Box<dyn entity::Entity>>,
        _buttons: u32,
        _tile_map: &tilemap::TileMap,
    ) {
        self.buoyancy += d_t;
        self.ypos += self.buoyancy.sin() * 0.5;
    }

    fn draw(&self, context: &mut gfx::RenderContext) {
        context.draw_image(
            (self.xpos as i32, self.ypos as i32),
            &assets::SPR_BALLOON,
            0.0,
            false,
        );
    }

    fn is_live(&self) -> bool {
        !self.popped
    }

    fn get_collision_class(&self) -> u32 {
        COLL_OBJ
    }

    fn get_collision_mask(&self) -> u32 {
        COLL_MISSILE
    }

    fn get_bounding_box(&self) -> util::Rect<i32> {
        util::Rect::<i32>::new(self.xpos as i32 - 10, self.ypos as i32 - 15, 20, 30)
    }

    fn collide(&mut self, _other: &dyn entity::Entity) {
        self.popped = true;
        audio::play_effect(assets::SFX_POP);
        // XXX start an animation
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
