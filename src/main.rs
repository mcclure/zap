// Entry point

mod constants;
mod quad;
mod room;
mod texture;

use std::borrow::Cow;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use glam::{IVec2};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch="wasm32")]
use winit::platform::web::WindowExtWebSys;

use crate::constants::*;
use crate::quad::*;
use crate::room::*;
use crate::texture::*;

// Silently fails if texture is bigger than 2^31 on either axis. Whatever
fn extent_xy_to_ivec(v:wgpu::Extent3d) -> IVec2 {
    IVec2::new(v.width as i32, v.height as i32)
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let init_size = window.inner_size();

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) };

    // If window create failed on web, assume webgpu versioning is the cause.
    #[cfg(target_arch="wasm32")]
    if surface.is_err() {
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| Some(
                doc.body()
                    .and_then(|body| {
                        let div = doc.create_element("p").unwrap();
                        div.set_class_name("alert");
                        div.append_child(&doc.create_text_node("This app requires WebGPU. Either your browser does not support WebGPU, or you must enable an experimental flag to access it.")).unwrap();
                        body.replace_child(
                            &div,
                            &web_sys::Element::from(window.canvas()))
                            .ok()
                    })
                    .expect("couldn't append canvas to document body")
            ));
        return
    }

    let surface = surface.unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Build scene
    let (sprite_atlas, sprite_atlas_view) = make_texture(&device, &queue, load_sprite_atlas(), "sprite");

    let (root_vertex_buffer, root_index_buffer, root_vertex_layout) = make_quad_root_buffer(&device);

    let (instance_buffer, instance_layout) = make_quad_instance_buffer(&device, "0"); // Returns mapped

    // Write scene
    let instance_buffer_count = room_push_fill_random(
        &queue,
        &instance_buffer,
        IVec2::new(CANVAS_SIDE as i32, CANVAS_SIDE as i32),
        extent_xy_to_ivec(sprite_atlas.size())
    );

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("single bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false }, /* FIXME: Is nearest a filter? */
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("single pipeline"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_quad",
            buffers: &[root_vertex_layout, instance_layout],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_quad_direct",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            ..wgpu::PrimitiveState::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: init_size.width,
        height: init_size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    // Get viewport within window
                    // FIXME: Better to let IVec handle this?
                    let (offset, size) =  {
                        let size = IVec2::new(config.width.try_into().unwrap(),
                                              config.height.try_into().unwrap());
                        let diff = size.y - size.x;
                        if diff == 0 {
                            (IVec2::ZERO, size)
                        } else if diff < 0 {
                            (IVec2::new(-diff/2, 0), IVec2::new(size.y, size.y))
                        } else {
                            (IVec2::new(0, diff/2), IVec2::new(size.x, size.x))
                        }
                    };

                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&sprite_atlas_view),
                            }
                        ],
                        layout: &bind_group_layout,
                        label: Some("frame bind group"),
                    });

                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    rpass.set_viewport(offset.x as f32, offset.y as f32, size.x as f32, size.y as f32, 0., 1.);
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_vertex_buffer(0, root_vertex_buffer.slice(..));
                    rpass.set_vertex_buffer(1, instance_buffer.slice(..));
                    rpass.set_index_buffer(root_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.draw_indexed(0..6, 0, 0..(instance_buffer_count as u32));
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
