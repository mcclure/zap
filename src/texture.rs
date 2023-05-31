// Basic routines for working with textured quads

use crate::constants::*;
use seq_macro;
use image::{GenericImage, GrayImage, ImageBuffer, Luma, imageops::{rotate90_in, rotate180_in, rotate270_in}};
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

// FIXME: Return error
// FIXME: Option<tuple> is slightly space heavier than Option<context>
struct FlexDecoder {
    #[cfg(target_arch = "wasm32")]
    canvas: Option<(web_sys::OffscreenCanvasRenderingContext2d, width, height)>
}

impl FlexDecoder {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            FlexDecoder {}
        }
        #[cfg(target_arch = "wasm32")]
        {
            FlexDecoder {canvas: None}
        } 
    }

    pub fn with_capacity(width:u32, height:u32) -> Self {
        let mut it = Self::new();
        it.try_reserve(width, height).unwrap();
        it
    }

    pub fn try_reserve(&mut self, _width:u32, _height:u32) -> Result<(), ()> {
        #[cfg(target_arch = "wasm32")]
        {
            let reset = if let Some((_, width, height)) = self.canvas.as_ref() {
                _width > width || _height > height
            } else { true };
            if reset {
                let canvas = web_sys::OffscreenCanvas::new(_width, _height).unwrap();
                let mut attributes = web_sys::ContextAttributes2d::new();
                attributes.will_read_frequently(true);

                let context:web_sys::OffscreenCanvasRenderingContext2d =
                    canvas.get_context_with_context_options("2d", &attributes)
                        .unwrap().unwrap().dyn_into().unwrap();

                self.canvas = Some((context, width, height))
            }
        }
        Ok(())
    }

    pub async fn load_from_memory(&mut self, buffer: &[u8]) -> image::ImageResult<image::DynamicImage> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            image::load_from_memory(buffer)
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;

            // Setup
            let window = web_sys::window().expect("no global `window` exists");
            let document = window.document().expect("should have a document on window");
            let body = document.body().expect("document should have a body");

            // Make object
            let bytes = js_sys::Array::new();
            bytes.push(&js_sys::Uint8Array::from(buffer));

            let blob = web_sys::Blob::new_with_u8_array_sequence(&bytes).unwrap();

            let bitmap:web_sys::ImageBitmap =
                wasm_bindgen_futures::JsFuture::from(
                    window.create_image_bitmap_with_blob(&blob).unwrap()
                ).await.unwrap().dyn_into().unwrap();

            let (width, height) = (bitmap.width(), bitmap.height());
            self.try_reserve(width, height).unwrap();
            let (context, _, _) = self.canvas.as_ref().unwrap(); // Unless try_reserve fails, we have Some

            context.draw_image_with_image_bitmap(&bitmap, 0., 0.);
            let data = context.get_image_data(0., 0., width as f64, height as f64).unwrap().data().0;

            Ok(image::DynamicImage::ImageRgba8(image::ImageBuffer::from_raw(width, height, data).unwrap()))
        }
    }
}

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

pub fn _make_sampler(device: &wgpu::Device) -> wgpu::Sampler { // Currently unused
    device.create_sampler(&wgpu::SamplerDescriptor::default())
}

pub async fn load_sprite_atlas() -> GrayImage {
    let mut decoder = FlexDecoder::with_capacity(LARGEST_PNG_SIDE, LARGEST_PNG_SIDE);

    seq_macro::seq! { N in 0..8 {
        const ACTOR: [&[u8]; 8] = [
            #(
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/sprite_zap", stringify!(N), ".png")),
            )*
        ];
    }};
    seq_macro::seq! { N in 0..5 {
        const TILE: [&[u8]; 5] = [
            #(
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/sprite_walls", stringify!(N), ".png")),
            )*
        ];

        let tile_img:[GrayImage;5] = [
            #(
                decoder.load_from_memory(TILE[N]).await.unwrap().to_luma8(),
            )*
        ];
    }};

    let mut canvas = ImageBuffer::from_pixel(128, 32, Luma([0xFFu8])); //GrayImage::new(64, 32);

    for idx in 0..8 {
        let img = decoder.load_from_memory(ACTOR[idx]).await.unwrap().to_luma8();
        canvas.copy_from(&img, (idx as u32)*ACTOR_SIDE, ACTOR_Y_ORIGIN).unwrap();
    }

    let mut rng = rand::thread_rng();
    for y in 0..6 {
        for x8 in 0..MONSTER_COUNT {
            for col in 1..4 {
                if col != 0 && col < 4 {
                    let value = Luma([rng.gen_range(0..=255) as u8]);
                    let x = MONSTER_X_ORIGIN+x8*8;
                    let y = MONSTER_Y_ORIGIN+1+y;
                    canvas.put_pixel(x+col, y, value);
                    canvas.put_pixel(x+7-col, y, value);
                }
            }
        }
    }

    {
        let mut temp = GrayImage::new(TILE_SIDE, TILE_SIDE);
        for idx in WallRot::Right as usize..WallRot::Count as usize {
            let sem = WALL_ROT_SEMANTICS[idx];
            let img:&GrayImage;
            let target = &tile_img[sem[0] as usize];
            let rot = sem[1];
            if rot == 0 {
                img = &target;
            } else {
                match rot {
                    1 => rotate90_in(target, &mut temp),
                    2 => rotate180_in(target, &mut temp),
                    3 => rotate270_in(target, &mut temp),
                    _ => unreachable!()
                }.unwrap();
                img = &temp;
            }
            let idx32 = idx as u32;
            canvas.copy_from(img, (idx32%TILE_ROW_MAX)*TILE_SIDE, TILE_Y_ORIGIN+(idx32/TILE_ROW_MAX)*TILE_SIDE).unwrap();
        }
    }

    //canvas.save("sprite_atlas_debug.png").unwrap(); // Debug

    canvas
}
