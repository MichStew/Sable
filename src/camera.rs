use winit::keyboard::KeyCode;
use std::time::Duration;
use cgmath::*;
use std::f32::consts::FRAC_PI_2 as SAFE_PI;
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
  pub position: Point3<f32>,
  yaw: Rad<f32>,
  pitch: Rad<f32>,
}

// lets just have all the information all at once
impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>> 
        (position: V, yaw: Y, pitch: P) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
    }
}

    // return the world
    pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        // destructure elements for use 
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos(); 

        Matrix4::look_to_rh(
            self.position, // from position
            Vector3::new(
                cos_pitch * cos_yaw, // Some(point) defined on a sphere 
                sin_pitch, 
                cos_pitch * sin_yaw
            ).normalize(),
            Vector3::unit_y(), // up
        )
    }

}

pub struct Projection {
    aspect: f32, 
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection { 
    pub fn new<F: Into<Rad<f32>>>(
        width: u32, 
        height: u32, 
        fovy: F, 
        znear: f32, 
        zfar: f32,
    ) -> Self { 
        Self {
            aspect : width as f32 / height as f32, 
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}


// we will create a camera uniform - a uniform means that this data will be available to to every
// shader invocation...is this like a global variable? idk
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]// allows buffer allocation of x
pub struct CameraUniform {
    view_pos: [f32;4],
    view: [[f32;4];4], // 4 4 element matrices within one matrix.
    view_proj: [[f32;4];4],
    inv_proj: [[f32;4];4],
    inv_view: [[f32;4];4],
    }

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self { 
            view_pos: [0.0;4],
            view_proj: cgmath::Matrix4::identity().into(),
            view: cgmath::Matrix4::identity().into(),
            inv_proj: cgmath::Matrix4::identity().into(),
            inv_view: cgmath::Matrix4::identity().into(),
        }
    }
    
    pub fn update_view_projection(&mut self, camera: &Camera, projection: &Projection) {
        self.view_pos = camera.position.to_homogeneous().into();
        let proj = projection.calc_matrix();
        let view = camera.calc_matrix();
        let view_proj = proj * view;
        self.view_proj = view_proj.into();
        self.view = view.into();
        self.inv_proj = proj.invert().unwrap().into();
        self.inv_view = view.transpose().into();

        //println!(" the view projection has been updated to {:?}", self.view_proj);
    }
}
// like a pause menu


// this will only handle operations on an existing camera struct
pub struct CameraController {
    amount_left: f32, 
    amount_right : f32, 
    amount_forward: f32, 
    amount_backward: f32, 
    amount_up: f32, 
    amount_down: f32,
    rotate_horizontal: f32, 
    rotate_vertical: f32, 
    scroll: f32, 
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
        amount_left: 0.0, 
        amount_right : 0.0, 
        amount_forward: 0.0, 
        amount_backward: 0.0, 
        amount_up: 0.0, 
        amount_down: 0.0,
        rotate_horizontal: 0.0, 
        rotate_vertical: 0.0, 
        scroll: 0.0, 
        speed,
        sensitivity,
    }           
}

    pub fn handle_mouse(&mut self, delta: (f64, f64)) -> bool {
        // I want to return a bool to track if the mouse is moving for debug stuff
        // also just useful to have. 
        // so winit will handle the recognition of Some(deviceEvent) 
        // it will call impl handleMouse in state which will come here
        // here we will do some operations on the camera based on the mouse movement

        // here I imagine we will only change the target based on the normalized values of dx, dy

        //println!("here I will do some ops on the camera");
        match (delta.0, delta.1) {
            (0.0,0.0) => {false},
            _ => {
                //println!("dx is {}, and dy is {}", delta.0, delta.1);
                self.rotate_horizontal = delta.0 as f32;
                self.rotate_vertical = delta.1 as f32;
                true
            }
        }
    }

    // this returns a bool so we can keep track of whether or not we are pressed in state
    pub fn handle_key(&mut self, key: KeyCode, pressed: bool) -> bool {
        let mut amount = 0.0;
        if pressed {amount = 1.0;} else {amount = 0.0;}
            match key {
            KeyCode::KeyW => {
            self.amount_forward = amount;
            //println!("{}", self.amount_forward);
            true
            },
            KeyCode::KeyA => {
            self.amount_left = amount;
            true
            },
            KeyCode::KeyS => {
             self.amount_backward = amount;
             true
            },
            KeyCode::KeyD => {
             self.amount_right = amount;
             true
            },
            KeyCode::Space => {
             self.amount_up = amount;
             true
            },
            KeyCode::ShiftLeft => {
             self.amount_down = amount;
             true
            },
            _ => { false }
            }
        }

    // do some operation on the camera when an input is received
    // speed in setup in main
    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration){ 
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        // sorta build a movement matrix from the input and apply that to position 
        // instead of a bunch of if statements that move one axis at once
        camera.position += forward  * (self.amount_forward - self.amount_backward) * self.speed; 
        camera.position += right * (self.amount_right - self.amount_left) * self.speed; 
        
        // stuff for zooming, not sure I care for now
        let(pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity;
        
        self.scroll = 0.0;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed; 
        
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity; 
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity; 

        //println!("{:?}", camera.position);
        // stop idle rotation
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
        /* 
        self.amount_forward = 0.0;
        self.amount_backward = 0.0;
        self.amount_left = 0.0;
        self.amount_right = 0.0;
        self.amount_up = 0.0;
        self.amount_down = 0.0;
        */
        // stop weird behavior from rotational limitation
        if camera.pitch < -Rad(SAFE_PI) { 
            camera.pitch = -Rad(SAFE_PI);
        } else if camera.pitch > Rad(SAFE_PI) {
            camera.pitch = Rad(SAFE_PI);
        }

}}

