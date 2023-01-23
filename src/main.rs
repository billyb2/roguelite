mod attacks;
mod config;
mod draw;
mod enchantments;
mod init_game;
mod input;
mod items;
mod map;
mod math;
mod monsters;
mod net;
mod player;

use std::time::{Duration, Instant};

use attacks::*;
use draw::*;
use egui::{FontId, RichText};
use ggrs::{GGRSEvent, P2PSession, SessionState};
use init_game::*;
use input::*;
use map::*;
use monsters::*;
use net::{handle_requests, GGRSConfig};
use player::*;

use macroquad::miniquad::conf::Platform;
use macroquad::prelude::*;
use macroquad::ui::root_ui;

use rayon::prelude::*;

use crate::enchantments::EnchantmentKind;
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

pub const NUM_PLAYERS: usize = 2;

pub const FPS: f64 = 60.0;

pub static mut NET_SESSION: Option<P2PSession<GGRSConfig>> = None;

fn update_game(game_info: &mut GameInfo) -> Option<Screen> {
	if let Some(net_session) = unsafe { &mut NET_SESSION } {
		net_session.poll_remote_clients();

		net_session.events().for_each(|ev| {
			if let GGRSEvent::WaitRecommendation { skip_frames } = ev {
				game_info.frames_to_skip = skip_frames
			}
		});

		if game_info.frames_to_skip > 0 {
			game_info.frames_to_skip -= 1;
			render_game(game_info);
			return None;
		}

		let mut fps_delta = 1. / FPS;
		if net_session.frames_ahead() > 0 {
			fps_delta *= 1.1;
		}

		// get delta time from last iteration and accumulate it
		let delta = Instant::now().duration_since(game_info.last_update);
		game_info.accumulator = game_info.accumulator.saturating_add(delta);
		game_info.last_update = Instant::now();

		while game_info.accumulator.as_secs_f64() > fps_delta {
			game_info.accumulator = game_info
				.accumulator
				.saturating_sub(Duration::from_secs_f64(fps_delta));

			// Frames are only happening if sessions are synced
			if net_session.current_state() == SessionState::Running {
				// Add input for all local players
				let local_input = movement_input(
					&game_info.game_state.players[0],
					Some(0),
					&game_info.cameras[0],
				);

				net_session
					.local_player_handles()
					.into_iter()
					.for_each(|handle| {
						net_session.add_local_input(handle, local_input).unwrap();
					});

				match net_session.advance_frame() {
					Ok(requests) => {
						handle_requests(requests, game_info);
					},
					Err(ggrs::GGRSError::PredictionThreshold) => {
						// println!("Frame {} skipped",
						// net_session.current_frame());
					},
					Err(e) => println!("{e:?}"),
				}
			}
		}
	}

	render_game(game_info);

	/*
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

	#[cfg(feature = "native")]
	if let Some(gamepad_id) = game_info.gamepad_info.active_gamepad {
		let gamepad = game_info.gamepad_info.gilrs.gamepad(gamepad_id);

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

	if game_info
		.map
		.current_floor()
		.should_descend(&game_info.players)
	{
		game_info.map.descend(&mut game_info.players);
	}
	*/

	None
}

fn render_game(game_info: &mut GameInfo) {
	clear_background(BLACK);

	game_info.material.set_uniform(
		"window_height",
		game_info.cameras[0].viewport.unwrap().3 as f32,
	);

	let current_floor = game_info.game_state.map.current_floor_mut();

	let exit = current_floor.exit().clone();

	let objects = current_floor.floor.objects_mut();

	let monsters = &mut current_floor.monsters;

	objects
		.par_iter_mut()
		.for_each(|obj| obj.clear_currently_visible());

	game_info.game_state.players.iter().for_each(|player| {
		Floor::set_visible_objects(player, None, objects);
	});

	// Draw all objects that have been seen in the past but are not visible now
	let seen_objects = objects
		.iter()
		.filter(|object: &&Object| object.has_been_seen() && !object.currently_visible());

	let visible_objects: Vec<&Object> = objects
		.iter()
		.filter(|object| object.currently_visible())
		.collect();

	let monsters_to_draw = monsters.iter().filter(|m| {
		let monster_tile_pos = pos_to_tile(&m.as_polygon());
		visible_objects
			.iter()
			.any(|obj| obj.tile_pos() == monster_tile_pos)
	});

	let player = &game_info.game_state.players[0];
	let camera = &mut game_info.cameras[0];

	camera.target = player.center();

	camera.zoom = Vec2::new(
		CAMERA_ZOOM,
		-CAMERA_ZOOM * (screen_width() / game_info.viewport_screen_height),
	) * 0.7;
	camera.viewport = Some((
		0,
		game_info.viewport_screen_height as i32 * 0 as i32,
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

		visible_objects.iter().for_each(|o| {
			o.draw();
			o.items().iter().rev().for_each(|item| {
				item.draw();
			});
		});

		// Draw all monsters on top of a visible object tile
		monsters_to_draw.for_each(|m| m.draw());

		game_info
			.material
			.set_uniform("lowest_light_level", 0.25_f32);

		seen_objects.for_each(|o| {
			o.draw();
		});

		exit.draw();

		game_info
			.material
			.set_uniform("lowest_light_level", 0.6_f32);

		visible_objects
			.iter()
			.flat_map(|o| o.items().iter())
			.for_each(|i| i.draw());

		game_info
			.material
			.set_uniform("lowest_light_level", 1.0_f32);

		game_info.game_state.attacks.iter().for_each(|a| a.draw());
	}

	gl_use_default_material();
	game_info.game_state.players.iter().for_each(|p| p.draw());

	// Draw UI
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
					let config_info = game_info.config_info.clone();
					config_info.set_config(game_info);

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
								&game_info.config_info.class() == &class,
								RichText::new(class.to_string())
									.strong()
									.font(FontId::proportional(30.0)),
							)
							.clicked()
						{
							game_info.config_info.set_class(class);
						}
					};

					class_button(PlayerClass::Warrior);
					class_button(PlayerClass::Wizard);
					class_button(PlayerClass::Rogue);
				});

				ui.horizontal(|ui| {
					let button_text = match game_info.config_info.multiplayer() {
						false => "Singleplayer",
						true => "Multiplayer",
					};

					if ui
						.button(
							RichText::new(button_text)
								.strong()
								.font(FontId::proportional(30.0)),
						)
						.clicked()
					{
						game_info.config_info.set_opposite_multiplayer();
					}
				});

				ui.horizontal(|ui| {
					ui.label(
						RichText::new("Local Port: ")
							.strong()
							.font(FontId::proportional(30.0)),
					);

					let mut local_port_str = game_info.config_info.local_port().to_string();

					ui.text_edit_singleline(&mut local_port_str);

					let new_local_port: u16 = match local_port_str.parse() {
						Ok(port) => port,
						Err(_) => 0,
					};

					game_info.config_info.set_local_port(new_local_port);
				});

				ui.horizontal(|ui| {
					ui.label(
						RichText::new("Remote Port: ")
							.strong()
							.font(FontId::proportional(30.0)),
					);

					let mut remote_port_str = game_info.config_info.remote_port().to_string();

					ui.text_edit_singleline(&mut remote_port_str);

					let new_remote_port: u16 = match remote_port_str.parse() {
						Ok(port) => port,
						Err(_) => 0,
					};

					game_info.config_info.set_remote_port(new_remote_port);
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
	rand::srand(1000);

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
