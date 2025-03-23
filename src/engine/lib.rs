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
pub mod ui;
pub mod util;
extern crate sdl2;
use std::collections::HashMap;

pub const LEFT_SCROLL_BOUNDARY: i32 = gfx::WINDOW_WIDTH / 3;
pub const RIGHT_SCROLL_BOUNDARY: i32 = gfx::WINDOW_WIDTH * 2 / 3;
pub const TOP_SCROLL_BOUNDARY: i32 = gfx::WINDOW_HEIGHT / 3;
pub const BOTTOM_SCROLL_BOUNDARY: i32 = gfx::WINDOW_HEIGHT * 2 / 3;

pub type EntityCreateFn = fn(i32, i32) -> Box<dyn entity::Entity>;

pub struct GameEngine {
    _sdl: sdl2::Sdl,
    pub render_context: gfx::RenderContext,
    pub tile_map: tilemap::TileMap,
    event_pump: sdl2::EventPump,
    pub entities: Vec<Box<dyn entity::Entity>>,
    pub max_x_scroll: i32,
    pub max_y_scroll: i32,
    pub entity_fns: HashMap<String, EntityCreateFn>,
    pub buttons: u32,
    pub quit: bool,
}

fn get_key_mask(key: sdl2::keyboard::Keycode) -> u32 {
    match key {
        sdl2::keyboard::Keycode::Up => entity::CONTROL_UP,
        sdl2::keyboard::Keycode::Down => entity::CONTROL_DOWN,
        sdl2::keyboard::Keycode::Left => entity::CONTROL_LEFT,
        sdl2::keyboard::Keycode::Right => entity::CONTROL_RIGHT,
        sdl2::keyboard::Keycode::X => entity::CONTROL_FIRE,
        sdl2::keyboard::Keycode::Z => entity::CONTROL_JUMP,
        sdl2::keyboard::Keycode::Escape => entity::CONTROL_MENU,
        _ => 0,
    }
}

impl GameEngine {
    pub fn new(audio_file_list: &[&str]) -> GameEngine {
        let sdl = sdl2::init().unwrap();
        audio::init_audio(audio_file_list);

        GameEngine {
            render_context: gfx::RenderContext::new(&sdl),
            tile_map: tilemap::TileMap::default(),
            event_pump: sdl.event_pump().unwrap(),
            entities: Vec::new(),
            _sdl: sdl,
            max_x_scroll: 0,
            max_y_scroll: 0,
            entity_fns: HashMap::new(),
            buttons: 0,
            quit: false,
        }
    }

    pub fn register_entity(&mut self, name: &str, create_fn: EntityCreateFn) {
        self.entity_fns.insert(name.to_string(), create_fn);
    }

    pub fn load_tile_map(&mut self, file_name: &str) {
        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();
        let tile_map_path = exe_dir.join(file_name);
        self.tile_map = tilemap::TileMap::new(&tile_map_path);
        self.max_x_scroll = self.tile_map.width * tilemap::TILE_SIZE - gfx::WINDOW_WIDTH;
        self.max_y_scroll = self.tile_map.height * tilemap::TILE_SIZE - gfx::WINDOW_HEIGHT;
    }

    pub fn create_entities(&mut self) {
        self.entities
            .extend(self.tile_map.objects.iter().map(|(name, x, y)| {
                let create_fn = self.entity_fns.get(name).unwrap();
                create_fn(*x, *y)
            }));
    }

    // This needs to be called before run, as the player is the first entity in the list.
    pub fn spawn_player(&mut self, create_fn: EntityCreateFn) {
        let entity = create_fn(self.tile_map.player_start_x, self.tile_map.player_start_y);
        self.entities.push(entity);
    }

    pub fn poll_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => self.quit = true,
                sdl2::event::Event::KeyDown {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => {
                    self.buttons |= get_key_mask(keycode);
                }

                sdl2::event::Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    self.buttons &= !get_key_mask(keycode);
                }

                _ => {}
            }
        }
    }
}
