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
use engine::{audio, GameEngine};

fn main() {
    let mut eng = GameEngine::new(&assets::AUDIO_FILE_LIST);
    for (name, ctor) in entities::ENTITY_LIST {
        eng.register_entity(name, *ctor);
    }

    eng.load_tile_map("map.bin");

    eng.spawn_player(|x, y| Box::new(entities::Player::new(x as f32, y as f32)));

    let _temp = audio::play_music("music_track1.mp3");
    eng.run();
}
