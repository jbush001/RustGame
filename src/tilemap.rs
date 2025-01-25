use crate::gfx;

const TILE_SIZE: i32 = 64;

pub struct TileMap {
    tiles: Vec<u8>,
    width: i32,
    height: i32,
}

impl TileMap {
    pub fn new() -> TileMap {
        let width: i32 = 20;
        let height: i32 = 7;

        let mut map = TileMap {
            tiles: vec![0; (width * height) as usize],
            width,
            height,
        };

        for x in 0..width {
            map.tiles[((height - 1) * map.width + x) as usize] = 1;
        }

        map.tiles[(5 * map.width + 5) as usize] = 1;
        map
    }

    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        self.tiles[((y / TILE_SIZE) * self.width + (x / TILE_SIZE)) as usize] != 0
    }

    pub fn draw(&mut self, context: &mut gfx::RenderContext) {
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.tiles[(y * self.width + x) as usize];
                if tile != 0 {
                    context.draw_image(
                        (TILE_SIZE * x as i32, TILE_SIZE * y as i32),
                        &gfx::TILE_BRICK,
                        0.0,
                        (0, 0),
                        false,
                    );
                }
            }
        }
    }
}
