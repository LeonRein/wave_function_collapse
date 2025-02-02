use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .resizable()
        .vulkan()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut last_frametime = Instant::now();
    let mut frametime_buffer: VecDeque<f32> = VecDeque::new();
    let mut last_fps_update = Instant::now();

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let now = Instant::now();
        let frametime = now.duration_since(last_frametime);
        last_frametime = now;

        if frametime_buffer.len() > 100 {
            frametime_buffer.pop_back();
        }
        frametime_buffer.push_front(frametime.as_secs_f32() * 1000.0);
        let frametime_avg = frametime_buffer.iter().sum::<f32>() / frametime_buffer.len() as f32;

        if last_fps_update.elapsed() >= Duration::from_secs(1) {
            println!("FPS: {:.2}", 1000.0 / frametime_avg);
            last_fps_update = now;
        }

        canvas.clear();
        canvas.present();
        // The rest of the game loop goes here...
    }

    Ok(())
}
