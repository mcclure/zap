use std::mem;
use wgpu::util::DeviceExt;

const SQUARE_VERTEX : [f32;8] = [
	0., 0.,
	0., 1.,
	1., 0.,
	1., 1.
];

const SQUARE_INDEX : [u16;6] = [0, 1, 2, 3, 1, 2];

const SQUARE_LAYOUT : wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
    array_stride: (mem::size_of::<f32>()*2) as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 4 * 4,
            shader_location: 1,
        },
    ],
};

pub fn make_quad_root_buffer(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, wgpu::VertexBufferLayout) {
	let root_vertex = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Unified vertex buffer"),
        contents: bytemuck::cast_slice(&SQUARE_VERTEX),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let root_index = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Unified index buffer"),
        contents: bytemuck::cast_slice(&SQUARE_INDEX),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
    });

    (root_vertex, root_index, SQUARE_LAYOUT)
}
