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

pub mod audio;
pub mod entity;
pub mod gfx;
pub mod tilemap;
extern crate sdl2;

const LEFT_SCROLL_BOUNDARY: i32 = gfx::WINDOW_WIDTH as i32 / 4;
const RIGHT_SCROLL_BOUNDARY: i32 = gfx::WINDOW_WIDTH as i32 * 3 / 4;
const TOP_SCROLL_BOUNDARY: i32 = gfx::WINDOW_HEIGHT as i32 / 4;
const BOTTOM_SCROLL_BOUNDARY: i32 = gfx::WINDOW_HEIGHT as i32 * 3 / 4;

pub struct GameEngine {
    _sdl: sdl2::Sdl,
    context: gfx::RenderContext,
    tile_map: tilemap::TileMap,
    event_pump: sdl2::EventPump,
    entities: Vec<Box<dyn entity::Entity>>,
}

fn get_key_mask(key: sdl2::keyboard::Keycode) -> u32 {
    match key {
        sdl2::keyboard::Keycode::Up => entity::CONTROL_UP,
        sdl2::keyboard::Keycode::Down => entity::CONTROL_DOWN,
        sdl2::keyboard::Keycode::Left => entity::CONTROL_LEFT,
        sdl2::keyboard::Keycode::Right => entity::CONTROL_RIGHT,
        sdl2::keyboard::Keycode::X => entity::CONTROL_FIRE,
        sdl2::keyboard::Keycode::Z => entity::CONTROL_JUMP,
        _ => 0,
    }
}

impl GameEngine {
    pub fn new() -> GameEngine {
        let mut sdl = sdl2::init().unwrap();
        audio::init_audio();

        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();
        let tile_map_path = exe_dir.join("map.bin");

        GameEngine {
            context: gfx::RenderContext::new(&mut sdl).unwrap(),
            tile_map: tilemap::TileMap::new(&tile_map_path),
            event_pump: sdl.event_pump().unwrap(),
            entities: Vec::new(),
            _sdl: sdl,
        }
    }

    pub fn run(&mut self) {
        let mut buttons: u32 = 0;
        let mut x_scroll: i32 = 0;
        let mut y_scroll: i32 = 0;

        'main: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => break 'main,
                    sdl2::event::Event::KeyDown {
                        keycode: Some(keycode),
                        repeat: false,
                        ..
                    } => {
                        buttons |= get_key_mask(keycode);
                    }

                    sdl2::event::Event::KeyUp {
                        keycode: Some(keycode),
                        ..
                    } => {
                        buttons &= !get_key_mask(keycode);
                    }

                    _ => {}
                }
            }

            let player_rect = self.entities[0].get_bounding_box();
            let right = (player_rect.0 + player_rect.2) as i32;
            let bottom = (player_rect.1 + player_rect.3) as i32;

            if right > x_scroll + RIGHT_SCROLL_BOUNDARY {
                x_scroll = right - RIGHT_SCROLL_BOUNDARY;
            } else if (player_rect.0 as i32) < x_scroll + LEFT_SCROLL_BOUNDARY {
                x_scroll = std::cmp::max(0, (player_rect.0 as i32) - LEFT_SCROLL_BOUNDARY);
            }

            if bottom > y_scroll + BOTTOM_SCROLL_BOUNDARY {
                y_scroll = bottom - BOTTOM_SCROLL_BOUNDARY;
            } else if (player_rect.1 as i32) < y_scroll + TOP_SCROLL_BOUNDARY {
                y_scroll = std::cmp::max(0, (player_rect.1 as i32) - TOP_SCROLL_BOUNDARY);
            }

            self.context.set_offset(x_scroll, y_scroll);

            let visible_rect = (
                x_scroll,
                y_scroll,
                gfx::WINDOW_WIDTH as i32,
                gfx::WINDOW_HEIGHT as i32,
            );

            self.tile_map.draw(&mut self.context, &visible_rect);
            entity::do_frame(
                &mut self.entities,
                1.0 / 60.0,
                &mut self.context,
                buttons,
                &self.tile_map,
                &visible_rect,
            );

            self.context.render();
        }
    }

    pub fn spawn_entity(&mut self, ent: Box<dyn entity::Entity>) {
        self.entities.push(ent);
    }
}
