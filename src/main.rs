#![feature(generic_const_exprs, get_many_mut)]

mod grid;
mod tileset;
use grid::Grid;
use image::{ImageReader, RgbImage};
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use sdl2::{event::Event, rect::Rect};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use tileset::TileSet;

const SCALE: u32 = 10;
const TILE_SIZE: usize = 3;
const GRID_HEIGHT: usize = 20;
const GRID_WIDTH: usize = 40;

struct App<'a> {
    canvas: Canvas<sdl2::video::Window>,
    texture_creator: TextureCreator<WindowContext>,
    event_pump: sdl2::EventPump,
    font: Font<'a, 'a>,
    n_frame: u32,
    last_frametime: Instant,
    frametime_buffer: VecDeque<f32>,
    last_fps_update: Instant,
    grid: Grid<TILE_SIZE, TILE_SIZE, GRID_WIDTH, GRID_HEIGHT>,
}


impl<'a> App<'a> {
    fn new(sdl_context: &sdl2::Sdl, font: Font<'a, 'a>) -> Result<Self, String> {
        let video_subsystem = sdl_context.video()?;

        // Create a resizable window
        let window = video_subsystem
            .window("rust-sdl2 demo: Video", 800, 600)
            .position_centered()
            .resizable()
            .vulkan()
            .build()
            .map_err(|e| e.to_string())?;

        // Create a canvas for rendering
        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        // Initialize the event pump
        let event_pump = sdl_context.event_pump()?;

        let image: RgbImage = ImageReader::open("samples/City.png")
            .unwrap()
            .decode()
            .unwrap()
            .into_rgb8();

        Ok(App {
            canvas,
            texture_creator,
            event_pump,
            font,
            last_frametime: Instant::now(),
            frametime_buffer: VecDeque::new(),
            last_fps_update: Instant::now(),
            grid: Grid::new(TileSet::new(&image)),
            n_frame: 0,
        })
    }

    fn handle_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return false,

                Event::KeyDown {
                    keycode: Some(Keycode::SPACE),
                    ..
                } => self.grid.collapse_step(),

                // Handle window resize events
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    // Update the canvas's viewport to match the new window size
                    self.canvas
                        .set_viewport(Rect::new(0, 0, width as u32, height as u32));
                }

                _ => {}
            }
        }
        true
    }

    fn display_fps(&mut self) -> Result<(), String> {
        self.n_frame += 1;
        let now = Instant::now();
        let frametime = now.duration_since(self.last_frametime);
        self.last_frametime = now;

        // Maintain a buffer of the last 100 frame times
        if self.frametime_buffer.len() > 100 {
            self.frametime_buffer.pop_back();
        }
        self.frametime_buffer
            .push_front(frametime.as_secs_f32() * 1000.0);

        // Calculate the average frame time
        let frametime_avg =
            self.frametime_buffer.iter().sum::<f32>() / self.frametime_buffer.len() as f32;

        // Print FPS every second
        if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
            println!("FPS: {:.2}", 1000.0 / frametime_avg);
            self.last_fps_update = now;
        }

        // Render FPS to the canvas
        let fps_text = format!("FPS: {:.2}", 1000.0 / frametime_avg);
        let surface = self
            .font
            .render(&fps_text)
            .blended(Color::WHITE)
            .map_err(|e| e.to_string())?;
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        // Set the destination rect to position the text at the top-left corner
        let dest_rect = Rect::new(10, 10, surface.width(), surface.height());

        // Copy the texture to the canvas
        self.canvas.copy(&texture, None, Some(dest_rect))?;
        Ok(())
    }

    fn clear_canvas(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
    }

    fn draw_scene(&mut self) {
        let grid = &mut self.grid;
        // let tileset = &grid.tileset;
        // tileset.draw(&mut self.canvas, 9, SCALE);
        // let index = (self.n_frame / 10) as usize % (tileset.len() * 4);
        // let direction = index / tileset.len();
        // let index = index % tileset.len();

        // tileset.draw_neighbors(
        //     &mut self.canvas,
        //     index,
        //     Direction::VALUES[direction],
        //     9,
        //     SCALE,
        // );

        grid.draw(&mut self.canvas, &self.texture_creator, &self.font, SCALE);
        // grid.collapse_step();
        // let _ = self.canvas.fill_rect(Rect::new(0, 0, 100, 100));
    }

    fn present_canvas(&mut self) {
        self.canvas.present();
    }

    fn update(&mut self) -> Result<(), String> {
        // Clear canvas, display FPS, and draw scene
        self.clear_canvas();
        self.draw_scene();
        self.display_fps()?;
        self.present_canvas();
        Ok(())
    }

    // Main loop
    fn run(&mut self) -> Result<(), String> {
        // Main loop
        'running: loop {
            // Handle events
            if !self.handle_events() {
                break 'running;
            }

            // Update frame
            self.update()?;
        }
        Ok(())
    }
}

pub(crate) fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font = ttf_context
        .load_font("OpenSans-Regular.ttf", 15)
        .map_err(|e| e.to_string())?;

    let mut app = App::new(&sdl_context, font)?;

    // Start the application main loop
    app.run()?;

    Ok(())
}
