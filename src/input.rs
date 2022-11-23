use std::collections::HashMap;

use crate::attacks::Attack;
use crate::map::FloorInfo;
use crate::math::get_angle;
use crate::player::{
	interact_with_door, move_player, pickup_items, player_attack, DoorInteraction, Player,
};
use macroquad::prelude::*;

pub fn movement_input(
	player: &mut Player, attacks: &mut Vec<Box<dyn Attack>>, textures: &HashMap<String, Texture2D>,
	floor_info: &mut FloorInfo,
) {
	if player.hp() == 0 {
		return;
	}

	let mut x_movement: f32 = 0.0;
	let mut y_movement: f32 = 0.0;

	if is_key_down(KeyCode::W) {
		y_movement -= 1.0;
	}

	if is_key_down(KeyCode::S) {
		y_movement += 1.0;
	}

	if is_key_down(KeyCode::A) {
		x_movement -= 1.0;
	}

	if is_key_down(KeyCode::D) {
		x_movement += 1.0;
	}

	if is_key_down(KeyCode::Z) {
		player.changing_spell = true;
		player.time_til_change_spell = 15;
	}

	let mouse_pos = mouse_position_local();
	player.angle = get_angle(mouse_pos, Vec2::ZERO);

	if is_mouse_button_down(MouseButton::Left) {
		player_attack(player, textures, attacks, floor_info, true);
	}

	if is_mouse_button_down(MouseButton::Right) {
		player_attack(player, textures, attacks, floor_info, false);
	}

	if is_key_down(KeyCode::P) {
		pickup_items(player, &mut floor_info.floor);
	}

	if x_movement != 0.0 || y_movement != 0.0 {
		let angle = y_movement.atan2(x_movement);
		move_player(player, angle, None, &floor_info.floor);
	}
}

pub fn door_interaction_input(
	player: &Player, players: &[Player], floor: &mut FloorInfo,
	textures: &HashMap<String, Texture2D>,
) {
	if is_key_pressed(KeyCode::O) {
		interact_with_door(player, players, DoorInteraction::Opening, floor, textures);
	}

	if is_key_pressed(KeyCode::C) {
		interact_with_door(player, players, DoorInteraction::Closing, floor, textures);
	}
}
