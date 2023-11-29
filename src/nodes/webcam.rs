use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

static NTHREADS: i32 = 2;

use flowrs::RuntimeConnectable;
use flowrs::{
    connection::Output,
    node::{ChangeObserver, Node, UpdateError},
};
use futures_executor::block_on;
use image::DynamicImage;
use nokhwa::{
    js_camera::{JSCamera, JSCameraConstraintsBuilder},
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::thread::spawn;
use wasm_bindgen_futures::spawn_local;

fn debug_info(inp: &str) {
    #[cfg(target_arch = "wasm32")]
    crate::log(inp);
    #[cfg(not(target_arch = "wasm32"))]
    println!("{}", inp)
}

#[derive(RuntimeConnectable)]
pub struct WebcamNode {
    #[cfg(not(target_arch = "wasm32"))]
    camera: Option<Camera>,
    #[cfg(target_arch = "wasm32")]
    camera: Option<JSCamera>,
    #[output]
    pub output: Output<DynamicImage>,
}

impl WebcamNode {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            camera: None,
            output: Output::new(change_observer),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn init_webcam(&mut self) -> anyhow::Result<(), UpdateError> {
        let index = CameraIndex::Index(0);
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let camera = Camera::new(index, requested);

        match camera {
            Ok(mut cam) => {
                let _ = cam.open_stream();
                let test_frame = cam.frame();
                match test_frame {
                    Err(err) => Err(UpdateError::Other(err.into())),
                    Ok(_) => {
                        self.camera = Some(cam);
                        Ok(())
                    }
                }
            }
            Err(err) => Err(UpdateError::Other(err.into())),
        }
    }
    #[cfg(target_arch = "wasm32")]
    fn init_webcam(&mut self) -> anyhow::Result<(), UpdateError> {
        use std::sync::mpsc::TryRecvError;

        let camera_contraints = JSCameraConstraintsBuilder::new().build();
        let mut cam_new: Option<JSCamera> = None;

        let (tx, rx) = mpsc::channel();
        let mut children = Vec::new();

        spawn_local(async move {
            debug_info("In async");
            let cam = JSCamera::new(camera_contraints).await;
            tx.send(cam);
            debug_info("After new");
        });

        debug_info("Waiting for camera");
        loop {
            match rx.try_recv() {
                Ok(cam_res) => {
                    self.camera = Some(cam_res.map_err(|e| UpdateError::Other(e.into()))?);
                    break;
                }
                Err(TryRecvError::Empty) => {
                    continue;
                }
                Err(TryRecvError::Disconnected) => unreachable!(),
            }
        }

        debug_info("Out of async");

        match self.camera.unwrap().frame() {
            Ok(_) => Ok(()),
            Err(err) => Err(UpdateError::Other(err.into())),
        }
    }
}

impl Node for WebcamNode {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        let no_cam_err = "No camera available";
        debug_info("updating...");

        match self.camera {
            None => self.init_webcam()?,
            Some(_) => todo!(),
        }

        let cam = match self.camera {
            Some(ref mut camera) => camera,
            None => {
                return Err(UpdateError::SendError {
                    message: no_cam_err.to_string(),
                })
            }
        };

        let frame_buffer = match cam.frame() {
            Ok(frame) => frame,
            Err(err) => return Err(UpdateError::Other(err.into())),
        };

        #[cfg(not(target_arch = "wasm32"))]
        let decoded_frame = match frame_buffer.decode_image::<RgbFormat>() {
            Ok(frame) => frame,
            Err(err) => return Err(UpdateError::Other(err.into())),
        };

        #[cfg(target_arch = "wasm32")]
        let decoded_frame = frame_buffer;

        let dyn_img = DynamicImage::ImageRgb8(decoded_frame);

        match self.output.clone().send(dyn_img) {
            Ok(_) => Ok(()),
            Err(err) => Err(UpdateError::Other(err.into())),
        }
    }
}
