use core::f32;
use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct AppData {
    size: (u32, u32),
    surface: Surface<Rc<Window>, Rc<Window>>,
    window: Rc<Window>,
}

struct App {
    data: Option<AppData>,
    last_frametime: Instant,
    frametime_buffer: VecDeque<f32>,
    n_frame: u32,
    last_fps_update: Instant,
}

impl App {
    fn new() -> Self {
        App {
            data: None,
            n_frame: 0,
            last_frametime: Instant::now(),
            frametime_buffer: VecDeque::new(),
            last_fps_update: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let context = Context::new(Rc::clone(&window)).unwrap();
        let surface = softbuffer::Surface::new(&context, Rc::clone(&window)).unwrap();
        self.data = Some(AppData {
            surface,
            window,
            size: (0, 0),
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let _ = id;
        let Some(data) = &mut self.data else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.frametime_buffer.clear();
                let _ = data.surface.resize(
                    NonZeroU32::new(size.width).unwrap(),
                    NonZeroU32::new(size.height).unwrap(),
                );
                data.size = (size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                data.window.request_redraw();
                self.n_frame += 1;
                let now = Instant::now();
                let frametime = now.duration_since(self.last_frametime);
                self.last_frametime = now;

                if self.frametime_buffer.len() > 100 {
                    self.frametime_buffer.pop_back();
                }
                self.frametime_buffer.push_front(frametime.as_secs_f32());
                let frametime_avg =
                    self.frametime_buffer.iter().sum::<f32>() / self.frametime_buffer.len() as f32;

                if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
                    println!("FPS: {:.2}", 1.0 / frametime_avg);
                    self.last_fps_update = now;
                }

                let mut pixel_buffer = data.surface.buffer_mut().unwrap();
                pixel_buffer.fill(0x00FF0000);
                pixel_buffer.present().unwrap();
            }
            _ => (),
        }
    }
}

pub fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
