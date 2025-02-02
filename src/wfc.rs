use std::array::from_fn;

use bittyset::BitSet;
use image::{GenericImageView, Rgb, RgbImage};

enum Directions {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
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
    pub fn new(image: RgbImage) -> TileSet<TILE_WIDTH, TILE_HEIGHT> {
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
        let mut tiles = Vec::new();
        for x in 0..(width - TILE_WIDTH) {
            for y in 0..(height - TILE_HEIGHT) {
                let mut pixels = Vec::new();
                for dx in 0..TILE_WIDTH {
                    for dy in 0..TILE_HEIGHT {
                        let index = (x + dx) + (y + dy) * width;
                        pixels.push(image[index]);
                    }
                }
                tiles.push(Tile::new(pixels))
            }
        }
        TileSet { tiles }
    }
}

struct Tile<const WIDTH: usize, const HEIGHT: usize>
where
    [(); WIDTH * HEIGHT]:,
{
    pixels: [u32; WIDTH * HEIGHT],
    neighbors: [BitSet<usize>; 4],
}

impl<const WIDTH: usize, const HEIGHT: usize> Tile<WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT]:,
{
    fn new(pixels: Vec<u32>) -> Tile<WIDTH, HEIGHT> {
        Self {
            pixels: pixels.try_into().unwrap(),

            neighbors: from_fn(|_| BitSet::new()),
        }
    }
}
