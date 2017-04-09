extern crate sdl2;

use self::sdl2::render::Texture;
use self::sdl2::render::Renderer;

use self::sdl2::keyboard::Keycode;

use self::sdl2::mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG,
                    AUDIO_S16LSB};
use self::sdl2::mixer::Music;
use self::sdl2::Sdl;

use self::sdl2::GameControllerSubsystem;
use self::sdl2::controller::GameController;

use std::collections::HashSet;
use std::path::Path;


pub struct AudioMixer {
    pub frequency: i32,
    pub format: u16,
    pub channels: i32,
    pub chunk_size: i32,

    pub audio: Option<sdl2::AudioSubsystem>,
    pub mixer_context: Option<sdl2::mixer::Sdl2MixerContext>,
}

impl AudioMixer {
    pub fn new(sdl: &Sdl) -> AudioMixer {
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

pub fn play_music<'a> (filename: &Path) -> Music<'a> {
    let music: Music = sdl2::mixer::Music::from_file(filename).unwrap();
    // NOTE(erick): -1 loops forever.
    if !music.play(-1).is_ok() {
        println!("Could not play file: {:?}", filename);
    }

    music
}

pub fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

pub fn init_controller(game_controller_subsystem : &GameControllerSubsystem) -> Option<GameController> {
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

pub fn texture_from_path(path: &Path, renderer: &Renderer) -> (Texture, u32, u32) {
    let temp_surface = sdl2::surface::Surface::load_bmp(path).unwrap();

    let texture = renderer.create_texture_from_surface(&temp_surface).unwrap();

    (texture, temp_surface.width(), temp_surface.height())
}

pub fn pressed_keycode_set(e: &sdl2::EventPump) -> HashSet<Keycode> {
    e.keyboard_state().pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect()
}
