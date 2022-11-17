use std::collections::HashMap;

use crate::attacks::Attack;
use crate::enchantments::Enchantable;
use crate::map::Floor;
use crate::math::get_angle;
use crate::monsters::Monster;
use crate::player::{interact_with_door, move_player, player_attack, DoorInteraction, Player};
use macroquad::prelude::*;

pub fn movement_input(
	player: &mut Player, attacks: &mut Vec<Box<dyn Attack>>, textures: &HashMap<String, Texture2D>,
	floor: &mut Floor,
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
		player_attack(player, textures, attacks, floor, true);
	}

	if is_mouse_button_down(MouseButton::Right) {
		player_attack(player, textures, attacks, floor, false);
	}

	if x_movement != 0.0 || y_movement != 0.0 {
		let angle = y_movement.atan2(x_movement);
		move_player(player, angle, None, floor)
	}
}

pub fn door_interaction_input(
	player: &Player, players: &[Player], monsters: &[Box<dyn Monster>], floor: &mut Floor,
	textures: &HashMap<String, Texture2D>,
) {
	if is_key_pressed(KeyCode::O) {
		interact_with_door(
			player,
			players,
			monsters,
			DoorInteraction::Opening,
			floor,
			textures,
		);
	}

	if is_key_pressed(KeyCode::C) {
		interact_with_door(
			player,
			players,
			monsters,
			DoorInteraction::Closing,
			floor,
			textures,
		);
	}
}
