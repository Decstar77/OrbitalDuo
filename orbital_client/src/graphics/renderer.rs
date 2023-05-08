use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::{collections::HashMap, ffi::CString, fs::File, io::Read, ptr};

use gl::types::*;

use crate::config::get_config;
use crate::types::*;
use cgmath::*;

use super::ui::*;
use super::window::{FrameState, Window};

pub struct ShaderProgram {
    program_handle: u32,
    uniform_ids: HashMap<String, GLint>,
}

impl ShaderProgram {
    pub fn new() -> ShaderProgram {
        ShaderProgram {
            program_handle: 0,
            uniform_ids: HashMap::new(),
        }
    }

    pub fn new_from_file(vertex_shader_path: &str, fragment_shader_path: &str) -> ShaderProgram {
        let mut vertex_shader_file = File::open(vertex_shader_path)
            .unwrap_or_else(|_| panic!("Failed to open {}", vertex_shader_path));
        let mut fragment_shader_file = File::open(fragment_shader_path)
            .unwrap_or_else(|_| panic!("Failed to open {}", fragment_shader_path));

        let mut vertex_shader_source = String::new();
        let mut fragment_shader_source = String::new();

        vertex_shader_file
            .read_to_string(&mut vertex_shader_source)
            .expect("Failed to read vertex shader");

        fragment_shader_file
            .read_to_string(&mut fragment_shader_source)
            .expect("Failed to read fragment shader");

        unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str_vert = CString::new(vertex_shader_source.as_bytes()).unwrap();
            gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
            gl::CompileShader(vertex_shader);

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str_frag = CString::new(fragment_shader_source.as_bytes()).unwrap();
            gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
            gl::CompileShader(fragment_shader);

            let program_handle = gl::CreateProgram();
            gl::AttachShader(program_handle, vertex_shader);
            gl::AttachShader(program_handle, fragment_shader);
            gl::LinkProgram(program_handle);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            ShaderProgram {
                program_handle,
                uniform_ids: HashMap::new(),
            }
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.program_handle);
        }
    }

    pub fn unuse_program() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn create_uniform(&mut self, uniform_name: &str) {
        let uniform_location = unsafe {
            let c_string = CString::new(uniform_name).unwrap();
            gl::GetUniformLocation(self.program_handle, c_string.as_ptr())
        };

        if uniform_location < 0 {
            panic!("Cannot locate uniform: {}", uniform_name);
        } else {
            self.uniform_ids
                .insert(uniform_name.to_string(), uniform_location);
        }
    }

    pub fn set_int(&mut self, uniform_name: &str, value: i32) {
        if !self.uniform_ids.contains_key(uniform_name) {
            self.create_uniform(uniform_name);
        }

        unsafe {
            gl::Uniform1i(self.uniform_ids[uniform_name], value);
        }
    }

    pub fn set_vec4(&mut self, uniform_name: &str, value: Vec4) {
        if !self.uniform_ids.contains_key(uniform_name) {
            self.create_uniform(uniform_name);
        }

        unsafe {
            gl::Uniform4fv(self.uniform_ids[uniform_name], 1, value.as_ptr());
        }
    }

    pub fn set_mat4(&mut self, uniform_name: &str, matrix: Mat4) {
        if !self.uniform_ids.contains_key(uniform_name) {
            self.create_uniform(uniform_name);
        }

        unsafe {
            gl::UniformMatrix4fv(
                self.uniform_ids[uniform_name],
                1,
                gl::FALSE,
                matrix.as_ptr(),
            )
        }
    }
}

pub struct Texture {
    pub id: u32,
    pub width: i32,
    pub height: i32,
}

impl Texture {
    pub fn new() -> Texture {
        Texture {
            id: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn new_from_file(path: &str) -> Texture {
        let image = stb_image::image::load(path);
        match image {
            stb_image::image::LoadResult::Error(_) => {
                println!("Failed to load image {}", path);
                return Texture::new();
            }
            stb_image::image::LoadResult::ImageU8(image) => unsafe {
                let mut id = 0;
                gl::GenTextures(1, &mut id);
                gl::BindTexture(gl::TEXTURE_2D, id);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    image.width as i32,
                    image.height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    image.data.as_ptr() as *const std::ffi::c_void,
                );
                gl::BindTexture(gl::TEXTURE_2D, 0);

                Texture {
                    id,
                    width: image.width as i32,
                    height: image.height as i32,
                }
            },
            stb_image::image::LoadResult::ImageF32(image) => {
                unsafe {
                    let mut id = 0;
                    gl::GenTextures(1, &mut id);
                    gl::BindTexture(gl::TEXTURE_2D, id);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA as i32,
                        image.width as i32,
                        image.height as i32,
                        0,
                        gl::RGBA,
                        gl::FLOAT,
                        image.data.as_ptr() as *const std::ffi::c_void,
                    );

                    Texture {
                        id,
                        width: image.width as i32,
                        height: image.height as i32,
                    }
                }
            }
        }
    }
}

pub struct VertexBuffer {
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    size: i32,
    stride: i32,
}

impl VertexBuffer {
    fn new() -> VertexBuffer {
        VertexBuffer {
            vao: 0,
            vbo: 0,
            size: 0,
            stride: 0,
        }
    }
}

pub struct DrawCommandCircle {
    rad: f32,
    pos: Vec2,
    col: Vec4,
}

impl DrawCommandCircle {
    pub fn with_color(&mut self, col: Vec4) -> &mut DrawCommandCircle {
        self.col = col;
        self
    }
}

pub struct DrawCommandRect {
    tr: Vec2,
    br: Vec2,
    bl: Vec2,
    tl: Vec2,
    col: Vec4,
}

impl DrawCommandRect {
    pub fn with_color(&mut self, col: Vec4) -> &mut DrawCommandRect {
        self.col = col;
        self
    }
}

pub struct DrawCommandTri {
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
    pub col: Vec4,
}

pub enum TextHAlignment {
    Left,
    Center,
    Right,
}

pub enum TextVAlignment {
    Top,
    Center,
    Bottom,
}

pub struct DrawCommandText {
    pub text: String,
    pub pos: Vec2,
    pub col: Vec4,
    pub h_alignment: TextHAlignment,
    pub v_alignment: TextVAlignment,
}

impl DrawCommandText {
    pub fn with_color(&mut self, col: Vec4) -> &mut DrawCommandText {
        self.col = col;
        self
    }

    pub fn with_horizontal_alignment(&mut self, alignment: TextHAlignment) -> &mut DrawCommandText {
        self.h_alignment = alignment;
        self
    }

    pub fn with_vertical_alignment(&mut self, alignment: TextVAlignment) -> &mut DrawCommandText {
        self.v_alignment = alignment;
        self
    }
}

struct DrawCommandSprite {
    pub texture: Texture,
    pub pos: Vec2,
    pub size: Vec2,
    pub col: Vec4,
}

pub enum DrawCommand {
    CIRCLE(DrawCommandCircle),
    RECT(DrawCommandRect),
    TRI(DrawCommandTri),
    TEXT(DrawCommandText),
    SPRITE(DrawCommandSprite),
}

impl DrawCommand {
    fn as_circle_mut(&mut self) -> &mut DrawCommandCircle {
        match self {
            DrawCommand::CIRCLE(c) => c,
            _ => panic!("Cannot cast to circle"),
        }
    }

    fn as_rect_mut(&mut self) -> &mut DrawCommandRect {
        match self {
            DrawCommand::RECT(r) => r,
            _ => panic!("Cannot cast to rect"),
        }
    }

    fn as_tri_mut(&mut self) -> &mut DrawCommandTri {
        match self {
            DrawCommand::TRI(t) => t,
            _ => panic!("Cannot cast to tri"),
        }
    }

    fn as_text_mut(&mut self) -> &mut DrawCommandText {
        match self {
            DrawCommand::TEXT(t) => t,
            _ => panic!("Cannot cast to text"),
        }
    }
}

pub struct FontChar {
    texture: gl::types::GLuint,
    size: Vec2,
    bearing: Vec2,
    advance: i32,
}

pub struct Font {
    font_chars: HashMap<char, FontChar>,
}

impl Font {
    pub fn new() -> Font {
        Font {
            font_chars: HashMap::new(),
        }
    }

    pub fn new_from_file(file: &str, font_size: u32) -> Font {
        use freetype::face::LoadFlag;
        use freetype::Library;

        let cfg = get_config();
        let mut file_path = cfg.assets_path.clone();
        file_path.push_str(file);

        let lib = Library::init().expect("Failed to init freetype");
        let face = lib.new_face(file_path, 0).expect("Failed to load font");

        face.set_pixel_sizes(0, font_size)
            .expect("Failed to set font size");

        let mut font_chars = HashMap::new();
        for ci in 0..128 {
            let c = char::from(ci);
            face.load_char(c as usize, LoadFlag::RENDER)
                .expect("Failed to load char");

            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            let bitmap_buffer = bitmap.buffer();
            let bitmap_width = bitmap.width();
            let bitmap_height = bitmap.rows();
            let bitmap_buffer_len = bitmap_buffer.len();

            let mut texture: gl::types::GLuint = 0;
            unsafe {
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    bitmap_width,
                    bitmap_height,
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    bitmap_buffer.as_ptr() as *const c_void,
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            };

            let font_char = FontChar {
                texture,
                size: Vec2::new(bitmap_width as f32, bitmap_height as f32),
                bearing: Vec2::new(glyph.bitmap_left() as f32, glyph.bitmap_top() as f32),
                advance: glyph.advance().x as i32,
            };

            font_chars.insert(c, font_char);
        }

        Font { font_chars }
    }
}

#[derive(Clone, Copy)]
pub struct Camera {
    pos: Vec2,
    zoom: f32,
    view_matrix: Mat4,
    inverse_view_matrix: Mat4,
    projection_matrix: Mat4,
    inverse_projection_matrix: Mat4,
}

impl Camera {
    pub fn default() -> Camera {
        Camera {
            pos: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            view_matrix: Mat4::identity(),
            inverse_view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
            inverse_projection_matrix: Mat4::identity(),
        }
    }

    pub fn new(width: i32, height: i32) -> Camera {
        let mut cam = Camera {
            pos: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            view_matrix: Mat4::identity(),
            inverse_view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
            inverse_projection_matrix: Mat4::identity(),
        };

        cam.update_matrices();

        return cam;
    }

    fn update_matrices(&mut self) {
        self.view_matrix = self.calc_view_matrix();
        self.inverse_view_matrix = self.view_matrix.invert().unwrap();
        self.projection_matrix = self.calc_projection_matrix();
        self.inverse_projection_matrix = self.projection_matrix.invert().unwrap();
    }

    fn calc_view_matrix(&self) -> Mat4 {
        Mat4::from_translation(Vector3 {
            x: -self.pos.x,
            y: -self.pos.y,
            z: 0.0,
        })
    }

    fn calc_projection_matrix(&self) -> Mat4 {
        // let mut right = self.surface_width / 2.0;
        // let mut left = -right;

        // let mut top = self.surface_height / 2.0;
        // let mut bottom = -top;

        // let n = 256.0 * self.zoom;
        // let aspect = self.surface_width / self.surface_height;

        // left = -n * 0.5 * aspect;
        // right = n * 0.5 * aspect;
        // bottom = -n * 0.5;
        // top = n * 0.5;

        // left = left.round();
        // right = right.round();
        // bottom = bottom.round();
        // top = top.round();

        let left = -250.0;
        let right = 250.0;
        let bottom = -150.0;
        let top = 150.0;

        cgmath::ortho(left, right, bottom, top, -1.0, 1.0)
    }
}

extern "system" fn gl_debug_output(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    _user_param: *mut c_void,
) {
    if id == 131_169 || id == 131_185 || id == 131_218 || id == 131_204 {
        // ignore these non-significant error codes
        return;
    }

    println!("---------------");
    let message = unsafe { CStr::from_ptr(message).to_str().unwrap() };
    println!("Debug message ({}): {}", id, message);
    match source {
        gl::DEBUG_SOURCE_API => println!("Source: API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => println!("Source: Window System"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => println!("Source: Shader Compiler"),
        gl::DEBUG_SOURCE_THIRD_PARTY => println!("Source: Third Party"),
        gl::DEBUG_SOURCE_APPLICATION => println!("Source: Application"),
        gl::DEBUG_SOURCE_OTHER => println!("Source: Other"),
        _ => println!("Source: Unknown enum value"),
    }

    match gltype {
        gl::DEBUG_TYPE_ERROR => println!("Type: Error"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => println!("Type: Deprecated Behaviour"),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => println!("Type: Undefined Behaviour"),
        gl::DEBUG_TYPE_PORTABILITY => println!("Type: Portability"),
        gl::DEBUG_TYPE_PERFORMANCE => println!("Type: Performance"),
        gl::DEBUG_TYPE_MARKER => println!("Type: Marker"),
        gl::DEBUG_TYPE_PUSH_GROUP => println!("Type: Push Group"),
        gl::DEBUG_TYPE_POP_GROUP => println!("Type: Pop Group"),
        gl::DEBUG_TYPE_OTHER => println!("Type: Other"),
        _ => println!("Type: Unknown enum value"),
    }

    match severity {
        gl::DEBUG_SEVERITY_HIGH => println!("Severity: high"),
        gl::DEBUG_SEVERITY_MEDIUM => println!("Severity: medium"),
        gl::DEBUG_SEVERITY_LOW => println!("Severity: low"),
        gl::DEBUG_SEVERITY_NOTIFICATION => println!("Severity: notification"),
        _ => println!("Severity: Unknown enum value"),
    }
}

pub struct RenderState {
    pub commands: Vec<DrawCommand>,
    pub shape_program: ShaderProgram,
    pub shape_vertex_buffer: VertexBuffer,
    pub font_program: ShaderProgram,
    pub font_vertex_buffer: VertexBuffer,
    pub surface_width: f32,
    pub surface_height: f32,
    pub surface_aspect: f32,
    pub surface_projection: Mat4,
    pub camera_active: bool,
    pub camera: Camera,
    pub default_font: Font,
}

impl RenderState {
    pub fn new(window: &mut Window) -> RenderState {
        let mut rs = RenderState {
            commands: Vec::new(),
            shape_program: ShaderProgram::new(),
            shape_vertex_buffer: VertexBuffer::new(),
            font_program: ShaderProgram::new(),
            font_vertex_buffer: VertexBuffer::new(),
            surface_width: 0.0,
            surface_height: 0.0,
            surface_aspect: 0.0,
            surface_projection: Mat4::identity(),
            camera_active: false,
            camera: Camera::new(window.client_width, window.client_height),
            default_font: Font::new(),
        };

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_debug_output), ptr::null());
            gl::DebugMessageControl(
                gl::DONT_CARE,
                gl::DONT_CARE,
                gl::DONT_CARE,
                0,
                ptr::null(),
                gl::TRUE,
            );
        }

        rs.surface_resized(window.client_width, window.client_height);
        rs.default_font = Font::new_from_file("arial.ttf", 24);

        rs.shape_program = rs.create_shader_program(
            include_str!("shaders/shape_rendering.vs.glsl"),
            include_str!("shaders/shape_rendering.fs.glsl"),
        );

        rs.shape_vertex_buffer = rs.create_vertex_buffer_dynamic(2 * 4 * 6, || unsafe {
            let stride = 2 * mem::size_of::<f32>() as i32;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
            stride
        });

        rs.font_program = rs.create_shader_program(
            include_str!("shaders/font_rendering.vs.glsl"),
            include_str!("shaders/font_rendering.fs.glsl"),
        );

        rs.font_vertex_buffer = rs.create_vertex_buffer_dynamic(4 * 4 * 6, || unsafe {
            let stride = 4 * mem::size_of::<f32>() as i32;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, stride, ptr::null());
            stride
        });

        return rs;
    }

    pub fn screen_pos_to_world_pos(&self, camera: &Camera, screen_pixel_pos: Vec2) -> Vec2 {
        let x = (screen_pixel_pos.x / self.surface_width) * 2.0 - 1.0;
        let y = -((screen_pixel_pos.y / self.surface_height) * 2.0 - 1.0);

        let mouse_pos_clip_space = Vec4::new(x, y, 0.0, 1.0);
        let mouse_pos_eye_space = camera.inverse_projection_matrix * mouse_pos_clip_space;
        let mouse_pos_world_space = camera.inverse_view_matrix * mouse_pos_eye_space;

        let pos = Vec2::new(mouse_pos_world_space.x, mouse_pos_world_space.y);

        return pos;
    }

    pub fn world_pos_to_screen_pos(&self, camera: &Camera, world_pos: Vec2) -> Vec2 {
        let clip_space_pos = camera.projection_matrix
            * camera.view_matrix
            * Vec4::new(world_pos.x, world_pos.y, 0.0, 1.0);

        let ndc =
            Vec3::new(clip_space_pos.x, clip_space_pos.y, clip_space_pos.z) / clip_space_pos.w;

        let mut screen_pos = Vec2::new(0.0, 0.0);
        screen_pos.x = ((ndc.x + 1.0) / 2.0) * self.surface_width;
        screen_pos.y = ((1.0 - ndc.y) / 2.0) * self.surface_height;

        return screen_pos;
    }

    pub fn world_length_to_screen_length(&self, camera: &Camera, world_length: f32) -> f32 {
        let clip_space_pos = camera.projection_matrix * Vec4::new(world_length, 0.0, 0.0, 1.0);
        let l = ((clip_space_pos.x + 1.0) / 2.0) * self.surface_width - self.surface_width / 2.0;
        return l;
    }

    pub fn world_dimension_to_screen_dimension(&self, camera: &Camera, world_dim: Vec2) -> Vec2 {
        let clip_space_pos =
            camera.projection_matrix * Vec4::new(world_dim.x, world_dim.y, 0.0, 1.0);
        let x = ((clip_space_pos.x + 1.0) / 2.0) * self.surface_width - self.surface_width / 2.0;
        let y = ((clip_space_pos.y + 1.0) / 2.0) * self.surface_height - self.surface_height / 2.0;
        return Vec2::new(x, y);
    }

    pub fn world_direction_to_screen_direction(&self, camera: &Camera, world_dir: Vec2) -> Vec2 {
        let w = Vec3::new(world_dir.x, world_dir.y, 0.0);
        let v = camera.view_matrix * Vec4::new(w.x, w.y, w.z, 0.0);
        return Vec2::new(v.x, -v.y);
    }

    pub fn create_shader_program(&mut self, vs_source: &str, fs_source: &str) -> ShaderProgram {
        unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str_vert = CString::new(vs_source.as_bytes()).unwrap();
            gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
            gl::CompileShader(vertex_shader);

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str_frag = CString::new(fs_source.as_bytes()).unwrap();
            gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
            gl::CompileShader(fragment_shader);

            let program_handle = gl::CreateProgram();
            gl::AttachShader(program_handle, vertex_shader);
            gl::AttachShader(program_handle, fragment_shader);
            gl::LinkProgram(program_handle);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            ShaderProgram {
                program_handle,
                uniform_ids: HashMap::new(),
            }
        }
    }

    pub fn create_vertex_buffer_dynamic<F: Fn() -> i32>(
        &mut self,
        size_bytes: i32,
        enable_attribs: F,
    ) -> VertexBuffer {
        unsafe {
            let mut vao = 0;
            let mut vbo = 0;

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_bytes as isize,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );

            let stride = enable_attribs();

            gl::BindVertexArray(0);

            VertexBuffer {
                vao: vao,
                vbo: vbo,
                size: size_bytes,
                stride: stride,
            }
        }
    }

    pub fn surface_resized(&mut self, w: i32, h: i32) {
        unsafe {
            gl::Viewport(0, 0, w, h);
        }

        self.surface_width = w as f32;
        self.surface_height = h as f32;

        let w = w as f32;
        let h = h as f32;

        self.surface_aspect = w / h;
        self.surface_projection = cgmath::ortho(0.0, w, h, 0.0, -1.0, 1.0);
    }

    pub fn enable_alpha_blending(&self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    pub fn enable_pre_multiplied_alpha_blending(&self) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    pub fn camera_begin(&mut self, camera: Camera) {
        self.camera = camera;
        self.camera_active = true;
    }

    pub fn camera_end(&mut self) {
        self.camera_active = false;
    }

    pub fn add_draw_command(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    pub fn draw_circle(&mut self, mut rad: f32, mut pos: Vec2) -> &mut DrawCommandCircle {
        if self.camera_active {
            pos = self.world_pos_to_screen_pos(&self.camera, pos);
            rad = self.world_length_to_screen_length(&self.camera, rad);
        }

        self.commands.push(DrawCommand::CIRCLE(DrawCommandCircle {
            rad: rad,
            pos: pos,
            col: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }));

        self.commands.last_mut().unwrap().as_circle_mut()
    }

    pub fn draw_rect_min_max(&mut self, mut min: Vec2, mut max: Vec2) -> &mut DrawCommandRect {
        if self.camera_active {
            min = self.world_pos_to_screen_pos(&self.camera, min);
            max = self.world_pos_to_screen_pos(&self.camera, max);
        }

        let bl = Vec2::new(min.x, max.y);
        let tr = Vec2::new(max.x, min.y);
        let br = max;
        let tl = min;

        self.commands.push(DrawCommand::RECT(DrawCommandRect {
            tr: tr,
            br: br,
            bl: bl,
            tl: tl,
            col: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }));

        self.commands.last_mut().unwrap().as_rect_mut()
    }

    pub fn draw_rect_center_dims(&mut self, mut pos: Vec2, mut dims: Vec2) {
        if self.camera_active {
            pos = self.world_pos_to_screen_pos(&self.camera, pos);
            dims = self.world_dimension_to_screen_dimension(&self.camera, dims);
        }

        let half_dims = dims / 2.0;
        let bl = -half_dims;
        let tr = half_dims;
        let br = Vec2::new(tr.x, bl.y);
        let tl = Vec2::new(bl.x, tr.y);

        self.commands.push(DrawCommand::RECT(DrawCommandRect {
            tr: tr + pos,
            br: br + pos,
            bl: bl + pos,
            tl: tl + pos,
            col: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }))
    }

    pub fn draw_tri(&mut self, mut p1: Vec2, mut p2: Vec2, mut p3: Vec2) {
        if self.camera_active {
            p1 = self.world_pos_to_screen_pos(&self.camera, p1);
            p2 = self.world_pos_to_screen_pos(&self.camera, p2);
            p3 = self.world_pos_to_screen_pos(&self.camera, p3);
        }

        self.commands.push(DrawCommand::TRI(DrawCommandTri {
            p1: p1,
            p2: p2,
            p3: p3,
            col: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }))
    }

    pub fn draw_text(&mut self, text: &str, mut pos: Vec2) -> &mut DrawCommandText {
        if self.camera_active {
            pos = self.world_pos_to_screen_pos(&self.camera, pos);
        }

        self.commands.push(DrawCommand::TEXT(DrawCommandText {
            text: text.to_string(),
            pos: pos,
            col: Vec4::new(1.0, 1.0, 1.0, 1.0),
            h_alignment: TextHAlignment::Left,
            v_alignment: TextVAlignment::Top,
        }));

        self.commands.last_mut().unwrap().as_text_mut()
    }

    pub fn get_text_width(&self, text: &str) -> f32 {
        let mut len = 0.0;

        for c in text.chars() {
            let glyph = self
                .default_font
                .font_chars
                .get(&c)
                .expect("Glyph not found");

            len += (glyph.advance >> 6) as f32;
        }

        len
    }

    pub fn get_text_height(&self, text: &str) -> f32 {
        let mut height: f32 = 0.0;

        for c in text.chars() {
            let glyph = self
                .default_font
                .font_chars
                .get(&c)
                .expect("Glyph not found");
            height = height.max(glyph.size.y as f32);
        }

        height
    }

    pub fn draw_submit(&mut self) {
        unsafe {
            //gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::ClearColor(0.9, 0.9, 0.9, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            //gl::Enable(gl::CULL_FACE);
            //gl::Enable(gl::DEPTH_TEST);
        }

        self.enable_alpha_blending();

        for cmd in &self.commands {
            match cmd {
                DrawCommand::CIRCLE(circle) => unsafe {
                    let x1 = circle.pos.x - circle.rad;
                    let y1 = circle.pos.y - circle.rad;
                    let x2 = circle.pos.x + circle.rad;
                    let y2 = circle.pos.y + circle.rad;

                    let vertices: [(f32, f32); 6] =
                        [(x1, y2), (x1, y1), (x2, y1), (x1, y2), (x2, y1), (x2, y2)];

                    let shape_pos_and_size = Vec4::new(
                        circle.pos.x,
                        self.surface_height as f32 - circle.pos.y,
                        circle.rad,
                        circle.rad,
                    );

                    let shape_radius = Vec4::new(circle.rad - 2.0, 0.0, 0.0, 0.0);
                    self.shape_program.use_program();
                    self.shape_program.set_mat4("p", self.surface_projection);
                    self.shape_program.set_int("mode", 1);
                    self.shape_program.set_vec4("color", circle.col);
                    self.shape_program
                        .set_vec4("shapePosAndSize", shape_pos_and_size);
                    self.shape_program.set_vec4("shapeRadius", shape_radius);

                    gl::BindVertexArray(self.shape_vertex_buffer.vao);
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.shape_vertex_buffer.vbo);
                    gl::BufferSubData(
                        gl::ARRAY_BUFFER,
                        0,
                        mem::size_of_val(&vertices) as isize,
                        vertices.as_ptr() as *const c_void,
                    );
                    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                    gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as i32);
                    gl::BindVertexArray(0);
                },

                DrawCommand::RECT(rect) => unsafe {
                    let vertices: [(f32, f32); 6] = [
                        (rect.tl.x, rect.tl.y),
                        (rect.bl.x, rect.bl.y),
                        (rect.br.x, rect.br.y),
                        (rect.tl.x, rect.tl.y),
                        (rect.br.x, rect.br.y),
                        (rect.tr.x, rect.tr.y),
                    ];

                    self.shape_program.use_program();
                    self.shape_program.set_mat4("p", self.surface_projection);
                    self.shape_program.set_int("mode", 0);
                    self.shape_program.set_vec4("color", rect.col);

                    gl::BindVertexArray(self.shape_vertex_buffer.vao);
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.shape_vertex_buffer.vbo);
                    gl::BufferSubData(
                        gl::ARRAY_BUFFER,
                        0,
                        mem::size_of_val(&vertices) as isize,
                        vertices.as_ptr() as *const c_void,
                    );
                    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                    gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as i32);
                    gl::BindVertexArray(0);
                },

                DrawCommand::TRI(tri) => unsafe {
                    let vertices: [(f32, f32); 3] = [
                        (tri.p1.x, tri.p1.y),
                        (tri.p2.x, tri.p2.y),
                        (tri.p3.x, tri.p3.y),
                    ];

                    self.shape_program.set_int("mode", 0);
                    self.shape_program.set_vec4("color", tri.col);

                    gl::BindVertexArray(self.shape_vertex_buffer.vao);
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.shape_vertex_buffer.vbo);
                    gl::BufferSubData(
                        gl::ARRAY_BUFFER,
                        0,
                        mem::size_of_val(&vertices) as isize,
                        vertices.as_ptr() as *const c_void,
                    );
                    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                    gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as i32);
                    gl::BindVertexArray(0);
                },
                DrawCommand::TEXT(text_cmd) => unsafe {
                    let text = &text_cmd.text;
                    let mut x = text_cmd.pos.x;
                    let mut y = text_cmd.pos.y;

                    match text_cmd.h_alignment {
                        TextHAlignment::Left => (),
                        TextHAlignment::Center => {
                            let text_width = self.get_text_width(text);
                            x -= text_width / 2.0;
                        }
                        TextHAlignment::Right => {
                            let text_width = self.get_text_width(text);
                            x -= text_width;
                        }
                    }

                    match text_cmd.v_alignment {
                        TextVAlignment::Top => (),
                        TextVAlignment::Center => {
                            let text_height = self.get_text_height(text);
                            y += text_height / 2.0;
                        }
                        TextVAlignment::Bottom => {
                            let text_height = self.get_text_height(text);
                            y += text_height;
                        }
                    }

                    self.font_program.use_program();
                    self.font_program
                        .set_mat4("projection", self.surface_projection);
                    self.font_program.set_int("text", 0);

                    gl::BindVertexArray(self.font_vertex_buffer.vao);

                    for c in text.chars() {
                        let ch = self
                            .default_font
                            .font_chars
                            .get(&c)
                            .expect("Font char not found");
                        let xpos = x + ch.bearing.x;
                        let ypos = y + (ch.size.y - ch.bearing.y);
                        let w = ch.size.x;
                        let h = ch.size.y;
                        let vertices: [(f32, f32, f32, f32); 6] = [
                            (xpos, ypos - h, 0.0, 0.0),
                            (xpos, ypos, 0.0, 1.0),
                            (xpos + w, ypos, 1.0, 1.0),
                            (xpos, ypos - h, 0.0, 0.0),
                            (xpos + w, ypos, 1.0, 1.0),
                            (xpos + w, ypos - h, 1.0, 0.0),
                        ];

                        gl::BindTextureUnit(0, ch.texture);
                        gl::BindBuffer(gl::ARRAY_BUFFER, self.font_vertex_buffer.vbo);
                        gl::BufferSubData(
                            gl::ARRAY_BUFFER,
                            0,
                            mem::size_of_val(&vertices) as isize,
                            vertices.as_ptr() as *const c_void,
                        );
                        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                        gl::DrawArrays(gl::TRIANGLES, 0, 6);
                        x += (ch.advance >> 6) as f32;
                    }

                    gl::BindVertexArray(0);
                },
                DrawCommand::SPRITE(sprite) => {

                },
            }
        }

        self.commands.clear();
    }
}
