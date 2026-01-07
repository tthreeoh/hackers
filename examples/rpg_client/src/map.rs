use abi_stable::std_types::RString;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Empty,
    Floor,
    Wall,
}

#[derive(Clone, Debug)]
pub struct LevelTile {
    pub position: [f32; 2], // Top-left corner
    pub size: [f32; 2],     // Width, Height
    pub tile_type: TileType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelMap {
    #[serde(skip)]
    pub tiles: Vec<LevelTile>,
    pub ascii_source: String,
    pub tile_size: f32,
    pub spawn_point: [f32; 2],
    #[serde(skip)]
    pub width: f32,
    #[serde(skip)]
    pub height: f32,
}

impl Default for LevelMap {
    fn default() -> Self {
        Self {
            tiles: Vec::new(),
            // Default level: Floor at bottom, player start at 100,500
            ascii_source: String::from(
                "                                \n\
                                                 \n\
                 @                              \n\
                 _______   ______   ___#___     \n\
                                                 \n
                       _______   ______   _______\n
                 ______                          \n
                 _   ______   _______            \n
                 _______   _____                  \n
                 _   _______                      \n
                 ___                              \n
                 _   ______   _______             \n",
            ),
            tile_size: 50.0,
            spawn_point: [100.0, 500.0],
            width: 0.0,
            height: 0.0,
        }
    }
}

impl LevelMap {
    pub fn parse(&mut self) {
        self.tiles.clear();
        // Reset spawn point to default if not found
        // self.spawn_point = [100.0, 500.0];

        let s = self.ascii_source.as_str();
        for (row, line) in s.lines().enumerate() {
            for (col, char) in line.chars().enumerate() {
                let x = col as f32 * self.tile_size;
                let y = row as f32 * self.tile_size;

                let tile_type = match char {
                    '_' => TileType::Floor,
                    '#' => TileType::Wall,
                    '@' => {
                        self.spawn_point = [x, y];
                        TileType::Empty
                    }
                    _ => TileType::Empty,
                };

                if tile_type != TileType::Empty {
                    self.tiles.push(LevelTile {
                        position: [x, y],
                        size: [self.tile_size, self.tile_size],
                        tile_type,
                    });
                }
            }
        }

        let rows = s.lines().count();
        let cols = s.lines().map(|l| l.chars().count()).max().unwrap_or(0);
        self.width = cols as f32 * self.tile_size;
        self.height = rows as f32 * self.tile_size;
    }
}
