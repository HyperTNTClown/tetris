use crate::components::{BufferUpdate, Drawable, Locked, Tetr, Updated};
use bevy::ecs::system::Resource;
use bevy::prelude::{Commands, EventReader, Has, NonSendMut, Query, Res, ResMut};
use bevy::tasks::block_on;
use bevy::time::{Fixed, Time};
use bevy::window::{RequestRedraw, WindowResized};
use std::thread;
use std::time::{Duration, Instant};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferBindingType, BufferUsages, include_wgsl, SamplerBindingType, ShaderStages, TextureDimension};
use wgsl_preprocessor::ShaderBuilder;
use winit::dpi::LogicalSize;
use winit::window::Window;

// TODO: Think about maybe using two Drawable-buffers which can be switched so when writing to one the other (cached one) can be used
// and updated once we stop writing to the primary one... We wanna increase our FPS
// https://www.youtube.com/watch?v=YNFaOnhaaso

// TODO: More passes for deffered rendering???
// Should be easier to do normals etc, but not sure if do vertex shader etc... Or rather compute ones
// Acerola has a video about Lethal Company's rendering
// https://en.wikipedia.org/wiki/Ray_marching#Deferred_shading (Pretty interesting stuff)
// https://en.wikipedia.org/wiki/Deferred_shading
// https://webgpu.github.io/webgpu-samples/samples/deferredRendering (sample in Typescript, but should be pretty easy to translate)

// Raymarching Volumetrics???

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0],
    },
];

const SCALE: f32 = 1f32 / 4f32;

pub struct Renderer {
    apply_render_pipeline: wgpu::RenderPipeline,
    clear_color: wgpu::Color,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    drawables: Drawables,
    drawables_buffer: wgpu::Buffer,
    drawables_buffer_bind_group: wgpu::BindGroup,
    num_vertices: u32,
    queue: wgpu::Queue,
    render_texture: wgpu::Texture,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    texture_bind_group: wgpu::BindGroup,
    texture_render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniforms_buffer: wgpu::Buffer,
    uniforms_buffer_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    window: &'static Window,
}

impl Renderer {
    pub(crate) fn new(window: &Window) -> Renderer {
        // Pointer hack to be able to get a constant reference to the window...
        // I'm not sure if this is the best way to do this, buuut it works.
        // TODO: Could try to instead use an Arc<Window>, might be safer...
        let window = window as *const Window;
        let window = unsafe { &*window };

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None,
        ))
        .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            // .filter(|f| f.is_srgb())
            // .next()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        //let shader = device.create_shader_module(ShaderBuilder::new("main.wgsl").unwrap().build());
        let shader = device.create_shader_module(include_wgsl!("main_combined.wgsl"));

        let uniforms = Uniforms::default();

        let uniforms_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: uniforms.as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniforms_buffer_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        });

        let uniforms_buffer_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &uniforms_buffer_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
        });

        let drawables = Drawables::default();

        let drawables_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Drawables Buffer"),
            contents: drawables.as_bytes(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let drawables_buffer_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Drawables Buffer Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        });

        let drawables_buffer_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Drawables Buffer Bind Group"),
            layout: &drawables_buffer_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: drawables_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniforms_buffer_layout, &drawables_buffer_layout],
                push_constant_ranges: &[],
            });

        let texture_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = VERTICES.len() as u32;

        let clear_color = wgpu::Color::BLACK;

        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width: (config.width as f32 * SCALE) as u32,
                height: (config.height as f32 * SCALE) as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &render_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        //let shader =
        //    device.create_shader_module(ShaderBuilder::new("apply_texture.wgsl").unwrap().build());

        let shader = device.create_shader_module(include_wgsl!("apply_texture.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniforms_buffer_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let apply_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Texture Apply Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            vertex_buffer,
            texture_render_pipeline,
            apply_render_pipeline,
            num_vertices,
            clear_color,
            uniforms,
            uniforms_buffer,
            uniforms_buffer_bind_group,
            drawables,
            drawables_buffer,
            drawables_buffer_bind_group,
            render_texture,
            texture_bind_group,
            window,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::LogicalSize<f32>) {
        let physical = new_size.to_physical(self.window.scale_factor());
        if physical.width > 0 && physical.height > 0 {
            self.size = physical;
            self.config.width = physical.width;
            self.config.height = physical.height;
            self.surface.configure(&self.device, &self.config);
            self.uniforms.window_size = [physical.width as f32, physical.height as f32];
            self.uniforms.window_scale = self.window.scale_factor() as f32;
            self.window.request_redraw()
        }
    }
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let view = self
            .render_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Texture Render Pass (getting rendered at half resolution)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.texture_render_pipeline);
            render_pass.set_bind_group(0, &self.uniforms_buffer_bind_group, &[]);
            render_pass.set_bind_group(1, &self.drawables_buffer_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        // Apply Texture to surface_view
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Apply Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.apply_render_pipeline);
            render_pass.set_bind_group(0, &self.uniforms_buffer_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            //let rect = (
            //    ((self.uniforms.window_size[0] - (self.uniforms.window_size[0] * self.uniforms.scale)) / 2.0) as u32,
            //    ((self.uniforms.window_size[1] - (self.uniforms.window_size[1] * self.uniforms.scale)) / 2.0) as u32,
            //    self.uniforms.window_size[0].min(self.uniforms.window_size[0] * self.uniforms.scale) as u32,
            //    self.uniforms.window_size[1].min(self.uniforms.window_size[1] * self.uniforms.scale) as u32,
            //);
            //render_pass.set_scissor_rect(rect.0, rect.1, rect.2, rect.3);
            render_pass.draw(0..6, 0..1);
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}

unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Uniforms {
    pub mouse: [f32; 2],
    pub time: f32,
    pub pad: f32,
    pub window_size: [f32; 2],
    pub scale: f32,
    pub window_scale: f32,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            mouse: [0.0, 0.0],
            time: 0.0,
            pad: 0.0,
            window_size: [0.0, 0.0],
            scale: SCALE,
            window_scale: 1.0,
        }
    }
}

impl Uniforms {
    fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

unsafe impl bytemuck::Zeroable for Uniforms {}

unsafe impl bytemuck::Pod for Uniforms {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Drawables([Drawable; 256]);

impl Drawables {
    fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(&self.0)
    }
}

impl Default for Drawables {
    fn default() -> Self {
        Self([Default::default(); 256])
    }
}

unsafe impl bytemuck::Zeroable for Drawables {}

unsafe impl bytemuck::Pod for Drawables {}

pub fn render(
    mut renderer: NonSendMut<Renderer>,
    time: Res<Time<Fixed>>,
    mut tetrs: Query<(&Tetr, &mut Updated, Has<Locked>)>,
    mut buffer_update: ResMut<BufferUpdate>,
    mut _commands: Commands,
) {
    static mut FRAME_COUNT: u32 = 0;
    static mut LAST_TIME: f32 = 0.0;

    let start_time = Instant::now();

    unsafe {
        FRAME_COUNT += 1;
        let elapsed = time.elapsed_seconds_wrapped();
        if elapsed - LAST_TIME >= 1.0 {
            println!("FPS: {}", FRAME_COUNT);
            FRAME_COUNT = 0;
            LAST_TIME = elapsed;
        }
    }

    buffer_update.0 = true; // TODO: remove this and fix the buffer update logic. This is just to get it working not - performance isn't a concern right now

    let vec = match buffer_update.0 {
        false => tetrs
            .iter()
            .filter(|e| e.1.0)
            .map(|e| e.0)
            .collect::<Vec<&Tetr>>(),
        true => tetrs
            .iter()
            .map(|e| e.0)
            .collect::<Vec<&Tetr>>()
    };


    let e = vec
        .iter()
        .map(|e| e.as_drawables())
        .flatten()
        .filter(|e| e.shape != 0)
        .map(|d| d.as_bytes().to_vec())
        .flatten()
        .collect::<Vec<u8>>();

    if !e.is_empty() {
        match buffer_update.0 {
            false => {
                let offset = tetrs.iter().filter(|e| e.2).map(|e| e.0.offset()).sum::<u64>();

                renderer
                    .queue
                    .write_buffer(&renderer.drawables_buffer, offset as u64, e.as_slice());
            },
            true => {
                // fill e with 0s until size of buffer is reached to overwrite old data
                let mut e = e;
                e.resize(renderer.drawables_buffer.size() as usize, 0);
                renderer
                    .queue
                    .write_buffer(&renderer.drawables_buffer, 0, e.as_slice());
            }
        }
    }

    renderer
        .queue
        .write_buffer(&renderer.uniforms_buffer, 0, renderer.uniforms.as_bytes());
    renderer.render().unwrap();

    let elapsed_time = start_time.elapsed();
    let frame_time = Duration::from_secs_f32(1.0 / 120.0);
    if elapsed_time < frame_time {
        thread::sleep(frame_time - elapsed_time);
    }

    for mut e in tetrs.iter_mut() {
        e.1 .0 = false;
    }
    buffer_update.0 = false;
}

pub fn render_events(
    mut renderer: NonSendMut<Renderer>,
    mut redraw: EventReader<RequestRedraw>,
    mut resize: EventReader<WindowResized>,
    instant: Res<Time<Fixed>>,
) {
    redraw.read().for_each(|_| {
        println!("redraw");
        renderer
            .queue
            .write_buffer(&renderer.uniforms_buffer, 0, renderer.uniforms.as_bytes());
        renderer.render().unwrap();
    });
    resize.read().for_each(|event| {
        let size = LogicalSize::new(event.width, event.height * 2f32);
        renderer.resize(size);
    });

    renderer.uniforms.time = instant.elapsed_seconds_wrapped();
}
