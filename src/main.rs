#![feature(drain_filter)]
#![feature(const_fn_floating_point_arithmetic)]

mod player;
mod attacks;
mod input;
mod math;
mod map;
mod monsters;
mod draw;

use std::{
    fs, 
    collections::HashMap, f32::consts::PI
};

use attacks::*;
use map::*;
use player::*;
use draw::*;
use input::*;
use monsters::*;

use macroquad::prelude::*;

#[macroquad::main("roguelite")]
async fn main() {
    let mut textures = HashMap::new();

    fs::read_dir("assets").unwrap().for_each(|file| {
        if let Ok(file) = file {
            let file_name = file.file_name().to_str().unwrap().to_string();

            let image_bytes = fs::read(file.path()).unwrap();
            let texture = Texture2D::from_file_with_format(&image_bytes, Some(ImageFormat::WebP));

            textures.insert(file_name, texture);

        }

    });
    
    let mut monsters: Vec<Box<dyn Monster>> = Vec::new();

    let map = Map::new(&textures, &mut monsters);
    let mut players = vec![Player::new(map.current_spawn())];

    let mut camera = Camera2D {
        target: players[0].pos(),
        zoom: Vec2::new(0.005, -0.005 * (screen_width() / screen_height())),
        ..Default::default()
    };

    loop {
        // Logic
        keyboard_input(&mut players[0], &textures, &map);
        update_cooldowns(&mut players);
        update_attacks(&mut players, &mut monsters, &map);
        update_monsters(&mut monsters, &mut players, &map);

        camera.target = players[0].pos();

        // Rendering
        clear_background(WHITE);

//        /*
        set_camera(&camera);
//        */

        map.draw();
        monsters.iter().for_each(|m| m.draw());
        players.iter().flat_map(|p| p.attacks.iter()).for_each(|a| a.draw());
        players.iter().for_each(|p| p.draw());

        next_frame().await
    }
}
