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

extern crate sdl2;

use sdl2::mixer;

static mut EFFECTS: Vec<mixer::Chunk> = Vec::new();


pub fn init_audio(audio_file_list: &[&str]) {
    mixer::open_audio(44100, mixer::AUDIO_S16LSB, mixer::DEFAULT_CHANNELS, 1024).unwrap();
    mixer::init(mixer::InitFlag::MP3).unwrap();

    mixer::allocate_channels(4);

    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    for path in audio_file_list {
        unsafe { EFFECTS.push(mixer::Chunk::from_file(exe_dir.join(path)).unwrap()); }
    }
}

pub fn play_effect(num: usize) {
    unsafe {
        sdl2::mixer::Channel::all()
        .play(&EFFECTS[num], 0)
        .unwrap();
    }
}
