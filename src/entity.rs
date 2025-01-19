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

pub trait Entity {
    fn update(&mut self, d_t: f32);
    fn draw(&mut self, context: &mut gfx::RenderContext);
}

pub struct Arrow {
    xpos: f32,
    ypos: f32,
    xvec: f32,
    yvec: f32,
    angle: f32,
    active: bool,
}

impl Arrow {
    pub fn new(xpos: f32, ypos: f32, angle: f32, velocity: f32) -> Arrow {
        Arrow {
            xpos,
            ypos,
            xvec: angle.cos() * velocity,
            yvec: angle.sin() * velocity,
            angle,
            active: true,
        }
    }
}

impl Entity for Arrow {
    fn update(&mut self, d_t: f32) {
        if !self.active {
            return;
        }

        self.xpos += self.xvec * d_t;
        self.ypos += self.yvec * d_t;
        self.angle = self.yvec.atan2(self.xvec);
        self.yvec += 400.0 * d_t;

        if self.ypos > gfx::WINDOW_HEIGHT as f32
            || self.xpos < 0.0
            || self.xpos > gfx::WINDOW_WIDTH as f32
        {
            self.active = false;
        }
    }

    fn draw(&mut self, context: &mut gfx::RenderContext) {
        if !self.active {
            return;
        }

        context.draw_image(
            (self.xpos as i32, self.ypos as i32),
            &gfx::SPR_ARROW,
            self.angle,
            (gfx::SPR_ARROW.4 as i32/ 2, gfx::SPR_ARROW.5 as i32 / 2)
        );
    }
}

pub fn do_frame(entities: &mut Vec<Box<dyn Entity>>, d_t: f32, context: &mut gfx::RenderContext) {
    for entity in entities.iter_mut() {
        entity.update(d_t);
    }

    for entity in entities.iter_mut() {
        entity.draw(context);
    }
}

