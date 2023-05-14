// Data structure for a world map tile

use crate::constants::*;

use rand::Rng;
use std::mem;
use glam::{IVec2, Vec2};

fn make_float(v:IVec2, offset:i32, scale:Vec2) -> [f32;2] {
	(
		Vec2::new((v.x - offset) as f32, (v.y - offset) as f32)
		/ scale
	).to_array()
}

pub fn room_push_fill_random(queue: &wgpu::Queue, buffer: &wgpu::Buffer, scale:IVec2) -> u64 {
	const OFFSET:i32 = (TILE_SIDE as i32 - (CANVAS_SIDE%TILE_SIDE) as i32)/2;
	const TILE_SIZE:IVec2 = IVec2::new(TILE_SIDE as i32, TILE_SIDE as i32);
	let mut rng = rand::thread_rng();

	let mut write_to:u64 = 0;

	let scale = scale.as_vec2();

	let mf = |v:IVec2| { make_float(v, OFFSET, scale) };

	let mut storage:Vec<u8> = Vec::default(); 

	'grid: for y in 0..CANVAS_SIDE {
		for x in 0..CANVAS_SIDE {
			let tile_which = rng.gen_range(0..MONSTER_COUNT);
			let sprite = [
				mf(IVec2::new((x*TILE_SIDE) as i32, (y*TILE_SIDE) as i32)),
				mf(TILE_SIZE),
				mf(IVec2::new((tile_which*TILE_SIDE) as i32, TILE_Y_ORIGIN as i32)),
				mf(TILE_SIZE)
			];

			let sprite_size = mem::size_of_val(&sprite) as u64;
//			assert!(sprite_size == 32);

			let write_to_next = write_to + sprite_size;

			if write_to_next > buffer.size() as u64 { break 'grid }

			let bytes = bytemuck::bytes_of(&sprite);

			storage.extend_from_slice(bytes);

			write_to = write_to_next;
		}
	}

	queue.write_buffer(&buffer, 0, &storage);

	write_to // byte count
}
