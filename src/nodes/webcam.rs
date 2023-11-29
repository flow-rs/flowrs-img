use flowrs::RuntimeConnectable;
use flowrs::{
    connection::Output,
    node::{ChangeObserver, Node, UpdateError},
};

use image::DynamicImage;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
#[derive(RuntimeConnectable)]
pub struct WebcamNode {
    camera: Option<Camera>,
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
}

impl Node for WebcamNode {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        let no_cam_err = "No camera available";

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

        let decoded_frame = match frame_buffer.decode_image::<RgbFormat>() {
            Ok(frame) => frame,
            Err(err) => return Err(UpdateError::Other(err.into())),
        };

        let dyn_img = DynamicImage::ImageRgb8(decoded_frame);

        match self.output.clone().send(dyn_img) {
            Ok(_) => Ok(()),
            Err(err) => Err(UpdateError::Other(err.into())),
        }
    }
}
