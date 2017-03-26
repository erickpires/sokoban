extern crate sdl2;
extern crate gl;

use sdl2::render::Texture;
use sdl2::render::Renderer;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use sdl2::ttf::Font;
use sdl2::pixels::Color;
use sdl2::render::TextureQuery;

use sdl2::mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                 AUDIO_S16LSB};
use sdl2::mixer::Music;
use sdl2::Sdl;

use sdl2::GameControllerSubsystem;
use sdl2::controller::GameController;
use sdl2::controller::Axis::*;

use std::path::Path;
use std::collections::HashSet;
use std::rc::Rc;

use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;

use std::ops::Add;
use std::ops::Mul;
use std::ops::AddAssign;


#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    fn zero() -> Vector2 {
        Vector2 {
            x: 0.0f32,
            y: 0.0f32
        }
    }

    // TODO(erick): Compare with epsilon
    fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    fn new(x0: f32, y0: f32) -> Vector2 {
        Vector2{
            x: x0,
            y: y0
        }
    }

    fn normalize_or_zero(&mut self) {
        let denom = (self.x * self.x + self.y * self.y).sqrt();

        if denom != 0.0f32 {
            self.x /= denom;
            self.y /= denom;
        } else {
            // if denom is zero the vector is already zero.
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
struct Rect2 {
    // NOTE(erick): (x0, y0) is always the left-bottom point
    // and (x1, y1) is always the right-top point.
    x0: f32,
    y0: f32,

    x1: f32,
    y1: f32,
}

impl Rect2 {
    fn from_point_and_dimensions(point: Vector2, width: f32, height: f32) -> Rect2 {
        Rect2 {
            x0: point.x,
            y0: point.y,

            x1: point.x + width,
            y1: point.y + height,
        }
    }

    // TODO(erick): It would be nice if we had some unit-test for this thing.
    fn collides_with(&self, other: &Rect2) -> bool {
        if self.x1 >= other.x0 && self.x1 <= other.x0 {
            if self.y1 < other.y0 {
                return false;
            }
            if self.y0 > other.y1 {
                return false;
            }

            return true;
        }
        if self.x0 <= other.x1 && self.x1 >= other.x0 {
            if self.y1 < other.y0 {
                return false;
            }
            if self.y0 > other.y1 {
                return false;
            }
            return true;
        }
        if self.y1 >= other.y0 && self.y1 <= other.y0 {
            if self.x1 < other.x0 {
                return false;
            }
            if self.x0 > other.x1 {
                return false;
            }

            return true;
        }
        if self.y0 <= other.y1 && self.y1 >= other.y0 {
            if self.x1 < other.x0 {
                return false;
            }
            if self.x0 > other.x1 {
                return false;
            }

            return true;
        }

        return false;
    }
}

impl<'a> Add<Vector2> for &'a Rect2 {
    type Output = Rect2;

    fn add(self, translation: Vector2) -> Rect2 {
        Rect2 {
            x0: self.x0 + translation.x,
            y0: self.y0 + translation.y,

            x1: self.x1 + translation.x,
            y1: self.y1 + translation.y,
        }
    }
}

fn allowed_motion_before_collision(moving: &Rect2, direction: Vector2, obstacle: &Rect2) -> f32 {
    // TODO(erick): We should binary search and find the correct movement amount.
    if moving.collides_with(obstacle) {
        0.0
    } else {
        1.0
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: f32) -> Vector2 {
        Vector2 {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

impl<'a> AddAssign<&'a Vector2> for Vector2 {
    fn add_assign(&mut self, rhs: &Vector2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

struct AudioMixer {
    frequency: i32,
    format: u16,
    channels: i32,
    chunk_size: i32,

    audio: Option<sdl2::AudioSubsystem>,
    mixer_context: Option<sdl2::mixer::Sdl2MixerContext>,
}

impl AudioMixer {
    fn new(sdl: &Sdl) -> AudioMixer {
        let mut mixer = AudioMixer {
            frequency: 44100,
            format: AUDIO_S16LSB,
            channels: 2,
            chunk_size: 1024,
            audio: None,
            mixer_context: None
        };

        let _audio = sdl.audio().unwrap();
        mixer.audio = Some(_audio);
        let _mixer_context = sdl2::mixer::init(INIT_MP3 | INIT_FLAC | INIT_MOD | INIT_FLUIDSYNTH |
                                               INIT_MODPLUG |
                                               INIT_OGG)
                                               .unwrap();

        mixer.mixer_context = Some(_mixer_context);

        sdl2::mixer::open_audio(mixer.frequency, mixer.format, mixer.channels, mixer.chunk_size).unwrap();
        sdl2::mixer::allocate_channels(0);

        mixer
    }
}

fn play_music<'a> (filename: &Path) -> Music<'a> {
    let music: Music = sdl2::mixer::Music::from_file(filename).unwrap();
    if !music.play(1).is_ok() {
        println!("Could not play file: {:?}", filename);
    }
    if !music.fade_in_from_pos(1, 10000, 10.0).is_ok() {
        println!("Could not play file: {:?}", filename);
    }

    music
}

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

fn init_controller(game_controller_subsystem : &GameControllerSubsystem) -> Option<GameController> {
    let available =
        match game_controller_subsystem.num_joysticks() {
            Ok(n)  => n,
            Err(_) => 0,
        };

    println!("{} joysticks available", available);

    let mut controller = None;

    // Iterate over all available joysticks and look for game
    // controllers.
    for id in 0..available {
        if game_controller_subsystem.is_game_controller(id) {
            println!("Attempting to open controller {}", id);

            match game_controller_subsystem.open(id) {
                Ok(c) => {
                    // We managed to find and open a game controller,
                    // exit the loop
                    println!("Success: opened \"{}\"", c.name());
                    controller = Some(c);
                    break;
                },
                Err(e) => println!("failed: {:?}", e),
            }

        } else {
             println!("{} is not a game controller", id);
        }
    }

    controller
}

const GAME_NAME : &'static str = "Sokoban";
const WINDOW_WIDTH  : u32 = 800;
const WINDOW_HEIGHT : u32 = 592;

#[derive(Debug)]
struct GameState {
    is_running: bool,
    old_ticks: u32,
}

impl GameState {
    fn new() -> GameState {
        GameState {
            is_running: true,
            old_ticks: 0,
        }
    }
}

#[derive(Debug)]
struct GameInputState {
    left_x_axis: f32,
    left_y_axis: f32,

    right_x_axis: f32,
    right_y_axis: f32,

    action_a: bool,
    action_b: bool,
}

impl GameInputState {
    fn new() -> GameInputState {
        GameInputState {
            left_x_axis: 0.0f32,
            left_y_axis: 0.0f32,

            right_x_axis: 0.0f32,
            right_y_axis: 0.0f32,

            action_a: false,
            action_b: false,
        }
    }

    fn no_left_axis_input(&self) -> bool {
        self.left_x_axis == 0.0f32 && self.left_y_axis == 0.0f32
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct Sprite {
    height : u32,
    width  : u32,

    texture_x : i32,
    texture_y : i32,
    texture_width  : u32,
    texture_height : u32,

    texture :  Rc<Texture>,
    is_animating: bool,

    // TODO(erick): This should not be here. We need an AnimationPlayer
    acc_dt      : f32,
    fps         : u32,
    frame_time  : f32,
}

impl Sprite {
    fn new(_texture: Rc<Texture>, texture_w: u32, texture_h: u32, w: u32, h: u32) -> Sprite {
        let _fps = 16;


        Sprite {
            height : h,
            width  : w,

            texture_x : 0,
            texture_y : 0,
            texture_width  : texture_w,
            texture_height : texture_h,

            texture : _texture,
            is_animating : false,

            acc_dt      : 0.0f32,
            fps         : _fps,
            frame_time  : 1.0f32 / (_fps) as f32,
        }
    }

    fn accumulate_time(&mut self, dt: f32) {
        self.acc_dt += dt;

        while self.acc_dt >= self.frame_time {
            self.acc_dt -= self.frame_time;
            self.texture_x += self.width as i32;

            // TODO(erick): Hard-coded
            if self.texture_x >= (self.texture_width) as i32 {
                self.texture_x = 0;
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct Entity {
    position    : Vector2,
    width       : f32,
    height      : f32,

    sprite : Sprite,
}

impl Entity {
    fn new(s: Sprite, p0: Vector2, w: f32, h: f32) -> Entity {
        Entity{
            position    : p0,
            width       : w,
            height      : h,

            sprite : s,
        }
    }

    fn center_in_current_tile_rect(&mut self) {
        let x_diff = self.width.ceil() - self.width;
        let y_diff = self.height.ceil() - self.height;

        self.position.x = self.position.x.floor() + x_diff * 0.5;
        self.position.y = self.position.y.floor() + y_diff * 0.5;
    }

    fn containing_rect(&self) -> Rect2 {
        Rect2 {
            x0: self.position.x,
            y0: self.position.y,

            x1: self.position.x + self.width,
            y1: self.position.y + self.height,
        }
    }

    fn collision_against_entities(&self, entities: &Vec<Entity>, movement: Vector2) -> isize {
        let self_rect = self.containing_rect();
        let target_rect = &self_rect + movement;

        for index in 0..entities.len() {
            let ref entity = entities[index];
            let entity_rect = entity.containing_rect();

            if target_rect.collides_with(&entity_rect) {
                return index as isize;
            }
        }

        -1
    }

    fn collision_against_tiles(&self, map: &Map, mut direction: Vector2) -> Vector2 {
        let entity_rect = self.containing_rect();

        let target_rect = &entity_rect + direction;

        // TODO(erick): We should probably write an iterator for this operation
        // TODO(erick): If we know where we are and where we are heading to we don't
        // need to look at all the tiles
        'outter: for tile_y in 0..map.n_lines() {
            for tile_x in 0..map.n_cols() {
                let tile_type = map.tile_at(tile_x, tile_y);
                if let TileType::Wall = tile_type {
                    let tile_rect = Rect2 {
                        x0: tile_x as f32,
                        y0: tile_y as f32,

                        x1: 1.0 + tile_x as f32,
                        y1: 1.0 + tile_y as f32,
                    };

                    if target_rect.collides_with(&tile_rect) {
                        direction = Vector2::zero();
                        break 'outter;
                    }
                }
            }
        }

        direction
    }

    fn draw(&self, renderer: &mut Renderer) {
         // TODO(erick): Should the camera move?
        const CAMERA_Y0     : u32 = 0;
        const CAMERA_X0     : u32 = 0;

        const CAMERA_HEIGHT : u32 = 16;
        const CAMERA_WIDTH  : u32 = 20;

        let x_camera_coord = self.position.x - CAMERA_X0 as f32;
        let y_camera_coord = self.position.y - CAMERA_Y0 as f32;

        let x_screen_coord = (x_camera_coord * (WINDOW_WIDTH / CAMERA_WIDTH) as f32) as i32;
        let y_screen_coord = WINDOW_HEIGHT as i32 - ( (y_camera_coord + self.height) * (WINDOW_HEIGHT / CAMERA_HEIGHT) as f32) as i32;

        let w_screen_coord = (self.width * (WINDOW_WIDTH / CAMERA_WIDTH) as f32) as u32;
        let h_screen_coord = (self.height * (WINDOW_HEIGHT / CAMERA_HEIGHT) as f32) as u32;


        let source_rect = Rect::new(self.sprite.texture_x, self.sprite.texture_y, self.sprite.width, self.sprite.height);
        let dest_rect = Rect::new(x_screen_coord, y_screen_coord, w_screen_coord, h_screen_coord);
        renderer.copy_ex(&self.sprite.texture, Some(source_rect), Some(dest_rect), 0.0, None, true, false).unwrap();
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
enum TileType {
    Floor,
    Wall,
    Target,
    Blank,
}

impl TileType {
    fn from_code(code: u32) -> Option<TileType> {
        match code {
            0 | 2 | 3   => Some(TileType::Floor),
            1           => Some(TileType::Wall),
            4           => Some(TileType::Target),
            5           => Some(TileType::Blank),
            _           => None,
        }
    }
}

struct MapData {
    floor_texture   : Rc<Texture>,
    wall_texture    : Rc<Texture>,
    target_texture  : Rc<Texture>,
    box_texture     : Rc<Texture>,

    tile_texture_width  : u32,
    tile_texture_height : u32,

    box_texture_width   : u32,
    box_texture_height  : u32,
}

impl MapData {
    fn load(renderer: &Renderer) -> MapData {
        let (floor, w, h)   = texture_from_path(Path::new("assets/floor.bmp"), renderer);
        let (wall, ..)      = texture_from_path(Path::new("assets/wall.bmp"), renderer);
        let (target, ..)    = texture_from_path(Path::new("assets/target.bmp"), renderer);

        let(_box, b_w, b_h) = texture_from_path(Path::new("assets/box.bmp"), renderer);

        MapData {
            floor_texture   : Rc::new(floor),
            wall_texture    : Rc::new(wall),
            target_texture  : Rc::new(target),
            box_texture     : Rc::new(_box),

            tile_texture_width  : w,
            tile_texture_height : h,

            box_texture_width   : b_w,
            box_texture_height  : b_h,
        }
    }
}

struct Map {
    tiles: Vec<TileType>,
    tiles_stride: i32,

    map_data: MapData,
    boxes: Vec<Entity>,
}

impl Map {
    fn is_box(code: u32) -> bool {
        code == 2
    }

    fn is_player(code: u32) -> bool {
        code == 3
    }

    fn add_box(map: &mut Map, _x: u32, _y: u32) {
        // TODO(erick): We probably don't need the unsafe here, but this is language is driving me mad.
        let _sprite = Sprite::new(map.map_data.box_texture.clone(),
                        map.map_data.box_texture_width,
                        map.map_data.box_texture_height,
                        map.map_data.box_texture_width,
                        map.map_data.box_texture_height);

        let e_box = Entity {
            position : Vector2 {
                x: _x as f32,
                y: _y as f32,
            },
            height  : 1.0,
            width   : 1.0,

            sprite: _sprite,
        };
        map.boxes.push(e_box);
    }

    fn from_left_to_right_handed(position : (u32, u32), n_lines: u32) -> (u32, u32) {
        (position.0, n_lines - position.1 - 1)
    }

    fn from_path(path: &Path, renderer: &Renderer) -> (Result<Map, io::Error>, (i32, i32)) {
        let _map_data = MapData::load(&renderer);

        let mut result = Map {
            tiles: Vec::new(),
            tiles_stride: -1,

            map_data: _map_data,
            boxes: Vec::new(),
        };

        let mut boxes_position = Vec::new();
        let mut player_position = (-1, -1);

        let input_file = match File::open(path) {
            Ok(file)    => file,
            Err(e)      => {return (Err(e), player_position)}
        };

        let file_data = BufReader::new(&input_file);

        let mut n_lines: u32 = 0;
        for line in file_data.lines() {
            n_lines += 1;

            let line = line.unwrap();
            let tiles_code = line.split_whitespace();

            let mut n_tiles: u32 = 0;
            for code in tiles_code {
                n_tiles += 1;
                let code = code.parse::<u32>().unwrap();
                let tile_type = TileType::from_code(code).unwrap();
                result.tiles.push(tile_type);

                if Map::is_box(code) {
                    // README(erick): These numbers are off-by-one
                    boxes_position.push((n_tiles - 1, n_lines - 1));
                } else if Map::is_player(code) {
                    // README(erick): These numbers are off-by-one
                    player_position = ((n_tiles - 1) as i32, (n_lines - 1) as i32);
                }
            }

            if result.tiles_stride < 0 {
                result.tiles_stride = n_tiles as i32;
            } else {
                if result.tiles_stride != n_tiles as i32 {
                    // TODO(erick): Error
                    println!("Invalid line ({}) at file {:?}", n_lines, path);
                }
            }
        }

        // Now we add the boxes, converting the coordinate system
        for box_position in boxes_position {
            let (pos_x, pos_y) = Map::from_left_to_right_handed(box_position, n_lines);
            Map::add_box(&mut result, pos_x, pos_y);
        }

        if !(player_position.0 < 0 || player_position.1 < 0) {
            let p_x = player_position.0 as u32;
            let p_y = player_position.1 as u32;

            let p = Map::from_left_to_right_handed((p_x, p_y), n_lines);
            player_position = (p.0 as i32, p.1 as i32)
        }

        (Ok(result), (player_position))
    }

    fn tile_at(&self, x: u32, y: u32) -> TileType {
        // NOTE(erick): Tiles are storage in a left-handed coordinate system.
        // We invert it here
        let y = self.n_lines() - y - 1;
        let pos : usize = (y * self.n_cols() + x) as usize;

        self.tiles[pos]
    }

    fn n_cols(&self) -> u32 {
        if self.tiles_stride < 0 {
            0
        }
        else {
            self.tiles_stride as u32
        }
    }

    fn n_lines(&self) -> u32 {
        self.tiles.len() as u32 / self.tiles_stride as u32
    }

    fn draw_tile(tile: TileType, x: u32, y: u32, width: u32, height: u32, map_data: &MapData, renderer: &mut Renderer) {
        let tile_x_screen_coord = (x * width) as i32;
        let tile_y_screen_coord = (WINDOW_HEIGHT - y * height - height) as i32;

        let tile_texture = match tile {
            TileType::Floor   => Some(&map_data.floor_texture),
            TileType::Wall    => Some(&map_data.wall_texture),
            TileType::Target  => Some(&map_data.target_texture),
            _                 => None
        };

        if !tile_texture.is_none() {
            let tile_texture = tile_texture.unwrap();

            let source_rect = Rect::new(0, 0, map_data.tile_texture_width, map_data.tile_texture_height);
            let dest_rect = Rect::new(tile_x_screen_coord, tile_y_screen_coord, width, height);
            renderer.copy_ex(tile_texture, Some(source_rect), Some(dest_rect), 0.0, None, true, false).unwrap();
        }
    }

    fn draw(&self, renderer: &mut Renderer) {
        // TODO(erick): Should the camera move?
        const CAMERA_Y0     : u32 = 0;
        const CAMERA_X0     : u32 = 0;

        const CAMERA_HEIGHT : u32 = 16;
        const CAMERA_WIDTH  : u32 = 20;

        let tile_width_in_camera    = WINDOW_WIDTH / CAMERA_WIDTH;
        let tile_height_in_camera   = WINDOW_HEIGHT / CAMERA_HEIGHT;

        for tile_y in CAMERA_Y0..self.n_lines() {
            // NOTE(erick): We are outside the camera.
            if tile_y >= CAMERA_HEIGHT { break; }

            for tile_x in CAMERA_X0..self.n_cols() {
                // NOTE(erick): We are outside the camera.
                if tile_x >= CAMERA_WIDTH { break; }

                let tile = self.tile_at(tile_x, tile_y);
                Map::draw_tile(tile, tile_x, tile_y, tile_width_in_camera, tile_height_in_camera, &self.map_data, renderer);
            }
        }

        for _box in &self.boxes {
            _box.draw(renderer);
        }
    }
}

fn texture_from_path(path: &Path, renderer: &Renderer) -> (Texture, u32, u32) {
    let temp_surface = sdl2::surface::Surface::load_bmp(path).unwrap();

    let texture = renderer.create_texture_from_surface(&temp_surface).unwrap();

    (texture, temp_surface.width(), temp_surface.height())
}

fn main() {
    let mut game_state : GameState = GameState::new();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem.window(GAME_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
        .resizable()
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .present_vsync()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .expect("Failed to create renderer with given parameters");

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    renderer.window().unwrap().gl_set_context_to_current().unwrap();
    renderer.set_draw_color(sdl2::pixels::Color::RGBA(255,255,0,255));

    let game_controller_subsystem = sdl_context.game_controller().unwrap();
    let mut timer = sdl_context.timer().unwrap();
    let mut events = sdl_context.event_pump().unwrap();
    let mixer = AudioMixer::new(&sdl_context);


    // NOTE(erick): controller has to be here because
    // we stop receiving messages once it's dropped.
    let controller = init_controller(&game_controller_subsystem);

    let mut playing_musics  = Vec::new();
    playing_musics.push(play_music(Path::new("assets/guitar.mp3")));

    // Load a font
    let font = ttf_context.load_font(Path::new("assets/font.ttf"), 22).unwrap();
    // font.set_style(sdl2::ttf::STYLE_BOLD);

    //
    // Input
    //
    let mut keyboard_input = GameInputState::new();
    let mut joystick_input = GameInputState::new();


    let (map, player_position) = Map::from_path(Path::new("assets/maps/0-tutorial.map"), &renderer);
    let mut map = map.unwrap();

    //
    // Player
    //
    // TODO(erick): Since we now use a Rc to store the sprite texture we
    // don't need to hold the texture here anymore. This two lines can be handled by a single function
    let (player_texture, texture_w, texture_h) = texture_from_path(Path::new("assets/player.bmp"), &renderer);
    let player_sprite = Sprite::new(Rc::new(player_texture), texture_w, texture_h, texture_w, texture_h);

    let player_x = player_position.0 as f32;
    let player_y = player_position.1 as f32;
    let player_width_to_height_ratio = texture_w as f32 / texture_h as f32;
    let player_height = 0.8;
    let player_width  = player_height * player_width_to_height_ratio;

    let mut player = Entity::new(player_sprite, Vector2::new(player_x, player_y), player_width, player_height);
    player.center_in_current_tile_rect();

    let (running_cat_texture, texture_w, texture_h) = texture_from_path(Path::new("assets/animate.bmp"), &renderer);
    let running_cat_sprite = Sprite::new(Rc::new(running_cat_texture), texture_w, texture_h, 128, 82);
    let mut running_cat = Entity::new(running_cat_sprite, Vector2::new(16.0, 16.0), 4.0, 4.0 * 82.0 / 128.0);

    game_state.is_running = true;
    while game_state.is_running {

        for event in events.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    game_state.is_running = false;
                },
                Event::ControllerAxisMotion{ axis, value: val, .. } => {
                    fn handle_axis_input(axis: &mut f32, val: i16) {
                        // Axis motion is an absolute value in the range
                        // [-32768, 32767]. Let's simulate a very rough dead
                        // zone to ignore spurious events.
                        const JOYSTICK_MIN_VALUE : f32 = -32768.0;
                        const JOYSTICK_MAX_VALUE : f32 =  32767.0;
                        const DEAD_ZONE: i16 = 10000;
                        if val <= DEAD_ZONE && val >= -DEAD_ZONE {
                            *axis = 0.0;
                            return;
                        }

                        if val < 0 {
                            *axis = -((val as f32) / JOYSTICK_MIN_VALUE);
                        }
                        else {
                            *axis = (val as f32) / JOYSTICK_MAX_VALUE;
                        }
                    }

                    if axis == LeftX {
                        handle_axis_input(&mut joystick_input.left_x_axis, val);
                    }
                    if axis == LeftY {
                        // NOTE(erick): The controller coordinates are left-handed
                        handle_axis_input(&mut joystick_input.left_y_axis, val);
                        joystick_input.left_y_axis *= -1.0;
                    }if axis == RightX {
                        handle_axis_input(&mut joystick_input.right_y_axis, val);
                    }if axis == RightY {
                        // NOTE(erick): The controller coordinates are left-handed
                        handle_axis_input(&mut joystick_input.right_y_axis, val);
                        joystick_input.right_y_axis *= -1.0;
                    }
                },
                _ => {}
            }
        }

        //
        // Keyboard input
        //
        keyboard_input.left_x_axis = 0.0;
        keyboard_input.left_y_axis = 0.0;

        let keys = pressed_keycode_set(&events);
        if keys.contains(&Keycode::W) {
            keyboard_input.left_y_axis += 1.0;
        }
        if keys.contains(&Keycode::S) {
            keyboard_input.left_y_axis -= 1.0;
        }
        if keys.contains(&Keycode::A) {
            keyboard_input.left_x_axis -= 1.0;
        }
        if keys.contains(&Keycode::D) {
            keyboard_input.left_x_axis += 1.0;
        }

        let new_ticks = timer.ticks();
        let dt = ((new_ticks - game_state.old_ticks) as f32) / 1000.0;
        game_state.old_ticks = new_ticks;


        let mut move_direction = Vector2::zero();
        // NOTE(erick): We only read the keyboard when there is no input on the joystick
        // TODO(erick): We should check if the joystick is still connected.
        if joystick_input.no_left_axis_input() {
            move_direction.x = keyboard_input.left_x_axis;
            move_direction.y = keyboard_input.left_y_axis;
        } else {
            move_direction.x = joystick_input.left_x_axis;
            move_direction.y = joystick_input.left_y_axis;
        }

        let fps_text = format!("Frame time: {:.3}", dt);

        running_cat.sprite.accumulate_time(dt);

        {
            move_direction.normalize_or_zero();
            let movement = move_direction * (dt * 7.0f32);
            let mut allowed_movement = player.collision_against_tiles(&map, movement);
            if !allowed_movement.is_zero() {
                let box_index = player.collision_against_entities(&map.boxes, allowed_movement);
                if box_index != -1 {
                    let _box = map.boxes[box_index as usize].clone();

                    allowed_movement = _box.collision_against_tiles(&map, allowed_movement);
                    map.boxes[box_index as usize].position += &allowed_movement;
                }

                player.position += &allowed_movement;
            }
        }

        renderer.clear();
        map.draw(&mut renderer);
        running_cat.draw(&mut renderer);
        player.draw(&mut renderer);

        draw_text(&mut renderer, &font, Color::RGBA(255, 0, 0, 255), &fps_text, Vector2::new(0.02, 0.02));


        renderer.present();

        // use std::time::Duration;
        // std::thread::sleep(Duration::from_millis(10));
    }
}

fn draw_text(renderer: &mut Renderer, font: &Font, color: Color, string: &String, position: Vector2) {
    let text_surface = font.render(string)
        .blended(color).unwrap();
    let mut text_texture = renderer.create_texture_from_surface(&text_surface).unwrap();

    let text_x = (WINDOW_WIDTH as f32 * position.x) as i32;
    let text_y = (WINDOW_HEIGHT as f32 * position.y) as i32;


    let TextureQuery { width: text_width, height: text_height, .. } = text_texture.query();
    let text_rect = Rect::new(text_x, text_y, text_width, text_height);

    renderer.copy(&mut text_texture, None, Some(text_rect)).unwrap();
}

fn pressed_keycode_set(e: &sdl2::EventPump) -> HashSet<Keycode> {
    e.keyboard_state().pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect()
 }
