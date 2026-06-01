use std::sync::Arc; 
use winit::{
    application::ApplicationHandler, 
    event::*, 
    event_loop::{ActiveEventLoop, EventLoop}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::Window,
    };
use wgpu::util::DeviceExt;
mod texture;
// stop giving errors. from me...to compiler...with love
fn main() {run();}

pub fn run() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app);
    Ok(()) // return the expected tuple assuming all goes well
}



// Changed
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.99240386], }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.56958647], }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.05060294], }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.1526709], }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.7347359], }, // E
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

// vertices have a position and a color 
// instead of using raw rgb we can have coordinates to map texture to vertices
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2], // 2d texture ( on a face )
}

// unsafe impl bytemuck::Pod for Vertex{}
// unsafe impl bytemuck::Zeroable for Vertex{}
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device, 
    queue: wgpu::Queue, 
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer : wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer, 
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
}


// the reason state is an option is that we need to be in resumed state to create a window, maybe
// like a pause menu

pub struct App {
    state: Option<State>,
}


// implement a state trait, this will be state machine? 
impl ApplicationHandler<State> for App {
    // take a reference to ourself, to determine current state
    // then borrow event loop to determine next state

    // what does resumed do?
    // define attribues of the window
    // create the window with these attributes - wrap in Arc for multiple ownership
    // use pollster to await the state of the window
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { 
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes(); 

        
        // this window data will need to be passed to multiple threads
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap()); 
        
        #[cfg(not(target_arch="wasm32"))]
        {
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
            
        }}

        #[allow(unused_mut)]
        fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
            self.state = Some(event);
        }


        fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event:WindowEvent) { 
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
            };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() { 
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
        WindowEvent::KeyboardInput {
            event: 
                KeyEvent {
                    physical_key : PhysicalKey::Code(code),
                    state: key_state,
                    ..
            },
            ..
        } => state.handle_key(event_loop, code, key_state.is_pressed()),
        _ => {}
        }
    }
}

impl App {
    pub fn new(_event_loop: &EventLoop<State>) -> Self {
        Self {
            state: None,
        }
    }
}

impl Vertex {
// array stride sets a width for the buffer
// step mode tells whether this is per vertex or per instance data 
// shader location differentiates vertex and shader data
// format tells the shape of the attribute (max value of float32)
    const ATTRIBS: [wgpu::VertexAttribute; 2] = 
      wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem; 

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }]}}}

impl State { 
    
    fn update(&mut self) {
        // TODO 
    }
    pub async fn new(window : Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size(); 
        // docs say I can just call this method wihtout arguments and get the same struct
        // doing this since my compiler states InstanceDescriptor does not have display field.
        // either my compiler is a liar or the docs are not up to date, either way I must go on...
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
            }
        );

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions { 
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }).await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor{
                label: None,
                required_features : wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            }).await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1, // struct returned from texture.rs 
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let shader = device.create_shader_module( //wgpu::include_wgsl!("shader.wgsl")
                wgpu::ShaderModuleDescriptor{
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                });
        
        // defines sets of bind group layouts that the pipeline can use, these layouts must be
        // defined - see texture_bind_group_layout to see how this happens
        let render_pipeline_layout = 
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[Some(&texture_bind_group_layout)],
                immediate_size: 0,
            });

        // struct docs can be found in wgpu::RenderPipelineDescriptor for v 29.3.0
        // this really just describes the wgsl files I wrote just a bit ago
        // this is why we had those @vertex and @fragment
        // buffers are also handled in those files so we leave them blank here.
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader,
                entry_point: Some("vs_main"),
                buffers:  &[Vertex::desc()], // goes to Vram
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader, 
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            // the primitive describes how I want the program to interpret the vertices 
            // triangle list means every three vertices corresponds to one triangle !note this 4 IO
            // ccw describes which way wgpu will determine the triangle is facing based on the
            // relationship between the points
            //
            // Throwback: triangles that are not facing forward are not rendered 
            // NCOT Technology has a very intriguing video on this topic that applies here.
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None, 
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_vertices = VERTICES.len() as u32;
        let num_indices = INDICES.len() as u32;

        Ok(Self {surface, device, queue, config, is_surface_configured: false, window, 
            render_pipeline, vertex_buffer, num_vertices, index_buffer, num_indices,
            diffuse_bind_group, diffuse_texture,})
    }

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> { 
        // winit only draws one windoww unless requested to draw a new one.
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        // output here has many many states

        let output =  match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture, 
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.surface.configure(&self.device, &self.config);
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout 
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // skip frames that aren't done?? ie. occluded, took too long, or validation(not
                // sure here)
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("lost device...exiting");
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
             
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // encoder is used for a render pass. which has all methods that handle actual drawing 
        // there are some nesting things going on...
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view, 
                        depth_slice: None,
                        resolve_target: None,                         
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {r: 0.1,g: 0.2,b: 0.3,a: 1.0,}),
                            store: wgpu::StoreOp::Store,
                        },
                    }
                )
                ], // closing for color attachments
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None, 
                multiview_mask: None, 
            });

            // set render pass pipeline to the new pipeline within state::new()
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_vertices, 0, 0..1);
        }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    
    Ok(())
    
    }}



