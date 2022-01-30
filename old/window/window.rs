use std::{error::Error, ffi::c_void, sync::mpsc::Receiver};

use glfw::Context;

use super::Event;

pub struct Window {
    window: glfw::Window,
    glfw: glfw::Glfw,
}
pub struct EventPump {
    glfw: glfw::Glfw,
    receiver: Receiver<(f64, glfw::WindowEvent)>,
}

pub struct EventIter<'a> {
    receiver: glfw::FlushedMessages<'a, (f64, glfw::WindowEvent)>,
}
impl<'a> Iterator for EventIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let event = match self.receiver.next() {
            Some((_, window_event)) => {
                let event = Event::new(window_event);
                Some(event)
            }
            None => None,
        };
        event
    }
}
impl EventPump {
    pub fn poll_events(&mut self) -> EventIter {
        self.glfw.poll_events();

        let receiver = &self.receiver;
        let receiver = glfw::flush_messages(receiver);

        EventIter { receiver }
    }
}
pub struct GLFWContext {
    glfw: glfw::Glfw,
    window: glfw::Window,
    receiver: Receiver<(f64, glfw::WindowEvent)>,
}
fn create_glfw_window(
    window_width: u32,
    window_height: u32,
) -> Result<GLFWContext, Box<dyn Error>> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    use glfw::WindowHint::*;
    glfw.window_hint(ContextVersionMajor(4));
    glfw.window_hint(ContextVersionMinor(5));
    glfw.window_hint(OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(OpenGlForwardCompat(true));

    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

    let (mut window, events) = glfw
        .create_window(window_width, window_height, "", glfw::WindowMode::Windowed)
        .ok_or("Failed to create GLFW Window")?;

    window.make_current();

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    window.set_framebuffer_size_polling(true);

    window.set_all_polling(true);

    Ok(GLFWContext {
        glfw,
        window,
        receiver: events,
    })
}

impl Window {
    pub fn create(width: u32, height: u32) -> Result<(Window, EventPump), Box<dyn Error>> {
        let glfw_context = create_glfw_window(width, height)?;
        let glfw = glfw_context.glfw;

        let window = Window {
            window: glfw_context.window,
            glfw: glfw.clone(),
        };

        let event_pump = EventPump {
            glfw,
            receiver: glfw_context.receiver,
        };

        Ok((window, event_pump))
    }

    pub fn set_vsync(&mut self, yes: bool) {
        match yes {
            true => self.glfw.set_swap_interval(glfw::SwapInterval::Sync(1)),
            false => self.glfw.set_swap_interval(glfw::SwapInterval::Sync(0)),
        }
    }
    pub fn request_close(&mut self) {
        self.window.set_should_close(true);
    }
    // pub fn pollEvents(&mut self) {
    //     let events = &self.glfwContext.receiver;

    //     for (_, event) in glfw::flush_messages(events) {
    //         let windowEvent=match event {
    //             glfw::WindowEvent::Key(k, _, glfw::Action::Press, _) => {
    //                 Event::Key(glfwKey2OurKey(k), Action::Pressed)
    //             }

    //             // glfw::WindowEvent::FramebufferSize(width, height) => {
    //             //     self.engine.onViewportResize(width as u32, height as u32);
    //             // }
    //             _=>{Event::Unknown}
    //         };
    //     }
    // }
    pub fn is_open(&self) -> bool {
        !self.window.should_close()
    }
    pub fn get_size(&self) -> (u32, u32) {
        let (w, h) = self.window.get_size();

        (w as u32, h as u32)
    }
    pub fn get_proc_address(&mut self, procname: &str) -> *const c_void {
        self.window.get_proc_address(procname)
    }
    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }
    pub fn get_raw(&self) -> &glfw::Window {
        &self.window
    }
    pub fn get_raw_mut(&mut self) -> &mut glfw::Window {
        &mut self.window
    }
    pub fn set_cursor_mode(&mut self, cursor_mode: glfw::CursorMode) {
        self.window.set_cursor_mode(cursor_mode);
    }
    pub fn set_cursor(&mut self, cursor: glfw::Cursor) {
        self.window.set_cursor(Some(cursor));
    }
    pub fn update(&mut self) {
        self.window.swap_buffers();
    }
}
