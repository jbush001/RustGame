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

mod assets;
mod entities;
use engine::{audio, entity, gfx, ui, util, GameEngine};

fn main() {
    let mut eng = GameEngine::new(&assets::AUDIO_FILE_LIST);
    for (name, ctor) in entities::ENTITY_LIST {
        eng.register_entity(name, *ctor);
    }

    eng.load_tile_map("map.bin");

    eng.spawn_player(|x, y| Box::new(entities::Player::new(x as f32, y as f32)));

    let _temp = audio::play_music("music_track1.mp3");

    const NINE_TILE: [gfx::SpriteInfo; 9] = [
        assets::SPR_9TILE_A,
        assets::SPR_9TILE_B,
        assets::SPR_9TILE_C,
        assets::SPR_9TILE_D,
        assets::SPR_9TILE_E,
        assets::SPR_9TILE_F,
        assets::SPR_9TILE_G,
        assets::SPR_9TILE_H,
        assets::SPR_9TILE_I,
    ];

    let mut x_scroll: i32 = 0;
    let mut y_scroll: i32 = 0;
    let mut new_entities: Vec<Box<dyn entity::Entity>> = Vec::new();
    let mut menu_open = false;
    let mut menu_anim = ui::Interpolator::new(0.0, ui::cubic_inout);

    // XXX Ideally these would be spawned dynamically as the user moves into new
    // areas
    eng.create_entities();

    let mut old_menu_pressed = false;

    loop {
        eng.poll_events();

        if eng.quit {
            break;
        }

        let menu_pressed = (eng.buttons & entity::CONTROL_MENU) != 0;
        if menu_pressed && !old_menu_pressed {
            menu_open = !menu_open;
            if menu_open {
                audio::play_effect(assets::SFX_PAUSE);
                audio::pause_music();
                menu_anim.start(0.4, 0.0, 1.0);
            } else {
                audio::resume_music();
            }
        }

        old_menu_pressed = menu_pressed;

        // Ideally we compute this dynamically, but there are complications
        // because the first few calls to poll events in SDL take a
        // significantly longer period of time.
        const D_T: f32 = 1.0 / 60.0;

        if !menu_open {
            let player_rect = eng.entities[0].get_bounding_box();
            if player_rect.right() > x_scroll + engine::RIGHT_SCROLL_BOUNDARY {
                x_scroll = std::cmp::min(
                    player_rect.right() - engine::RIGHT_SCROLL_BOUNDARY,
                    eng.max_x_scroll,
                );
            } else if player_rect.left < x_scroll + engine::LEFT_SCROLL_BOUNDARY {
                x_scroll = std::cmp::max(0, player_rect.left - engine::LEFT_SCROLL_BOUNDARY);
            }

            if player_rect.bottom() > y_scroll + engine::BOTTOM_SCROLL_BOUNDARY {
                y_scroll = std::cmp::min(
                    player_rect.bottom() - engine::BOTTOM_SCROLL_BOUNDARY,
                    eng.max_y_scroll,
                );
            } else if player_rect.top < y_scroll + engine::TOP_SCROLL_BOUNDARY {
                y_scroll = std::cmp::max(0, player_rect.top - engine::TOP_SCROLL_BOUNDARY);
            }

            eng.render_context.set_offset(x_scroll, y_scroll);

            entity::handle_collisions(&mut eng.entities);
            eng.entities.iter_mut().for_each(|entity| {
                entity.update(
                    D_T,
                    &mut new_entities,
                    eng.buttons,
                    &eng.tile_map,
                    &player_rect,
                );
            });

            eng.entities.append(&mut new_entities);
            new_entities.clear();

            // XXX despawn things that are too far outsize visible rect
            eng.entities.retain(|entity| entity.is_live());
        }

        let visible_rect =
            util::Rect::<i32>::new(x_scroll, y_scroll, gfx::WINDOW_WIDTH, gfx::WINDOW_HEIGHT);

        eng.tile_map.draw(&mut eng.render_context, &visible_rect);

        eng.entities.iter().for_each(|entity| {
            entity.draw(&mut eng.render_context);
        });

        if menu_open {
            let scale = menu_anim.update(D_T);

            ui::draw_nine_tile(
                &mut eng.render_context,
                50,
                20,
                40 + (scale * 250.0) as i32,
                40 + (scale * 350.0) as i32,
                &NINE_TILE,
            );
        }

        eng.render_context.render();
    }
}
