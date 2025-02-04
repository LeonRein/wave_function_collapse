use crate::tileset::{Direction, TileSet};
use bittyset::BitSet;
use rand::seq::IndexedRandom;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use std::collections::VecDeque;

// trait FromVec<T>
// where
//     T: Into<usize>,
// {
//     fn from_vec(v: Vec<T>) -> BitSet<usize>;
// }

// impl<T: Into<usize>> FromVec<T> for BitSet<usize> {
//     fn from_vec(v: Vec<T>) -> BitSet<usize> {
//         let mut bitset: BitSet<usize> = BitSet::new();
//         v.into_iter().for_each(|e| {
//             bitset.insert(e.into());
//         });
//         bitset
//     }
// }

pub(crate) struct Grid<
    const TILE_WIDTH: usize,
    const TILE_HEIGHT: usize,
    const WIDTH: usize,
    const HEIGHT: usize,
> where
    [(); TILE_WIDTH * TILE_HEIGHT]:,
    [(); WIDTH * HEIGHT]:,
{
    pub(crate) tileset: TileSet<TILE_WIDTH, TILE_HEIGHT>,
    grid: [Cell; WIDTH * HEIGHT],
    uncollapsed: BitSet<usize>,
}

impl<const TILE_WIDTH: usize, const TILE_HEIGHT: usize> Grid<TILE_WIDTH, TILE_HEIGHT, 0, 0>
where
    [(); TILE_WIDTH * TILE_HEIGHT]:,
{
    pub(crate) fn new<const WIDTH: usize, const HEIGHT: usize>(
        tileset: TileSet<TILE_WIDTH, TILE_HEIGHT>,
    ) -> Grid<TILE_WIDTH, TILE_HEIGHT, WIDTH, HEIGHT>
    where
        [(); WIDTH * HEIGHT]:,
    {
        let all_options: BitSet = (0..tileset.len()).collect();
        Grid {
            tileset,
            grid: std::array::from_fn(|_| Cell::new(all_options.clone())),
            uncollapsed: (0..(WIDTH * HEIGHT)).collect(),
        }
    }
}

impl<const TILE_WIDTH: usize, const TILE_HEIGHT: usize, const WIDTH: usize, const HEIGHT: usize>
    Grid<TILE_WIDTH, TILE_HEIGHT, WIDTH, HEIGHT>
where
    [(); TILE_WIDTH * TILE_HEIGHT]:,
    [(); WIDTH * HEIGHT]:,
{
    pub(crate) fn collapse_step(&mut self) {
        let Some(min_cell_ix) = self.min_cell() else {
            return;
        };
        let min_cell = &mut self.grid[min_cell_ix];
        if min_cell.final_tile.is_some() {
            println!("DBG");
        }
        let options: Vec<usize> = min_cell.options.iter().collect();
        // let Some(&option) = options.first() else {
        //     println!("ERROR: no options for cell {}", min_cell_ix);
        //     return;
        // };
        let Some(&option) = options.choose(&mut rand::rng()) else {
            println!("ERROR: no options for cell {}", min_cell_ix);
            return;
        };
        // let &option = options.choose(&mut rand::rng()).unwrap();
        min_cell.options.clear();
        min_cell.options.insert(option);
        min_cell.final_tile = Some(option);
        self.uncollapsed.remove(min_cell_ix);
        self.propagate_options(min_cell_ix);
    }

    fn propagate_options(&mut self, index: usize) {
        let mut to_update = VecDeque::new();
        to_update.push_back(index);
        while let Some(cell_ix) = to_update.pop_front() {
            println!("to_update.len() = {}", to_update.len());
            for direction in Direction::VALUES {
                let neighbor_ix = self.get_neighbor(cell_ix, direction);
                println!("### cell_ix = {}, neighbor_ix = {}", cell_ix, neighbor_ix);
                let [cell, neighbor] = self.grid.get_many_mut([cell_ix, neighbor_ix]).unwrap();
                if neighbor.final_tile.is_some() {
                    continue;
                }
                println!("cell.options = {:?}, {}", cell.options, cell.options.len());
                println!("neighbor.options = {:?}, {}", neighbor.options, neighbor.options.len());
                let mut tile_neighbor_options_iter = cell
                    .options
                    .iter()
                    .map(|index| self.tileset.get_tile(index).get_neighbors(direction));
                let Some(mut tile_neighbor_options) = tile_neighbor_options_iter.next().cloned()
                else {
                    break;
                };
                // println!("tile_neighbor_options = {:?}", tile_neighbor_options);
                tile_neighbor_options =
                    tile_neighbor_options_iter.fold(tile_neighbor_options, |acc, e| acc.union(e));
                println!("tile_neighbor_options = {:?}, {}", tile_neighbor_options, tile_neighbor_options.len());
                if (neighbor_ix == 13) {
                    println!("DBG");
                }
                if tile_neighbor_options.is_proper_subset(&neighbor.options) {
                    println!("neighbor.options = {:?}", neighbor.options);
                    if !to_update.contains(&neighbor_ix) {
                        to_update.push_front(neighbor_ix);
                    }
                }
                neighbor.options =
                        BitSet::intersection(&neighbor.options, &tile_neighbor_options);
            }
        }
    }

    fn get_neighbor(&self, index: usize, direction: Direction) -> usize {
        let mut x = (index % WIDTH) as isize;
        let mut y = (index / WIDTH) as isize;
        match direction {
            Direction::North => y -= 1,
            Direction::East => x += 1,
            Direction::South => y += 1,
            Direction::West => x -= 1,
        };
        x = x.rem_euclid(WIDTH as isize);
        y = y.rem_euclid(HEIGHT as isize);
        x as usize + y as usize * WIDTH
    }

    fn entropy(&self, index: usize) -> usize {
        self.grid[index].entropy()
    }

    fn min_cell(&self) -> Option<usize> {
        self.uncollapsed
            .iter()
            .min_by_key(|&index| self.entropy(index))
    }

    pub(crate) fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
        scale: u32,
    ) {
        for (index, cell) in self.grid.iter().enumerate() {
            let x = (index % WIDTH) as i32 * (scale + 3) as i32 * TILE_WIDTH as i32;
            let y = (index / WIDTH) as i32 * (scale + 3) as i32 * TILE_HEIGHT as i32;
            let rect = Rect::new(x, y, TILE_WIDTH as u32 * scale, TILE_HEIGHT as u32 * scale);

            if let Some(tile_i) = cell.final_tile {
                let tile = self.tileset.get_tile(tile_i);
                // tile.draw(canvas, x, y, scale);
                canvas.set_draw_color(tile.get_color());
                canvas.fill_rect(rect);
            } else {
                canvas.set_draw_color(Color::MAGENTA);
                let _ = canvas.draw_rect(rect);
                let text = format!("{}", cell.options.len());
                Self::write_text(canvas, texture_creator, font, &text, rect);
            }
        }
    }

    pub(crate) fn write_text(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
        text: &str,
        rect: Rect,
    ) {
        let surface = font.render(text).blended(Color::WHITE).unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        canvas.copy(&texture, None, Some(rect)).unwrap();
    }
}

struct Cell {
    final_tile: Option<usize>,
    options: BitSet,
}

impl Cell {
    fn new(options: BitSet) -> Self {
        Self {
            final_tile: None,
            options,
        }
    }

    #[inline(always)]
    fn entropy(&self) -> usize {
        self.options.len()
    }
}
