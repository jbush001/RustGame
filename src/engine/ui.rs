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

pub struct Interpolator {
    t: f32,
    max_t: f32,
    start_value: f32,
    end_value: f32,
    multiplier: f32,
    easing_fn: fn(f32) -> f32,
}

pub fn cubic_inout(x: f32) -> f32 {
    if x < 0.5 { 4.0 * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powf(3.0) / 2.0 }
}

impl Interpolator {
    pub fn new(value: f32, easing_fn: fn(f32) -> f32) -> Interpolator {
        Interpolator {
            t: 1.0,
            max_t: 1.0,
            start_value: value,
            end_value: value,
            multiplier: 0.0,
            easing_fn,
        }
    }

    pub fn start(&mut self, time: f32, start: f32, end: f32) {
        self.start_value = start;
        self.end_value = end;
        self.multiplier = 1.0 / (end - start);
        self.t = 0.0;
        self.max_t = time;
    }

    pub fn update(&mut self, d_t: f32) -> f32 {
        if self.t < self.max_t {
            self.t = self.t + d_t;
            if self.t > self.max_t {
                self.t = self.max_t;
            }

            let x = self.t / self.max_t;
            self.start_value + (self.easing_fn)(x) * self.multiplier
        } else {
            self.end_value
        }
    }
}

// 012
// 345
// 678
pub fn draw_nine_tile(context: &mut gfx::RenderContext, left: i32, top: i32, width: i32, height: i32, assets: &[gfx::SpriteInfo; 9]) {
    const TILE_SIZE: i32 = 20;

    let right = left + width;
    let bottom = top + height;
    let inner_left = left + TILE_SIZE;
    let inner_right = right - TILE_SIZE;
    let inner_top = top + TILE_SIZE;
    let inner_bottom = bottom - TILE_SIZE;

    context.draw_quad(
        (left, top),
        (inner_left, top),
        (left, inner_top),
        (inner_left, inner_top),
        assets[0].0,
        assets[0].1,
        assets[0].2,
        assets[0].3,
    );

    context.draw_quad(
        (inner_left, top),
        (inner_right, top),
        (inner_left, inner_top),
        (inner_right, inner_top),
        assets[1].0,
        assets[1].1,
        assets[1].2,
        assets[1].3,
    );

    context.draw_quad(
        (inner_right, top),
        (right, top),
        (inner_right, inner_top),
        (right, inner_top),
        assets[2].0,
        assets[2].1,
        assets[2].2,
        assets[2].3,
    );

    context.draw_quad(
        (left, inner_top),
        (inner_left, inner_top),
        (left, inner_bottom),
        (inner_left, inner_bottom),
        assets[3].0,
        assets[3].1,
        assets[3].2,
        assets[3].3,
    );

    context.draw_quad(
        (inner_left, inner_top),
        (inner_right, inner_top),
        (inner_left, inner_bottom),
        (inner_right, inner_bottom),
        assets[4].0,
        assets[4].1,
        assets[4].2,
        assets[4].3,
    );

    context.draw_quad(
        (inner_right, inner_top),
        (right, inner_top),
        (inner_right, inner_bottom),
        (right, inner_bottom),
        assets[5].0,
        assets[5].1,
        assets[5].2,
        assets[5].3,
    );

    context.draw_quad(
        (left, inner_bottom),
        (inner_left, inner_bottom),
        (left, bottom),
        (inner_left, bottom),
        assets[6].0,
        assets[6].1,
        assets[6].2,
        assets[6].3,
    );

    context.draw_quad(
        (inner_left, inner_bottom),
        (inner_right, inner_bottom),
        (inner_left, bottom),
        (inner_right, bottom),
        assets[7].0,
        assets[7].1,
        assets[7].2,
        assets[7].3,
    );

    context.draw_quad(
        (inner_right, inner_bottom),
        (right, inner_bottom),
        (inner_right, bottom),
        (right, bottom),
        assets[8].0,
        assets[8].1,
        assets[8].2,
        assets[8].3,
    );
}
