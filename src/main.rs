use std::sync::Arc; 

use winit::{
    application::ApplicationHandler, 
    event::*, 
    event_loop::{ActiveEventLoop, EventLoop}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::Window,
    };

pub struct State { 
    window: Arc<Window>,
}


// the reason state is an option is that we need to be in resumed state to create a window, maybe
// like a pause menu
//
// The proxy resource is used for creating wgpu processes on web, which is async.
// consider removing this since the plan for this game is to run as a process (like regular games)
// not on the web. 


pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>,
}




// implement a state trait, this will be state machine? 
impl ApplicationHandler<State> for App {
    // take a reference to ourself, to determine current state
    // then borrow event loop to determine next state 
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { 
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes(); 

        // this seems to be only for web applications, I will include it for now but likely delete
        // it later when optimizing for readability and scope
        #[cfg(target_arch = "wasm32")]
        {
            use wasm::bindgen::JsCast; 
            use winit::platform::web::WindowAttributesExtWebSys; 

            const CANVAS_ID: &str = "canvas";

            let window = wgpyu::web_sys::window().unwrap_throw(); 
            let document = window.document().unwrap_throw(); 
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw(); 
            let html_canvas_element = canvas.unchecked_into(); 
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }
        // this window data will need to be passed to multiple threads
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap()); 
        
        #[cfg(not(target_arch="wasm32"))]
        {
            self.state = Some(pollster::block_on(State::new(window).unwrap());
            
        }

        #[cfg(target_arch="wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(
                            State::new(window)
                            .await
                            .expect("cant create canvas");
                        }
                        .is_ok())
                        });
                    }
                }
            }

            #[allow(unused_mut)]
            fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
                #[cfg(target_arch = "wasm32")]
                {
                    event.window.request_redraw(); 
                    event.resize(
                        event.window.inner_size().width,
                        event.window.inner_size().height,
                    );
                }
                self.state = Some(event);
            }
}








impl App {
    pub  fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl State { 
    pub async fn new( window : Arc<Window>) -> anyhow::Result<Self> {
        Ok( Self { 
            window,
        })
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {
        // TODO: handle resizing 
        // imagine something like
        // window.x += x; 
        // window.y += y
        // render(); 
    }

    pub fn render(&mut self) { 
        // winit only draws one window unless requested to draw a new one.
        self.window.request_redraw(); 
    }

}
}

