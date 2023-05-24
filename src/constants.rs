// Constants used in all files

use std::mem;
use int_enum::IntEnum;
use glam::IVec2;

// Graphics

pub const ACTOR_SIDE:u32 = 8; // Corresponds to sprite_zap
pub const TILE_SIDE:u32 = 10;  // Corresponds to sprite_walls
pub const TILE_ROW_MAX:u32 = 8;
pub const ACTOR_Y_ORIGIN:u32 = 0;
pub const MONSTER_X_ORIGIN:u32 = 64;
pub const MONSTER_Y_ORIGIN:u32 = 0;
pub const MONSTER_COUNT:u32 = 8;
pub const TILE_Y_ORIGIN:u32 = 8;

pub const CANVAS_SIDE:u32 = 128;

pub const SPRITE_SIZE:u64 = 8*mem::size_of::<f32>() as u64;
pub const SPRITES_MAX:u64 = 512; // 13*13*2 = 338, round up for room for bullets. (Realistically, 256 would be enough)

// Walls

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum Dir {
	Right = 0,
	Down = 1,
	Left = 2,
	Up = 3
}

pub const DIR_COMPASS:[IVec2;4] = [IVec2::new(1,0), IVec2::new(0,1), IVec2::new(-1,0) , IVec2::new(0,-1)];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum DirMask {
	Right = 1,
	Down = 2,
	Left = 4,
	Up = 8
}

// From lowest to highest signficance, walls go RIGHT DOWN LEFT UP
// L0 is right+down; T0 is right+down+left; each rotates90deg
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum WallRot {
	Blank     = 0b0000,
	Right     = 0b0001,
	Down      = 0b0010,
	L0        = 0b0011,
	Left      = 0b0100,
	LeftRight = 0b0101,
	L1        = 0b0110,
	T0        = 0b0111,
	Up        = 0b1000,
	L3        = 0b1001,
	UpDown    = 0b1010,
	T3        = 0b1011,
	L2        = 0b1100,
	T2        = 0b1101,
	T1        = 0b1110,
	Full      = 0b1111,
    Count = 16
}

// Map WallRot to [sprite_walls%d.png, rotations]
pub const WALL_ROT_SEMANTICS:[[u8;2];WallRot::Count as usize] = [
	[0, 0], // Blank [[ FALSE ; DON'T USE ]]
	[4, 0], // Right
	[4, 1], // Down
	[2, 0], // L0
	[4, 2], // Left
	[1, 0], // LeftRight
	[2, 1], // L1
	[3, 0], // T0
	[4, 3], // Up
	[2, 3], // L3
	[1, 1], // UpDown
	[3, 3], // T3
	[2, 2], // L2
	[3, 2], // T2
	[3, 1], // T1
	[0, 0], // Full
];

// Actors

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Actor {
	Player(Dir),
	Door,
	Key(bool), // True for rightward
	Ammo,
	Shot,
	Monster(Dir, u8) // Direction, sprite
}

