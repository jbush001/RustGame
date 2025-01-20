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

use image::ImageReader;

// Load all constants pointing to individual images from the texture atlas,
// which is generated during the build process.
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 450;

pub struct RenderContext {
    window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    vbo: gl::types::GLuint,
    atlas_texture_id: gl::types::GLuint,
    vertices: Vec<f32>,
}

const POSITION_ATTRIB: gl::types::GLuint = 0;
const TEXCOORD_ATTRIB: gl::types::GLuint = 1;

const VERTEX_SHADER: &str = r#"
attribute vec2 aPosition;
attribute vec2 aTexcoord;
varying vec2 vTexcoord;

void main() {
    gl_Position = vec4(aPosition, 0.0, 1.0);
    vTexcoord = aTexcoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"
varying vec2 vTexcoord;
uniform sampler2D texture0;

void main() {
    gl_FragColor = texture2D(texture0, vTexcoord);
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

// We use a single texture atlas with all images to avoid state changes during
// rendering.
fn init_texture_atlas() -> gl::types::GLuint {
    // The atlas file is copied into the same directory as our executable.
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

    unsafe {
        let mut atlas_texture_id: gl::types::GLuint = 0;
        gl::Enable(gl::TEXTURE_2D);
        gl::GenTextures(1, &mut atlas_texture_id);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, atlas_texture_id);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            atlas_width as i32,
            atlas_height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            raster_data.as_ptr() as *const _,
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        check_gl_error();

        atlas_texture_id
    }
}

//
// The matrix is a, b, c, d
// the point is x, y
//
// | a b | * | x | = | x' |
// | c d |   | y |   | y' |
//
fn rotate(point: &(f32, f32), matrix: &(f32, f32, f32, f32)) -> (f32, f32) {
    (
        matrix.0 * point.0 + matrix.1 * point.1,
        matrix.2 * point.0 + matrix.3 * point.1,
    )
}

impl RenderContext {
    pub fn new(sdl: &mut sdl2::Sdl) -> Result<Self, String> {
        let video_subsystem = sdl.video().unwrap();

        // Note: doubling resolutions here because SDL seems
        // to be halving them on my system (Chromebook). I think it's some
        // kind of high-DPI thing. This will probably break on other systems.
        let window = video_subsystem
            .window("Game", WINDOW_WIDTH * 3, WINDOW_HEIGHT * 3)
            .opengl()
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
            gl::BindAttribLocation(program, POSITION_ATTRIB, c"aPosition".as_ptr().cast());
            gl::BindAttribLocation(program, TEXCOORD_ATTRIB, c"aTexcoord".as_ptr().cast());
        }


        let vbo = unsafe {
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            vbo
        };

        let atlas_texture_id = init_texture_atlas();

        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);

            // Assign texture unit
            let image_attr = gl::GetUniformLocation(program, c"texture0".as_ptr().cast());
            assert!(image_attr != -1);
            gl::Uniform1i(image_attr, 0);

            // Enable source alpha blending
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        check_gl_error();

        Ok(RenderContext {
            window,
            _gl_context: gl_context,
            vbo,
            atlas_texture_id,
            vertices: Vec::new(),
        })
    }

    // Add an image to the display list.
    pub fn draw_image(
        &mut self,
        position: (i32, i32),
        image_info: &(f32, f32, f32, f32, u32, u32),
        rotation: f32,
        origin: (i32, i32),
        flip_h: bool,
    ) {
        let (mut atlas_left, atlas_top, mut atlas_right, atlas_bottom, width, height) = *image_info;

        if flip_h {
            std::mem::swap(&mut atlas_left, &mut atlas_right);
        }

        // Images are square. We compose them of two abutting triangles, with four
        // vertices:
        // 0      1
        // +------+
        // |    / |
        // |   /  |
        // |  /   |
        // | /    |
        // +------+
        // 2      3

        let display_left = -origin.0 as f32;
        let display_top = -origin.1 as f32;
        let display_right = display_left + width as f32;
        let display_bottom = display_top + height as f32;

        let (mut p0, mut p1, mut p2, mut p3) = if rotation == 0.0 {
            // Fast path if there is no rotation
            (
                (display_left, display_top),
                (display_right, display_top),
                (display_left, display_bottom),
                (display_right, display_bottom)
            )
        } else {
            let crot = f32::cos(rotation);
            let srot = f32::sin(rotation);
            let rotmat = (crot, -srot, srot, crot);

            (
                rotate(&(display_left, display_top), &rotmat),
                rotate(&(display_right, display_top), &rotmat),
                rotate(&(display_left, display_bottom), &rotmat),
                rotate(&(display_right, display_bottom), &rotmat),
            )
        };

        // Convert from pixel coordinates to OpenGL coordinate space.
        #[inline]
        fn to_ogl_coord(pt: &(f32, f32), position: &(i32, i32)) -> (f32, f32) {
            (
                ((pt.0 + position.0 as f32) / WINDOW_WIDTH as f32) * 2.0 - 1.0,
                1.0 - ((pt.1 + position.1 as f32) / WINDOW_HEIGHT as f32) * 2.0,
            )
        }

        p0 = to_ogl_coord(&p0, &position);
        p1 = to_ogl_coord(&p1, &position);
        p2 = to_ogl_coord(&p2, &position);
        p3 = to_ogl_coord(&p3, &position);

        #[cfg_attr(any(), rustfmt::skip)]
        self.vertices.extend_from_slice(&[
            // Upper left triangle (CW winding)
            p0.0, p0.1, atlas_left, atlas_top, // 0
            p1.0, p1.1, atlas_right, atlas_top, // 1
            p2.0, p2.1, atlas_left, atlas_bottom, // 2
            // Lower right triangle
            p1.0, p1.1, atlas_right, atlas_top, // 1
            p3.0, p3.1, atlas_right, atlas_bottom, // 3
            p2.0, p2.1, atlas_left, atlas_bottom, // 2
        ]);
    }

    pub fn render(&mut self) {
        const ATTRIBS_PER_VERTEX: usize = 4;
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr().cast(),
                gl::STREAM_DRAW,
            );

            gl::BindTexture(gl::TEXTURE_2D, self.atlas_texture_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            // Screen coordinate attribute
            gl::VertexAttribPointer(
                POSITION_ATTRIB,
                2, // Size (elements)
                gl::FLOAT,
                gl::FALSE,
                (ATTRIBS_PER_VERTEX * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null(),
            );

            // Texture coordinate attribute
            gl::VertexAttribPointer(
                TEXCOORD_ATTRIB,
                2, // Size (elements)
                gl::FLOAT,
                gl::FALSE,
                (ATTRIBS_PER_VERTEX * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null::<f32>().add(2).cast(), // Offset into packed array.
            );

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                (self.vertices.len() / ATTRIBS_PER_VERTEX) as i32,
            );
            check_gl_error();
        }

        self.window.gl_swap_window();

        self.vertices.clear();
    }
}

fn compile_shader(
    shader_type: gl::types::GLuint,
    source: &str,
) -> Result<gl::types::GLuint, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        check_gl_error();
        gl::ShaderSource(
            shader,
            1,
            &(source.as_bytes().as_ptr().cast()),
            &(source.len().try_into().unwrap()),
        );

        gl::CompileShader(shader);
        let mut status: gl::types::GLint = 1;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status == 0 {
            let mut v: [u8; 1024] = [0; 1024];
            let mut log_length = 0i32;
            gl::GetShaderInfoLog(shader, 1024, &mut log_length, v.as_mut_ptr().cast());
            return Err(format!(
                "Shader compile error {}",
                String::from_utf8_lossy(&v[..log_length as usize])
            ));
        }

        Ok(shader)
    }
}

fn compile_program(
    vertex_source: &str,
    fragment_source: &str,
) -> Result<gl::types::GLuint, String> {
    let vertex_shader = compile_shader(gl::VERTEX_SHADER, vertex_source)?;
    let fragment_shader = compile_shader(gl::FRAGMENT_SHADER, fragment_source)?;

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
            gl::GetProgramInfoLog(program, 1024, &mut log_length, v.as_mut_ptr());

            let temp_msg = std::mem::transmute::<[i8; 1024], [u8; 1024]>(v);

            return Err(format!(
                "Error linking shaders {}",
                String::from_utf8_lossy(&temp_msg[..log_length as usize] as &[u8])
            ));
        }

        check_gl_error();

        Ok(program)
    }
}
