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

pub fn room_push_fill_random(view: &mut wgpu::BufferViewMut, scale:IVec2) -> u64 {
	const offset:i32 = (TILE_SIDE as i32 - (CANVAS_SIDE%TILE_SIDE) as i32)/2;
	const tile_size:IVec2 = IVec2::new(TILE_SIDE as i32, TILE_SIDE as i32);
	let mut rng = rand::thread_rng();

	let mut write_to:usize = 0; // If view is bigger than size_of(ptr) we'll fail earlier anyway?!

	let scale = scale.as_vec2();

	let mf = |v:IVec2| { make_float(v, offset, scale) };

	'grid: for y in 0..CANVAS_SIDE {
		for x in 0..CANVAS_SIDE {
			let tile_which = rng.gen_range(0..MONSTER_COUNT);
			let sprite = [
				mf(IVec2::new((x*TILE_SIDE) as i32, (y*TILE_SIDE) as i32)),
				mf(tile_size),
				mf(IVec2::new((tile_which*TILE_SIDE) as i32, TILE_Y_ORIGIN as i32)),
				mf(tile_size)
			];

			let sprite_size = mem::size_of_val(&sprite);
			println!("{}", sprite_size);

			let write_to_next = write_to + sprite_size;

			if write_to_next > view.len() { break 'grid }

			let bytes = bytemuck::bytes_of(&sprite);

			view[write_to..write_to_next].copy_from_slice(bytes);

			write_to = write_to_next;
		}
	}

	write_to as u64 // byte count
}