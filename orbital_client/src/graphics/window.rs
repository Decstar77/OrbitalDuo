use glfw::{Action, Context, WindowEvent};
use std::sync::mpsc::Receiver;

use crate::{config::get_config, types::Vec2};

use super::renderer::RenderState;

pub type KeyCode = glfw::Key;

pub struct FrameState {
    pub prev_time: f32,
    pub time: f32,
    pub delta_time: f32,
    pub mouse_scroll: f32,
    pub keys: [bool; 1024],
    pub prev_keys: [bool; 1024],
    pub mouse_buttons: [bool; 8],
    pub prev_mouse_buttons: [bool; 8],
    pub mouse_pos: Vec2,
}

impl FrameState {
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys[key as usize]
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        !self.keys[key as usize]
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys[key as usize] && !self.prev_keys[key as usize]
    }

    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        !self.keys[key as usize] && self.prev_keys[key as usize]
    }

    pub fn is_mouse_pressed(&self, button: u8) -> bool {
        self.mouse_buttons[button as usize]
    }

    pub fn is_mouse_released(&self, button: u8) -> bool {
        !self.mouse_buttons[button as usize]
    }

    pub fn is_mouse_just_pressed(&self, button: u8) -> bool {
        self.mouse_buttons[button as usize] && !self.prev_mouse_buttons[button as usize]
    }

    pub fn is_mouse_just_released(&self, button: u8) -> bool {
        !self.mouse_buttons[button as usize] && self.prev_mouse_buttons[button as usize]
    }

    pub fn new() -> FrameState {
        FrameState {
            prev_time: 0.0,
            time: 0.0,
            delta_time: 0.0,
            mouse_scroll: 0.0,
            keys: [false; 1024],
            prev_keys: [false; 1024],
            mouse_buttons: [false; 8],
            prev_mouse_buttons: [false; 8],
            mouse_pos: Vec2::new(0.0, 0.0),
        }
    }
}

pub struct Window {
    glfw: glfw::Glfw,
    window_handle: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    pub client_width: i32,
    pub client_height: i32,
}

impl Window {
    /// Create new window with settings
    pub fn new(title: &str) -> Window {
        let cfg = get_config();

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::ScaleToMonitor(false));
        //glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

        let width = cfg.window_size.unwrap().x as u32;
        let height = cfg.window_size.unwrap().y as u32;

        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window!");

        window.set_framebuffer_size_polling(true);
        window.set_key_polling(true);
        window.set_cursor_enter_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);

        match cfg.window_pos {
            Some(pos) => window.set_pos(pos.x as i32, pos.y as i32),
            None => {
                glfw.with_primary_monitor(|_, primary_monitor| {
                    if let Some(primary_monitor) = primary_monitor {
                        if let Some(monitor_video_mode) = primary_monitor.get_video_mode() {
                            let monitor_width = monitor_video_mode.width;
                            let monitor_height = monitor_video_mode.height;

                            let window_pos_x = (monitor_width as i32 - width as i32) / 2;
                            let window_pos_y = (monitor_height as i32 - height as i32) / 2;

                            window.set_pos(window_pos_x, window_pos_y);
                        }
                    }
                });
            }
        }

        let (client_width, client_height) = window.get_framebuffer_size();

        let mut window = Window {
            glfw,
            window_handle: window,
            events,
            client_width,
            client_height,
        };

        window.init_gl();

        window
    }

    pub fn init_gl(&mut self) {
        self.window_handle.make_current();
        gl::load_with(|s| self.window_handle.get_proc_address(s) as *const _);

        self.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        //self.glfw.set_swap_interval(glfw::SwapInterval::None);

        unsafe {
            let version = gl::GetString(gl::VERSION);
            let vendor = gl::GetString(gl::VENDOR);
            let renderer = gl::GetString(gl::RENDERER);
            println!(
                "OpenGL version {}, vendor {}, renderer {}",
                std::str::from_utf8_unchecked(
                    std::ffi::CStr::from_ptr(version as *const _).to_bytes()
                ),
                std::str::from_utf8_unchecked(
                    std::ffi::CStr::from_ptr(vendor as *const _).to_bytes()
                ),
                std::str::from_utf8_unchecked(
                    std::ffi::CStr::from_ptr(renderer as *const _).to_bytes()
                ),
            );
        }
    }

    pub fn should_close(&self) -> bool {
        self.window_handle.should_close()
    }

    pub fn update(&mut self, rs: &mut RenderState, frame_input: &mut FrameState) {
        self.process_events(rs, frame_input);
        self.glfw.poll_events();
        self.window_handle.swap_buffers();
    }

    fn process_events(&mut self, rs: &mut RenderState, frame_input: &mut FrameState) {
        frame_input.prev_time = frame_input.time;
        frame_input.time = glfw::Glfw::get_time(&self.glfw) as f32;
        frame_input.delta_time = frame_input.time - frame_input.prev_time;
        frame_input.prev_keys = frame_input.keys;
        frame_input.prev_mouse_buttons = frame_input.mouse_buttons;

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    rs.surface_resized(width, height);
                },

                glfw::WindowEvent::Key(key, scancode, action, modifiers) => {
                    if key == glfw::Key::Escape && action == Action::Press {
                        self.window_handle.set_should_close(true)
                    }

                    frame_input.keys[key as usize] = action != Action::Release;
                }

                glfw::WindowEvent::MouseButton(button, action, modifiers) => {
                    frame_input.mouse_buttons[button as usize] = action != Action::Release;
                }

                glfw::WindowEvent::CursorPos(xpos, ypos) => {
                    frame_input.mouse_pos = Vec2::new(xpos as f32, ypos as f32);
                }

                _ => {}
            }
        }
    }
}
