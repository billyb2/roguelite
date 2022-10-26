#![feature(drain_filter)]

mod player;
mod attack;
mod input;
mod math;
mod map;
mod draw;

use std::{
    fs, 
    collections::HashMap
};

use attack::update_attacks;
use map::*;
use player::*;
use draw::*;
use input::*;

use macroquad::prelude::*;

#[macroquad::main("roguelite")]
async fn main() {
    let mut attacks = Vec::new();
    let mut textures = HashMap::new();

    fs::read_dir("assets").unwrap().for_each(|file| {
        if let Ok(file) = file {
            let file_name = file.file_name().to_str().unwrap().to_string();

            let image_bytes = fs::read(file.path()).unwrap();
            let texture = Texture2D::from_file_with_format(&image_bytes, Some(ImageFormat::WebP));

            textures.insert(file_name, texture);

        }

    });
    
    let map = Map::new(&textures);
    let mut players = vec![Player::new(map.current_spawn())];

    loop {
        // Logic
        keyboard_input(&mut players[0], &mut attacks, &textures, &map);
        update_cooldowns(&mut players);
        update_attacks(&mut attacks);

        // Rendering
        clear_background(WHITE);

        set_camera(&Camera2D {
            target: players[0].pos(),
            zoom: Vec2::new(0.005, 0.005 * (screen_width() / screen_height())),
            ..Default::default()
        });

        players.iter().for_each(|p| p.draw());
        attacks.iter().for_each(|a| a.draw());

        map.draw();

        next_frame().await
    }
}
