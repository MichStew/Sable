use std::sync::Arc; 
use winit::{
    application::ApplicationHandler, 
    event::*, 
    event_loop::{ActiveEventLoop, EventLoop}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::Window,
    };

// stop giving errors. from me...to compiler...with love
fn main() {run();}

pub fn run() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app);
    Ok(()) // return the expected tuple assuming all goes well
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device, 
    queue: wgpu::Queue, 
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
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
        fn user_event(&mut self, event_loop: &ActiveEventLoop, mut event: State) {
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
    pub  fn new(_event_loop: &EventLoop<State>) -> Self {
        Self {
            state: None,
        }
    }
}

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


        let modes = &surface_caps.present_modes;

        Ok(Self {surface, device, queue, config, is_surface_configured: false, window, })
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
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    
    Ok(())
    
    }}



