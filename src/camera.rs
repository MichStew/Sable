use winit::keyboard::KeyCode;
#[rustfmt::skip]
pub const OPENGL_TO_WGPU : cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0), // this w coordinate should shift perspective RTR
);

// my camera struct will hold the basic needs for the camera, the controller will handle movements
// the camera is really only useful for calling look at and building the view
// it has to be translated to a uniform camera for use by the render pipeline
pub struct Camera {
  eye: cgmath::Point3<f32>,
  target: cgmath::Point3<f32>,
  up: cgmath::Vector3<f32>,
  aspect: f32,
  fovy: f32, 
  znear: f32, 
  zfar: f32,
}

// lets just have all the information all at once
impl Camera {
    pub fn new( height: f32, width: f32) -> Self {
        Self {
            eye:(0.4, 0.4, -0.2).into(), // look from where? 
            target: (0.0,0.0,0.0).into(), // default: look at the origin
            up: cgmath::Vector3::unit_y(), // y axis is up
            aspect: height as f32/ width as f32, // aspect ratio
            fovy: 45.0, // consider a 2mp with a 45deg fov
            znear: 0.1,
            zfar: 100.0,
    }
}

    // return the world
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
    // orients the camera to look at the target from the position of the eye, with up defining the
    // vertical position
    
      let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
      let projection = cgmath::perspective(cgmath::Deg(self.fovy),self.aspect, self.znear, self.zfar);
    
      // normalize the cgmath matrix to OpenGL bounds
      return OPENGL_TO_WGPU * projection * view; 
    }

}

// we will create a camera uniform - a uniform means that this data will be available to to every
// shader invocation...is this like a global variable? idk
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]// allows buffer allocation of x
pub struct CameraUniform {
    view_pos: [f32;4],
    view_proj: [[f32;4];4] // 4 4 element matrices within one matrix.
    }

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self { 
            view_pos: [0.0;4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
    
    // here we calculate some values and then organize them in a way that the shaders can use them
    //
    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_pos = camera.eye.to_homogeneous().into();
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
// like a pause menu


// this will only handle operations on an existing camera struct
pub struct CameraController {
    speed: f32,
    ahead_pressed: bool,
    back_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,
    x_rotating: bool,
    y_rotating: bool,
    }

impl CameraController {
    pub fn new (magnitude: f32) -> Self {
        Self {
            speed: magnitude,
            ahead_pressed: false, 
            back_pressed: false, 
            left_pressed: false,
            right_pressed: false,
            x_rotating: false,
            y_rotating: false,
        }
    }

    pub fn rotate(&self, dy: f32 , dx: f32) {
        //TODO
    }
// state.handle_key(event_loop, code, key_state.ispressed())
// so this will intake the key and perform matrix ops based on the input
//
    pub fn handle_key(&self, key: KeyCode, pressed: bool) {
        if pressed { match key {
            KeyCode::KeyW => {println!("w was pressed");},
            KeyCode::KeyA => {println!("A was pressed");},
            KeyCode::KeyS => {println!("s was pressed");},
            KeyCode::KeyD => {println!("d was pressed");} ,
            _ => {println!("no key has been pressed - error");}
        }}

    }

    // wanting to have fps mouse input so 
    pub fn handle_mouse() {}

    // do some operation on the camera when an input is received
    pub fn update_camera(&self, camera: &mut Camera) {
        println!("I am gonna update so hard");
        /*use cgmath::InnerSpace;
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
        } */
    }
}
