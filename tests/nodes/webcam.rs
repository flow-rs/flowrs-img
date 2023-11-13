#[cfg(test)]
mod nodes {
    use flowrs::connection::{connect, Edge};
    use flowrs::node::{ChangeObserver, Node, ReceiveError};
    use flowrs_img::webcam::WebcamNode;

    #[test]
    fn should_return_some_frame() -> Result<(), ReceiveError> {
        let change_observer: ChangeObserver = ChangeObserver::new();
        let mut webcam = WebcamNode::new(Some(&change_observer));

        let mock_output = Edge::new();
        connect(webcam.output.clone(), mock_output.clone());
        let _ = webcam.on_update();

        let actual_image = mock_output.next();
        match actual_image {
            Ok(dyn_img) => Ok(assert!(dyn_img.width() > 0)),
            Err(err) => Err(err),
        }
    }
}
