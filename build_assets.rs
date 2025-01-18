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
use std::env;
use std::fs;
use std::path::Path;
use std::io::Write;

fn main() {
    println!("cargo::rerun-if-changed=assets/");
    println!("cargo::rerun-if-changed=build.rs");

    // Scan the manifest and load all images into it.
    let mut images: Vec<(String, DynamicImage)> = Vec::new();
    let manifest = std::fs::read_to_string("assets/manifest.txt").unwrap();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split
        let tokens: Vec<&str> = line.split(' ').collect();
        if tokens.len() != 2 {
            panic!("Invalid manifest line: {}", line);
        }

        let path = format!("assets/{}", tokens[1]);
        let path = std::path::Path::new(&path);
        let path = path.canonicalize().unwrap();
        let path = path.to_str().unwrap();
        println!("cargo:warning=asset: {:?}", path);

        let img = ImageReader::open(path);
        if let Err(msg) = img {
            panic!("{}", msg);
        }

        images.push((tokens[0].to_string(), img.unwrap().decode().unwrap()));
    }

    // Sort images by vertical size, which will make them pack better.
    images.sort_by(|a, b| {
        let a = a.1.dimensions();
        let b = b.1.dimensions();
        a.1.cmp(&b.1)
    });

    // Pack images, left to right, top to bottom. There are much more
    // sophisticated ways to do this that waste less space, but this
    // is fairly simple and does an okay job.
    const ATLAS_SIZE: u32 = 1024;
    const BORDER_SIZE: u32 = 2;
    let mut atlas = DynamicImage::new_rgba8(ATLAS_SIZE, ATLAS_SIZE);
    let mut image_coordinates: Vec<(String, (u32, u32, u32, u32))> = Vec::new();
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
        image_coordinates.push((name.clone(), (x, y, img.width(), img.height())));
        x += img.width() + BORDER_SIZE;
    }

    // Write out a rust file with all of the locations. This will be linked
    // into the executable.
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("assets.rs");
    println!("cargo:warning=writing asset indices to {:?}", dest_path);

    let mut file = fs::File::create(&dest_path).unwrap();
    for (name, coords) in image_coordinates.iter() {
        // The tuple contains left, top, right, bottom in texture coordinate
        // space, width and height in pixels.
        writeln!(file, "pub const {}: (f32, f32, f32, f32, u32, u32) = ({:?}, {:?}, {:?}, {:?}, {:?}, {:?});",
            name,
            coords.0 as f32 / ATLAS_SIZE as f32,
            coords.1 as f32 / ATLAS_SIZE as f32,
            (coords.0 + coords.2) as f32 / ATLAS_SIZE as f32,
            (coords.1 + coords.3) as f32 / ATLAS_SIZE as f32,
            coords.2,
            coords.3,
        ).unwrap();
    }

    // Write out the new atlas image.
    let result = image::save_buffer(
        "target/debug/atlas.png", // XXX HACK, hardcoded dest path.
        &atlas.to_rgba8().into_raw(),
        atlas.width(),
        atlas.height(),
        image::ColorType::Rgba8,
    );

    if let Err(msg) = result {
        panic!("{}", msg);
    }
}
