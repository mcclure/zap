// Data structure for a world map tile

use crate::constants::*;

//use std::mem;
use glam::{IVec2, Vec2};
use divrem::DivCeil;
use ndarray::{Array2, Axis};
use rand::seq::SliceRandom;

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

pub fn room_push_fill_random(queue: &wgpu::Queue, buffer: &wgpu::Buffer, pos_scale:IVec2, tex_scale:IVec2) -> u64 {
	const TILES:u32 = CANVAS_SIDE/TILE_SIDE - 1;

	// NDArray helpers
	fn to_index(v:IVec2) -> (usize, usize) { (v.y as usize, v.x as usize) }
	fn within (at:IVec2, size:IVec2) -> bool {
		IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
	}

	// Make map
	let routes_bound = IVec2::new(TILES as i32, TILES as i32);
	let mut routes:Array2<u8> = Array2::default(to_index(routes_bound));
	let mut rng = rand::thread_rng();
	{
		// Must randomize indices rather than directions because rotation identity matters
		const COMPASS:[IVec2;4] = [IVec2::new(1,0), IVec2::new(0,1), IVec2::new(-1,0) , IVec2::new(0,-1)];
		const COMPASS_IDX:[usize;4] = [2,1,0,3];
		
		let mut stack = vec![(routes_bound/2, COMPASS_IDX, 0)];
		loop {
			let top = stack.pop();
			if top == None { break }
			let (at, compass_order, compass_order_idx) = top.unwrap();

			if compass_order_idx < 3 {
				stack.push((at, compass_order, compass_order_idx+1));
			}

			let compass_idx = compass_order[compass_order_idx];
			let cand = at + COMPASS[compass_idx];

			if within(cand, routes_bound) {
				let cand_value = routes[to_index(cand)];
				let is_free = cand_value == 0;
				if is_free {
					let mut random_compass = COMPASS_IDX.clone();
					random_compass.shuffle(&mut rng);
					stack.push((cand, random_compass, 0));
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

	// Notice y,x order
	const OFFSET:i32 = (CANVAS_SIDE as i32 - TILE_SIDE as i32*(TILES as i32 + 1))/2;
	const TILE_SIZE:IVec2 = IVec2::new(TILE_SIDE as i32, TILE_SIDE as i32);

	let (pos_scale, tex_scale) = (pos_scale.as_vec2(), tex_scale.as_vec2());

	// Make position, make tile
	let mp = |v:IVec2| { make_float(v, pos_scale) };
	let mt = |v:IVec2| { make_float(v, tex_scale) };

	let mut storage:Vec<u8> = Vec::default(); 

	'grid: for (y,col) in walls.axis_iter(Axis(0)).enumerate() {
		for (x,&tile_mask) in col.iter().enumerate() {
			let tile_which = WALL_ROT_MASK[tile_mask as usize] as u32; // Notice y,x order
			let sprite = [
				mp(IVec2::new((x as u32*TILE_SIDE) as i32 + OFFSET, (y as u32*TILE_SIDE) as i32 + OFFSET)),
				mp(TILE_SIZE),
				mt(IVec2::new((tile_which*TILE_SIDE) as i32, TILE_Y_ORIGIN as i32)),
				mt(TILE_SIZE)
			];

//			assert!(mem::size_of_val(&sprite) as u64 == SPRITE_SIZE);

			if storage.len() as u64 + SPRITE_SIZE > buffer.size() as u64 { break 'grid }

			let bytes = bytemuck::bytes_of(&sprite);

			storage.extend_from_slice(bytes);
		}
	}

	let len = storage.len() as u64;
//	println!("FLOOR LEN {} ({})", len, len / SPRITE_SIZE);

	queue.write_buffer(&buffer, 0, &storage);

	len / SPRITE_SIZE // byte count
}
