use anyhow::Error;
use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, InitError, Node, ShutdownError, UpdateError},
};

use image::DynamicImage;

#[cfg(not(target_arch = "wasm32"))]
use opencv::{
    core::Mat,
    imgproc::*,
    prelude::*,
    videoio::{VideoCapture, CAP_ANY, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH},
};
use serde::{Deserialize, Serialize};

/// This struct is cloneable, can be serialized and deserialized, and
/// contains configurations for the camera.
///
/// # Attributes
///
/// * `device_index` - An i32 that indicates the index of the device.
/// * `frame_width` - An u32 that indicates the captured frame width.
/// * `frame_height` - An u32 that indicates the captured frame height.
///
/// # Remarks
///
/// This is derived from both `Serialize` and `Deserialize` to allow
/// transformation to/from String format
///
/// # Examples
///
/// ```
/// use flowrs_img::webcam::WebcamNodeConfig;
/// let config = WebcamNodeConfig { device_index: 0 };
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Deserialize, Serialize)]
pub struct WebcamNodeConfig {
    pub device_index: i32,
    pub frame_width: u32,
    pub frame_height: u32,
}

/// `WebcamNode<T>` struct defines configuration for webcam node with generic
/// parameter `T`.
///
/// This struct is used to manage and control the webcam node. It contains
/// runtime-connectable attribute, an optional `VideoCapture` object,
/// `WebcamNodeConfig` instance, and output and input object which hold DynamicImage
/// and generic type `T` respectively. `T` is used to type the input of the node.
/// An image is only returned if an item is in the input
///
/// # Derivation
///
/// This struct derives `RuntimeConnectable` trait for dynamic input and output
/// connections at runtime.
///
/// # Type Parameters
///
/// * `T` - Clone trait bound. This enables WebcamNode to hold any data
///         type that implements the Clone trait.
///
/// # Attributes
///
/// * `camera` - Optional `VideoCapture` object represents the camera device.
/// * `config` - A `WebcamNodeConfig` object used for configuring the webcam node.
/// * `output` - The output attribute `Output<DynamicImage>`, expected to be a node output
///              end points capable of transferring `DynamicImage` data type.
/// * `input`  - The input attribute `Input<T>`, expected to be a node input end points
///              capable of accepting `T` data type.
///
/// # Examples
///
/// ```
/// use flowrs::node::ChangeObserver;
/// use flowrs_img::webcam::WebcamNodeConfig;
/// use flowrs_img::webcam::WebcamNode;
///
/// let config = WebcamNodeConfig { device_index: 0 };
/// let co = ChangeObserver::new();
/// let observer = Some(&co);
///
/// let node: WebcamNode<i32> = WebcamNode::new(config, observer);
/// ```
#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
impl<T> WebcamNode<T>
where
    T: Clone,
{
    /// Struct initialization function for `WebcamNode`.
    ///
    /// This method creates a new instance of `WebcamNode`,
    /// it takes two inputs which include a `WebcamNodeConfig` object and
    /// an optional reference, '&ChangeObserver'.
    /// The ChangeObserver is used for receiving tokens in the input.
    ///
    /// # Parameters
    ///
    /// * `value` - A WebcamNodeConfig object taken ownership by the function.
    /// * `change_observer` - An Optional reference to a ChangeObserver.
    ///
    /// # Return
    ///
    /// It returns a `WebcamNode` instance with the following attributes initialized:
    /// * `camera`: This is None initially because the camera hasn't started yet.
    /// * `output`: This is a new output node which references the ChangeObserver if it is provided.
    /// * `input`: A new uninitialized `Input` instance.
    /// * `config`: Cloning of the passed `value` parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use flowrs::node::ChangeObserver;
    /// use flowrs_img::webcam::WebcamNodeConfig;
    /// use flowrs_img::webcam::WebcamNode;
    ///
    /// let config = WebcamNodeConfig { device_index: 0 };
    /// let co = ChangeObserver::new();
    /// let observer = Some(&co);
    /// let node: WebcamNode<i32> = WebcamNode::new(config, observer);
    /// ```
    pub fn new(value: WebcamNodeConfig, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            camera: None,
            output: Output::new(change_observer),
            input: Input::new(),
            config: value.clone(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> Node for WebcamNode<T>
where
    T: Clone + Send,
{
    /// The `on_init` function is used to initialize a webcam node. It attempts to create a new`VideoCapture`
    /// instance that represents a webcam device with the specified index in `self.config.device_index`, and
    /// verify if the webcam device is successfully opened. The function may fail if unable to create the
    /// `VideoCapture` instance or the webcam device can't be opened, which results in an `InitError`.
    ///
    /// If the function executes successfully, the `self.camera` field of the `WebcamNode` is assigned the
    /// `VideoCapture` instance.
    ///
    /// # Parameters
    ///
    /// No input parameters.
    ///
    /// # Return
    ///
    /// * `Ok(())`: Successfully initialized the object and the camera is opened properly.
    /// * `Err(InitError::Other(e))`: An error occurred during initialization. It can be due to:
    ///    * The camera could not be opened.
    ///    * Any other issues while initializing the VideoCapture object or checking its status.
    ///
    /// # Errors
    ///
    /// The function can produce an error of type `InitError` due to either of these scenarios:
    /// * VideoCapture initialization fails.
    /// * The camera represented by VideoCapture instance isn't opened successfully.
    ///
    /// In the case of these errors, an `InitError::Other` is returned with a detailed description of the issue wrapped in an `Error`.
    ///
    /// # Panics
    ///
    /// This function does not explicitly panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use flowrs::node::ChangeObserver;
    /// use flowrs::node::Node;
    /// use flowrs_img::webcam::WebcamNodeConfig;
    /// use flowrs_img::webcam::WebcamNode;
    ///
    /// let config = WebcamNodeConfig { device_index: 0, frame_width:640, frame_height:480 };
    /// let co = ChangeObserver::new();
    /// let observer = Some(&co);
    /// let mut node: WebcamNode<i32> = WebcamNode::new(config, observer);
    ///
    /// match node.on_init() {
    ///     Ok(_) => println!("WebcamNode has been successfully initialized"),
    ///     Err(e) => println!("An error occurred when trying to initialize the WebcamNode: {}", e),
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// This function doesn't have any specific safety considerations as it doesn't involve `unsafe` operations.
    ///
    /// Please note that you need to make sure the webcam device specified by the index in `self.config.device_index` is available and can be opened.
    fn on_init(&mut self) -> Result<(), InitError> {
        let mut camera = VideoCapture::new(self.config.device_index, CAP_ANY)
            .map_err(|e| InitError::Other(e.into()))?;
        let opened = VideoCapture::is_opened(&camera).map_err(|e| InitError::Other(e.into()))?;

        if !opened {
            return Err(InitError::Other(Error::msg("Camera could not be opened!")));
        }

        camera
            .set(CAP_PROP_FRAME_WIDTH, self.config.frame_width as f64)
            .map_err(|e| InitError::Other(e.into()))?;
        camera
            .set(CAP_PROP_FRAME_HEIGHT, self.config.frame_height as f64)
            .map_err(|e| InitError::Other(e.into()))?;
        self.camera = Some(camera);
        Ok(())
    }

    /// `on_update` checks for new inputs and updates the camera frame accordingly.
    /// Then it converts the new frame into RGB format and sends this processed image
    /// output as a `DynamicImage`.
    ///
    /// # Errors
    ///
    /// * `Err(UpdateError::Other(Error::msg("There is no cam to update!")))` is returned when the camera is not available or not set up correctly.
    /// * `Err(UpdateError::Other(Error::msg("Could not read a new frame")))` is returned when the camera fails to read a new frame.
    /// * `Err(UpdateError::Other(err.into()))` is returned when the output fails to send updated frame.
    ///
    /// # Safety
    ///
    /// The method uses an `unsafe` block to convert a raw pointer to a slice. The safety
    /// of this operation is guaranteed by the fact that correct size is used when slicing from raw parts,
    /// which is `(width * height * 3)`, and it is ensured that the slice will not outlive the data it points to.
    /// However, ensure careful use of this method, as it involves `unsafe` operations.
    ///
    /// # Example
    ///
    /// ```
    /// use flowrs::node::Node;
    /// use flowrs::node::ChangeObserver;
    /// use flowrs_img::webcam::WebcamNodeConfig;
    /// use flowrs_img::webcam::WebcamNode;
    ///
    /// let config = WebcamNodeConfig { device_index: 0, frame_width: 640, frame_height: 480 };
    /// let co = ChangeObserver::new();
    /// let observer = Some(&co);
    /// let mut node: WebcamNode<i32> = WebcamNode::new(config, observer);
    /// match node.on_init() {
    ///     Ok(_) => println!("WebcamNode has been successfully initialized"),
    ///     Err(e) => println!("An error occurred when trying to initialize the WebcamNode: {}", e),
    /// }
    ///
    /// //send someting into the input
    /// match node.on_update() {
    ///     Ok(_) => println!("WebcamNode has been successfully updated"),
    ///     Err(e) => println!("An error occurred when trying to update the WebcamNode: {}", e),
    /// }
    /// ```
    ///
    /// Before calling `on_update`, ensure that a valid camera and other necessary fields are properly initialized.
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Err(_) = self.input.next() {
            return Ok(());
        }

        let cam = match self.camera.as_mut() {
            None => return Err(UpdateError::Other(Error::msg("There is no cam to update!"))),
            Some(cam) => cam,
        };

        let mut frame = Mat::default();
        match cam.read(&mut frame) {
            Ok(true) => {}
            Ok(false) => return Err(UpdateError::Other(Error::msg("Could not read a new frame"))),
            Err(e) => return Err(UpdateError::Other(e.into())),
        };

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

    /// Releases all resource held by the node.
    ///
    /// # Returns
    ///
    /// * `Ok(())`: If the camera is successfully shut down.
    /// * `Err(ShutdownError::Other(Error::msg("There is no cam to shutdown!")))`: If there's no camera to shut down.
    /// * `Err(ShutdownError::Other(e.into()))`: If an error occurs when trying to release the camera.
    ///
    /// # Errors
    ///
    /// May return `flowrs::node::ShutdownError::Other` if there is no camera to shut down or if
    /// the shutdown process encounters an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use flowrs::node::Node;
    /// use flowrs::node::ChangeObserver;
    /// use flowrs_img::webcam::WebcamNodeConfig;
    /// use flowrs_img::webcam::WebcamNode;
    ///
    /// let config = WebcamNodeConfig { device_index: 0, frame_width: 640, frame_height: 480 };
    /// let co = ChangeObserver::new();
    /// let observer = Some(&co);
    /// let mut node: WebcamNode<i32> = WebcamNode::new(config, observer);
    ///
    /// match node.on_shutdown() {
    ///     Ok(_) => println!("WebcamNode has been successfully shut down"),
    ///     Err(e) => println!("An error occurred when trying to shut down the WebcamNode: {}", e),
    /// }
    /// ```
    fn on_shutdown(&mut self) -> Result<(), flowrs::node::ShutdownError> {
        match self.camera.as_mut() {
            None => Err(ShutdownError::Other(Error::msg(
                "There is no cam to shutdown!",
            ))),
            Some(cam) => {
                cam.release().map_err(|e| ShutdownError::Other(e.into()))?;
                Ok(())
            }
        }
    }
}
