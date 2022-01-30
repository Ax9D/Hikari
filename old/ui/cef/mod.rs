use cef::{app::App, browser_host::BrowserHost, browser_process_handler::BrowserProcessHandler, client::{life_span_handler::LifeSpanHandler, render_handler::RenderHandler}, settings::{LogSeverity, Settings}, window::{RawWindow, WindowInfo}};
use winit::event_loop::EventLoop;

use self::primitives::{AppCallbacksImpl, BrowserProcessHandlerCallbacksImpl, ClientCallbacksImpl, LifeSpanHandlerImpl, RenderHandlerCallbacksImpl};

mod primitives;


pub struct CEFContext {
    
}

impl CEFContext {
    fn new(window: glfw::Window) {
        let framework_dir_path = {
            #[cfg(target_os = "macos")] {
                Some(cef::load_framework(None).unwrap())
            }
            #[cfg(not(target_os = "macos"))] {
                None
            }
        };
        

        log::info!("testing logs");
        
        let app = App::new(AppCallbacksImpl {
            browser_process_handler: BrowserProcessHandler::new(
                BrowserProcessHandlerCallbacksImpl {
                },
            ),
        });

        let mut settings = Settings::new()
            .log_severity(LogSeverity::Verbose)
            .windowless_rendering_enabled(true)
            .external_message_pump(true);

        settings.framework_dir_path = framework_dir_path;

        let context = cef::Context::initialize(settings, Some(app), None).unwrap();

        let window_builder = WindowBuilder::new()
            .with_title("CEF Example Window");

        let width = renderer.window().inner_size().width;
        let height = renderer.window().inner_size().height;

        
        //let window = glfw::init().unwrap();
        let window = window.create_window(width, height, title, mode).unwrap();
        let h=window.0.raw_window_handle();
        let window_info = WindowInfo {
            windowless_rendering_enabled: true,
            parent_window: Some(unsafe { RawWindow::from_window(renderer.window()) }),
            width: width as _,
            height: height as _,
            ..WindowInfo::new()
        };

        let browser_settings = BrowserSettings {
            background_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            ..BrowserSettings::new()
        };

        let renderer = Arc::new(Mutex::new(renderer));
        let client = Client::new(ClientCallbacksImpl {
            life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {
            }),
            render_handler: RenderHandler::new(RenderHandlerCallbacksImpl {
                renderer: Arc::clone(&renderer),
            }),
        });

        let browser = BrowserHost::create_browser_sync(
            &window_info,
            client,
            "http://html5advent2011.digitpaint.nl/3/index.html",
            &browser_settings,
            None,
            None,
        );

        println!("initialize done");

    }
}