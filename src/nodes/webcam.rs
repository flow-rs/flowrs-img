use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, InitError, Node, ShutdownError, UpdateError},
};

use image::DynamicImage;
use opencv::{
    core::Mat,
    imgproc::*,
    prelude::*,
    videoio::{VideoCapture, CAP_ANY},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct WebcamNodeConfig {
   pub device_index: i32
}

#[derive(RuntimeConnectable)]
pub struct WebcamNode<T>
where
    T: Clone,
{
    camera: Option<VideoCapture>,
    config: WebcamNodeConfig,

    #[output]
    pub output: Output<DynamicImage>,
    
    #[input]
    pub input: Input<T>,
}

impl<T> WebcamNode<T>
where
    T: Clone,
{
    pub fn new(value: WebcamNodeConfig, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            camera: None,
            output: Output::new(change_observer),
            input: Input::new(),
            config: value.clone()
        }
    }
}

impl<T> Node for WebcamNode<T>
where
    T: Clone + Send,
{
    fn on_init(&mut self) -> Result<(), InitError> {
        let camera = VideoCapture::new(self.config.device_index, CAP_ANY).map_err(|e| InitError::Other(e.into()))?;
        let opened = VideoCapture::is_opened(&camera).map_err(|e| InitError::Other(e.into()))?;

        if !opened {
            return Err(InitError::Other(anyhow::Error::msg(
                "Camera could not be opened!",
            )));
        }

        self.camera = Some(camera);
        Ok(())
    }

    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Err(_) = self.input.next() {
            return Ok(())
        }
        
        if self.camera.is_none() {
            return Err(UpdateError::Other(anyhow::Error::msg(
                "There is no cam to update!",
            )));
        }

        let mut frame = Mat::default();
        self.camera
            .as_mut()
            .unwrap()
            .read(&mut frame)
            .map_err(|e| UpdateError::Other(e.into()))?;

        let mut rgb_mat = Mat::default();
        cvt_color(&frame, &mut rgb_mat, COLOR_BGR2RGB, 0).unwrap();
        let data = rgb_mat.data();
        let width = rgb_mat.cols() as u32;
        let height = rgb_mat.rows() as u32;

        // Convert raw pointer to a slice
        let data_slice = unsafe { std::slice::from_raw_parts(data, (width * height * 3) as usize) };

        // Create a Vec<u8> from the slice
        let data_vec = data_slice.to_vec();

        let dyn_img =
            DynamicImage::ImageRgb8(image::RgbImage::from_raw(width, height, data_vec).unwrap());

        match self.output.clone().send(dyn_img) {
            Ok(_) => Ok(()),
            Err(err) => Err(UpdateError::Other(err.into())),
        }
    }

    fn on_shutdown(&mut self) -> anyhow::Result<(), flowrs::node::ShutdownError> {
        if self.camera.is_none() {
            return Err(ShutdownError::Other(anyhow::Error::msg(
                "There is no cam to shutdown!",
            )));
        }
        self.camera
            .as_mut()
            .unwrap()
            .release()
            .map_err(|e| ShutdownError::Other(e.into()))?;
        Ok(())
    }
}
