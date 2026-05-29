use std::sync::Arc; 

use winit::{
    application::ApplicationHandler, 
    event::*, 
    event_loop::{ActiveEventLoop, EventLoop}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::Window,
    };

// stop giving errors. from me...to compiler...with love
fn main() {}


pub struct State { 
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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { 
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes(); 

        
        // this window data will need to be passed to multiple threads
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap()); 
        
        #[cfg(not(target_arch="wasm32"))]
        {
            self.state = Some(pollster::block_on(State::new(window).unwrap()));
            
        }}

        #[allow(unused_mut)]
        fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
            self.state = Some(event);
        }
}

impl App {
    pub  fn new(event_loop: &EventLoop<State>) -> Self {
        Self {
            state: None,
        }
    }
}

impl State { 
    pub async fn new( window : Arc<Window>) -> anyhow::Result<Self> {
        Ok(Self {window})
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


