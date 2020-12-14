use ggez;
use ggez::audio::{self, SoundSource};
use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Color, Scale, TextFragment};
use ggez::nalgebra::{self, Point2};
use ggez::{Context, GameResult};

use std::env;
use std::path;
use std::time::{Duration, Instant};
use array2d::Array2D;

use rand::Rng;

// If on shows some debug texts
const DEBUG_ON: bool = false;

// Returns variable name and value in string for debugging
/*macro_rules! debug2 {
    (x => $e:expr) => {
        format!("{}={}, ", stringify!($e), $e)
    };
}*/

const START_X: i16 = 4;
const START_Y: i16 = 2;

const TILEMAP_SIZE_X: i16 = 12;
const TILEMAP_SIZE_Y: i16 = 30;
const CELL_SIZE: i16 = 32;

// Making often used name easier
type Vector2 = nalgebra::Vector2<i16>;

#[derive(Debug, Clone, Copy)]
enum Rotation {
    Cw0,
    Cw90,
    Cw180,
    Cw270,
}

impl Rotation {
    pub fn next(&self) -> Rotation {
        match self {
            Rotation::Cw0   => Rotation::Cw90,
            Rotation::Cw90  => Rotation::Cw180,
            Rotation::Cw180 => Rotation::Cw270,
            Rotation::Cw270 => Rotation::Cw0,
        }
    }
}

// Game states
enum GameStates {
    GameOver,
    GameOn,
    Pause,
    Restart,
}

#[derive(Debug, Clone)]
pub struct Tile{
    id: u16,
    color: graphics::Color,
}

impl Tile {
    pub fn new(id: u16, color: graphics::Color) -> Self {
        Tile { id, color }
    }
}

pub struct TileSet {
    tiles: Vec<Tile>,
}

impl TileSet {
    pub fn new() -> Self {
        TileSet {
            tiles: Vec::new(),
        }
    }

    pub fn add_tile(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }
}

// Screen resolution / window size
pub struct Screen {
    pub size: Vector2,
    pub center: Vector2,
}

impl Screen {
    fn get_size() -> Vector2 {
        return Vector2::new(1800, 1000);
    }

    fn get_center() -> Vector2 {
        return Vector2::new(900, 500);
    }

}

// TileMap
struct TileMap {
    size: Vector2,
    cell_size: i16,
    tile_set: TileSet,
    // 2d array of U16 representing tiles/cells
    array : array2d::Array2D<i16>,
    spritebatch: graphics::spritebatch::SpriteBatch,
}

impl TileMap {
    pub fn new(ctx: &mut Context, size: Vector2, cell_size: i16, tile_set: TileSet) -> Self {

        let array = Array2D::filled_with(0, size.x as usize, size.y as usize);

        let image = graphics::Image::new(ctx, "/element_white_square.png").unwrap();
        let spritebatch = graphics::spritebatch::SpriteBatch::new(image);

        TileMap {
            size,
            cell_size,
            tile_set,
            array,
            spritebatch,
        }
    }

    fn get_center(&self) -> Vector2 {
        return Vector2::new((0.5 * self.size.x as f32) as i16, (0.5 * self.size.y as f32) as i16);
    }

    fn _get_cell(&self, x: i16, y: i16) -> i16 {
        if x < 0 || x >= self.size.x || y < 0 || y >= self.size.y {
            1
        }
        else {
            self.array[(x as usize, y as usize)]
        }
    }

    fn _get_cellv(&self, position: Vector2) -> i16 {
        return self._get_cell(position.x, position.y);
    }

    fn set_cell(&mut self, x: i16, y: i16, tile: i16) {
        self.array[(x as usize, y as usize)] = tile;
    }

    fn get_pixel_center(&self) -> Vector2 {
       return self.cell_size * self.get_center();
    }

    // TileMap pixel offset
    fn get_offset(&self) -> Vector2 {
        return Screen::get_center() - self.get_pixel_center();
    }

    fn clear_center(&mut self) {

        self.spritebatch.clear();

        for x in 1..self.size.x-1 {
            for y in 0..self.size.y-1 {
                self.array[(x as usize, y as usize)] = 0;
            }
        }
    }

    fn update_spritebatches(&mut self) {

        self.spritebatch.clear();

        for ix in 0..self.size.x {
            for iy in 0..self.size.y {
                let fx = ix as f32;
                let fy = iy as f32;
                let tile = self.array[(ix as usize,iy as usize)];
                let p = graphics::DrawParam::new()
                    .dest(Point2::new(fx * self.cell_size as f32, fy * self.cell_size as f32))
                    .color(self.tile_set.tiles[tile as usize].color);
                self.spritebatch.add(p);
            }
        }

    }
    
    // Add images to spritebatch and draw tile_map.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {

        // Transform and scale background
        let param = graphics::DrawParam::new()
            .dest(Point2::new(
               self.get_offset().x as f32,
               self.get_offset().y as f32,
            ));

        // Draw background
        graphics::draw(ctx, &self.spritebatch, param)?;

        Ok(())
    }

    fn remove_row(&mut self, y: i16) {

        // Remove row 'y'
        for x in 1..self.size.x-1 {
            self.set_cell(x, y, 0);
        }

        // Move rows down
        for y2 in (0..y).rev() {
            for x in 1..self.size.x-1 {
                let cell = self._get_cell(x, y2);
                if y2 < (self.size.y) {
                    self.set_cell(x, y2+1, cell);
                }
            }
        }
    }

    fn check_full_rows(&mut self) -> u32 {
        let mut added_points = 0;
        for y in 0..self.size.y-1 {
            let mut row_is_full = true;
            for x in 1..self.size.x-1 {
                if self._get_cell(x, y) < 1 {
                    row_is_full = false;
                }
            }
            if row_is_full {
                self.remove_row(y);
                added_points = 1 + self.check_full_rows();
            }
        }
        added_points
    }
}

#[derive(Debug, Clone)]
struct Block {
    position: Vector2,
    previous_position: Vector2,
    block_type: u8,
    shape: array2d::Array2D<u8>,
    rotation: Rotation,
    previous_rotation: Rotation,
    down: bool,
    moving_down: bool,
}

impl Block {
    pub fn new(position: Vector2, block_type: u8) -> Self {
        
        let shapes = vec![  
            // O-shape
            vec![   
                0,0,0,0,0,
                0,0,0,0,0,
                0,0,1,1,0,
                0,0,1,1,0,
                0,0,0,0,0
            ],
            // I-shape
            vec![   
                0,0,0,0,0,
                0,0,1,0,0,
                0,0,1,0,0,
                0,0,1,0,0,
                0,0,1,0,0
            ],
            // T-shape
            vec![   
                0,0,0,0,0,
                0,0,0,0,0,
                0,1,1,1,0,
                0,0,1,0,0,
                0,0,0,0,0
            ],
            // S-shape
            vec![   
                0,0,0,0,0,
                0,0,0,1,0,
                0,0,1,1,0,
                0,0,1,0,0,
                0,0,0,0,0
            ],
            // Z-shape
            vec![   
                0,0,0,0,0,
                0,0,1,0,0,
                0,0,1,1,0,
                0,0,0,1,0,
                0,0,0,0,0
            ],
            // L-shape
            vec![   
                0,0,0,0,0,
                0,0,1,0,0,
                0,0,1,0,0,
                0,0,1,1,0,
                0,0,0,0,0
            ],
            // J-shape
            vec![   
                0,0,0,0,0,
                0,0,1,0,0,
                0,0,1,0,0,
                0,1,1,0,0,
                0,0,0,0,0
            ]
        ];

        let shape = array2d::Array2D::from_column_major(&shapes[block_type as usize], 5, 5);

        Block { 
            position, 
            previous_position: position, 
            block_type, 
            shape, 
            rotation: Rotation::Cw0, 
            previous_rotation: Rotation::Cw0, 
            down: false, 
            moving_down: false 
        }
    }

    fn get_cell(&mut self, x: usize, y: usize, previous: bool) -> bool {
        
        let r = match previous {
            true    => &self.previous_rotation,
            false   => &self.rotation,
        };

        let c = match r {
            Rotation::Cw0   => self.shape[(x,y)],
            Rotation::Cw90  => self.shape[(y,4-x)],
            Rotation::Cw180 => self.shape[(4-x,4-y)],
            Rotation::Cw270 => self.shape[(4-y,x)],
        };

        c > 0
    }

    pub fn _mark_to_tile_map(&mut self, tile_map: &mut TileMap) {

        for x in 0..5 {
            for y in 0..5 {
                if self.get_cell(x,y,false) {
                    tile_map.set_cell(self.position.x + x as i16, self.position.y + y as i16, self.block_type as i16 + 1);
                }    
            }
        }
    }

    pub fn _delete_from_tile_map(&mut self, tile_map: &mut TileMap) {

        for x in 0..5 {
            for y in 0..5 {
                if self.get_cell(x,y,false) {
                    tile_map.set_cell(self.previous_position.x + x as i16, self.previous_position.y + y as i16, 0);
                }    
            }
        }
    }
    
    pub fn move_down(&mut self, tile_map: &mut TileMap) {
        if self.position.y < tile_map.size.y - 1 {
            self.position.y += 1;
        }
    }

    pub fn is_down(&mut self) -> bool {
        return self.down;
    }

    fn test_position(&mut self, tile_map: &mut TileMap) {
        for x in 0..5 {
            for y in 0..5 {
                if self.get_cell(x,y,false) {
                    if tile_map._get_cell(self.position.x + x as i16, self.position.y + y as i16) > 0 {
                        if self.position.y > self.previous_position.y {
                            if self.position.x == self.previous_position.x {
                                self.down = true;
                            }
                        }
                        self.position = self.previous_position;
                    }
                }
            }
        }
    }
    
    fn test_rotation(&mut self, tile_map: &mut TileMap) {
        for x in 0..5 {
            for y in 0..5 {
                if self.get_cell(x,y,false) {
                    if tile_map._get_cell(self.position.x + x as i16, self.position.y + y as i16) > 0 {
                        self.rotation = self.previous_rotation;
                        //self.position = self.previous_position;
                        //self._delete_from_tile_map(&mut tile_map);
                    }
                }
            }
        }
    }

}

/// Now we have the heart of our game, the GameState. This struct
/// will implement ggez's `EventHandler` trait and will therefore drive
/// everything ele that happens in our game.
struct GameState {
    tile_map: TileMap,
    block: Block,
    /// Whether the game is over or not
    _gameover: bool,
    last_update: Instant,
    last_move_down_time: Instant,

    text: graphics::Text,
    text_game_over: graphics::Text,
    text_try_again: graphics::Text,
    text_pause: graphics::Text,
    text_debug: graphics::Text,

    game_state: GameStates,
    music_on: bool,
    music: audio::Source,
    sound_remove_row: audio::Source,
    points: u32,
}

impl GameState {
    /// Our new function will set up the initial state of our game.
    pub fn new(_ctx: &mut Context) -> GameResult<GameState> {
        let block = Block::new(Vector2::new(START_X, START_Y), 1);

        let _font = graphics::Font::new(_ctx, "/DejaVuSerif.ttf");

        let mut sound_remove_row = audio::Source::new(_ctx, "/13_item1.wav")?;
        sound_remove_row.set_volume(2.0);

        let mut music = audio::Source::new(_ctx, "/BoxCat_Games_-_10_-_Epic_Song.mp3")?;
        music.set_volume(0.2);
        music.set_repeat(true);
        let _ = music.play();
        
        let mut tile_set = TileSet::new();
        
        // Background tiles
        tile_set.add_tile(Tile::new(0, graphics::Color::new(0.2, 0.2, 0.2, 1.0)));
        // O-shape block tiles
        tile_set.add_tile(Tile::new(1, graphics::Color::new(0.7, 0.7, 0.1, 1.0)));
        // I-shape block tiles
        tile_set.add_tile(Tile::new(2, graphics::Color::new(0.0, 0.7, 0.7, 1.0)));
        // T-shape block tiles
        tile_set.add_tile(Tile::new(3, graphics::Color::new(0.5, 0.1, 0.8, 1.0)));
        // S-shape block tiles
        tile_set.add_tile(Tile::new(4, graphics::Color::new(0.1, 0.7, 0.0, 1.0)));
        // Z-shape block tiles
        tile_set.add_tile(Tile::new(5, graphics::Color::new(0.8, 0.1, 0.1, 1.0)));
        // L-shape block tiles
        tile_set.add_tile(Tile::new(6, graphics::Color::new(0.8, 0.4, 0.1, 1.0)));
        // J-shape block tiles
        tile_set.add_tile(Tile::new(7, graphics::Color::new(0.1, 0.3, 0.9, 1.0)));
        // Wall tiles
        tile_set.add_tile(Tile::new(8, graphics::Color::new(0.6, 0.6, 0.6, 1.0)));
        
        let mut tile_map = TileMap::new(_ctx, Vector2::new(TILEMAP_SIZE_X, TILEMAP_SIZE_Y), CELL_SIZE, tile_set);

        // Bottom wall
        for i in 0..=10 {
            tile_map.set_cell(i, TILEMAP_SIZE_Y-1, 8);
        }

        // Left and right wall
        for i in 0..TILEMAP_SIZE_Y {
            tile_map.set_cell( 0, i, 8);
            tile_map.set_cell(TILEMAP_SIZE_X-1, i, 8);
        }
        
        let s = GameState {
            tile_map,
            block,
            _gameover: false,
            last_update: Instant::now(),
            last_move_down_time: Instant::now(),
            text: graphics::Text::new("Hello world!"),
            text_try_again: graphics::Text::new(TextFragment {
                // `TextFragment` stores a string, and optional parameters which will override those
                // of `Text` itself. This allows inlining differently formatted lines, words,
                // or even individual letters, into the same block of text.
                text: "Do you want to try again? Y/N".to_string(),
                color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
                // `Font` is a handle to a loaded TTF, stored inside the `Context`.
                // `Font::default()` always exists and maps to DejaVuSerif.
                font: Some(graphics::Font::default()),
                scale: Some(Scale::uniform(30.0)),
                // This doesn't do anything at this point; can be used to omit fields in declarations.
                ..Default::default()
            }),
            text_game_over: graphics::Text::new(TextFragment {
                text: "GAME OVER".to_string(),
                color: Some(Color::new(1.0, 0.0, 0.0, 1.0)),
                font: Some(graphics::Font::default()),
                scale: Some(Scale::uniform(100.0)),
                ..Default::default()
            }),
           text_pause: graphics::Text::new(TextFragment {
                text: "PAUSED".to_string(),
                color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
                font: Some(graphics::Font::default()),
                scale: Some(Scale::uniform(100.0)),
                ..Default::default()
            }),
            text_debug: graphics::Text::new(TextFragment {
                text: "DEBUG".to_string(),
                color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
                font: Some(graphics::Font::default()),
                scale: Some(Scale::uniform(14.0)),
                ..Default::default()
            }),
            game_state: GameStates::GameOn,
            music_on: true,
            music,
            sound_remove_row,
            points: 0,
        };

        Ok(s)
    }
}

/// Now we implement EventHandler for GameState. This provides an interface
/// that ggez will call automatically when different events happen.
impl event::EventHandler for GameState {
    /// Update will happen on every frame before it is drawn. This is where we update
    /// our game state to react to whatever is happening in the game world.
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // First we check to see if enough has elapsed since our last update based on
        // the update rate we defined at the top.
        if Instant::now() - self.last_update >= Duration::from_millis(10) {
            self.text = graphics::Text::new(format!(
                "FPS: {:.0} Points: {}",
                ggez::timer::fps(_ctx), self.points,
            ));

            match self.game_state {
                GameStates::GameOver | GameStates::Pause => None,
                GameStates::Restart => Some({
                    self.points = 0;
                    self.tile_map.clear_center();
                    self.game_state = GameStates::GameOn;
                }),
                GameStates::GameOn => Some({
                    if Instant::now() - self.last_move_down_time >= Duration::from_millis(300) {
                        if !self.block.down {
                            self.block.moving_down = true;
                        }
                        self.last_move_down_time = Instant::now();
                    }
                    self.block._delete_from_tile_map(&mut self.tile_map);
                    if self.block.moving_down {
                        self.block.move_down(&mut self.tile_map);
                        self.block.moving_down = false;
                    }
                    self.block.test_position(&mut self.tile_map);
                    self.block._mark_to_tile_map(&mut self.tile_map);
                    self.block.previous_position = self.block.position;

                    self.tile_map.update_spritebatches();

                    if self.block.is_down() {
                        if self.block.position.y < 5 {
                            self.game_state = GameStates::GameOver;
                        }
                        let mut rng = rand::thread_rng();
                        let block_type: u8 = rng.gen_range(1,7); // generates
                        self.block = Block::new(Vector2::new(START_X, START_Y), block_type);
                        let added_points = self.tile_map.check_full_rows();
                        if added_points > 0 {
                            let _ = self.sound_remove_row.play();
                            self.points += added_points; 
                        }
                        
                    }
                }),
            };

            // If we updated, we set our last_update to be now
            self.last_update = Instant::now();
        }

        // Finally we return `Ok` to indicate we didn't run into any errors
        Ok(())
    }

    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.2, 0.3, 0.6, 1.0].into());

        // Draw tile_map.
        self.tile_map.draw(ctx)?;
        let dest_point = mint::Vector2 { x: (0.0), y: (0.0) };
        graphics::draw(ctx, &self.text, (dest_point,))?;

        match self.game_state {
            GameStates::GameOver => Some({
                let dest_point = mint::Vector2 {
                    x: (Screen::get_center().x as u32 - (0.5 * self.text_game_over.width(ctx) as f32) as u32) as f32,
                    y: (Screen::get_center().y as u32 - self.text_game_over.height(ctx)) as f32,
                };
                graphics::draw(ctx, &self.text_game_over, (dest_point,))?;

                let dest_point = mint::Vector2 {
                    x: (Screen::get_center().x as u32 - (0.5 * self.text_try_again.width(ctx) as f32) as u32) as f32,
                    y: (Screen::get_center().y as u32 + 50) as f32,
                };
                graphics::draw(ctx, &self.text_try_again, (dest_point,))?;
            }),
            GameStates::Pause => Some({
                let dest_point = mint::Vector2 {
                    x: (Screen::get_size().x as u32 - self.text_game_over.width(ctx)) as f32,
                    y: (Screen::get_size().y as u32 - self.text_game_over.height(ctx)) as f32,
                };
                graphics::draw(ctx, &self.text_pause, (dest_point,))?;
            }),
            _ => None,
        };

        if DEBUG_ON {
            let dest_point = mint::Vector2 {
                x: 0.0,
                y: Screen::get_size().y as f32 - self.text_debug.height(ctx) as f32,
            };
            graphics::draw(ctx, &self.text_debug, (dest_point,))?;
        }

        // Finally we call graphics::present to 1.0 the gpu's framebuffer and display
        // the new frame we just drew.
        graphics::present(ctx)?;
        // We yield the current thread until the next update
        ggez::timer::yield_now();

        // And return success.
        Ok(())
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        _ctx.continuing = match keycode {
            KeyCode::Q | KeyCode::Escape => false,
            _ => true,
        };
        
        if !self.block.down {
            match keycode {
                KeyCode::Left => Some({
                    self.block._delete_from_tile_map(&mut self.tile_map);
                    self.block.position.x -= 1;
                    self.block.test_position(&mut self.tile_map);
                }),
                KeyCode::Right => Some({
                    self.block._delete_from_tile_map(&mut self.tile_map);
                    self.block.position.x += 1;
                    self.block.test_position(&mut self.tile_map);
                }),
                KeyCode::Down => Some({
                    self.block.moving_down = true;
                }),
                KeyCode::Up | KeyCode::Space => Some({
                    self.block._delete_from_tile_map(&mut self.tile_map);
                    self.block.previous_rotation = self.block.rotation;
                    self.block.rotation = self.block.rotation.next();
                    self.block.test_rotation(&mut self.tile_map);
                }),
                _ => None,
            };
        }

        match keycode {
                KeyCode::P => Some(
                    self.game_state = match self.game_state {
                        GameStates::GameOn => GameStates::Pause,
                        _ => GameStates::GameOn,
                    },
                ),
                KeyCode::M => Some({
                    self.music_on = !self.music_on;

                    match self.music_on {
                        true => Some({
                            self.music.resume();
                        }),
                        false => Some({
                            self.music.pause();
                        }),
                    };
                }),
                _ => None,
        };
        match self.game_state {
            GameStates::GameOver => Some({
                match keycode {
                    KeyCode::N => Some(_ctx.continuing = false),
                    KeyCode::Y => Some(self.game_state = GameStates::Restart),
                    _ => None,
                };
            }),
            _ => None,
        };      
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    // Here we use a ContextBuilder to setup metadata about our game. First the title and author
    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("Jetris", "jotalamp")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Jetris"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(Screen::get_size().x.into(), Screen::get_size().y.into()),
        )
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .add_resource_path(resource_dir)
        .build()?;

    // Fullscreen (set right resolution to Screen::get_size())
    //let window = graphics::window(ctx);
    //let monitor = window.get_current_monitor();
    //window.set_fullscreen(Some(monitor));

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = &mut GameState::new(ctx)?;

    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}
