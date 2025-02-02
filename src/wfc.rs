use std::array::from_fn;

use bittyset::BitSet;
use image::Rgb;
use image::RgbImage;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

#[derive(Clone, Copy)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    pub const VALUES: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];
}

pub struct TileSet<const TILE_WIDTH: usize, const TILE_HEIGHT: usize>
where
    [(); TILE_WIDTH * TILE_HEIGHT]:,
{
    tiles: Vec<Tile<TILE_WIDTH, TILE_HEIGHT>>,
}

impl<const TILE_WIDTH: usize, const TILE_HEIGHT: usize> TileSet<TILE_WIDTH, TILE_HEIGHT>
where
    [(); TILE_WIDTH * TILE_HEIGHT]:,
{
    pub fn new(image: &RgbImage) -> TileSet<TILE_WIDTH, TILE_HEIGHT> {
        let width = image.dimensions().0 as usize;
        let height = image.dimensions().1 as usize;
        let image: Vec<u32> = image
            .pixels()
            .map(|pixel| {
                let Rgb(data) = pixel;
                let bytes = [data[0], data[1], data[2], 0]; // Add zero for the alpha channel
                u32::from_le_bytes(bytes)
            })
            .collect();
        assert!(width - TILE_WIDTH > 0);
        assert!(height - TILE_HEIGHT > 0);
        let mut tiles: Vec<Tile<TILE_WIDTH, TILE_HEIGHT>> = Vec::new();
        for x in 0..width {
            for y in 0..height {
                let mut pixels = Vec::new();
                for dx in 0..TILE_WIDTH {
                    for dy in 0..TILE_HEIGHT {
                        let x = (x + dx) % width;
                        let y = (y + dy) % height;
                        let index = x + y * width;
                        pixels.push(image[index]);
                    }
                }
                // let exists = tiles
                //     .iter_mut()
                //     .find_map(|tile| {
                //         if tile.pixels == pixels.as_slice() {
                //             tile.frequency += 1;
                //             Some(())
                //         } else {
                //             None
                //         }
                //     })
                //     .is_some();
                let exists = false;
                if !exists {
                    tiles.push(Tile::new(pixels));
                }
            }
        }
        let mut tile_set = TileSet { tiles };
        tile_set.generate_neighbors();
        tile_set
    }

    fn generate_neighbors(&mut self) {
        for ia in 0..self.tiles.len() {
            for ib in 0..self.tiles.len() {
                let tile_b = self.tiles[ib].pixels.to_vec();
                let tile_b = Tile::new(tile_b);
                let tile_a = &mut self.tiles[ia];
                for direction in Direction::VALUES {
                    if tile_a.cmp_adjacent(&tile_b, direction.clone()) {
                        tile_a.neighbors[direction as usize].insert(ib);
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, width: usize, scale: u32) {
        for (index, tile) in self.tiles.iter().enumerate() {
            let x = (index % width) as i32 * (scale + 3) as i32 * TILE_WIDTH as i32;
            let y = (index / width) as i32 * (scale + 3) as i32 * TILE_HEIGHT as i32;
            tile.draw(canvas, x, y, scale);
        }
    }

    pub fn draw_neighbors(
        &self,
        canvas: &mut Canvas<Window>,
        index: usize,
        direction: Direction,
        width: usize,
        scale: u32,
    ) {
        let tile = &self.tiles[index];
        tile.draw(canvas, 0, 0, scale);
        let mut tiles = Vec::new();
        for ix in tile.neighbors[direction as usize].iter() {
            tiles.push(&self.tiles[ix]);
        }
        let x_offset = scale as i32 * TILE_WIDTH as i32 + 30 as i32;
        let y_offset = 0;
        for (index, &tile) in tiles.iter().enumerate() {
            let x = (index % width) as i32 * (scale + 3) as i32 * TILE_WIDTH as i32 + x_offset;
            let y = (index / width) as i32 * (scale + 3) as i32 * TILE_HEIGHT as i32 + y_offset;
            tile.draw(canvas, x, y, scale);
        }
    }
}

struct Tile<const WIDTH: usize, const HEIGHT: usize>
where
    [(); WIDTH * HEIGHT]:,
{
    pixels: [u32; WIDTH * HEIGHT],
    neighbors: [BitSet<usize>; 4],
    frequency: u32,
}

impl<const WIDTH: usize, const HEIGHT: usize> Tile<WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    fn new(pixels: Vec<u32>) -> Tile<WIDTH, HEIGHT> {
        Self {
            pixels: pixels.try_into().unwrap(),
            neighbors: from_fn(|_| BitSet::new()),
            frequency: 1,
        }
    }

    fn adjacent_north(&self, other: &Self) -> bool {
        for xa in 0..WIDTH {
            for ya in 0..HEIGHT - 1 {
                let xb = xa;
                let yb = ya + 1;
                let ia = xa + ya * WIDTH;
                let ib = xb + yb * WIDTH;
                if self.pixels[ia] != other.pixels[ib] {
                    return false;
                }
            }
        }
        return true;
    }

    fn adjacent_south(&self, other: &Self) -> bool {
        for xa in 0..WIDTH {
            for ya in 1..HEIGHT {
                let xb = xa;
                let yb = ya - 1;
                let ia = xa + ya * WIDTH;
                let ib = xb + yb * WIDTH;
                if self.pixels[ia] != other.pixels[ib] {
                    return false;
                }
            }
        }
        return true;
    }

    fn ajacetn_east(&self, other: &Self) -> bool {
        for xa in 1..WIDTH {
            for ya in 0..HEIGHT {
                let xb = xa - 1;
                let yb = ya;
                let ia = xa + ya * WIDTH;
                let ib = xb + yb * WIDTH;
                if self.pixels[ia] != other.pixels[ib] {
                    return false;
                }
            }
        }
        return true;
    }

    fn adjacent_west(&self, other: &Self) -> bool {
        for xa in 0..WIDTH - 1 {
            for ya in 0..HEIGHT {
                let xb = xa + 1;
                let yb = ya;
                let ia = xa + ya * WIDTH;
                let ib = xb + yb * WIDTH;
                if self.pixels[ia] != other.pixels[ib] {
                    return false;
                }
            }
        }
        return true;
    }

    fn cmp_adjacent(&self, other: &Self, direction: Direction) -> bool {
        match direction {
            Direction::North => self.adjacent_north(other),
            Direction::East => self.ajacetn_east(other),
            Direction::South => self.adjacent_south(other),
            Direction::West => self.adjacent_west(other),
        }
    }
    fn draw(&self, canvas: &mut Canvas<Window>, x: i32, y: i32, scale: u32) {
        for tile_x in 0..WIDTH {
            for tile_y in 0..HEIGHT {
                let color = self.pixels[tile_x + tile_y * WIDTH];
                let color = Color::from_u32(&PixelFormatEnum::RGBA32.try_into().unwrap(), color);
                let x = x + tile_x as i32 * scale as i32;
                let y = y + tile_y as i32 * scale as i32;
                canvas.set_draw_color(color);
                let rect = Rect::new(x, y, scale, scale);
                let _ = canvas.fill_rect(rect);
                canvas.set_draw_color(Color::WHITE);
                let _ = canvas.draw_rect(rect);
            }
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> PartialEq for Tile<WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    fn eq(&self, other: &Self) -> bool {
        self.pixels == other.pixels
    }
}
