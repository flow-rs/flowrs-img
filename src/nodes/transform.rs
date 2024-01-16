use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};

use anyhow::anyhow;
use image::{io::Reader as ImageReader, DynamicImage, ImageBuffer, Pixel};
use ndarray::{Array3, ArrayBase, Dim, OwnedRepr};
use nshare::ToNdarray3;
use std::io::Cursor;

use serde::{Deserialize, Serialize};

extern crate alloc;

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct DecodeImageNode {
    #[output]
    pub output: Output<DynamicImage>,

    #[input]
    pub input: Input<Vec<u8>>,
}

impl DecodeImageNode {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new(),
        }
    }
}

impl Node for DecodeImageNode {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(data) = self.input.next() {
            let img = ImageReader::new(Cursor::new(data))
                .with_guessed_format()
                .map_err(|e| UpdateError::Other(e.into()))?
                .decode()
                .map_err(|e| UpdateError::Other(e.into()))?;

            self.output
                .send(img)
                .map_err(|e| UpdateError::Other(e.into()))?;
        }
        Ok(())
    }
}

// TODO:    - EncodeImageNode, Array3ToImage,
//          - How to replace DynamicImage with something like ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct ImageToArray3Node<T> {
    #[output]
    pub output: Output<Array3<T>>,

    #[input]
    pub input: Input<DynamicImage>,
}

impl<T> ImageToArray3Node<T>
where
    T: Send + Sync,
{
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new(),
        }
    }

    fn handle_image<U: Pixel + 'static>(
        &mut self,
        img: ImageBuffer<U, Vec<U::Subpixel>>,
    ) -> Result<(), UpdateError>
    where
        T: From<U::Subpixel>,
    {
        let a: ArrayBase<OwnedRepr<T>, Dim<[usize; 3]>> = img.into_ndarray3().mapv(|x| x.into());

        self.output
            .send(a)
            .map_err(|e| UpdateError::Other(e.into()))?;
        Ok(())
    }
}

impl<T> Node for ImageToArray3Node<T>
where
    T: Send + Sync + From<u8> + From<u16> + From<f32>,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(data) = self.input.next() {
            match data {
                DynamicImage::ImageLuma8(img) => self.handle_image(img),
                DynamicImage::ImageLumaA8(img) => self.handle_image(img),
                DynamicImage::ImageRgb8(img) => self.handle_image(img),
                DynamicImage::ImageRgba8(img) => self.handle_image(img),
                DynamicImage::ImageLuma16(img) => self.handle_image(img),
                DynamicImage::ImageLumaA16(img) => self.handle_image(img),
                DynamicImage::ImageRgb16(img) => self.handle_image(img),
                DynamicImage::ImageRgba16(img) => self.handle_image(img),
                DynamicImage::ImageRgb32F(img) => self.handle_image(img),
                DynamicImage::ImageRgba32F(img) => self.handle_image(img),
                _ => Err(UpdateError::Other(anyhow!("Image type not supported."))),
            }?
        }
        Ok(())
    }
}
