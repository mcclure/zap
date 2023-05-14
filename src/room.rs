// Data structure for a world map tile

use crate::constants::*;

use rand::Rng;
use std::mem;
use glam::{IVec2, Vec2};
use divrem::DivCeil;

fn make_float(v:IVec2, offset:i32, scale:Vec2) -> [f32;2] {
	(
		Vec2::new((v.x - offset) as f32, (v.y - offset) as f32)
		/ scale
	).to_array()
}

pub fn room_push_fill_random(queue: &wgpu::Queue, buffer: &wgpu::Buffer, pos_scale:IVec2, tex_scale:IVec2) -> u64 {
	let tiles:u32 = CANVAS_SIDE.div_ceil(TILE_SIDE);
	const OFFSET:i32 = (TILE_SIDE as i32 - (CANVAS_SIDE%TILE_SIDE) as i32)/2;
	const TILE_SIZE:IVec2 = IVec2::new(TILE_SIDE as i32, TILE_SIDE as i32);
	let mut rng = rand::thread_rng();

	let (pos_scale, tex_scale) = (pos_scale.as_vec2(), tex_scale.as_vec2());

	// Make position, make tile
	let mp = |v:IVec2| { make_float(v, OFFSET, pos_scale) };
	let mt = |v:IVec2| { make_float(v, 0, tex_scale) };

	let mut storage:Vec<u8> = Vec::default(); 

	'grid: for y in 0..tiles {
		for x in 0..tiles {
			let tile_which = rng.gen_range(0..MONSTER_COUNT);
			let sprite = [
				mp(IVec2::new((x*TILE_SIDE) as i32, (y*TILE_SIDE) as i32)),
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
