use std::fmt::Debug;

use flowrs::{
    connection::{Output, Input},
    node::{ChangeObserver, ReadyError, Node, UpdateError},
};
use flowrs::RuntimeConnectable;
use nokhwa::{utils::{CameraIndex, RequestedFormat, RequestedFormatType}, pixel_format::RgbFormat, Camera};
use serde::{Deserialize, Serialize};

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct WebcamNode<I>
where
    I: Clone,
{
    #[input]
    pub input: Input<I>,
    #[output]
    pub output: Output<u32>,
}

impl<I> WebcamNode<I>
where
    I: Clone,
{
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            input: Input::new(),
            output: Output::new(change_observer),
        }
    }
}

impl<I> Node for WebcamNode<I>
where
    I: Clone + Debug + Send,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        // first camera in system
        let index = CameraIndex::Index(0); 
        // request the absolute highest resolution CameraFormat that can be decoded to RGB.
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        // make the camera
        let mut camera = Camera::new(index, requested).unwrap();

        // get a frame
        let frame = camera.frame().unwrap();
        println!("Captured Single Frame of {}", frame.buffer().len());
        // decode into an ImageBuffer
        let decoded = frame.decode_image::<RgbFormat>().unwrap();
        println!("Decoded Frame of {}", decoded.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        // first camera in system
        let index = CameraIndex::Index(0); 
        // request the absolute highest resolution CameraFormat that can be decoded to RGB.
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        // make the camera
        let mut camera = Camera::new(index, requested).unwrap();

        let _ = camera.open_stream();

        // get a frame
        let frame = camera.frame().unwrap();
        println!("Captured Single Frame of {}", frame.buffer().len());
        // decode into an ImageBuffer
        let decoded = frame.decode_image::<RgbFormat>().unwrap();
        println!("Decoded Frame of {}", decoded.len());

        decoded.save("test.png").unwrap();

        assert!(decoded.len() > 0);
    }


}
