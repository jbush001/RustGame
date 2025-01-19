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
mod entity;
extern crate sdl2;

fn main() {
    let mut sdl = sdl2::init().unwrap();
    let mut context = gfx::RenderContext::new(&mut sdl).unwrap();

    let mut event_pump = sdl.event_pump().unwrap();
    let mut entities: Vec<Box<dyn entity::Entity>> = Vec::new();

    // start at 45 degrees
    let mut fire_angle: f32 = -std::f32::consts::PI / 4.0;
    const fire_pos_x: f32 = 20.0;
    const fire_pos_y: f32 = (gfx::WINDOW_HEIGHT - 10) as f32;

    let mut bow_draw_time: u32 = 0;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,

                // When the space key is held, the bow is drawn, when released it is
                // fired. The duration of hold determines the distance. Record
                // the time. Grab timestamp from event type. the repeat field should be false.
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Space), repeat: false, timestamp, .. } => {
                    println!("Drawing bow at time: {}", timestamp);
                    bow_draw_time = timestamp;
                }

                sdl2::event::Event::KeyUp { keycode: Some(sdl2::keyboard::Keycode::Space), timestamp, .. } => {
                    let elapsed = (timestamp - bow_draw_time) as f32 / 1000.0;
                    let velocity = elapsed.clamp(0.2, 0.3) * 2000.0;
                    println!("Firing arrow at time {} with velocity: {}", elapsed, velocity);
                    entities.push(Box::new(entity::Arrow::new(fire_pos_x, fire_pos_y, fire_angle, velocity)));
                }

                // Adjust the firing angle with the up and down arrow keys
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Up), .. } => {
                    fire_angle -= 0.1;
                }

                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Down), .. } => {
                    fire_angle += 0.1;
                }

                _ => {},
            }
        }

        // Draw an arrow to show angle
        context.draw_image(
            (fire_pos_x as i32, fire_pos_y as i32),
            &gfx::SPR_ARROW,
            fire_angle,
            (gfx::SPR_ARROW.4 as i32 / 2, gfx::SPR_ARROW.5 as i32 / 2)
        );

        entity::do_frame(&mut entities, 1.0 / 60.0, &mut context);
        context.render();
    }
}
