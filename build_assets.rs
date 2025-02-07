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
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

const TARGET_DIR: &str = "target/debug"; // XXX HACK, hardcoded dest path.

// XXX should scale this based on number of assets...
const ATLAS_SIZE: u32 = 1024;

type AtlasLocation = (f32, f32, f32, f32, u32, u32);

fn main() {
    println!("cargo::rerun-if-changed=assets/");
    println!("cargo::rerun-if-changed=build.rs");

    let sprite_ids = read_sprite_list("assets/sprites.txt");
    let (map_width, map_height, encoded_map, tile_paths, tile_flags) =
        read_tile_map("assets/tiles.txt");

    let mut image_paths: Vec<String> = tile_paths.clone();
    image_paths.extend(sprite_ids.iter().map(|(_, path)| path.clone()));

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
    let sprite_define_path =
        env::var_os("OUT_DIR").unwrap().to_str().unwrap().to_owned() + "/sprites.rs";
    write_sprite_locations(&sprite_define_path, &sprite_ids, &image_coordinates);

    // Write out the new atlas image.
    let result = image::save_buffer(
        format!("{}/{}", &TARGET_DIR, "atlas.png"),
        &atlas.to_rgba8().into_raw(),
        atlas.width(),
        atlas.height(),
        image::ColorType::Rgba8,
    );

    if let Err(msg) = result {
        panic!("{}", msg);
    }

    write_map_file(
        format!("{}/{}", &TARGET_DIR, "map.bin").as_str(),
        &encoded_map,
        &tile_paths,
        &image_coordinates,
        &tile_flags,
        map_width,
        map_height,
    );

    let audio_define_path =
        env::var_os("OUT_DIR").unwrap().to_str().unwrap().to_owned() + "/sounds.rs";
    copy_audio_files("assets/sound-effects.txt", &audio_define_path);
}

// Returns a list of identifier->path mappings
fn read_sprite_list(path: &str) -> Vec<(String, String)> {
    let mut sprites: Vec<(String, String)> = Vec::new();
    let manifest = std::fs::read_to_string(path).unwrap();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = line.split(' ').collect();
        if tokens.len() != 2 {
            panic!("Invalid manifest line: {}", line);
        }

        sprites.push((tokens[0].to_string(), tokens[1].to_string()));
    }

    sprites
}

fn read_tile_map(path: &str) -> (usize, usize, Vec<u8>, Vec<String>, Vec<u8>) {
    let tiles = std::fs::read_to_string(path).unwrap();
    let mut char_to_tile: HashMap<char, usize> = HashMap::new();
    let mut tile_images: Vec<String> = Vec::new();
    let mut tile_flags: Vec<u8> = Vec::new();

    // XXX should scale these based on content.
    const MAP_WIDTH: usize = 64;
    const MAP_HEIGHT: usize = 64;

    let mut encoded_map = vec![0u8; MAP_WIDTH * MAP_HEIGHT];
    let mut reading_tile_images = true;
    let mut map_start = 0;

    for (linenum, line) in tiles.lines().enumerate() {
        if line.starts_with("------") {
            if !reading_tile_images {
                panic!("{}:{}: Unexpected separator", path, linenum + 1);
            }

            reading_tile_images = false;
            map_start = linenum + 1;
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
            let map_row = linenum - map_start;
            for (map_col, c) in line.chars().enumerate() {
                if c != ' ' {
                    if !char_to_tile.contains_key(&c) {
                        panic!("{}:{}: Invalid tile character", path, linenum + 1);
                    }

                    let tile_index = char_to_tile.get(&c).unwrap() + 1;
                    encoded_map[map_row * MAP_WIDTH + map_col] = tile_index as u8;
                }
            }
        }
    }

    (MAP_WIDTH, MAP_HEIGHT, encoded_map, tile_images, tile_flags)
}

// Given a list of paths, return corresponding images.
fn load_images(filenames: &[String]) -> Vec<(String, DynamicImage)> {
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

fn pack_images(
    images: &[(String, DynamicImage)],
) -> (DynamicImage, HashMap<String, AtlasLocation>) {
    const BORDER_SIZE: u32 = 2;
    let mut atlas = DynamicImage::new_rgba8(ATLAS_SIZE, ATLAS_SIZE);
    let mut image_coordinates: HashMap<String, AtlasLocation> = HashMap::new();
    let mut x = BORDER_SIZE;
    let mut y = BORDER_SIZE;
    let mut row_height = images[0].1.height();
    for (name, img) in images.iter() {
        if x + img.width() > atlas.width() {
            x = BORDER_SIZE;
            y += row_height + BORDER_SIZE;
            // Because these are sorted by row height, we know none of the subsequent
            // images in the row will be larger.
            row_height = img.height();
        }

        if y + img.height() > atlas.height() {
            panic!("Out of space in atlas");
        }

        let _ = atlas.copy_from(img, x, y);
        println!("Packing image {} at {},{}", name, x, y);
        image_coordinates.insert(
            name.clone(),
            (
                x as f32 / ATLAS_SIZE as f32,
                y as f32 / ATLAS_SIZE as f32,
                (x + img.width() - 1) as f32 / ATLAS_SIZE as f32,
                (y + img.height() - 1) as f32 / ATLAS_SIZE as f32,
                img.width(),
                img.height(),
            ),
        );
        x += img.width() + BORDER_SIZE;
    }

    (atlas, image_coordinates)
}

fn write_sprite_locations(
    dest_path: &str,
    sprite_ids: &Vec<(String, String)>,
    image_coordinates: &HashMap<String, AtlasLocation>,
) {
    let mut file = fs::File::create(dest_path).unwrap();
    for (name, path) in sprite_ids {
        let (left, top, right, bottom, width, height) = *image_coordinates.get(path).unwrap();
        writeln!(
            file,
            "pub const {}: (f32, f32, f32, f32, u32, u32) = ({:?}, {:?}, {:?}, {:?}, {:?}, {:?});",
            name, left, top, right, bottom, width, height,
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
fn write_map_file(
    dest_path: &str,
    encoded_map: &[u8],
    tile_paths: &[String],
    image_coordinates: &HashMap<String, AtlasLocation>,
    tile_flags: &[u8],
    width: usize,
    height: usize,
) {
    let output_file = fs::File::create(dest_path).unwrap();
    let mut writer = std::io::BufWriter::new(output_file);
    const MAGIC: &[u8; 4] = b"TMAP";
    writer.write_all(MAGIC.as_bytes()).unwrap();
    writer.write_all(&(width as u32).to_le_bytes()).unwrap();
    writer.write_all(&(height as u32).to_le_bytes()).unwrap();
    writer
        .write_all(&(tile_paths.len() as u32).to_le_bytes())
        .unwrap();
    for path in tile_paths.iter() {
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

    writer.write_all(tile_flags).unwrap();
    writer.write_all(encoded_map).unwrap();
    writer.flush().unwrap();
}

fn copy_audio_files(manifest_path: &str, defines_path: &str) {
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
            format!("{}/{}", &TARGET_DIR, dest)
        );
        std::fs::copy(&source_path, format!("{}/{}", &TARGET_DIR, dest)).unwrap();
    }

    // Write a source file
    let mut defines_file = fs::File::create(defines_path).unwrap();
    for (index, (name, _path)) in files.clone().into_iter().enumerate() {
        writeln!(defines_file, "pub const {}: usize = {};", name, index).unwrap();
    }

    writeln!(defines_file, "pub const AUDIO_FILE_LIST: [&str; {}] = [", files.len()).unwrap();
    for (_name, path) in files {
        writeln!(defines_file, "\"{}\",", path).unwrap();
    }

    writeln!(defines_file, "];").unwrap();
}
