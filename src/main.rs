#![feature(drain_filter)]

mod player;
mod attack;
mod input;
mod math;
mod draw;

use std::fs;

use attack::update_attacks;
use player::*;
use draw::*;
use input::*;

use macroquad::prelude::*;

#[macroquad::main("roguelite")]
async fn main() {
    let mut players = vec![Player::new()];
    let mut attacks = Vec::new();
    let image_bytes = fs::read("swipe.webp").unwrap();
    let texture = Texture2D::from_file_with_format(&image_bytes, Some(ImageFormat::WebP));

    loop {
        // Logic
        keyboard_input(&mut players[0], &mut attacks, texture);
        update_cooldowns(&mut players);
        update_attacks(&mut attacks);

        // Rendering
        clear_background(WHITE);

        players.iter().for_each(|p| p.draw());
        attacks.iter().for_each(|a| a.draw());

        next_frame().await
    }
}
