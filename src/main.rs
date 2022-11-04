mod player;
mod attacks;
mod input;
mod math;
mod map;
mod monsters;
mod draw;
mod enchantments;

use std::{
    fs, 
    collections::HashMap, io::{self, Write}, 
};

use attacks::*;
use map::*;
use player::*;
use draw::*;
use input::*;
use monsters::*;

use macroquad::{prelude::*, ui::root_ui, miniquad::conf::Platform};

#[macroquad::main(window_conf)]
async fn main() {
    print!("What player class are you? Warrior or Wizard?: ");
    io::stdout().flush().unwrap();
    
    let mut class = String::new();
    io::stdin().read_line(&mut class).unwrap();

    class = class.to_lowercase();
    class.pop();


    let class = 
        match class.is_empty() {
            true => PlayerClass::Wizard,
            false => match class.as_str() {
                "warrior" => PlayerClass::Warrior,
                "wizard" | "." => PlayerClass::Wizard,
                c => panic!("Invalid class given: {c}"),
            },

        };

    let textures: HashMap<String, Texture2D> = fs::read_dir("assets").unwrap().filter_map(|file| {
        if let Ok(file) = file {
            let file_name = file.file_name().to_str().unwrap().to_string();

            let image_bytes = fs::read(file.path()).unwrap();
            let texture = Texture2D::from_file_with_format(&image_bytes, None);

            Some((file_name, texture))

        } else {
            None

        }

    }).collect();
    
    let mut monsters: Vec<Box<dyn Monster>> = Vec::new();

    let mut map = Map::new(&textures, &mut monsters);
    let mut players = vec![Player::new(class, map.current_floor().current_spawn())];

    let mut camera = Camera2D {
        target: players[0].pos(),
        //zoom: Vec2::new(0.001, -0.001 * (screen_width() / screen_height())),
        zoom: Vec2::new(0.005, -0.005 * (screen_width() / screen_height())),
        ..Default::default()
    };

    const SHOW_FRAMERATE: bool = true;

    let mut frames_till_update_framerate: u8 = 30;
    let mut fps = 0.0;

    loop {
        let frame_time = get_frame_time();
        frames_till_update_framerate -= 1;

        // If running at more than 60 fps, slow down
        if frame_time < 1.0 / 60.0 {
            let time_to_sleep = ((1.0 / 60.0) - frame_time) * 1000.0;
            std::thread::sleep(std::time::Duration::from_millis(time_to_sleep as u64));

        }

        // Logic
        keyboard_input(&mut players[0], &mut monsters, &textures, map.current_floor());
        update_cooldowns(&mut players);
        update_attacks(&mut players, &mut monsters, map.current_floor());
        update_monsters(&mut monsters, &mut players, map.current_floor());


        if map.current_floor().should_descend(&players, &monsters) {
            map.descend(&mut players, &mut monsters, &textures)

        }

        // Rendering
        clear_background(WHITE);

        camera.target = players[0].pos();
        camera.zoom.y = -0.005 * (screen_width() / screen_height());
        set_camera(&camera);

        map.draw();
        monsters.iter().for_each(|m| m.draw());
        players.iter().flat_map(|p| p.attacks.iter()).for_each(|a| a.draw());
        players.iter().for_each(|p| p.draw());

        root_ui().label(Vec2::ZERO, &fps.to_string());

        if SHOW_FRAMERATE && frames_till_update_framerate == 0 {
            fps = (1.0 / frame_time).round();
            frames_till_update_framerate = 30;

        }

        next_frame().await
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Roguelite".to_string(),
        platform: Platform {
            //linux_backend: LinuxBackend::WaylandWithX11Fallback,

            ..Default::default()
        },

        ..Default::default()
    }

}
