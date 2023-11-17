pub mod nodes;

use wasm_bindgen::prelude::wasm_bindgen;

pub use self::nodes::transform;
pub use self::nodes::webcam;

// Required for debug node
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
