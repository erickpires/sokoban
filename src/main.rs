extern crate sdl2;
extern crate gl;
extern crate regex;

use sdl2::render::Texture;
use sdl2::render::Renderer;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use sdl2::ttf::Font;
use sdl2::pixels::Color;
use sdl2::render::TextureQuery;

use sdl2::controller::Axis::*;

use std::path::Path;
use std::rc::Rc;

use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;
use std::io::Write;

use regex::Regex;

use std::collections::hash_map::HashMap;

extern crate sokoban;
use sokoban::math::*;
use sokoban::sdl_misc::*;

#[allow(dead_code)]
fn allowed_motion_before_collision(moving: &Rect2, direction: Vector2, obstacle: &Rect2) -> f32 {
    // TODO(erick): We should binary search and find the correct movement amount.
    if !moving.collides_with(obstacle).is_none() {
        0.0
    } else {
        1.0
    }
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

#[derive(Debug)]
#[derive(Clone)]
struct AnimationLane {
    number_of_frames        : u32,
}

#[derive(Debug)]
#[derive(Clone)]
struct AnimationInfo {
    is_animating    : bool,

    acc_dt          : f32,
    fps             : u32,
    frame_time      : f32,

    current_frame   : u32,
    animation_lanes : Vec<AnimationLane>,
    // TODO(erick): We should either find out a way to disable bound checks or
    // see if we can use a reference here.
    current_animation_lane_index : isize,
}

impl AnimationInfo {
    fn new(animating: bool, _fps: u32) -> AnimationInfo {
        let _frame_time = if _fps > 0 {1.0 / _fps as f32} else {0.0};

        AnimationInfo {
            is_animating    : animating,

            acc_dt          : 0.0,
            fps             : _fps,
            frame_time      : _frame_time,

            current_frame   : 0,
            animation_lanes : Vec::new(),
            current_animation_lane_index : -1,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct SpriteSheet {
    sprite_height : u32,
    sprite_width  : u32,

    sprite_x_offset : i32,
    sprite_y_offset : i32,
    sprite_sheet_width  : u32,
    sprite_sheet_height : u32,

    texture :  Rc<Texture>,

    animation_info : AnimationInfo,
}

impl SpriteSheet {
    fn new(_texture: Rc<Texture>, sprite_sheet_w: u32, sprite_sheet_h: u32,
            w: u32, h: u32, _animation_info: AnimationInfo) -> SpriteSheet {
        // TODO(erick): We are not checking whether the sprite sheet dimensions are
        // multiples of the sprite dimensions.
        let _fps = 16;

        SpriteSheet {
            sprite_height : h,
            sprite_width  : w,

            sprite_x_offset : 0,
            sprite_y_offset : 0,
            sprite_sheet_width   : sprite_sheet_w,
            sprite_sheet_height  : sprite_sheet_h,

            texture : _texture,

            animation_info : _animation_info,
        }
    }

    fn animation_accumulate_dt(&mut self, dt: f32) {
        //
        // Early-outs
        //
        if !self.animation_info.is_animating {
            return;
        }

        if self.animation_info.current_animation_lane_index == -1 {
            return;
        }

        self.animation_info.acc_dt += dt;

        if self.animation_info.acc_dt < self.animation_info.frame_time {
            return
        }

        let ref current_lane = self.animation_info.animation_lanes[self.animation_info.current_animation_lane_index as usize];
        while self.animation_info.acc_dt >= self.animation_info.frame_time {
            self.animation_info.acc_dt -= self.animation_info.frame_time;

            self.animation_info.current_frame += 1;
            if self.animation_info.current_frame >= current_lane.number_of_frames {
                self.animation_info.current_frame = 0;
            }
        }
        self.sprite_x_offset = (self.sprite_width * self.animation_info.current_frame) as i32;
    }

    fn set_animation_lane_index(&mut self, index: usize) {
        if index < self.animation_info.animation_lanes.len() {
            self.animation_info.current_animation_lane_index = index as isize;
            self.sprite_y_offset = (self.sprite_height * index as u32) as i32;
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct Entity {
    position     : Vector2,
    velocity     : Vector2,
    acceleration : Vector2,

    draw_height      : f32,
    draw_width       : f32,

    collision_width  : f32,
    collision_height : f32,

    sprite_sheet : SpriteSheet,
}

impl Entity {
    fn new(s: SpriteSheet, p0: Vector2, collision_w: f32, collision_h: f32, draw_w: f32, draw_h: f32) -> Entity {
        Entity {
            position     : p0,
            velocity     : Vector2::zero(),
            acceleration : Vector2::zero(),

            draw_height         : draw_h,
            draw_width          : draw_w,

            collision_width     : collision_w,
            collision_height    : collision_h,

            sprite_sheet : s,
        }
    }

    fn center_on_current_tile_rect(&mut self) {
        let x_diff = self.draw_width.ceil() - self.draw_width;
        let y_diff = self.draw_height.ceil() - self.draw_height;

        self.position.x = self.position.x.floor() + x_diff * 0.5;
        self.position.y = self.position.y.floor() + y_diff * 0.5;
    }

    fn containing_rect(&self) -> Rect2 {
        Rect2::from_point_and_dimensions(self.position, self.collision_width, self.collision_height)
    }

    fn collision_against_entities(&self, entities: &Vec<Entity>, movement: Vector2) -> isize {
        let self_rect = self.containing_rect();
        let target_rect = &self_rect + movement;

        for index in 0..entities.len() {
            let ref entity = entities[index];
            let entity_rect = entity.containing_rect();

            if !target_rect.collides_with(&entity_rect).is_none() {
                return index as isize;
            }
        }

        -1
    }

    fn collision_against_tiles(&self, map: &Map, mut movement: Vector2) -> Vector2 {
        let entity_rect = self.containing_rect();

        let target_rect = &entity_rect + movement;

        // NOTE(erick): We only need to search inside the bounding rectangle
        let bounding_rect = Rect2::bounding_rect(&entity_rect, &target_rect);
        let min_point = bounding_rect.lower_left();
        let max_point = bounding_rect.upper_right();

        let min_tile_x = min_point.x.floor() as u32; // Inclusive
        let min_tile_y = min_point.y.floor() as u32; // Inclusive

        let max_tile_x = max_point.x.ceil() as u32; // Exclusive
        let max_tile_y = max_point.y.ceil() as u32; // Exclusive

        'outter: for tile_y in min_tile_y..max_tile_y {
            for tile_x in min_tile_x..max_tile_x {
                let tile_type = map.tile_at(tile_x, tile_y);
                if let TileType::Wall = tile_type {
                    let tile_rect = Rect2 {
                        x0: tile_x as f32,
                        y0: tile_y as f32,

                        x1: 1.0 + tile_x as f32,
                        y1: 1.0 + tile_y as f32,
                    };

                    if !target_rect.collides_with(&tile_rect).is_none() {
                        movement = Vector2::zero();
                        break 'outter;
                    }
                }
            }
        }

        movement
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
        let y_screen_coord = WINDOW_HEIGHT as i32 - ( (y_camera_coord + self.draw_height) * (WINDOW_HEIGHT / CAMERA_HEIGHT) as f32) as i32;

        let w_screen_coord = (self.draw_width * (WINDOW_WIDTH / CAMERA_WIDTH) as f32) as u32;
        let h_screen_coord = (self.draw_height * (WINDOW_HEIGHT / CAMERA_HEIGHT) as f32) as u32;

        let source_rect = Rect::new(self.sprite_sheet.sprite_x_offset, self.sprite_sheet.sprite_y_offset,
                                    self.sprite_sheet.sprite_width, self.sprite_sheet.sprite_height);
        let dest_rect = Rect::new(x_screen_coord, y_screen_coord, w_screen_coord, h_screen_coord);
        renderer.copy_ex(&self.sprite_sheet.texture, Some(source_rect), Some(dest_rect), 0.0, None, false, false).unwrap();
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
    fn load(renderer: &Renderer, floor_path: &Path, wall_path: &Path,
            target_path: &Path, box_path: &Path) -> MapData {
        let (floor, w, h)   = texture_from_path(floor_path, renderer);
        let (wall, ..)      = texture_from_path(wall_path, renderer);
        let (target, ..)    = texture_from_path(target_path, renderer);

        let(_box, b_w, b_h) = texture_from_path(box_path, renderer);

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

    #[allow(dead_code)]
    fn load_default(renderer: &Renderer) -> MapData {
        let default_floor_path  = Path::new("assets/floor.bmp");
        let default_wall_path   = Path::new("assets/wall.bmp");
        let default_target_path = Path::new("assets/target.bmp");
        let default_box_path    = Path::new("assets/box.bmp");

        MapData::load(renderer, default_floor_path, default_wall_path,
                        default_target_path, default_box_path)
    }
}

// TODO(erick): Tiles and box should be in separated structs, otherwise we can't borrow a mutable box
// and an immutable map. CHECK FIRST!!!!
struct Map {
    name        : String,
    level_music : Option<String>,
    next_level  : Option<String>,

    tiles: Vec<TileType>,
    tiles_stride: i32,

    map_data: MapData,
    boxes: Vec<Entity>,
}

impl Map {
    fn add_box(map: &mut Map, sprite_width: u32, sprite_height: u32, _x: u32, _y: u32) {
        let boxes_anim_info = AnimationInfo::new(false, 0);

        let _sprite = SpriteSheet::new(map.map_data.box_texture.clone(),
                        map.map_data.box_texture_width,
                        map.map_data.box_texture_height,
                        sprite_width,
                        sprite_height,
                        boxes_anim_info);

        let e_box = Entity {
            position : Vector2 {
                x: _x as f32,
                y: _y as f32,
            },
            velocity     : Vector2::zero(),
            acceleration : Vector2::zero(),

            draw_width          : 1.0,
            draw_height         : 1.0,
            collision_width     : 1.0,
            collision_height    : 1.0,

            sprite_sheet    : _sprite,
        };
        map.boxes.push(e_box);
    }

    #[allow(dead_code)]
    fn from_left_to_right_handed(position : (u32, u32), n_lines: u32) -> (u32, u32) {
        (position.0, n_lines - position.1 - 1)
    }

    fn fill_tiles_and_stride(map: &mut Map, map_file: &Path) {
        let input_file = File::open(map_file).expect(format!("Could not open file: {:?}", map_file).as_str());

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
                map.tiles.push(tile_type);
            }

            if map.tiles_stride < 0 {
                map.tiles_stride = n_tiles as i32;
            } else {
                if map.tiles_stride != n_tiles as i32 {
                    // TODO(erick): Error
                    println!("Invalid line ({}) at file {:?}", n_lines, map_file);
                }
            }
        }
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
            renderer.copy_ex(tile_texture, Some(source_rect), Some(dest_rect), 0.0, None, false, false).unwrap();
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
    #[allow(unused_variables)]
    let mixer = AudioMixer::new(&sdl_context);


    // NOTE(erick): controller has to be here because
    // we stop receiving messages once it's dropped.
    #[allow(unused_variables)]
    let controller = init_controller(&game_controller_subsystem);


    // Load a font
    let fps_font = ttf_context.load_font(Path::new("assets/font_pixelated.ttf"), 22).unwrap();
    let level_title_font = ttf_context.load_font(Path::new("assets/font_6809.ttf"), 54).unwrap();
    // font.set_style(sdl2::ttf::STYLE_BOLD);

    //
    // Input
    //
    let mut keyboard_input = GameInputState::new();
    let mut joystick_input = GameInputState::new();

    //
    // Player and Map
    //
    let (mut map, mut player) = parse_level("1-starting", &renderer).unwrap();

    #[allow(unused_variables)]
    let level_music;
    if !map.level_music.is_none() {
        level_music = play_music(Path::new(map.level_music.as_ref().unwrap().as_str()));
    }


    // NOTE(erick): Running cat animation stuff. This is only so we can have
    // an example of the animation code
    let mut cat_anim_info = AnimationInfo::new(true, 12);
    cat_anim_info.animation_lanes.push(AnimationLane{number_of_frames: 6});
    cat_anim_info.animation_lanes.push(AnimationLane{number_of_frames: 6});
    let (running_cat_texture, texture_w, texture_h) = texture_from_path(Path::new("assets/animate.bmp"), &renderer);
    let mut running_cat_sprite = SpriteSheet::new(Rc::new(running_cat_texture), texture_w, texture_h, 128, 82, cat_anim_info);
    running_cat_sprite.set_animation_lane_index(1);
    let running_cat_width = 4.0;
    let running_cat_height = running_cat_width * 82.0 / 128.0;
    let mut running_cat = Entity::new(running_cat_sprite, Vector2::new(16.0, 13.5), running_cat_width, running_cat_height, running_cat_width, running_cat_height);


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

                    // TODO(erick): This code must be transfered to sdl_misc
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

        // HACK(erick): We are moving the player (and the boxes) here.
        // This code need to be generalized, but I don't know to to generalize it yet.
        // TODO(erick): Entity-vs-entity collision when the second entity is not movable
        // is not handled yet.

        fn move_entity(entity: &mut Entity, mut force: Vector2, map: &mut Map, dt: f32) {
            const entity_mass : f32 = 0.0058;
            const drag : f32 = 20.0;

            force.normalize_or_zero();

            entity.acceleration = force / entity_mass - entity.velocity * drag;
            entity.velocity += entity.acceleration * dt;
            let mut target_movement = entity.velocity * dt;

            target_movement = entity.collision_against_tiles(map, target_movement);
            entity.position += target_movement;

        }

        move_entity(&mut player, move_direction, &mut map, dt);

        {
            let mut end_game = true;
            for _box in & map.boxes {
                let box_tile = map.tile_at(_box.position.x as u32, _box.position.y as u32);
                if let TileType::Target = box_tile {

                } else {
                    end_game = false;
                    break;
                }
            }

            if end_game {
                game_state.is_running = false;
            }
        }


        let fps_text = format!("Frame time: {:.3}", dt);

        running_cat.sprite_sheet.animation_accumulate_dt(dt);

        renderer.clear();
        map.draw(&mut renderer);
        player.draw(&mut renderer);
        running_cat.draw(&mut renderer);

        draw_text(&mut renderer, &fps_font, Color::RGB(255, 0, 0), &fps_text, Vector2::new(0.02, 0.02), false);

        draw_text(&mut renderer, &level_title_font, Color::RGBA(0, 167, 208, 127), &map.name, Vector2::new(0.5, 0.1), true);

        renderer.present();

        // use std::time::Duration;
        // std::thread::sleep(Duration::from_millis(100));
    }
}

#[derive(Debug)]
enum AssetType {
    Sound,
    Sprite,
    Map,
    Level,
}

fn create_player(player_position: (u32, u32), renderer: &Renderer) -> Entity {
    let player_anim_info = AnimationInfo::new(false, 0);

    let (player_texture, texture_w, texture_h) = texture_from_path(Path::new("assets/player.bmp"), &renderer);
    let player_sprite = SpriteSheet::new(Rc::new(player_texture), texture_w, texture_h, texture_w, texture_h, player_anim_info);

    let player_x = player_position.0 as f32;
    let player_y = player_position.1 as f32;
    let player_width_to_height_ratio = texture_w as f32 / texture_h as f32;

    let player_draw_height = 1.2;
    let player_draw_width = player_draw_height * player_width_to_height_ratio;

    let player_collision_height = 0.8;
    let player_colliion_width  = player_draw_width;

    let mut player = Entity::new(player_sprite, Vector2::new(player_x, player_y),
                        player_colliion_width, player_collision_height,
                        player_draw_width, player_draw_height);
    player.center_on_current_tile_rect();

    player
}

fn asset_path_string(asset_type: AssetType, asset_name: &str) -> String {
    let mut result = String::new();

    match asset_type {
        AssetType::Sound    => { result.push_str("assets/") },
        AssetType::Sprite   => { result.push_str("assets/") },
        AssetType::Map      => { result.push_str("assets/maps/") },
        AssetType::Level    => { result.push_str("assets/maps/") },
    }
    result.push_str(asset_name);

    result
}

fn remove_asset_path(asset_type: AssetType, asset_path: &str) -> &str {
    let offset : usize;

    match asset_type {
        AssetType::Sound    => { offset = "assets/".len() },
        AssetType::Sprite   => { offset = "assets/".len() },
        AssetType::Map      => { offset = "assets/maps/".len() },
        AssetType::Level    => { offset = "assets/maps/".len() },
    };

    &asset_path[offset..]
}

#[allow(dead_code)]
fn write_level_file(level_file_name: &str, map: &Map, textures_names: &HashMap<&str, String>, player_position: (u32, u32)) {
    //
    // Level file
    //
    let mut level_file_with_extension = String::from(level_file_name);
    level_file_with_extension.push_str(".lvl");

    let level_file_path = asset_path_string(AssetType::Level, level_file_with_extension.as_str());
    let level_output_path = Path::new(level_file_path.as_str());

    //
    // Map file
    //
    let mut map_file_with_extension = String::from(level_file_name);
    map_file_with_extension.push_str(".map");

    let map_file_path = asset_path_string(AssetType::Map, map_file_with_extension.as_str());
    let map_output_path = Path::new(map_file_path.as_str());


    if map.boxes.len() == 0 {
        panic!("Map must have at least one box");
    }

    let mut output_file = match File::create(level_output_path) {
        Ok(file)    => file,
        Err(_)      => panic!("Could not open file {:?}", level_output_path),
    };

    output_file.write_all(format!("level_name = {}\n", map.name).as_bytes())
        .expect("Failed to write");

    if !map.level_music.is_none() {
        output_file.write_all(format!("level_music = {}\n",
            remove_asset_path(AssetType::Sound, map.level_music.as_ref().unwrap())).as_bytes())
            .expect("Failed to write");
    }

    if !map.next_level.is_none() {
        output_file.write_all(format!("next_level = {}\n\n",
            remove_asset_path(AssetType::Level, map.next_level.as_ref().unwrap().as_str())).as_bytes())
            .expect("Failed to write");
    }

    output_file.write_all(format!("wall_tile = {}\n", textures_names.get("wall_tile").as_ref().unwrap()).as_bytes())
        .expect("Failed to write");
    output_file.write_all(format!("floor_tile = {}\n", textures_names.get("floor_tile").as_ref().unwrap()).as_bytes())
        .expect("Failed to write");
    output_file.write_all(format!("target_tile = {}\n\n", textures_names.get("target_tile").as_ref().unwrap()).as_bytes())
        .expect("Failed to write");

    output_file.write_all(format!("player_position = ({}, {})\n\n", player_position.0, player_position.1).as_bytes())
        .expect("Failed to write");

    output_file.write_all(format!("box_sprite_sheet = {}\n", textures_names.get("box_sprite_sheet").as_ref().unwrap()).as_bytes())
        .expect("Failed to write");
    output_file.write_all(format!("box_sprite_width = {}\n", map.boxes[0].sprite_sheet.sprite_width).as_bytes())
        .expect("Failed to write");
    output_file.write_all(format!("box_sprite_height = {}\n", map.boxes[0].sprite_sheet.sprite_height).as_bytes())
        .expect("Failed to write");

    output_file.write_all("box_positions = {".as_bytes())
        .expect("Failed to write");

    let mut first = true;
    for _box in &map.boxes {
        if !first {
            output_file.write_all(", ".as_bytes())
                .expect("Failed to write");
        }
        output_file.write_all(format!("({}, {})", _box.position.x as u32, _box.position.y as u32).as_bytes())
            .expect("Failed to write");
        first = false;
    }

    output_file.write_all("}\n\n".as_bytes())
        .expect("Failed to write");

    output_file.write_all(format!("tile_map = {}", map_file_with_extension).as_bytes())
        .expect("Failed to write");
    write_map_file(map_output_path, map);
}

#[allow(dead_code)]
fn write_map_file(map_path: &Path, map: &Map) {
    let mut map_file = File::create(map_path).expect(format!("Could not open file {:?}", map_path).as_str());

    let mut current_col = 0;
    for tile_type in &map.tiles {
        if current_col == map.tiles_stride {
            current_col = 0;
            map_file.write_all("\n".as_bytes())
            .expect("Failed to write");;
        }

        let tile_code = match tile_type {
            &TileType::Wall      => 1,
            &TileType::Floor     => 0,
            &TileType::Target    => 4,
            &TileType::Blank     => 5,
        };

        map_file.write_all(format!("{} ", tile_code).as_bytes())
        .expect("Failed to write");

        current_col += 1;
    }
}

fn parse_level(level_name: &str, renderer: &Renderer) -> (Option<(Map, Entity)>) {
    let mut filename = String::from(level_name);
    filename.push_str(".lvl");

    let level_full_path_string = asset_path_string(AssetType::Level, filename.as_str());
    let level_file_path = Path::new(level_full_path_string.as_str());


    let mut _level_name         = None;
    let mut _level_music        = None;
    let mut _next_level         = None;
    let mut _wall_tile          = None;
    let mut _floor_tile         = None;
    let mut _target_tile        = None;
    let mut _tile_map           = None;
    let mut _player_position    = None;
    let mut _box_sprite_sheet   = None;
    let mut _box_sprite_width   = None;
    let mut _box_sprite_height  = None;
    let mut _box_positions      = None;

    let level_file = match File::open(level_file_path) {
        Ok(file)    => file,
        Err(_)      => { return None; }
    };

    let level_data = BufReader::new(&level_file);

    let mut line_number = 0;
    for line in level_data.lines() {
        line_number += 1;

        let line = line.unwrap();
        if line == "" {
            continue;
        }
        if line.starts_with("//") {
            continue;
        }

        let attrib_index = line.find('=');
        if attrib_index.is_none() {
            println!("Error({:?} : {}): Could not find '=' sign", level_file_path, line_number);
            return None; // NOTE(erick): Maybe continue?
        }

        let _split = line.split_at(attrib_index.unwrap());
        let lhs = (_split.0).trim();
        let rhs = (_split.1)[1..].trim();

        match lhs {
            "level_name"          => {_level_name           = Some(rhs.to_string())},
            "level_music"         => {_level_music          = Some(rhs.to_string())},
            "next_level"          => {_next_level           = Some(rhs.to_string())},
            "tile_map"            => {_tile_map             = Some(rhs.to_string())},
            "wall_tile"           => {_wall_tile            = Some(rhs.to_string())},
            "floor_tile"          => {_floor_tile           = Some(rhs.to_string())},
            "target_tile"         => {_target_tile          = Some(rhs.to_string())},
            "box_sprite_sheet"    => {_box_sprite_sheet     = Some(rhs.to_string())},
            "player_position"     => {_player_position      = parse_position_tuple(rhs)},
            "box_positions"       => {_box_positions        = parse_position_tuple_vec(rhs)},
            "box_sprite_width"    => {_box_sprite_width     = parse_or_none::<u32>(rhs)},
            "box_sprite_height"   => {_box_sprite_height    = parse_or_none::<u32>(rhs)},
            _                     => {println!("Unknown variable: {}", lhs);}
        }
    }

    //
    // We got all data from the file. Now we need to check if we got all the information that we need.
    //
    if _level_name.is_none() {
        panic!("Error({:?}): The level has no name.", level_file_path);
    }


    if _wall_tile.is_none() {
        panic!("Error({:?}): A wall tile must be specified", level_file_path);
    }
    if _floor_tile.is_none() {
        panic!("Error({:?}): A floor tile must be specified", level_file_path);
    }
    if _target_tile.is_none() {
        panic!("Error({:?}): A target tile must be specified", level_file_path);
    }
    if _box_sprite_sheet.is_none() {
        panic!("Error({:?}): A box sprite sheet must be specified", level_file_path);
    }
    if _box_sprite_width.is_none() || _box_sprite_height.is_none() {
        panic!("Error({:?}): The box sprite dimensions must be specified", level_file_path);
    }


    if _tile_map.is_none() {
        panic!("Error({:?}): A tile map must be specified", level_file_path);
    }
    if _player_position.is_none() {
        panic!("Error({:?}): No player initial position", level_file_path);
    }
    if _box_positions.is_none() {
        panic!("Error({:?}): No boxes.", level_file_path);
    }

    //
    // Now we create the map and the player
    //
    let floor_path  = asset_path_string(AssetType::Sprite, _floor_tile.unwrap().as_str());
    let wall_path   = asset_path_string(AssetType::Sprite, _wall_tile.unwrap().as_str());
    let target_path = asset_path_string(AssetType::Sprite, _target_tile.unwrap().as_str());
    let box_path    = asset_path_string(AssetType::Sprite, _box_sprite_sheet.unwrap().as_str());

    let map_path    = asset_path_string(AssetType::Map, _tile_map.unwrap().as_str());

    let level_music_path = match _level_music {
        Some(path) => Some(asset_path_string(AssetType::Sound, path.as_str())),
        None => None,
    };
    let next_level_path = match _next_level {
        Some(path) => Some(asset_path_string(AssetType::Level, path.as_str())),
        None => None,
    };

    let _map_data = MapData::load(renderer,
                                    Path::new(floor_path.as_str()),
                                    Path::new(wall_path.as_str()),
                                    Path::new(target_path.as_str()),
                                    Path::new(box_path.as_str()));

    let mut result_map = Map {
        name        : _level_name.unwrap().to_string(),
        level_music : level_music_path,
        next_level  : next_level_path,

        tiles: Vec::new(),
        tiles_stride: -1,

        map_data: _map_data,
        boxes: Vec::new(),
    };

    Map::fill_tiles_and_stride(&mut result_map, Path::new(map_path.as_str()));

    for (box_position_x, box_position_y) in _box_positions.unwrap() {
        Map::add_box(&mut result_map, _box_sprite_width.unwrap(), _box_sprite_height.unwrap(),
                     box_position_x, box_position_y);
    }

    let player = create_player(_player_position.unwrap(), renderer);
    Some((result_map, player))
}

fn parse_or_none<T> (s: &str) -> Option<T> where T: std::str::FromStr {
    let result = match s.parse::<T>() {
        Ok(value)   => Some(value),
        Err(_)      => {
            println!("Error: Could parse {}", s);
            None
        },
    };

    result
}

fn tuple_from_strings<T> (v_0_str: &str, v_1_str: &str) -> Option< (T, T) > where T: std::str::FromStr {
    let v_0 = parse_or_none::<T>(v_0_str);
    let v_1 = parse_or_none::<T>(v_1_str);

    if v_0.is_none() || v_1.is_none() {
        return None
    }

    Some((v_0.unwrap(), v_1.unwrap()))
}

fn parse_position_tuple(s: &str) -> Option<(u32, u32)> {
    // NOTE(erick): This regex is almost identical to the one the the parse_position_tuple_vec function. Any modification here should
    // be reflected there.
    // NOTE(erick): Matches:
    // '(' <any number of white spaces> <an integer number> ',' <any number of white spaces> <an integer number> <any number of white spaces> ')'
    let tuple_re = Regex::new(r"\(\s*(-?[0-9]+),\s*(-?[0-9]+)\s*\)").unwrap();

    let captures = match tuple_re.captures(s) {
        Some(cap)   => cap,
        None        => {
            println!("Error: Could parse {} as a position tuple", s);
            return None;
        }
    };

    let v_0_str = captures.get(1).unwrap().as_str();
    let v_1_str = captures.get(2).unwrap().as_str();

    let result = tuple_from_strings::<u32>(v_0_str, v_1_str);

    result
}

fn parse_position_tuple_vec(s: &str) -> Option<Vec< (u32, u32) > > {
    let mut result = Vec::new();
    // TODO(erick): Some unit tests for this regexes would be nice!
    let tuple_vec_re = Regex::new(
        r"\{\s*((?:\(\s*(?:-?[0-9]+),\s*(?:-?[0-9]+)\s*\)\s*,\s*)*(?:\(\s*(?:-?[0-9]+),\s*(?:-?[0-9]+)\s*\)))\s*\}")
        .unwrap();

    // NOTE(erick): This regex is almost identical to the one the the parse_position_tuple function. Any modification here should
    // be reflected there.
    let tuple_and_rest_re = Regex::new(r"\s*\(\s*(-?[0-9]+),\s*(-?[0-9]+)\s*\)\s*(?:,\s*(.*))?").unwrap();

    let vec_captures = match tuple_vec_re.captures(s) {
        Some(cap)   => cap,
        None        => {
            println!("Error: Could parse a vector of position tuples: {}", s);
            return None;
        }
    };

    let mut vec_str =  vec_captures.get(1).unwrap().as_str();

    // NOTE(erick): It would be interesting to write an iterator for this loop.
    loop {
        let tuple_capture = tuple_and_rest_re.captures(vec_str).unwrap();
        let tuple_v0_str = tuple_capture.get(1).unwrap().as_str();
        let tuple_v1_str = tuple_capture.get(2).unwrap().as_str();

        let rest = tuple_capture.get(3);

        let tuple = tuple_from_strings::<u32>(tuple_v0_str, tuple_v1_str);
        if tuple.is_none() {
            println!("Failed to parse tuple vector. Could not parse tuple ({:?}, {:?}).\nAborting.",
                        tuple_v0_str, tuple_v1_str);
            return None;
        } else {
            result.push(tuple.unwrap());
        }

        if rest.is_none() {
            break;
        } else {
            vec_str = rest.unwrap().as_str();
        }
    }

    Some(result)
}

// TODO(erick): We should eventually create a tait Draw so we can move this
// function to a separate file and impl draw for Entity
fn draw_text(renderer: &mut Renderer, font: &Font, color: Color, string: &String, position: Vector2, centered: bool) {
    let text_surface = font.render(string)
        .blended(color).unwrap();
    let mut text_texture = renderer.create_texture_from_surface(&text_surface).unwrap();

    let text_x = (WINDOW_WIDTH as f32 * position.x) as i32;
    let text_y = (WINDOW_HEIGHT as f32 * position.y) as i32;


    let TextureQuery { width: text_width, height: text_height, .. } = text_texture.query();
    let text_rect;

    if centered {
        text_rect = Rect::new(text_x - (text_width / 2) as i32, text_y - (text_height / 2) as i32, text_width, text_height);
    } else {
        text_rect = Rect::new(text_x, text_y, text_width, text_height);
    }

    renderer.copy(&mut text_texture, None, Some(text_rect)).unwrap();
}
