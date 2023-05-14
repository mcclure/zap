// Data structure for a world map tile

use crate::constants::*;

use rand::Rng;
//use std::mem;
use glam::{IVec2, Vec2};
use divrem::DivCeil;
use ndarray::Array2;
use rand::seq::SliceRandom;

fn make_float(v:IVec2, scale:Vec2) -> [f32;2] {
	(
		Vec2::new(v.x as f32, v.y as f32)
		/ scale
	).to_array()
}

pub fn room_push_fill_random(queue: &wgpu::Queue, buffer: &wgpu::Buffer, pos_scale:IVec2, tex_scale:IVec2) -> u64 {
	let tiles:u32 = CANVAS_SIDE.div_ceil(TILE_SIDE);

	// Make map
	fn to_index(v:IVec2) -> (usize, usize) { (v.y as usize, v.x as usize) }
	let routes_bound = IVec2::new(tiles as i32, tiles as i32);
	let mut routes:Array2<u8> = Array2::default(to_index(routes_bound));
	let mut rng = rand::thread_rng();
	{
		fn within (at:IVec2, size:IVec2) -> bool {
			IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
		}

		const COMPASS:[IVec2;4] = [IVec2::new(1,0), IVec2::new(0,1), IVec2::new(-1,0), IVec2::new(0,-1)];
		
		let mut next = vec![routes_bound/2];
		while next.len() > 0 {
			let mut current = std::mem::take(&mut next);
			current.shuffle(&mut rng);

			for at in current {
				let mut at_value = 0;
				let mut random_compass = COMPASS.clone();
				random_compass.shuffle(&mut rng);

				for (idx,&dir) in random_compass.iter().enumerate() {
					let cand = at + dir;
					if within(cand, routes_bound) {
						let cand_value = routes[to_index(cand)];
						let is_free = cand_value == 0;
						if is_free {
							next.push(cand);
						}
						if is_free || 0 != cand_value & 1<<((idx+2)%4) {
							at_value |= 1<<idx;
						}
					}
				}
				routes[to_index(at)] = at_value;
			}
		}
	}

	const OFFSET:i32 = (TILE_SIDE as i32 - (CANVAS_SIDE%TILE_SIDE) as i32)/2;
	const TILE_SIZE:IVec2 = IVec2::new(TILE_SIDE as i32, TILE_SIDE as i32);

	let (pos_scale, tex_scale) = (pos_scale.as_vec2(), tex_scale.as_vec2());

	// Make position, make tile
	let mp = |v:IVec2| { make_float(v, pos_scale) };
	let mt = |v:IVec2| { make_float(v, tex_scale) };

	let mut storage:Vec<u8> = Vec::default(); 

	'grid: for y in 0..tiles {
		for x in 0..tiles {
			let tile_which = WALL_ROT_MASK[routes[(x as usize,y as usize)] as usize] as u32;
			let sprite = [
				mp(IVec2::new((x*TILE_SIDE) as i32 - OFFSET, (y*TILE_SIDE) as i32 - OFFSET)),
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
