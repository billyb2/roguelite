use crate::attacks::Attack;
use crate::draw::Textures;
use crate::map::FloorInfo;
use crate::math::{get_angle, AsAABB};
use crate::player::{
	interact_with_door, move_player, pickup_items, player_attack, DoorInteraction, Player,
};
use crate::NUM_PLAYERS;
use gilrs::{Axis, Button, Gamepad};
use macroquad::prelude::*;

pub fn movement_input(
	player: &mut Player, index: Option<usize>, attacks: &mut Vec<Box<dyn Attack>>,
	textures: &Textures, floor_info: &mut FloorInfo, camera: &Camera2D,
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

	let mouse_pos: Vec2 = mouse_position().into();

	player.angle = get_angle(
		mouse_pos,
		match NUM_PLAYERS == 1 {
			true => camera.world_to_screen(player.center()),
			false => {
				camera.world_to_screen(player.center())
					+ Vec2::new(
						0.0,
						(camera.viewport.unwrap().3 as f32) * (1.0 / NUM_PLAYERS as f32),
					)
			},
		},
	);

	if is_mouse_button_down(MouseButton::Left) {
		player_attack(player, index, textures, attacks, floor_info, true);
	}

	if is_mouse_button_down(MouseButton::Right) {
		player_attack(player, index, textures, attacks, floor_info, false);
	}

	if is_key_down(KeyCode::P) {
		pickup_items(player, &mut floor_info.floor);
	}

	if x_movement != 0.0 || y_movement != 0.0 {
		let angle = y_movement.atan2(x_movement);
		move_player(player, angle, None, &floor_info.floor);
	}
}

pub fn movement_input_controller(
	player: &mut Player, index: Option<usize>, attacks: &mut Vec<Box<dyn Attack>>,
	textures: &Textures, floor_info: &mut FloorInfo, gamepad: &Gamepad,
) {
	let x_movement = gamepad
		.axis_data(Axis::LeftStickX)
		.map(|a| a.value())
		.unwrap_or_default();

	let y_movement = -gamepad
		.axis_data(Axis::LeftStickY)
		.map(|a| a.value())
		.unwrap_or_default();

	if x_movement.abs() > f32::EPSILON || y_movement.abs() > f32::EPSILON {
		let angle = y_movement.atan2(x_movement);
		move_player(player, angle, None, &floor_info.floor);
	}

	let x_movement_r = gamepad
		.axis_data(Axis::RightStickX)
		.map(|a| a.value())
		.unwrap_or_default();

	let y_movement_r = -gamepad
		.axis_data(Axis::RightStickY)
		.map(|a| a.value())
		.unwrap_or_default();

	player.angle = y_movement_r.atan2(x_movement_r);

	if let Some(button_data) = gamepad.button_data(Button::LeftTrigger2) {
		if button_data.is_pressed() {
			player_attack(player, index, textures, attacks, floor_info, false);
		}
	}

	if let Some(button_data) = gamepad.button_data(Button::RightTrigger2) {
		if button_data.is_pressed() {
			player_attack(player, index, textures, attacks, floor_info, true);
		}
	}
}

pub fn door_interaction_input(
	player: &Player, players: &[Player], floor: &mut FloorInfo, textures: &Textures,
) {
	if is_key_pressed(KeyCode::O) {
		interact_with_door(player, players, DoorInteraction::Opening, floor, textures);
	}

	if is_key_pressed(KeyCode::C) {
		interact_with_door(player, players, DoorInteraction::Toggle, floor, textures);
	}
}

pub fn door_interaction_input_controller(
	player: &Player, players: &[Player], floor: &mut FloorInfo, textures: &Textures,
	gamepad: &Gamepad,
) {
	if let Some(button_data) = gamepad.button_data(Button::South) {
		if button_data.is_pressed() {
			interact_with_door(player, players, DoorInteraction::Opening, floor, textures);
		}
	}
}
