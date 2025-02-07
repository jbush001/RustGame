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

use engine::entity;
use engine::audio;
use engine::gfx;
use engine::tilemap;
use crate::assets;
use std::any::Any;

pub const GRAVITY: f32 = 1500.0;

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

    fn draw(&mut self, context: &mut gfx::RenderContext) {
        context.draw_image(
            (self.xpos as i32, self.ypos as i32),
            &assets::SPR_ARROW,
            self.angle + self.wobble.sin() * 0.1,
            (assets::SPR_ARROW.4 as i32 / 2, assets::SPR_ARROW.5 as i32 / 2),
            false,
        );
    }

    fn is_live(&self) -> bool {
        !self.collided
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
        entity::COLL_MISSILE
    }

    fn get_collision_mask(&self) -> u32 {
        !entity::COLL_MISSILE
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
    pub xpos: f32,
    pub ypos: f32,
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
}

const RUN_FRAME_DURATION: f32 = 0.1;
const MAX_JUMP_COUNTER: u32 = 5;

impl Player {
    pub fn new() -> Player {
        Player {
            angle: -std::f32::consts::PI / 4.0,
            xpos: 128.0,
            ypos: 128.0,
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

        const GROUND_OFFSET: i32 = 44;
        self.on_ground = tile_map.is_solid(self.xpos as i32 - 12, self.ypos as i32 + GROUND_OFFSET)
            || tile_map.is_solid(self.xpos as i32 + 12, self.ypos as i32 + GROUND_OFFSET);

        if self.on_ground {
            if buttons & entity::CONTROL_JUMP != 0 && !self.last_jump_button {
                self.yvec = -100.0;
                self.jump_counter = 0;
            } else {
                self.yvec = 0.0;

                // Ensure it is on the ground.
                self.ypos = (self.ypos / 64.0).floor() * 64.0 + (64.0 - GROUND_OFFSET as f32);
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

        self.last_jump_button = buttons & entity::CONTROL_JUMP != 0;
        self.ypos += self.yvec * d_t;

        // Movement
        if buttons & entity::CONTROL_LEFT != 0
            && !tile_map.is_solid(self.xpos as i32 - 16, self.ypos as i32 + GROUND_OFFSET - 3)
            && !tile_map.is_solid(self.xpos as i32 - 16, self.ypos as i32 - 15)
        {
            self.xpos -= 150.0 * d_t;
            self.facing_left = true;
            self.is_running = self.on_ground;
        } else if buttons & entity::CONTROL_RIGHT != 0
            && !tile_map.is_solid(self.xpos as i32 + 16, self.ypos as i32 + GROUND_OFFSET - 3)
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

    fn draw(&mut self, context: &mut gfx::RenderContext) {
        if self.killed {
            context.draw_image(
                (self.xpos as i32, self.ypos as i32),
                &assets::SPR_PLAYER_DEAD,
                0.0,
                (33, 20),
                false,
            );
            return;
        }

        if !self.bow_drawn {
            // Draw bow on back
            context.draw_image(
                (self.xpos as i32, self.ypos as i32),
                &assets::SPR_BOW_ON_BACK,
                0.0,
                (33, 20),
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
                (self.xpos as i32, self.ypos as i32),
                &assets::SPR_BOW_DRAWN,
                angle,
                (33, 20),
                self.facing_left,
            );
        } else {
            let arms_image = if self.is_running {
                match self.run_frame {
                    0 => &assets::SPR_ARMS_RUN1,
                    1 => &assets::SPR_ARMS_RUN2,
                    2 => &assets::SPR_ARMS_RUN3,
                    _ => &assets::SPR_ARMS_RUN1,
                }
            } else {
                &assets::SPR_ARMS_IDLE
            };

            context.draw_image(
                (self.xpos as i32, self.ypos as i32),
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
            (self.xpos - 32.0, self.ypos + 40.0, 64.0, 14.0)
        } else {
            // We only include the torso
            (self.xpos - 5.0, self.ypos - 5.0, 10.0, 15.0)
        }
    }

    fn get_collision_class(&self) -> u32 {
        entity::COLL_PLAYER
    }

    fn get_collision_mask(&self) -> u32 {
        !entity::COLL_PLAYER
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
