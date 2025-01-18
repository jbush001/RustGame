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

use gl;
use sdl2;
use image::ImageReader;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

pub const WINDOW_WIDTH: u32 = 900;
pub const WINDOW_HEIGHT: u32 = 700;

pub struct RenderContext {
    window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    vbo: gl::types::GLuint,
    shader_program: gl::types::GLuint,
    image_atlas: gl::types::GLuint,
    atlas_width: u32,
    atlas_height: u32,
    vertices: Vec<f32>,
}

const VERTEX_SHADER: &str = r#"
#version 330 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texcoord;
out vec2 frag_texcoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    frag_texcoord = texcoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec2 frag_texcoord;
out vec4 color;

uniform sampler2D texture0;

void main() {
    color = texture(texture0, frag_texcoord);
}

"#;

fn check_gl_error() {
    unsafe {
        let err = gl::GetError();
        if err != 0 {
            panic!("Error: {}", err);
        }
    }
}

impl RenderContext {
    pub fn new(sdl: &mut sdl2::Sdl) -> Result<Self, String> {
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window("Game", WINDOW_WIDTH, WINDOW_HEIGHT)
            .opengl()
            .resizable()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

        let compile_result = compile_program(VERTEX_SHADER, FRAGMENT_SHADER);
        if let Err(msg) = compile_result {
            panic!("{}", msg);
        }

        let program = compile_result.unwrap();
        unsafe {
            gl::UseProgram(program);
        }

        let vbo = unsafe {
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            vbo
        };

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        // Find out which directory this executable is located in
        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();
        let atlas_path = exe_dir.join("atlas.png");

        let img = ImageReader::open(atlas_path);
        if let Err(msg) = img {
            panic!("{}", msg);
        }

        let decode_result = img.unwrap().decode();
        if let Err(msg) = decode_result {
            panic!("{}", msg);
        }

        let decoded = decode_result.unwrap();
        let atlas_width = decoded.width();
        let atlas_height = decoded.height();

        let binding = decoded.into_rgba8();
        let raster_data = binding.as_raw();

        let image_atlas = unsafe {
            let mut image_atlas: gl::types::GLuint = 0;
            gl::Enable(gl::TEXTURE_2D);
            check_gl_error();
            gl::GenTextures(1, &mut image_atlas);
            check_gl_error();

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, image_atlas);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                atlas_width as i32,
                atlas_height as i32,
                0,
                gl::RGBA as u32,
                gl::UNSIGNED_BYTE,
                raster_data.as_ptr() as *const _,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);


            let image_attr = gl::GetUniformLocation(program, "texture0\0".as_ptr().cast());
            assert!(image_attr != -1);
            gl::Uniform1i(image_attr, 0);

            image_atlas
        };

        check_gl_error();

        Ok(RenderContext {
            window,
            _gl_context: gl_context,
            vbo,
            shader_program: program,
            image_atlas,
            vertices: Vec::new(),
            atlas_width,
            atlas_height,
        })
    }

    pub fn draw_image(&mut self, x: i32, y: i32, image_info: &(f32, f32, f32, f32, u32, u32)) {
        let (atlas_left, atlas_top, atlas_right, atlas_bottom, width, height) = image_info.clone();

        let screen_left = (x as f32 / WINDOW_WIDTH as f32) * 2.0 - 1.0;
        let screen_top = 1.0 - (y as f32 / WINDOW_HEIGHT as f32) * 2.0;
        let screen_right = ((x + width as i32) as f32 / WINDOW_WIDTH as f32) * 2.0 - 1.0;
        let screen_bottom = 1.0 - ((y + height as i32) as f32 / WINDOW_HEIGHT as f32) * 2.0;

        // 0      1
        // +------+
        // |    / |
        // |   /  |
        // |  /   |
        // | /    |
        // +------+
        // 2      3

        // Upper left triangle (CW winding)
        // 0
        self.vertices.push(screen_left);
        self.vertices.push(screen_top);
        self.vertices.push(atlas_left);
        self.vertices.push(atlas_top);

        // 1
        self.vertices.push(screen_right);
        self.vertices.push(screen_top);
        self.vertices.push(atlas_right);
        self.vertices.push(atlas_top);

        // 2
        self.vertices.push(screen_left);
        self.vertices.push(screen_bottom);
        self.vertices.push(atlas_left);
        self.vertices.push(atlas_bottom);

        // Lower right triangle
        // 1
        self.vertices.push(screen_right);
        self.vertices.push(screen_top);
        self.vertices.push(atlas_right);
        self.vertices.push(atlas_top);

        // 3
        self.vertices.push(screen_right);
        self.vertices.push(screen_bottom);
        self.vertices.push(atlas_right);
        self.vertices.push(atlas_bottom);

        // 2
        self.vertices.push(screen_left);
        self.vertices.push(screen_bottom);
        self.vertices.push(atlas_left);
        self.vertices.push(atlas_bottom);
    }

    pub fn render(&mut self) {
        const ATTRIBS_PER_VERTEX: usize = 4;
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::BindTexture(gl::TEXTURE_2D, self.image_atlas);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            // Screen coordinate attribute
            gl::VertexAttribPointer(
                0, // Input index
                2, // Size (elements)
                gl::FLOAT,
                gl::FALSE,
                (ATTRIBS_PER_VERTEX * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null(),
            );

            // Texture coordinate attribute
            gl::VertexAttribPointer(
                1, // Input index
                2, // Size (elements)
                gl::FLOAT,
                gl::FALSE,
                (ATTRIBS_PER_VERTEX * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null::<f32>().add(2).cast(), // Offset into packed array.
            );

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::DrawArrays(gl::TRIANGLES, 0, (self.vertices.len() / ATTRIBS_PER_VERTEX as usize) as i32);
            check_gl_error();
        }

        self.window.gl_swap_window();

        self.vertices.clear();
    }
}

fn compile_shader<'a>(shader_type: gl::types::GLuint, source: &str) -> Result<gl::types::GLuint, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        check_gl_error();
        gl::ShaderSource(
            shader,
            1,
            &(source.as_bytes().as_ptr().cast()),
            &(source.len().try_into().unwrap())
        );

        gl::CompileShader(shader);
        let mut status: gl::types::GLint = 1;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status == 0 {
            let mut v: [u8; 1024] = [0; 1024];
            let mut log_length = 0i32;
            gl::GetShaderInfoLog(
                shader,
                1024,
                &mut log_length,
                v.as_mut_ptr().cast(),
            );

            return Err(
                format!("Compile error {}", String::from_utf8_lossy(&v[..log_length as usize]))
            );
        }

        Ok(shader)
    }
}

fn compile_program<'a>(vertex_source: &str, fragment_source: &'a str) -> Result<gl::types::GLuint, String> {
    let vertex_shader = compile_shader(gl::VERTEX_SHADER, &vertex_source)?;
    let fragment_shader = compile_shader(gl::FRAGMENT_SHADER, &fragment_source)?;

    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut status: gl::types::GLint = 1;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status == 0 {
            let mut v: [i8; 1024] = [0; 1024];
            let mut log_length = 0i32;
            gl::GetProgramInfoLog(
                program,
                1024,
                &mut log_length,
                v.as_mut_ptr(),
            );

            let temp_msg = std::mem::transmute::<[i8; 1024], [u8; 1024]>(v);

            return Err(
                format!("Compile error {}", String::from_utf8_lossy(&temp_msg[..log_length as usize] as &[u8]))
            );
        }

        check_gl_error();

        Ok(program)
    }
}
