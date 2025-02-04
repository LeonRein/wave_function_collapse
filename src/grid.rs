use std::collections::VecDeque;

use bittyset::BitSet;

use crate::tileset::{Direction, TileSet};
use rand::seq::IndexedRandom;

trait FromVec<T>
where
    T: Into<usize>,
{
    fn from_vec(v: Vec<T>) -> BitSet<usize>;
}

impl<T: Into<usize>> FromVec<T> for BitSet<usize> {
    fn from_vec(v: Vec<T>) -> BitSet<usize> {
        let mut bitset: BitSet<usize> = BitSet::new();
        v.into_iter().for_each(|e| {
            bitset.insert(e.into());
        });
        bitset
    }
}

struct Cell {
    final_tile: Option<usize>,
    options: BitSet,
}

pub(crate) struct Grid<
    const TILE_WIDTH: usize,
    const TILE_HEIGHT: usize,
    const WIDTH: usize,
    const HEIGHT: usize,
> where
    [(); TILE_WIDTH * TILE_HEIGHT]:,
    [(); WIDTH * HEIGHT]:,
{
    tiles: TileSet<TILE_WIDTH, TILE_HEIGHT>,
    grid: [Cell; WIDTH * HEIGHT],
    uncollapsed: Vec<usize>,
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
        let n = tileset.len();
        Grid {
            tiles: tileset,
            grid: std::array::from_fn(|_| Self::new_cell(n)),
            uncollapsed: (0..(WIDTH * HEIGHT)).into_iter().collect(),
        }
    }
}

impl<const TILE_WIDTH: usize, const TILE_HEIGHT: usize, const WIDTH: usize, const HEIGHT: usize>
    Grid<TILE_WIDTH, TILE_HEIGHT, WIDTH, HEIGHT>
where
    [(); TILE_WIDTH * TILE_HEIGHT]:,
    [(); WIDTH * HEIGHT]:,
{
    fn new_cell(n: usize) -> Cell {
        let v: Vec<usize> = (0..n).into_iter().collect();
        Cell {
            final_tile: None,
            options: BitSet::from_vec(v),
        }
    }

    pub(crate) fn collapse_step(&mut self) {
        let min_cell_ix = self.min_cell();
        let min_cell = &mut self.grid[min_cell_ix];
        let options: Vec<usize> = min_cell.options.iter().collect();
        let &option = options.choose(&mut rand::rng()).unwrap();
        min_cell.options.clear();
        min_cell.options.insert(option);
        min_cell.final_tile = Some(option);
        self.uncollapsed.remove(min_cell_ix);
        self.propagate_options(min_cell_ix);
    }

    fn propagate_options(&mut self, index: usize) {
        let mut to_update = VecDeque::new();
        to_update.push_back(index);
        while let Some(index) = to_update.pop_front() {
            println!("to_update.len() = {}", to_update.len());
            for direction in Direction::VALUES {
                let neighbor_ix = self.get_neighbot(index, direction);
                let [cell, neighbor] = self.grid.get_many_mut([index, neighbor_ix]).unwrap();
                let tile_neighbor_options = 
                if cell.options.is_subset(&neighbor.options) {
                    neighbor.options = BitSet::union(&neighbor.options, &cell.options);
                    if !to_update.contains(&neighbor_ix) {
                        to_update.push_front(neighbor_ix);
                    }
                }
            }
        }
    }

    fn get_neighbot(&self, index: usize, direction: Direction) -> usize {
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
        self.grid[index].options.len()
    }

    fn min_cell(&self) -> usize {
        *self
            .uncollapsed
            .iter()
            .min_by_key(|&&index| self.entropy(index))
            .unwrap_or(&0)
    }
}
