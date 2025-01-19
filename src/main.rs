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

    entities.push(Box::new(entity::Player::new()));

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,

                sdl2::event::Event::KeyDown { keycode: Some(keycode), repeat: false, .. } => {
                    match keycode {
                        sdl2::keyboard::Keycode::Space => {
                            entity::set_control_bitmap(entity::CONTROL_FIRE);
                        }
                        sdl2::keyboard::Keycode::Up => {
                            entity::set_control_bitmap(entity::CONTROL_UP);
                        }
                        sdl2::keyboard::Keycode::Down => {
                            entity::set_control_bitmap(entity::CONTROL_DOWN);
                        }
                        sdl2::keyboard::Keycode::Left => {
                            entity::set_control_bitmap(entity::CONTROL_LEFT);
                        }
                        sdl2::keyboard::Keycode::Right => {
                            entity::set_control_bitmap(entity::CONTROL_RIGHT);
                        }
                        _ => {},
                    }
                }

                sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } => {
                    match keycode {
                        sdl2::keyboard::Keycode::Space => {
                            entity::clear_control_bitmap(entity::CONTROL_FIRE);
                        }
                        sdl2::keyboard::Keycode::Up => {
                            entity::clear_control_bitmap(entity::CONTROL_UP);
                        }
                        sdl2::keyboard::Keycode::Down => {
                            entity::clear_control_bitmap(entity::CONTROL_DOWN);
                        }
                        sdl2::keyboard::Keycode::Left => {
                            entity::clear_control_bitmap(entity::CONTROL_LEFT);
                        }
                        sdl2::keyboard::Keycode::Right => {
                            entity::clear_control_bitmap(entity::CONTROL_RIGHT);
                        }
                        _ => {},
                    }
                }

                _ => {},
            }
        }

        entity::do_frame(&mut entities, 1.0 / 60.0, &mut context);
        context.render();
    }
}
