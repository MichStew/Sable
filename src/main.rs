use std::sync::Arc; 
use winit::{
    application::ApplicationHandler, 
    event::*, 
    event_loop::{ActiveEventLoop, EventLoop}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::Window,
    };
use wgpu::util::DeviceExt;
use cgmath::prelude::*;
use model::{Vertex,DrawModel};

mod texture;
mod model;
mod resources;
// stop giving errors. from me...to compiler...with love
// const instances for now
const NUM_INSTANCES_PER_ROW : u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5); //z
#[rustfmt::skip]
pub const OPENGL_TO_WGPU : cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0), // this w coordinate should shift perspective RTR
);


fn main() {run();}

pub fn run() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app);
    Ok(()) // return the expected tuple assuming all goes well
}

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

// vertices have a position and a color 
// instead of using raw rgb we can have coordinates to map texture to vertices

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
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    obj_model: model::Model,
}

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32;4];4],
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
	    model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
	}
    }
}

// stride determines the distance the shader will need to go to find the next shading element
// so InstanceRaw is a struct of [[f43;4] ;4]. this should be 64 bytes between Vertex Attributes
// The instance step mode tells the shader to use the next instance when it beigns processing a new
// instance (as opposed to a vertex?)
// the first element is at 0. VertexAttribute has
impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
	wgpu::VertexBufferLayout{
	    array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
	    step_mode: wgpu::VertexStepMode::Instance,
	    attributes: &[
	        wgpu::VertexAttribute {
	            offset: 0,
	            shader_location: 5,
	            format: wgpu::VertexFormat::Float32x4,
	        }, 
	        wgpu::VertexAttribute {
	            offset: mem::size_of::<[f32;4]>() as wgpu::BufferAddress,
                    shader_location: 6,
	            format: wgpu::VertexFormat::Float32x4,
	        },
	        wgpu::VertexAttribute {
	            offset: mem::size_of::<[f32;8]>() as wgpu::BufferAddress,
	            shader_location: 7,
	            format: wgpu::VertexFormat::Float32x4,
	        },
                wgpu::VertexAttribute {
	            offset: mem::size_of::<[f32;12]>() as wgpu::BufferAddress,
	            shader_location: 8,
	            format: wgpu::VertexFormat::Float32x4,
	        },
	],
      }
    }
  }

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32, 
    zfar: f32,
}

impl Camera { 
    // theres some normalization that will have to occur here due to inconsistencies with DirectX
    // and OpenGL, specifically the coordinate definitions 
     fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // view matrix moves the world to be at position and rotation of the camera
         let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // projection allows for depth (orthographic / perspective) 
        let proj = cgmath::perspective(cgmath::Deg(self.fovy),self.aspect, self.znear, self.zfar);

    return OPENGL_TO_WGPU  * proj * view;
    }
}

// we will create a camera uniform - a uniform means that this data will be available to to every
// shader invocation...is this like a global variable? idk
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]// allows buffer allocation of x
struct CameraUniform {
    view_proj: [[f32;4];4] // 4 4 element matrices within one matrix.
    }

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self { view_proj: cgmath::Matrix4::identity().into(),
        }
    }
    
    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

struct CameraController { 
    speed: f32, 
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed:false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            },
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            },
             KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            },
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            },
            _ => false,
        }
    }
    
    fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // some extra logic that is supposed to normalize the camera, this will need to be refined
        // greatly
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }
        let right = forward_norm.cross(camera.up);
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();
        if self.is_right_pressed {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
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

impl State { 
    
    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera); 
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
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

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");


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

        let obj_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
            .await
            .unwrap();

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
	
	//Originally had this instancing up above the buffer layout stuffs but it wasnt rendering anything
	// TODO dynamic instancing
        const SPACE_BETWEEN : f32 = 3.0;
	let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
	    (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32/2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32/2.0);

	        let position = cgmath::Vector3 { x, y:0.0, z};
		let rotation = if position.is_zero() {
		    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
		} else {
		    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
		};

		Instance {position, rotation,}
	    })
	    }).collect::<Vec<_>>();
	
	let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
	let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
	    label: Some("Instance Buffer"),
	    contents: bytemuck::cast_slice(&instance_data),
	    usage: wgpu::BufferUsages::VERTEX,
	    });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
		    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let camera_controller = CameraController::new(0.2);

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
                bind_group_layouts: &[
                    Some(&texture_bind_group_layout),
                    Some(&camera_bind_group_layout),
                ],
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
                entry_point: Some("vs_main"), // Instance does not have enough memory
                buffers:  &[model::ModelVertex::desc(), InstanceRaw::desc()], // goes to Vram
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader, 
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState{
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less), //back to front
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, 
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });


        Ok(Self {
            surface, 
            device, 
            queue, 
            config, 
            is_surface_configured: false, 
            window, 
            render_pipeline,
            diffuse_bind_group, 
            diffuse_texture, 
            camera, 
            camera_uniform, 
            camera_buffer,
            camera_bind_group, 
            camera_controller, 
            instances, 
            instance_buffer, 
            depth_texture,
            obj_model,
        })
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
        event_loop.exit();
        } else {
            self.camera_controller.handle_key(code, is_pressed);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None, 
                multiview_mask: None, 
            });

            // set render pass pipeline to the new pipeline within state::new()
           

	    render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);
            use model::DrawModel;
            let mesh = &self.obj_model.meshes[0];
            let material = &self.obj_model.materials[mesh.material];
            render_pass.draw_model_instanced(&self.obj_model, 0..self.instances.len()as u32, &self.camera_bind_group);
        }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    
    Ok(())
    
    }}



