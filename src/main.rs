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

mod gfx;
extern crate sdl2;


fn main() {
    let mut sdl = sdl2::init().unwrap();
    let mut context = gfx::RenderContext::new(&mut sdl).unwrap();

    let mut event_pump = sdl.event_pump().unwrap();
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut xdir: i32 = 1;
    let mut ydir: i32 = 1;
    let mut rot: f32 = 0.0;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {},
            }
        }

        x += xdir * 5;
        y += ydir * 5;
        if x > gfx::WINDOW_WIDTH as i32 || x < 0 {
            xdir = -xdir;
        }

        if y > gfx::WINDOW_HEIGHT as i32 || y < 0 {
            ydir = -ydir;
        }
        context.draw_image((x, y), &gfx::SPRITE_0, rot, (gfx::SPRITE_0.4 as i32/ 2, gfx::SPRITE_0.5 as i32 / 2));
        rot += 0.02;
        context.render();
    }
}
