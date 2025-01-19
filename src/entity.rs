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

static mut CONTROL_BITMAP: u32 = 0x0;

pub const CONTROL_UP: u32 = 0x1;
pub const CONTROL_DOWN: u32 = 0x2;
pub const CONTROL_LEFT: u32 = 0x4;
pub const CONTROL_RIGHT: u32 = 0x8;
pub const CONTROL_FIRE: u32 = 0x10;

pub fn set_control_bitmap(mask: u32) {
    unsafe {
        CONTROL_BITMAP |= mask;
    }
}

pub fn clear_control_bitmap(mask: u32) {
    unsafe {
        CONTROL_BITMAP &= !mask;
    }
}

pub fn get_control_bitmap() -> u32 {
    unsafe {
        CONTROL_BITMAP
    }
}

pub trait Entity {
    fn update(&mut self, d_t: f32, new_entities: &mut Vec<Box<dyn Entity>>);
    fn draw(&mut self, context: &mut gfx::RenderContext);
    fn is_live(&self) -> bool;
}

pub struct Arrow {
    xpos: f32,
    ypos: f32,
    xvec: f32,
    yvec: f32,
    angle: f32,
}

impl Arrow {
    pub fn new(xpos: f32, ypos: f32, angle: f32, velocity: f32) -> Arrow {
        Arrow {
            xpos,
            ypos,
            xvec: angle.cos() * velocity,
            yvec: angle.sin() * velocity,
            angle,
        }
    }
}

impl Entity for Arrow {
    fn update(&mut self, d_t: f32, new_entities: &mut Vec<Box<dyn Entity>>) {
        self.xpos += self.xvec * d_t;
        self.ypos += self.yvec * d_t;
        self.angle = self.yvec.atan2(self.xvec);
        self.yvec += 400.0 * d_t;
    }

    fn draw(&mut self, context: &mut gfx::RenderContext) {
        context.draw_image(
            (self.xpos as i32, self.ypos as i32),
            &gfx::SPR_ARROW,
            self.angle,
            (gfx::SPR_ARROW.4 as i32/ 2, gfx::SPR_ARROW.5 as i32 / 2)
        );
    }

    fn is_live(&self) -> bool {
        self.ypos < gfx::WINDOW_HEIGHT as f32
            && self.xpos > 0.0
            && self.xpos < gfx::WINDOW_WIDTH as f32
    }
}

pub struct Player {
    angle: f32,
    pos_x: f32,
    pos_y: f32,
    bow_drawn: bool,
    bow_draw_time: f32,
}

impl Player {
    pub fn new() -> Player {
        Player {
            angle: -std::f32::consts::PI * 3.0 / 4.0,
            pos_x: 20.0,
            pos_y: (gfx::WINDOW_HEIGHT - 20) as f32,
            bow_drawn: false,
            bow_draw_time: 0.0,
        }
    }
}

impl Entity for Player {
    fn update(&mut self, _d_t: f32, new_entities: &mut Vec<Box<dyn Entity>>) {
        if get_control_bitmap() & CONTROL_FIRE == 0 {
            // Button not pressed
            if self.bow_drawn {
                // It was released
                let velocity = self.bow_draw_time.clamp(0.2, 0.4) * 2000.0;
                new_entities.push(Box::new(Arrow::new(self.pos_x, self.pos_y, self.angle, velocity)));
                self.bow_drawn = false;
            }
        } else {
            // Button pressed
            if self.bow_drawn {
                self.bow_draw_time += _d_t;
            } else {
                self.bow_drawn = true;
                self.bow_draw_time = 0.0;
            }
        }

        if get_control_bitmap() & CONTROL_UP != 0 {
            self.angle -= 0.1;
        }

        if get_control_bitmap() & CONTROL_DOWN != 0 {
            self.angle += 0.1;
        }

        if get_control_bitmap() & CONTROL_LEFT != 0 {
            self.pos_x -= 100.0 * _d_t;
        }

        if get_control_bitmap() & CONTROL_RIGHT != 0 {
            self.pos_x += 100.0 * _d_t;
        }
    }

    fn draw(&mut self, context: &mut gfx::RenderContext) {
        context.draw_image(
            (self.pos_x as i32, self.pos_y as i32),
            &gfx::SPR_ARROW,
            self.angle,
            (gfx::SPR_ARROW.4 as i32 / 2, gfx::SPR_ARROW.5 as i32 / 2)
        );
    }

    fn is_live(&self) -> bool {
        true
    }
}

pub fn do_frame(entities: &mut Vec<Box<dyn Entity>>, d_t: f32, context: &mut gfx::RenderContext) {
    let mut new_entities: Vec<Box<dyn Entity>> = Vec::new();
    for entity in entities.iter_mut() {
        entity.update(d_t, &mut new_entities);
    }

    entities.append(&mut new_entities);
    entities.retain(|entity| entity.is_live());

    for entity in entities.iter_mut() {
        entity.draw(context);
    }
}

