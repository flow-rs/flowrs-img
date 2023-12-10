#[cfg(test)]
mod flow {

    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_img::transform::ImageToArray3Node;
    use flowrs_img::webcam::WebcamNode;
    use flowrs_std::binary::ToBinaryNode;
    use flowrs_std::debug::DebugNode;
    use image::DynamicImage;
    use ndarray::Array3;
    use serial_test::serial;

    use crate::nodes::webcam;

    #[test]
    #[serial]
    fn transport_of_image_to_array() -> Result<(), anyhow::Error> {
        println!("Webcam_on_update_test:");

        let change_observer: ChangeObserver = ChangeObserver::new();

        let mut node_webcam = WebcamNode::<i32>::new(Some(&change_observer));
        let mut node_to_array = ImageToArray3Node::new(Some(&change_observer));
        let mut node_debug = DebugNode::new(Some(&change_observer));

        let mock_output = Edge::new();

        println!("Connecting:");

        connect(node_webcam.output.clone(), node_to_array.input.clone());
        connect(node_to_array.output.clone(), node_debug.input.clone());
        connect(node_debug.output.clone(), mock_output.clone());

        println!("On udpate:");

        let _ = node_webcam.on_init()?;

        let _ = node_webcam.input.send(1);

        let _ = node_webcam.on_update()?;
        let _ = node_to_array.on_update()?;
        let _ = node_debug.on_update()?;

        let _ = node_webcam.on_shutdown()?;

        let _: Array3<f32> = mock_output.next()?.into();

        Ok(())
    }
}
