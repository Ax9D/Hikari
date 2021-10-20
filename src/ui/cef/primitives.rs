use std::{ffi::c_void, sync::Arc, time::Instant};

use cef::{app::AppCallbacks, browser::Browser, browser_host::PaintElementType, browser_process_handler::{BrowserProcessHandler, BrowserProcessHandlerCallbacks}, client::{ClientCallbacks, life_span_handler::{LifeSpanHandler, LifeSpanHandlerCallbacks}, render_handler::{CursorType, RenderHandler, RenderHandlerCallbacks, ScreenInfo}}, command_line::CommandLine, drag::DragOperation, values::{Point, Rect}};
use cef_sys::cef_cursor_handle_t;
use glfw::Window;
use std::time::Duration;
use parking_lot::Mutex;
use winit::{dpi::{LogicalPosition, PhysicalPosition}, event_loop::EventLoopProxy, window::WindowBuilder};

pub struct AppCallbacksImpl {
    browser_process_handler: BrowserProcessHandler,
}
pub struct ClientCallbacksImpl {
    life_span_handler: LifeSpanHandler,
    render_handler: RenderHandler,
}
pub struct LifeSpanHandlerImpl {
    // proxy: Mutex<EventLoopProxy<CefEvent>>,
}
pub struct BrowserProcessHandlerCallbacksImpl {
    // proxy: Mutex<EventLoopProxy<CefEvent>>,
}
pub struct RenderHandlerCallbacksImpl<R: Renderer> {
    renderer: Arc<Mutex<R>>,
}

pub trait Renderer: 'static + Send {
    fn new<T>(window_builder: WindowBuilder) -> Self;
    fn window(&self) -> &Window;
    fn on_paint(
        &mut self,
        element_type: PaintElementType,
        dirty_rects: &[Rect],
        buffer: &[u8],
        width: i32,
        height: i32,
    );
    fn set_popup_rect(&mut self, rect: Option<Rect>);
}
impl<R: Renderer> RenderHandlerCallbacks for RenderHandlerCallbacksImpl<R> {
    fn get_view_rect(&self, _: Browser) -> Rect {
        let renderer = self.renderer.lock();
        let window = renderer.window();
        let inner_size = window.inner_size().to_logical::<i32>(window.scale_factor());
        Rect {
            x: 0,
            y: 0,
            width: inner_size.width,
            height: inner_size.height,
        }
    }
    fn on_popup_show(&self, _browser: Browser, show: bool) {
        if !show {
            self.renderer.lock().set_popup_rect(None);
        }
    }
    fn get_screen_point(&self, _browser: Browser, point: Point) -> Option<Point> {
        let renderer = self.renderer.lock();
        let window = renderer.window();

        let screen_pos = window
            .inner_position()
            .unwrap_or(PhysicalPosition::new(0, 0));
        let point_physical =
            LogicalPosition::new(point.x, point.y).to_physical::<i32>(window.scale_factor());
        Some(Point::new(
            screen_pos.x + point_physical.x,
            screen_pos.y + point_physical.y,
        ))
    }
    fn on_popup_size(&self, _: Browser, mut rect: Rect) {
        let mut renderer = self.renderer.lock();
        let window = renderer.window();

        let window_size: (u32, u32) = window.inner_size().into();
        let window_size = (window_size.0 as i32, window_size.1 as i32);
        rect.x = i32::max(rect.x, 0);
        rect.y = i32::max(rect.y, 0);
        rect.x = i32::min(rect.x, window_size.0 - rect.width);
        rect.y = i32::min(rect.y, window_size.1 - rect.height);
        renderer.set_popup_rect(Some(rect));
    }
    fn get_screen_info(&self, _: Browser) -> Option<ScreenInfo> {
        let renderer = self.renderer.lock();
        let window = renderer.window();

        let inner_size = window.inner_size().to_logical::<i32>(window.scale_factor());
        let rect = Rect {
            x: 0,
            y: 0,
            width: inner_size.width,
            height: inner_size.height,
        };

        Some(ScreenInfo {
            device_scale_factor: window.scale_factor() as f32,
            depth: 32,
            depth_per_component: 8,
            is_monochrome: false,
            rect: rect,
            available_rect: rect,
        })
    }
    fn on_paint(
        &self,
        _: Browser,
        element_type: PaintElementType,
        dirty_rects: &[Rect],
        buffer: &[u8],
        width: i32,
        height: i32,
    ) {
        // FIXME: this completely ignores dirty rects for now and only
        // just re-uploads and re-renders everything anew
        assert_eq!(buffer.len(), 4 * (width * height) as usize);

        println!("{:?}",buffer);

        let mut renderer = self.renderer.lock();
        renderer.on_paint(element_type, dirty_rects, buffer, width, height);
    }
    fn on_accelerated_paint(
        &self,
        _browser: Browser,
        _type_: PaintElementType,
        _dirty_rects: &[Rect],
        _shared_handle: *mut c_void,
    ) {
        unimplemented!()
    }
    fn on_cursor_change(&self, _browser: Browser, _cursor: cef_cursor_handle_t, type_: CursorType) {
        // this is a good website for testing cursor changes
        // http://html5advent2011.digitpaint.nl/3/index.html
        
    }
    fn update_drag_cursor(&self, _browser: Browser, _operation: DragOperation) {}
}

#[derive(Debug, Clone)]
enum CefEvent {
    ScheduleWork(Instant),
    Quit,
}

impl AppCallbacks for AppCallbacksImpl {
    fn on_before_command_line_processing(
        &self,
        process_type: Option<&str>,
        command_line: CommandLine,
    ) {
        if process_type == None {
            command_line.append_switch("disable-gpu");
            command_line.append_switch("disable-gpu-compositing");
        }
    }
    fn get_browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        Some(self.browser_process_handler.clone())
    }
}

impl ClientCallbacks for ClientCallbacksImpl {
    fn get_life_span_handler(&self) -> Option<LifeSpanHandler> {
        Some(self.life_span_handler.clone())
    }
    fn get_render_handler(&self) -> Option<RenderHandler> {
        Some(self.render_handler.clone())
    }
}

impl LifeSpanHandlerCallbacks for LifeSpanHandlerImpl {
    fn on_before_close(&self, _browser: Browser) {
        // self.proxy.lock().send_event(CefEvent::Quit).unwrap();
    }
}

impl BrowserProcessHandlerCallbacks for BrowserProcessHandlerCallbacksImpl {
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
        // self.proxy
        //     .lock()
        //     .send_event(CefEvent::ScheduleWork(
        //         Instant::now() + Duration::from_millis(delay_ms as u64),
        //     ))
        //     .ok();
    }
}