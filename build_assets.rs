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
use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

type AtlasLocation = (f32, f32, f32, f32, u32, u32);

#[derive(Debug)]
struct TileMapInfo {
    source_path: String,
    width: i32,
    height: i32,
    tile_data: Vec<u8>,
    image_paths: Vec<String>,
    tile_flags: Vec<u8>,
    objects: Vec<(String, i32, i32)>,
    player_start_x: i32,
    player_start_y: i32,
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

    let mut image_paths: HashSet<String> = HashSet::new();
    let tile_map = read_tmx_file("assets/map.tmx");

    println!("{:?}", tile_map);

    image_paths.extend(tile_map.image_paths.iter().cloned());
    image_paths.extend(sprite_ids.iter().map(|(_, path, _, _)| path.clone()));

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

    write_tile_map_file(&target_dir, &tile_map, &image_coordinates);

    let audio_define_path = build_dir.clone() + "/sounds.rs";
    copy_sound_effects("assets/sound-effects.txt", &audio_define_path, &target_dir);

    copy_music_files("assets/sounds", &target_dir);
}

// Returns a list of identifier->path mappings
fn read_sprite_list(path: &str) -> Vec<(String, String, i32, i32)> {
    let manifest = std::fs::read_to_string(path).unwrap();
    manifest
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            let tokens: Vec<&str> = line.split(' ').collect();
            if tokens.len() != 4 {
                panic!("Invalid manifest line: {}", line);
            }

            (
                tokens[0].to_string(),
                tokens[1].to_string(),
                tokens[2].parse::<i32>().unwrap(),
                tokens[3].parse::<i32>().unwrap(),
            )
        })
        .collect()
}

// Given a list of paths, return corresponding images.
fn load_images(filenames: &HashSet<String>) -> Vec<(String, DynamicImage)> {
    let images: Result<Vec<(String, DynamicImage)>, image::ImageError> = filenames
        .iter()
        .map(|filename| {
            let img = ImageReader::open(format!("assets/{}", filename))?.decode()?;
            Ok((filename.clone(), img))
        })
        .collect();

    images.unwrap()
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
//    width: i32
//    height: i32
//    player_start_x: i32,
//    player_start_y: i32,
//    num_tiles: u32
//    tile_locs: [(f32, f32, f32, f32); num_tiles]
//    tile_flags: [u8; num_tiles]
//    map: [u8; width * height]
//    num_objects: u32
//    objects: [name: [u8; 32], x: i32, y: i32]
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
        .write_all(&tile_map_info.width.to_le_bytes())
        .unwrap();
    writer
        .write_all(&tile_map_info.height.to_le_bytes())
        .unwrap();
    writer
        .write_all(&tile_map_info.player_start_x.to_le_bytes())
        .unwrap();
    writer
        .write_all(&tile_map_info.player_start_y.to_le_bytes())
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

    let num_objects: u32 = tile_map_info.objects.len() as u32;
    writer.write_all(&num_objects.to_le_bytes()).unwrap();
    for (name, x, y) in &tile_map_info.objects {
        let mut name_temp = [0u8; 32];
        name_temp[..name.len()].copy_from_slice(name.as_bytes());
        writer.write_all(&name_temp).unwrap();
        writer.write_all(&x.to_le_bytes()).unwrap();
        writer.write_all(&y.to_le_bytes()).unwrap();
    }

    writer.flush().unwrap();
}

fn copy_sound_effects(manifest_path: &str, defines_path: &str, output_dir: &str) {
    let manifest = std::fs::read_to_string(manifest_path).unwrap();
    let files: Vec<(String, String)> = manifest
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split_once(' ')
                .map(|(first, second)| (first.to_string(), second.to_string()))
                .unwrap()
        })
        .collect();

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

fn get_xml_attribute(attrs: &Attributes, name: &str) -> Option<String> {
    attrs.clone().find_map(|attr| {
        let attru = attr.as_ref().ok()?; // Handle potential errors from attr.as_ref()
        if std::str::from_utf8(attru.key.as_ref()).unwrap() == name {
            String::from_utf8(attru.value.as_ref().to_vec()).ok()
        } else {
            None
        }
    })
}

fn read_tileset(filename: &str) -> (Vec<String>, Vec<u8>) {
    let rawxml = std::fs::read_to_string(filename).unwrap();
    let mut reader = Reader::from_str(&rawxml);
    let mut buf = Vec::new();
    let mut current_tile_id = 0;
    let mut image_paths: Vec<String> = Vec::new();
    let mut tile_flags: Vec<u8> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                if e.name() == QName(b"tile") {
                    current_tile_id = get_xml_attribute(&e.attributes(), "id")
                        .unwrap()
                        .parse()
                        .unwrap();
                    while image_paths.len() <= current_tile_id {
                        image_paths.push(String::new());
                        tile_flags.push(0);
                    }
                }
            }
            Ok(Event::Empty(e)) => match e.name() {
                QName(b"image") => {
                    image_paths[current_tile_id] =
                        get_xml_attribute(&e.attributes(), "source").unwrap();
                }

                QName(b"property") => {
                    if get_xml_attribute(&e.attributes(), "value").unwrap() == "true" {
                        match get_xml_attribute(&e.attributes(), "name").unwrap().as_str() {
                            "ladder" => {
                                tile_flags[current_tile_id] |= 2;
                            }
                            "solid" => {
                                tile_flags[current_tile_id] |= 1;
                            }
                            _ => {
                                println!("unknown attribute");
                            }
                        }
                    }
                }

                _ => (),
            },
            _ => (),
        }
    }

    (image_paths, tile_flags)
}

fn read_tmx_file(filename: &str) -> TileMapInfo {
    let rawxml = std::fs::read_to_string(filename).unwrap();
    let mut reader = Reader::from_str(&rawxml);
    let mut buf = Vec::new();
    let mut tile_data: Vec<u8> = Vec::new();
    let mut image_paths: Vec<String> = Vec::new();
    let mut tile_flags: Vec<u8> = Vec::new();
    let mut objects: Vec<(String, i32, i32)> = Vec::new();
    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut player_start_x: i32 = 0;
    let mut player_start_y: i32 = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                if let QName(b"layer") = e.name() {
                    width = get_xml_attribute(&e.attributes(), "width")
                        .unwrap()
                        .parse()
                        .unwrap();
                    height = get_xml_attribute(&e.attributes(), "height")
                        .unwrap()
                        .parse()
                        .unwrap();
                    println!("layer {}x{}", width, height);
                }
            }
            Ok(Event::Empty(e)) => match e.name() {
                QName(b"tileset") => {
                    let first_gid: u32 = get_xml_attribute(&e.attributes(), "firstgid")
                        .unwrap()
                        .parse()
                        .unwrap();
                    if first_gid != 1 {
                        panic!("first_gid != 1");
                    }

                    let tsx_file = get_xml_attribute(&e.attributes(), "source").unwrap();
                    (image_paths, tile_flags) = read_tileset(&format!("assets/{}", tsx_file));
                }

                QName(b"object") => {
                    let x_loc: f32 = get_xml_attribute(&e.attributes(), "x")
                        .unwrap()
                        .parse()
                        .unwrap();
                    let y_loc: f32 = get_xml_attribute(&e.attributes(), "y")
                        .unwrap()
                        .parse()
                        .unwrap();
                    let objtype = get_xml_attribute(&e.attributes(), "type").unwrap();

                    // This is a special object that indicates the player's start location.
                    if objtype == "Player" {
                        player_start_x = ((x_loc as i32 + 32) / 64) * 64;
                        player_start_y = ((y_loc as i32 + 32) / 64) * 64;
                    } else {
                        objects.push((objtype.clone(), x_loc as i32, y_loc as i32));
                    }
                }

                _ => (),
            },
            Ok(Event::Text(e)) => {
                tile_data.extend(
                    e.unescape()
                        .unwrap()
                        .split(',')
                        .map(|elem| elem.trim())
                        .filter(|tok| !tok.is_empty())
                        .map(|tok| tok.parse::<u8>().expect("")),
                );
            }
            _ => (),
        }
    }

    TileMapInfo {
        source_path: filename.to_string(),
        width,
        height,
        tile_data,
        image_paths,
        tile_flags,
        objects,
        player_start_x,
        player_start_y,
    }
}
