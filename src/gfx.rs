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

pub struct RenderContext {
    window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    vbo: gl::types::GLuint,
    shader_program: gl::types::GLuint,
    image_atlas: gl::types::GLuint,
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
            .window("Game", 900, 700)
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
        let width = decoded.width();
        let height = decoded.height();

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

            println!("Width: {}, Height: {}", width, height);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA as u32,
                gl::UNSIGNED_BYTE,
                raster_data.as_ptr() as *const _,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

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
        })
    }

    pub fn render(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            let vertices: [f32; 12] = [
                -0.5, -0.5, 0.0, 0.0,
                0.5, -0.5, 1.0, 0.0,
                0.0, 0.5, 0.5, 1.0,
            ];

            gl::UseProgram(self.shader_program);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::BindTexture(gl::TEXTURE_2D, self.image_atlas);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            // Screen coordinate attribute
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null(),
            );

            // Texture coordinate attribute
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (4 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null::<f32>().add(2).cast(), // Offset into packed array.
            );

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            check_gl_error();
        }

        self.window.gl_swap_window();
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
