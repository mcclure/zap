// Gameplay mechanics

use crate::constants::*;
use crate::room::*;
use glam::IVec2;

pub struct GameState {
    pub player_idx: usize,
    keys: u32,
}
impl Default for GameState {
    fn default() -> Self { Self { player_idx:0, keys:0 } }
}

pub fn game_collide(state:&mut GameState, from:&Actor, into:&Actor, at:IVec2) -> (bool, bool) { // Returns halt? delete?
	match (from, into) {
		(Actor::Player(_), Actor::Key(_)) => {
			state.keys += 1;
			(false, true)
		},
		(Actor::Player(_), Actor::Door) => {
			if state.keys > 0 {
				state.keys -= 1;
				(false, true)
			} else {
				(true, false)
			}
		},
		_ => (false, false)
	}
}

pub fn game_move(state:&mut GameState, room:&mut Room, dir:Dir) {
    let (player@Actor::Player(mut player_dir), mut player_at) = room.actors[state.player_idx]
    	else { panic!("Player not found where expected"); };
    if dir == player_dir {
        if 0 != room.routes[ivec_to_index(player_at)] & (1 << player_dir as u8) {
            let want_at = player_at + DIR_COMPASS[dir as usize];
            let mut deletes = Vec::<usize>::default();

            'collide: {
            	for (idx, (actor, at)) in room.actors.iter().enumerate() {
	                if want_at == *at {
	                	let (halt, destroy) = game_collide(state, &player, &actor, want_at);
	                	if destroy { deletes.push(idx); }
	                	if halt { break 'collide; }
	                }
            	}

            	player_at = want_at;
            }

            for &target in deletes.iter().rev() { // FIXME: use slots or something
            	room.actors.remove(target);
            	if state.player_idx > target { state.player_idx -= 1 } // FIXME: USE SLOTS!!
            }
        }
    } else {
        player_dir = dir;
    }
    room.actors[state.player_idx] = (Actor::Player(player_dir), player_at);
}
