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
use engine::*;

fn main() {
    let mut eng = GameEngine::new(&assets::AUDIO_FILE_LIST);
    // XXX player always needs to be first entity spawned
    eng.spawn_entity(Box::new(entities::Player::new(128.0, 320.0)));
    eng.spawn_entity(Box::new(entities::Balloon::new(512.0, 64.0)));
    eng.load_tile_map("initial_map.bin");
    let _temp = audio::play_music("music_track1.mp3");
    eng.run();
}
