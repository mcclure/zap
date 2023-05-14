// Constants used in all files

use std::mem;

// Graphics

pub const SPRITE_SIDE:u32 = 8; // Corresponds to sprite_zap
pub const TILE_SIDE:u32 = 10;  // Corresponds to sprite_walls
pub const TILE_COUNT:u32 = 4;
pub const SPRITE_Y_ORIGIN:u32 = 0;
pub const MONSTER_Y_ORIGIN:u32 = 8;
pub const MONSTER_COUNT:u32 = 8;
pub const TILE_Y_ORIGIN:u32 = 16;

pub const CANVAS_SIDE:u32 = 128;

pub const SPRITE_SIZE:u64 = 8*mem::size_of::<f32>() as u64;
pub const SPRITES_MAX:u64 = 512; // 13*13*2 = 338, round up for room for bullets. (Realistically, 256 would be enough)
