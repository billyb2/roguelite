mod attacks;
mod draw;
mod enchantments;
mod init_game;
mod input;
mod items;
mod map;
mod math;
mod monsters;
mod player;

use std::io::{self, Write};

use attacks::*;
use draw::*;
use egui::{FontId, RichText};
#[cfg(feature = "native")]
use gilrs::Gilrs;
use init_game::*;
use input::*;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use map::*;
use monsters::*;
use player::*;

use macroquad::miniquad::conf::Platform;
use macroquad::prelude::*;
use macroquad::ui::{root_ui, Skin};

use crate::enchantments::{Enchantable, EnchantmentKind};
use crate::math::AsPolygon;

pub const MAX_VIEW_OF_PLAYER: f32 = 200.0;

const DEFAULT_FRAGMENT_SHADER: &str = "
#version 100
precision lowp float;
varying vec2 uv;
uniform sampler2D Texture;
uniform lowp float lowest_light_level;
uniform lowp float window_height;
const lowp float VISION_SIZE = 400.0;

void main() {
    gl_FragColor = texture2D(Texture, uv);

	float lighting = 1.0;
	lighting *= lowest_light_level;
	gl_FragColor.rgb *= vec3(lighting * 0.75);

}
";

const DEFAULT_VERTEX_SHADER: &str = "
#version 100
precision lowp float;
attribute vec3 position;
attribute vec2 texcoord;
varying vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";

const CAMERA_ZOOM: f32 = 0.0045;

pub const NUM_PLAYERS: usize = 1;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

fn update_game(game_info: &mut GameInfo) -> Option<Screen> {
	#[cfg(feature = "native")]
	while let Some(gilrs::Event {
		id,
		event: _,
		time: _,
	}) = game_info.gamepad_info.gilrs.next_event()
	{
		game_info.gamepad_info.active_gamepad = Some(id);
	}

	game_info.viewport_screen_height = screen_height() * (1.0 / NUM_PLAYERS as f32);

	// Logic
	game_info
		.players
		.iter_mut()
		.for_each(|player| player.update_enchantments());

	movement_input(
		&mut game_info.players[0],
		Some(0),
		&mut game_info.attacks,
		game_info.map.current_floor_mut(),
		&game_info.cameras[0],
	);

	#[cfg(feature = "native")]
	if let Some(gamepad_id) = game_info.gamepad_info.active_gamepad {
		let gamepad = game_info.gamepad_info.gilrs.gamepad(gamepad_id);

		movement_input_controller(
			&mut game_info.players[1],
			Some(1),
			&mut game_info.attacks,
			game_info.map.current_floor_mut(),
			&gamepad,
		);

		door_interaction_input_controller(
			&game_info.players[1],
			&game_info.players,
			game_info.map.current_floor_mut(),
			&gamepad,
		)
	}

	door_interaction_input(
		&game_info.players[0],
		&game_info.players,
		game_info.map.current_floor_mut(),
	);

	trigger_traps(&mut game_info.players, game_info.map.current_floor_mut());
	set_effects(&mut game_info.players, game_info.map.current_floor_mut());
	update_effects(&mut game_info.map.current_floor_mut().floor);
	update_cooldowns(&mut game_info.players);
	update_attacks(
		&mut game_info.players,
		game_info.map.current_floor_mut(),
		&mut game_info.attacks,
	);
	update_monsters(
		&mut game_info.players,
		game_info.map.current_floor_mut(),
		&mut game_info.attacks,
	);

	if game_info
		.map
		.current_floor()
		.should_descend(&game_info.players)
	{
		game_info.map.descend(&mut game_info.players);
	}

	render_game(game_info);

	None
}

fn render_game(game_info: &mut GameInfo) {
	clear_background(BLACK);

	game_info.material.set_uniform(
		"window_height",
		game_info.cameras[0].viewport.unwrap().3 as f32,
	);

	game_info.visible_objects.clear();

	game_info.players.iter().for_each(|player| {
		game_info.visible_objects.push(
			Floor::visible_objects_mut(
				player,
				None,
				game_info.map.current_floor_mut().floor.objects_mut(),
			)
			.into_iter()
			.cloned()
			.collect(),
		)
	});

	let only_show_past_seen_objects = |obj: &&Object| -> bool {
		let is_currently_visible = game_info
			.visible_objects
			.iter()
			.flatten()
			.any(|v_obj| v_obj.tile_pos() == obj.tile_pos());

		obj.has_been_seen() && !is_currently_visible
	};

	// Draw all objects that have been seen
	let seen_objects = game_info
		.map
		.current_floor()
		.floor
		.objects()
		.iter()
		.filter(only_show_past_seen_objects);

	let monsters_to_draw = game_info.map.current_floor().monsters.iter().filter(|m| {
		let monster_tile_pos = pos_to_tile(&m.as_polygon());
		game_info
			.visible_objects
			.iter()
			.flatten()
			.any(|obj| obj.tile_pos() == monster_tile_pos)
	});

	game_info
		.cameras
		.iter_mut()
		.zip(game_info.players.iter())
		.enumerate()
		.for_each(|(i, (camera, player))| {
			camera.target = player.center();

			camera.zoom = Vec2::new(
				CAMERA_ZOOM,
				-CAMERA_ZOOM * (screen_width() / game_info.viewport_screen_height),
			) * 0.7;
			camera.viewport = Some((
				0,
				game_info.viewport_screen_height as i32 * i as i32,
				screen_width() as i32,
				game_info.viewport_screen_height as i32,
			));

			set_camera(camera);

			if player
				.enchantments()
				.get(&EnchantmentKind::Blinded)
				.is_none()
			{
				gl_use_material(game_info.material);
				game_info
					.material
					.set_uniform("lowest_light_level", 0.6_f32);

				game_info.visible_objects.iter().flatten().for_each(|o| {
					o.draw();
					o.items().iter().rev().for_each(|item| {
						item.draw();
					});
				});

				// Draw all monsters on top of a visible object tile
				monsters_to_draw.clone().for_each(|m| m.draw());

				game_info
					.material
					.set_uniform("lowest_light_level", 0.25_f32);

				seen_objects.clone().for_each(|o| {
					o.draw();
				});

				game_info.map.current_floor().exit().draw();

				game_info
					.material
					.set_uniform("lowest_light_level", 0.6_f32);
				game_info
					.visible_objects
					.iter()
					.flatten()
					.flat_map(|o| o.items().iter())
					.for_each(|i| i.draw());

				game_info
					.material
					.set_uniform("lowest_light_level", 1.0_f32);

				game_info.attacks.iter().for_each(|a| a.draw());
			}

			gl_use_default_material();
			game_info.players.iter().for_each(|p| p.draw());

			// Draw UI
			set_default_camera();
			draw_inventory(player);

			root_ui().label(
				Vec2::new(
					(camera.viewport.unwrap().2 - 150) as f32,
					camera.viewport.unwrap().1 as f32,
				),
				&format!("HP: {}", player.hp()),
			);
			root_ui().label(
				Vec2::new(
					(camera.viewport.unwrap().2 - 150) as f32,
					(camera.viewport.unwrap().1 + 10) as f32,
				),
				&format!("MP: {}", player.mp()),
			);

			if let Some(spell) = player.spells().first() {
				root_ui().label(
					Vec2::new(
						(camera.viewport.unwrap().2 - 150) as f32,
						(camera.viewport.unwrap().1 + 20) as f32,
					),
					&match player.changing_spell {
						false => format!("Spell: {}", spell),
						true => "Cycling Spell...".to_string(),
					},
				);
			}
		});
}

enum Screen {
	MainMenu,
	Config,
	Game,
}

fn update_main_menu(game_info: &mut GameInfo) -> Option<Screen> {
	let mut new_screen = None;

	clear_background(WHITE);

	egui_macroquad::ui(|egui_ctx| {
		egui_ctx.set_visuals(egui::Visuals::dark());

		egui::CentralPanel::default().show(egui_ctx, |ui| {
			ui.vertical_centered(|ui| {
				ui.spacing_mut().button_padding = egui::Vec2::new(30.0, 15.5);

				ui.label(
					RichText::new("Roguelite")
						.strong()
						.font(FontId::proportional(45.0)),
				);

				ui.add_space(25.0);

				if ui
					.button(
						RichText::new("Play")
							.strong()
							.font(FontId::proportional(30.0)),
					)
					.clicked()
				{
					game_info.players = init_players(game_info.config_info.class, &game_info.map);
					new_screen = Some(Screen::Game);
				}

				ui.add_space(25.0);

				if ui
					.button(
						RichText::new("Settings")
							.strong()
							.font(FontId::proportional(30.0)),
					)
					.clicked()
				{
					new_screen = Some(Screen::Config);
				}
			});
		});
	});

	egui_macroquad::draw();

	new_screen
}

fn config_game_update(game_info: &mut GameInfo) -> Option<Screen> {
	let mut new_screen = None;

	egui_macroquad::ui(|egui_ctx| {
		egui_ctx.set_visuals(egui::Visuals::dark());

		egui::CentralPanel::default().show(egui_ctx, |ui| {
			ui.vertical_centered(|ui| {
				ui.spacing_mut().button_padding = egui::Vec2::new(30.0, 15.5);

				ui.label(
					RichText::new("Settings")
						.strong()
						.font(FontId::proportional(45.0)),
				);

				ui.add_space(25.0);

				ui.horizontal_top(|ui| {
					let mut class_button = |class: PlayerClass| {
						if ui
							.radio(
								&game_info.config_info.class == &class,
								RichText::new(class.to_string())
									.strong()
									.font(FontId::proportional(30.0)),
							)
							.clicked()
						{
							game_info.config_info.class = class;
						}
					};

					class_button(PlayerClass::Warrior);
					class_button(PlayerClass::Wizard);
					class_button(PlayerClass::Rogue);
				});

				if ui
					.button(
						RichText::new("Back")
							.strong()
							.font(FontId::proportional(30.0)),
					)
					.clicked()
				{
					new_screen = Some(Screen::MainMenu);
				}
			});
		});
	});

	egui_macroquad::draw();

	new_screen
}

#[macroquad::main(window_conf)]
async fn main() {
	for asset_name in ASSETS {
		let path = format!("assets/{asset_name}");
		let texture = load_texture(&path).await.unwrap();

		unsafe {
			TEXTURES.insert(asset_name.to_string(), texture);
		}
	}

	rand::srand(main as u64);

	let mut game_info = init_game();

	let mut update_fn: fn(&mut GameInfo) -> Option<Screen> = update_main_menu;

	loop {
		if let Some(new_screen) = update_fn(&mut game_info) {
			let new_update_fn: fn(&mut GameInfo) -> Option<Screen> = match new_screen {
				Screen::MainMenu => update_main_menu,
				Screen::Game => update_game,
				Screen::Config => config_game_update,
			};

			update_fn = new_update_fn;
		}

		update_fn(&mut game_info);

		next_frame().await;
	}
}

fn window_conf() -> Conf {
	Conf {
		window_title: "Roguelite".to_string(),
		platform: Platform {
			// linux_backend: macroquad::miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
			..Default::default()
		},

		..Default::default()
	}
}
