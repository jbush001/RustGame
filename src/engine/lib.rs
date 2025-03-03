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
pub mod util;
extern crate sdl2;
use std::collections::HashMap;

const LEFT_SCROLL_BOUNDARY: i32 = gfx::WINDOW_WIDTH / 3;
const RIGHT_SCROLL_BOUNDARY: i32 = gfx::WINDOW_WIDTH * 2 / 3;
const TOP_SCROLL_BOUNDARY: i32 = gfx::WINDOW_HEIGHT / 3;
const BOTTOM_SCROLL_BOUNDARY: i32 = gfx::WINDOW_HEIGHT * 2 / 3;

pub type EntityCreateFn = fn(i32, i32) -> Box<dyn entity::Entity>;

pub struct GameEngine {
    _sdl: sdl2::Sdl,
    render_context: gfx::RenderContext,
    tile_map: tilemap::TileMap,
    event_pump: sdl2::EventPump,
    entities: Vec<Box<dyn entity::Entity>>,
    max_x_scroll: i32,
    max_y_scroll: i32,
    entity_fns: HashMap<String, EntityCreateFn>,
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

    fn create_entities(&mut self) {
        for (name, x, y) in self.tile_map.objects.clone() {
            let create_fn = self.entity_fns.get(&name).unwrap();
            let entity = create_fn(x, y);
            self.entities.push(entity);
        }
    }

    pub fn run(&mut self) {
        let mut buttons: u32 = 0;
        let mut x_scroll: i32 = 0;
        let mut y_scroll: i32 = 0;
        let mut new_entities: Vec<Box<dyn entity::Entity>> = Vec::new();

        // XXX Ideally these would be spawned dynamically as the user moves into new
        // areas
        self.create_entities();

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
            if player_rect.right() > x_scroll + RIGHT_SCROLL_BOUNDARY {
                x_scroll = std::cmp::min(
                    player_rect.right() - RIGHT_SCROLL_BOUNDARY,
                    self.max_x_scroll,
                );
            } else if player_rect.left < x_scroll + LEFT_SCROLL_BOUNDARY {
                x_scroll = std::cmp::max(0, player_rect.left - LEFT_SCROLL_BOUNDARY);
            }

            if player_rect.bottom() > y_scroll + BOTTOM_SCROLL_BOUNDARY {
                y_scroll = std::cmp::min(
                    player_rect.bottom() - BOTTOM_SCROLL_BOUNDARY,
                    self.max_y_scroll,
                );
            } else if player_rect.top < y_scroll + TOP_SCROLL_BOUNDARY {
                y_scroll = std::cmp::max(0, player_rect.top - TOP_SCROLL_BOUNDARY);
            }

            self.render_context.set_offset(x_scroll, y_scroll);

            let visible_rect =
                util::Rect::<i32>::new(x_scroll, y_scroll, gfx::WINDOW_WIDTH, gfx::WINDOW_HEIGHT);

            self.tile_map.draw(&mut self.render_context, &visible_rect);

            // Ideally we compute this dynamically, but there are complications
            // because the first few calls to poll events in SDL take a
            // significantly longer period of time.
            const D_T: f32 = 1.0 / 60.0;

            entity::handle_collisions(&mut self.entities);
            for entity in self.entities.iter_mut() {
                entity.update(
                    D_T,
                    &mut new_entities,
                    buttons,
                    &self.tile_map,
                    &player_rect,
                );
            }

            self.entities.append(&mut new_entities);
            new_entities.clear();

            // XXX despawn things that are too far outsize visible rect
            self.entities.retain(|entity| entity.is_live());

            for entity in self.entities.iter() {
                entity.draw(&mut self.render_context);
            }

            self.render_context.render();
        }
    }

    // This needs to be called before run, as the player is the first entity in the list.
    pub fn spawn_player(&mut self, create_fn: EntityCreateFn) {
        let entity = create_fn(self.tile_map.player_start_x, self.tile_map.player_start_y);
        self.entities.push(entity);
    }
}
