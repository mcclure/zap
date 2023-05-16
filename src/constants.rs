// Constants used in all files

use std::mem;
use int_enum::IntEnum;

// Graphics

pub const ACTOR_SIDE:u32 = 8; // Corresponds to sprite_zap
pub const TILE_SIDE:u32 = 10;  // Corresponds to sprite_walls
pub const TILE_ROW_MAX:u32 = 6;
pub const ACTOR_Y_ORIGIN:u32 = 0;
pub const MONSTER_X_ORIGIN:u32 = 64;
pub const MONSTER_Y_ORIGIN:u32 = 0;
pub const MONSTER_COUNT:u32 = 8;
pub const TILE_Y_ORIGIN:u32 = 8;

pub const CANVAS_SIDE:u32 = 128;

pub const SPRITE_SIZE:u64 = 8*mem::size_of::<f32>() as u64;
pub const SPRITES_MAX:u64 = 512; // 13*13*2 = 338, round up for room for bullets. (Realistically, 256 would be enough)

// Walls

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum WallRot {
    Full = 0,
    T0 = 1,
    T1 = 2,
    T2 = 3,
    T3 = 4,
    L0 = 5,
    L1 = 6,
    L2 = 7,
    L3 = 8,
    LeftRight = 9,
    UpDown    = 10,
    Blank     = 11,
    Count     = 12
}

// Map WallRot to [sprite_walls%d.png, rotations]
pub const WALL_ROT_SEMANTICS:[[u8;2];WallRot::Blank as usize] = [
	[0, 0],
	[3, 0],
	[3, 1],
	[3, 2],
	[3, 3],
	[2, 0],
	[2, 1],
	[2, 2],
	[2, 3],
	[1, 0],
	[1, 1],
];

// From lowest to highest signficance, walls go RIGHT DOWN LEFT UP
// L0 is right+down; T0 is right+down+left; each rotates90deg
pub const WALL_ROT_MASK:[WallRot;16] = [
	WallRot::Blank,     // 0000
	WallRot::LeftRight, // 0001 -- INVALID
	WallRot::UpDown,    // 0010 -- INVALID
	WallRot::L0,        // 0011
	WallRot::LeftRight, // 0100 -- INVALID
	WallRot::LeftRight, // 0101
	WallRot::L1,        // 0110
	WallRot::T0,        // 0111
	WallRot::UpDown,    // 1000 -- INVALID
	WallRot::L3,        // 1001
	WallRot::UpDown,    // 1010
	WallRot::T3,        // 1011
	WallRot::L2,        // 1100
	WallRot::T2,        // 1101
	WallRot::T1,        // 1110
	WallRot::Full,      // 1111
];
