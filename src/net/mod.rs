use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use ggrs::{Config, GGRSRequest, P2PSession, SessionBuilder, UdpNonBlockingSocket};
use serde::{Deserialize, Serialize};

use crate::attacks::update_attacks;
use crate::init_game::{GameInfo, GameState};
use crate::input::PlayerInput;

use crate::map::{set_effects, trigger_traps, update_effects};
use crate::monsters::update_monsters;
use crate::player::{
	interact_with_door, move_player, player_attack, update_cooldowns, DoorInteraction,
};
use crate::FPS;

#[derive(Clone, Serialize, Deserialize)]
pub struct GGRSConfig {
	pub multiplayer: bool,
	pub local_port: u16,
	pub remote_port: u16,
}

impl Default for GGRSConfig {
	fn default() -> Self {
		Self {
			multiplayer: false,
			local_port: 1111,
			remote_port: 2222,
		}
	}
}

impl Config for GGRSConfig {
	type Input = PlayerInput;
	type State = GameState;
	type Address = SocketAddr;
}

pub fn init_net(conf: &GGRSConfig) -> P2PSession<GGRSConfig> {
	let local_sock = UdpNonBlockingSocket::bind_to_port(conf.local_port).unwrap();
	let remote = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, conf.remote_port));

	SessionBuilder::<GGRSConfig>::new()
		.with_num_players(2)
		.with_fps(FPS as usize)
		.unwrap()
		// .with_input_delay(1)
		.add_player(ggrs::PlayerType::Local, 0)
		.unwrap()
		.add_player(ggrs::PlayerType::Remote(remote), 1)
		.unwrap()
		.with_sparse_saving_mode(true)
		.start_p2p_session(local_sock)
		.unwrap()
}

pub fn handle_requests(reqs: Vec<GGRSRequest<GGRSConfig>>, game_info: &mut GameInfo) {
	reqs.iter().for_each(|req| match req {
		GGRSRequest::SaveGameState { cell, frame } => {
			// let bin = bincode::serialize(&game_info.game_state).unwrap();
			// let checksum = fletcher16(bin) as u128;
			cell.save(*frame, Some(game_info.game_state.clone()), None);
		},
		GGRSRequest::LoadGameState { cell, frame: _ } => {
			game_info.game_state = cell.load().unwrap();
		},
		GGRSRequest::AdvanceFrame { inputs } => {
			game_info.game_state.frame += 1;
			let players = &mut game_info.game_state.players;

			inputs.iter().zip(players.iter_mut().enumerate()).for_each(
				|((input, _input_status), (i, player))| {
					player.angle = input.rotation();

					if input.is_moving() {
						move_player(
							player,
							input.movement_angle(),
							None,
							&game_info.game_state.map.current_floor().floor,
						);
					}

					if input.using_primary() {
						player_attack(
							player,
							Some(i),
							&mut game_info.game_state.attacks,
							&game_info.game_state.map.current_floor(),
							true,
						);
					}

					if input.using_secondary() {
						player_attack(
							player,
							Some(i),
							&mut game_info.game_state.attacks,
							&game_info.game_state.map.current_floor(),
							false,
						);
					}

					if input.opening_door() {
						interact_with_door(
							player,
							DoorInteraction::Opening,
							game_info.game_state.map.current_floor_mut(),
						);
					}

					if input.closing_door() {
						interact_with_door(
							player,
							DoorInteraction::Closing,
							game_info.game_state.map.current_floor_mut(),
						);
					}
				},
			);

			update_attacks(
				&mut game_info.game_state.players,
				game_info.game_state.map.current_floor_mut(),
				&mut game_info.game_state.attacks,
			);

			update_cooldowns(&mut game_info.game_state.players);

			trigger_traps(
				&mut game_info.game_state.players,
				game_info.game_state.map.current_floor_mut(),
			);
			set_effects(
				&mut game_info.game_state.players,
				game_info.game_state.map.current_floor_mut(),
			);
			update_effects(&mut game_info.game_state.map.current_floor_mut().floor);
			update_monsters(
				&mut game_info.game_state.players,
				game_info.game_state.map.current_floor_mut(),
				&mut game_info.game_state.attacks,
			);
		},
	});
}
