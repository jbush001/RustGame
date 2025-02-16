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

//
// This runs during build time to take all of the different image files and
// copy them into a single image (atlas). It also generates a source code
// file with the coordinates, which will be compiled into the program.
//

use image::*;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

type AtlasLocation = (f32, f32, f32, f32, u32, u32);

struct TileMapInfo {
    source_path: String,
    width: usize,
    height: usize,
    tile_data: Vec<u8>,
    image_paths: Vec<String>,
    tile_flags: Vec<u8>,
}

fn main() {
    let build_dir = env::var_os("OUT_DIR").unwrap().to_str().unwrap().to_owned();
    let target_dir = Path::new(&build_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    println!("build dir {} target_dir {}", build_dir, target_dir);

    println!("cargo::rerun-if-changed=assets/");
    println!("cargo::rerun-if-changed=build.rs");

    let sprite_ids = read_sprite_list("assets/sprites.txt");
    let tile_maps = read_tile_maps("assets/tile_maps.txt");

    let mut image_paths: HashSet<String> = HashSet::new();
    for tile_map in &tile_maps {
        for path in &tile_map.image_paths {
            image_paths.insert(path.clone());
        }
    }

    for (_, path, _, _) in &sprite_ids {
        image_paths.insert(path.clone());
    }

    println!("All images {:?}", image_paths);
    let mut images = load_images(&image_paths);

    // Sort images by vertical size, which will make them pack better.
    images.sort_by(|a, b| {
        let a = a.1.dimensions();
        let b = b.1.dimensions();
        b.1.cmp(&a.1)
    });

    let (atlas, image_coordinates) = pack_images(&images);

    // Write out a rust file with all of the sprite locations. This will be linked
    // into the executable.
    let sprite_define_path = build_dir.clone() + "/sprites.rs";
    write_sprite_locations(&sprite_define_path, &sprite_ids, &image_coordinates);

    // Write out the new atlas image.
    let result = image::save_buffer(
        format!("{}/{}", &target_dir, "atlas.png"),
        &atlas.to_rgba8().into_raw(),
        atlas.width(),
        atlas.height(),
        image::ColorType::Rgba8,
    );

    if let Err(msg) = result {
        panic!("{}", msg);
    }

    for tile_map_info in &tile_maps {
        write_tile_map_file(&target_dir, tile_map_info, &image_coordinates);
    }

    let audio_define_path = build_dir.clone() + "/sounds.rs";
    copy_sound_effects("assets/sound-effects.txt", &audio_define_path, &target_dir);

    copy_music_files("assets/sounds", &target_dir);
}

fn read_tile_maps(list_file: &str) -> Vec<TileMapInfo> {
    let mut result: Vec<TileMapInfo> = Vec::new();
    let map_list = std::fs::read_to_string(list_file).unwrap();
    for filename in map_list.lines() {
        result.push(read_tile_map(filename));
    }

    result
}

// Returns a list of identifier->path mappings
fn read_sprite_list(path: &str) -> Vec<(String, String, i32, i32)> {
    let mut sprites: Vec<(String, String, i32, i32)> = Vec::new();
    let manifest = std::fs::read_to_string(path).unwrap();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = line.split(' ').collect();
        if tokens.len() != 4 {
            panic!("Invalid manifest line: {}", line);
        }

        sprites.push((
            tokens[0].to_string(),
            tokens[1].to_string(),
            tokens[2].parse::<i32>().unwrap(),
            tokens[3].parse::<i32>().unwrap(),
        ));
    }

    sprites
}

fn read_tile_map(filename: &str) -> TileMapInfo {
    let path = format!("assets/{}", &filename);
    println!("reading tile map {}", &path);

    let tiles = std::fs::read_to_string(&path).unwrap();
    let mut char_to_tile: HashMap<char, usize> = HashMap::new();
    let mut tile_images: Vec<String> = Vec::new();
    let mut tile_flags: Vec<u8> = Vec::new();

    let mut reading_tile_images = true;

    let mut map_data: Vec<Vec<u8>> = Vec::new();
    let mut map_width = 0;
    for (linenum, line) in tiles.lines().enumerate() {
        if line.starts_with("------") {
            if !reading_tile_images {
                panic!("{}:{}: Unexpected separator", path, linenum + 1);
            }

            reading_tile_images = false;
            continue;
        }

        if reading_tile_images {
            // Information about each tile type
            let tokens: Vec<&str> = line.split(' ').collect();
            if tokens.len() != 3 {
                panic!("{}:{}: Invalid line: needs 3 fields", path, linenum + 1);
            }

            if tokens[0].len() != 1 {
                panic!("{}:{}: Invalid tile character", path, linenum + 1);
            }

            let ch = tokens[0].chars().next().unwrap();
            if char_to_tile.contains_key(&ch) {
                panic!("{}:{}: Duplicate tile character", path, linenum + 1);
            }

            char_to_tile.insert(ch, tile_images.len());
            tile_images.push(tokens[1].trim().to_string());
            tile_flags.push(tokens[2].trim().parse().unwrap());
        } else {
            // Filling actual map data
            let mut row: Vec<u8> = Vec::new();
            for c in line.chars() {
                let tile_index = if c == ' ' {
                    0
                } else {
                    if !char_to_tile.contains_key(&c) {
                        panic!("{}:{}: Invalid tile character", path, linenum + 1);
                    }

                    char_to_tile.get(&c).unwrap() + 1
                };

                row.push(tile_index as u8);
            }

            if row.len() > map_width {
                map_width = row.len();
            }

            map_data.push(row);
        }
    }

    // Flatten the map
    let map_height = map_data.len();
    let mut encoded_map: Vec<u8> = Vec::new();
    for row in map_data {
        encoded_map.extend(row.clone().into_iter());
        let padding = map_width - row.len();
        if padding > 0 {
            encoded_map.extend(std::iter::repeat(0).take(padding));
        }
    }

    TileMapInfo {
        source_path: filename.to_string(),
        width: map_width,
        height: map_height,
        tile_data: encoded_map,
        image_paths: tile_images,
        tile_flags,
    }
}

// Given a list of paths, return corresponding images.
fn load_images(filenames: &HashSet<String>) -> Vec<(String, DynamicImage)> {
    let mut images: Vec<(String, DynamicImage)> = Vec::new();
    for filename in filenames.iter() {
        let img = ImageReader::open(format!("assets/{}", filename));
        if let Err(msg) = img {
            panic!("Failed to load {}: {}", filename, msg);
        }

        images.push((filename.clone(), img.unwrap().decode().unwrap()));
    }

    images
}

struct AtlasAllocator {
    free_regions: Vec<(u32, u32, u32, u32)>,
}

impl AtlasAllocator {
    fn new(width: u32, height: u32) -> AtlasAllocator {
        AtlasAllocator {
            free_regions: vec![(0, 0, width, height)],
        }
    }

    fn alloc(&mut self, sprite_width: u32, sprite_height: u32) -> (u32, u32) {
        // First fit allocator
        for index in 0..self.free_regions.len() {
            let (region_left, region_top, region_width, region_height) = self.free_regions[index];
            if region_width >= sprite_width && region_height >= sprite_height {
                self.free_regions.remove(index);

                // If there are left over regions after carving up this
                // block, stick them back into the list (in order)
                // +-----+------------+
                // |/////|     A      |
                // +-----+------------+
                // |        B         |
                // +------------------+
                //
                if region_height > sprite_height {
                    // B
                    self.free_regions.insert(
                        index,
                        (
                            region_left,
                            region_top + sprite_height,
                            region_width,
                            region_height - sprite_height,
                        ),
                    );
                }

                if region_width > sprite_width {
                    // A
                    self.free_regions.insert(
                        index,
                        (
                            region_left + sprite_width,
                            region_top,
                            region_width - sprite_width,
                            sprite_height,
                        ),
                    );
                }

                return (region_left, region_top);
            }
        }

        panic!(
            "Atlas full, trying to allocate {}x{}, regions {:?}",
            sprite_width, sprite_height, self.free_regions
        );
    }
}

fn pack_images(
    images: &[(String, DynamicImage)],
) -> (DynamicImage, HashMap<String, AtlasLocation>) {
    const BORDER_SIZE: u32 = 2;
    const ATLAS_SIZE: u32 = 512;
    let mut atlas = DynamicImage::new_rgba8(ATLAS_SIZE, ATLAS_SIZE);
    let mut allocator = AtlasAllocator::new(ATLAS_SIZE, ATLAS_SIZE);
    let mut image_coordinates: HashMap<String, AtlasLocation> = HashMap::new();
    for (name, img) in images.iter() {
        let (x, y) = allocator.alloc(img.width() + BORDER_SIZE, img.height() + BORDER_SIZE);
        let _ = atlas.copy_from(img, x, y);
        println!("Packing image {} at {},{}", name, x, y);
        image_coordinates.insert(
            name.clone(),
            (
                x as f32 / ATLAS_SIZE as f32,
                y as f32 / ATLAS_SIZE as f32,
                (x + img.width()) as f32 / ATLAS_SIZE as f32,
                (y + img.height()) as f32 / ATLAS_SIZE as f32,
                img.width(),
                img.height(),
            ),
        );
    }

    (atlas, image_coordinates)
}

fn write_sprite_locations(
    dest_path: &str,
    sprite_ids: &Vec<(String, String, i32, i32)>,
    image_coordinates: &HashMap<String, AtlasLocation>,
) {
    let mut file = fs::File::create(dest_path).unwrap();
    for (name, path, xorigin, yorigin) in sprite_ids {
        let (left, top, right, bottom, width, height) = *image_coordinates.get(path).unwrap();
        writeln!(
            file,
            "pub const {}: (f32, f32, f32, f32, u32, u32, i32, i32) = ({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?});",
            name, left, top, right, bottom, width, height, xorigin, yorigin
        )
        .unwrap();
    }
}

//
// Format
//    magic [u8; 4]  "TMAP"
//    width: u32
//    height: u32
//    num_tiles: u32
//    tile_locs: [(f32, f32, f32, f32), num_tiles]
//    tile_flags: [u8, num_tiles]
//    map: [u8; width * height]
//  "255 tiles should be enough for anyone"
//
fn write_tile_map_file(
    target_dir: &str,
    tile_map_info: &TileMapInfo,
    image_coordinates: &HashMap<String, AtlasLocation>,
) {
    let output_file_name = Path::new(&tile_map_info.source_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let dest_path = format!("{}/{}.bin", target_dir, output_file_name);
    let output_file = fs::File::create(dest_path).unwrap();
    let mut writer = std::io::BufWriter::new(output_file);
    const MAGIC: &[u8; 4] = b"TMAP";
    writer.write_all(MAGIC.as_bytes()).unwrap();
    writer
        .write_all(&(tile_map_info.width as u32).to_le_bytes())
        .unwrap();
    writer
        .write_all(&(tile_map_info.height as u32).to_le_bytes())
        .unwrap();
    writer
        .write_all(&(tile_map_info.image_paths.len() as u32).to_le_bytes())
        .unwrap();
    for path in tile_map_info.image_paths.iter() {
        assert!(image_coordinates.contains_key(path));
        let (left, top, right, bottom, _width, _height) = image_coordinates.get(path).unwrap();
        println!(
            "Writing tile location for {}: {:?} {:?} {:?} {:?}",
            path, left, top, right, bottom
        );
        writer.write_all(&left.to_le_bytes()).unwrap();
        writer.write_all(&top.to_le_bytes()).unwrap();
        writer.write_all(&right.to_le_bytes()).unwrap();
        writer.write_all(&bottom.to_le_bytes()).unwrap();
    }

    writer
        .write_all(tile_map_info.tile_flags.as_slice())
        .unwrap();
    writer
        .write_all(tile_map_info.tile_data.as_slice())
        .unwrap();
    writer.flush().unwrap();
}

fn copy_sound_effects(manifest_path: &str, defines_path: &str, output_dir: &str) {
    let manifest = std::fs::read_to_string(manifest_path).unwrap();
    let mut files: Vec<(String, String)> = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = line.split(' ').collect();
        if tokens.len() != 2 {
            panic!("Invalid manifest line: {}", line);
        }

        files.push((tokens[0].to_string(), tokens[1].to_string()));
    }

    // Copy the files
    for (_, path) in files.clone().into_iter() {
        let source_path = "assets/".to_owned() + &path.clone();
        let dest = Path::new(&source_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        println!(
            "Copy {:?} {:?}",
            &source_path,
            format!("{}/{}", output_dir, dest)
        );
        std::fs::copy(&source_path, format!("{}/{}", output_dir, dest)).unwrap();
    }

    // Write a source file
    let mut defines_file = fs::File::create(defines_path).unwrap();
    for (index, (name, _path)) in files.clone().into_iter().enumerate() {
        writeln!(defines_file, "pub const {}: usize = {};", name, index).unwrap();
    }

    writeln!(
        defines_file,
        "pub const AUDIO_FILE_LIST: [&str; {}] = [",
        files.len()
    )
    .unwrap();
    for (_name, path) in files {
        let file_name = Path::new(&path).file_name().unwrap().to_str().unwrap();

        writeln!(defines_file, "\"{}\",", file_name).unwrap();
    }

    writeln!(defines_file, "];").unwrap();
}

fn copy_music_files(from_dir: &str, to_dir: &str) {
    for name in fs::read_dir(from_dir).unwrap() {
        let source_path = name.unwrap().path();
        if source_path.is_file() {
            if let Some(file_name) = source_path.file_name().and_then(|name| name.to_str()) {
                if file_name.ends_with(".mp3") {
                    println!("copy {:?} to {}/{:?}", source_path, to_dir, file_name);
                    std::fs::copy(&source_path, format!("{}/{}", to_dir, file_name)).unwrap();
                }
            }
        }
    }
}
