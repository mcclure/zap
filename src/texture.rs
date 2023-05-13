use crate::constants::*;
use seq_macro;
use image::{GenericImage, GrayImage, ImageBuffer, Luma};
use rand::Rng;

const STANDARD_TEXTURE_DESCRIPTOR:wgpu::TextureDescriptor = wgpu::TextureDescriptor {
    size: wgpu::Extent3d {width:1,height:1,depth_or_array_layers:1},
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: wgpu::TextureFormat::R8Unorm,
    usage: wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::COPY_DST),
    label: None,
    view_formats: &[],
};

pub fn make_texture(device: &wgpu::Device, queue: &wgpu::Queue, img:GrayImage, label:&str) -> (wgpu::Texture, wgpu::TextureView) {
    let size = wgpu::Extent3d {width:img.width(), height:img.height(), ..STANDARD_TEXTURE_DESCRIPTOR.size};
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        view_formats: &[],
        size: size,
        ..STANDARD_TEXTURE_DESCRIPTOR
    });

    queue.write_texture(
        texture.as_image_copy(),
        &img,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(size.width),
            rows_per_image: Some(size.height), // Unnecessary
        },
        size, // TODO size from image
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    (texture, view)
}

pub fn make_sampler(device: &wgpu::Device) -> wgpu::Sampler { // Expected only called once
    device.create_sampler(&wgpu::SamplerDescriptor::default())
}

pub fn load_sprite_atlas() -> GrayImage {
    seq_macro::seq! { N in 0..8 {
        const SPRITE: [&[u8]; 8] = [
            #(
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/sprite_zap", stringify!(N), ".png")),
            )*
        ];
    }};
    seq_macro::seq! { N in 0..4 {
        const TILE: [&[u8]; 4] = [
            #(
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/sprite_walls", stringify!(N), ".png")),
            )*
        ];
    }};

    let mut canvas = ImageBuffer::from_pixel(64, 32, Luma([0xFFu8])); //GrayImage::new(64, 32);

    for idx in 0..8 {
        let img = image::load_from_memory(SPRITE[idx]).unwrap().to_luma8();
        canvas.copy_from(&img, (idx as u32)*SPRITE_SIDE, SPRITE_Y_ORIGIN).unwrap();
    }

    let mut rng = rand::thread_rng();
    for y in 0..6 {
        for x8 in 0..8 {
            for col in 1..4 {
                if col != 0 && col < 4 {
                    let value = Luma([rng.gen_range(0..=255) as u8]);
                    let y = MONSTER_Y_ORIGIN+1+y;
                    canvas.put_pixel(x8*8+col, y, value);
                    canvas.put_pixel(x8*8+7-col, y, value);
                }
            }
        }
    }

    for idx in 0..4 {
        let img = image::load_from_memory(TILE[idx]).unwrap().to_luma8();
        canvas.copy_from(&img, (idx as u32)*TILE_SIDE, TILE_Y_ORIGIN).unwrap();
    }

    //canvas.save("sprite_atlas_debug.png").unwrap(); // Debug

    canvas
}
