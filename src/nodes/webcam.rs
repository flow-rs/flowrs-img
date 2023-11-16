use core::result::Result::Ok;
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
        let camera_contraints = JSCameraConstraintsBuilder::new().build();
        let camera = block_on(async move {JSCamera::new(camera_contraints).await});
        match camera {
            Ok(mut cam) => {
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
            _ => {}
        }

        match self.camera {
            None => Err(UpdateError::SendError {
                message: no_cam_err.to_string(),
            }),
            Some(ref mut cam) => {
                let r_frame = cam.frame();
                match r_frame {
                    Err(err) => Err(UpdateError::Other(err.into())),
                    Ok(frame) => {
                        #[cfg(not(target_arch = "wasm32"))]
                        let r_decode = frame.decode_image::<RgbFormat>();
                        #[cfg(target_arch = "wasm32")]
                        let r_decode = frame;
                        match r_decode {
                            Err(err) => Err(UpdateError::Other(err.into())),
                            Ok(decoded) => {
                                let dyn_img = DynamicImage::ImageRgb8(decoded);
                                self.output
                                    .clone()
                                    .send(dyn_img)
                                    .map_err(|e| UpdateError::Other(e.into()))?;
                                Ok(())
                            }
                        }
                    }
                }
            }
        }
    }
}
