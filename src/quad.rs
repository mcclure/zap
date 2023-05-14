// Basic routines for working with textured quads

use std::mem;
use wgpu::util::DeviceExt;

use crate::constants::*;

const SQUARE_VERTEX : [f32;8] = [
    0., 0.,
    1., 0.,
    0., 1.,
    1., 1.
];

const SQUARE_INDEX : [u16;6] = [0, 1, 2, 1, 3, 2];

const SQUARE_LAYOUT : wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
    array_stride: (mem::size_of::<f32>()*2) as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        },
    ],
};

pub fn make_quad_root_buffer(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, wgpu::VertexBufferLayout) {
	let root_vertex = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Unified vertex buffer"),
        contents: bytemuck::cast_slice(&SQUARE_VERTEX),
        usage: wgpu::BufferUsages::VERTEX, // Immutable
    });

    let root_index = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Unified index buffer"),
        contents: bytemuck::cast_slice(&SQUARE_INDEX),
        usage: wgpu::BufferUsages::INDEX, // Immutable
    });

    (root_vertex, root_index, SQUARE_LAYOUT)
}

// Makes some assumptions about usage
pub fn make_quad_instance_buffer(device:&wgpu::Device, tag:&str) -> wgpu::Buffer {
    let instance = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("Instance buffer {}", tag)),
        size: (SPRITE_SIZE*(SPRITES_MAX as usize)) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::MAP_WRITE,
        mapped_at_creation:true
    });

    instance
}
