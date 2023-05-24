// Data structure for a world map tile

use std::cmp::Reverse;

use crate::constants::*;

//use std::mem;
use glam::{IVec2, Vec2};
use ndarray::{Array2, Axis};
use rand::{seq::SliceRandom, Rng};

pub struct Room {
	pub routes:Array2<u8>,
	pub walls:Array2<u8>,
	pub actors:Vec<(Actor, IVec2)> // Data, location
}

fn make_float(v:IVec2, scale:Vec2) -> [f32;2] {
	(
		Vec2::new(v.x as f32, v.y as f32)
		/ scale
	).to_array()
}

fn _debug_room(routes: &Array2<u8>, origin:IVec2, player:IVec2, dir:usize) {
	for (y,col) in routes.axis_iter(Axis(0)).enumerate() {
		for (x,tile_mask) in col.iter().enumerate() {
			const R:usize = Dir::Right as usize; const D:usize = Dir::Down as usize; const L:usize = Dir::Left as usize; const U:usize = Dir::Up as usize;
			print!("{}{}{}{}{} ", if tile_mask&DirMask::Up as u8!=0 {'U'} else {'_'}, if tile_mask&DirMask::Left as u8!=0 {'L'} else {'_'}, 
				if tile_mask&DirMask::Down as u8!=0 {'D'} else {'_'}, if tile_mask&DirMask::Right as u8!=0 {'R'} else {'_'},
				if player.x==x as i32 && player.y==y as i32 {match dir {
					R => '>', D => 'v', L => '<', U => '^', _ => '?'
				}} else
				if origin.x==x as i32 && origin.y==y as i32 {'*'} else {' '});
		}
		println!("");
	}
}

const TILES:u32 = CANVAS_SIDE/TILE_SIDE - 1;

pub fn room_make(add_actors:bool) -> Room {
	// NDArray helpers
	fn to_index(v:IVec2) -> (usize, usize) { (v.y as usize, v.x as usize) }
	fn within (at:IVec2, size:IVec2) -> bool {
		IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
	}

	// Make map
	let routes_bound = IVec2::new(TILES as i32, TILES as i32);
	let mut routes:Array2<u8> = Array2::default(to_index(routes_bound));
	let mut actors:Vec<(Actor, IVec2)> = Default::default();
	let mut rng = rand::thread_rng();
	type ObjCand = (IVec2, u32);
	let mut path_max: [Option<ObjCand>; 4] = [None, None, None, None];
	{
		// Must randomize indices rather than directions because rotation identity matters
		const COMPASS_IDX:[usize;4] = [2,1,0,3];
		
		let mut stack = vec![(routes_bound/2, COMPASS_IDX, 0, None::<usize>, 0)]; // See…
		loop {
			let top = stack.pop();
			if top == None { break }
			let (at, compass_order, compass_order_idx, root_branch, root_distance) = top.unwrap(); // …here
			// (Note: root_s only needed for player_decorate case)

			if compass_order_idx < 3 {
				stack.push((at, compass_order, compass_order_idx+1, root_branch, root_distance));
			}

			let compass_idx = compass_order[compass_order_idx];
			let cand = at + DIR_COMPASS[compass_idx];

			let root_branch = root_branch.unwrap_or(compass_idx);

			if add_actors { // We could do this one-fourth as often, but then we'd need special behavior for the case where the root is blocked on 3 sides
				let current_path_max = path_max[root_branch];
				let replace;
				if let Some((_, distance)) = current_path_max {
					replace = distance < root_distance;
				} else { replace = true; }
				if replace {
					path_max[root_branch] = Some((at, root_distance));
				}
			}

			if within(cand, routes_bound) {
				let cand_value = routes[to_index(cand)];
				let is_free = cand_value == 0;
				if is_free {
					let mut random_compass = COMPASS_IDX.clone();
					random_compass.shuffle(&mut rng);
					stack.push((cand, random_compass, 0, Some(root_branch), root_distance+1));
				}
//println!("\nFrom {} check {}: {}, {}", at, cand, is_free, 0 != cand_value & 1<<((compass_idx+2)%4)); _debug_room(&routes, routes_bound/2, at, compass_idx);
				if is_free || 0 != cand_value & 1<<((compass_idx+2)%4) {
					routes[to_index(at)] |= 1<<compass_idx; // Reciprocate
				}
			}
		}
	}

	let walls_bound = routes_bound + IVec2::ONE;
	let mut walls:Array2<u8> = Array2::default(to_index(walls_bound));
	// Instead of iterating over the members of the array imagine the grid separating members of the array,
	// and imagine iterating over the intersection points.
	for y in 0..(TILES+1) {
		for x in 0..(TILES+1) {
			let at = (y as usize, x as usize);
			let up_left = IVec2::new(x as i32-1,y as i32-1);
			let down_left = IVec2::new(x as i32-1,y as i32);

			if !within(up_left, routes_bound) || 0==routes[to_index(up_left)]&DirMask::Right as u8
				{ walls[at] |= DirMask::Up as u8 }
			if !within(down_left, routes_bound) || 0==routes[to_index(down_left)]&DirMask::Right as u8
				{ walls[at] |= DirMask::Down as u8 }
		}
	}
	for y in 0..(TILES+1) {
		for x in 0..(TILES+1) {
			let at = (y as usize, x as usize);
			let up_left = IVec2::new(x as i32-1,y as i32-1);
			let up_right = IVec2::new(x as i32,y as i32-1);

			if !within(up_left, routes_bound) || 0==routes[to_index(up_left)]&DirMask::Down as u8
				{ walls[at] |= DirMask::Left as u8 }
			if !within(up_right, routes_bound) || 0==routes[to_index(up_right)]&DirMask::Down as u8
				{ walls[at] |= DirMask::Right as u8 }
		}
	}

	if add_actors {
		path_max.sort_by_key(|cand| {let (_, x) = cand.unwrap(); Reverse(x) });

		while path_max[2].unwrap().0 == path_max[0].unwrap().0 || path_max[2].unwrap().0 == path_max[1].unwrap().0 {
			path_max[2] = Some((IVec2::new(rng.gen_range(0..TILES) as i32, rng.gen_range(0..TILES) as i32), 0));
		}

		for ord in 0..=2 {
			let (at, _) = path_max[ord].unwrap();
			let actor = match ord { 0 => Actor::Door, 1 => Actor::Player(if rng.gen_range(0..=1) == 0 {Dir::Left} else {Dir::Right}), _ => Actor::Key({
				let route = routes[to_index(at)];
				     if 0==route&DirMask::Left as u8 && 0!=route&DirMask::Right as u8 { false }
				else if 0!=route&DirMask::Left as u8 && 0==route&DirMask::Right as u8 { true }
				else { rng.gen_range(0..=1) != 0 }
			})};

			actors.push((actor, at));
		}
	}

	Room { routes, walls, actors }
}

pub fn room_render(room: &Room, queue: &wgpu::Queue, buffer: &wgpu::Buffer, pos_scale:IVec2, tex_scale:IVec2, actor_draw:bool) -> u64 {
	// Notice y,x order
	const OFFSET:i32 = (CANVAS_SIDE as i32 - TILE_SIDE as i32*(TILES as i32 + 1))/2;
	const TILE_SIZE:IVec2 = IVec2::new(TILE_SIDE as i32, TILE_SIDE as i32);

	let (pos_scale, tex_scale) = (pos_scale.as_vec2(), tex_scale.as_vec2());
	let tex_scale_reflect = Vec2::new(-tex_scale.x, tex_scale.y);

	// Make position, make tile
	let mp = |v:IVec2| { make_float(v, pos_scale) };
	let mt = |v:IVec2, reflect:bool| { make_float(v, if reflect { tex_scale_reflect } else { tex_scale }) };

	let mut storage:Vec<u8> = Vec::default(); 

	'grid: for (y,col) in room.walls.axis_iter(Axis(0)).enumerate() {
		for (x,&tile_which) in col.iter().enumerate() {
			let tile_which = tile_which as u32; // Notice y,x order
			let sprite = [
				mp(IVec2::new((x as u32*TILE_SIDE) as i32 + OFFSET, (y as u32*TILE_SIDE) as i32 + OFFSET)),
				mp(TILE_SIZE),
				mt(IVec2::new(((tile_which%TILE_ROW_MAX)*TILE_SIDE) as i32, (TILE_Y_ORIGIN+(tile_which/TILE_ROW_MAX)*TILE_SIDE) as i32), false),
				mt(TILE_SIZE, false)
			];

//			assert!(mem::size_of_val(&sprite) as u64 == SPRITE_SIZE);

			if storage.len() as u64 + SPRITE_SIZE > buffer.size() as u64 { break 'grid }

			let bytes = bytemuck::bytes_of(&sprite);

			storage.extend_from_slice(bytes);
		}
	}

	if actor_draw {
		const ACTOR_SIZE:IVec2 = IVec2::new(ACTOR_SIDE as i32, ACTOR_SIDE as i32);

		'sprite: for actor in &room.actors {
			let (actor, at) = actor;
			let (actor_which, reflect) = match actor {
				Actor::Player(Dir::Right) => (0, false),
				Actor::Player(Dir::Left) => (0, true),
				Actor::Player(Dir::Down) => (1, false),
				Actor::Player(Dir::Up) => (2, false),
				Actor::Door => (3, false),
				Actor::Key(true) => (4, false),
				Actor::Key(false) => (5, false),
				Actor::Shot => (6, false),
				Actor::Ammo => (7, false),
				Actor::Monster(_, n) => (8+*n as u32, false), // FIXME: Must assert MONSTER_Y_ORIGIN == 0, MONSTER_X_ORIGIN == 8*8
			};
			let sprite = [
				mp(IVec2::new(at.x*TILE_SIDE as i32 + OFFSET + 6, at.y*TILE_SIDE as i32 + OFFSET + 6)),
				mp(ACTOR_SIZE),
				mt(IVec2::new(((actor_which + if reflect { 1 } else { 0 })*ACTOR_SIDE) as i32, ACTOR_Y_ORIGIN as i32), false),
				mt(ACTOR_SIZE, reflect)
			];

//			assert!(mem::size_of_val(&sprite) as u64 == SPRITE_SIZE);

			if storage.len() as u64 + SPRITE_SIZE > buffer.size() as u64 { break 'sprite }

			let bytes = bytemuck::bytes_of(&sprite);

			storage.extend_from_slice(bytes);
		}
	}

	let len = storage.len() as u64;
//	println!("FLOOR LEN {} ({})", len, len / SPRITE_SIZE);

	queue.write_buffer(&buffer, 0, &storage);

	len / SPRITE_SIZE // byte count
}
