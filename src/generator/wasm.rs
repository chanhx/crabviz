use {
    super::GraphGenerator,
    crate::lsp_types::{
        CallHierarchyIncomingCall, CallHierarchyOutgoingCall, DocumentSymbol, Location, Position,
    },
    std::cell::RefCell,
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: String);
}

#[wasm_bindgen(js_name = GraphGenerator)]
pub struct GraphGeneratorWasm {
    inner: RefCell<GraphGenerator>,
}

#[wasm_bindgen(js_class = GraphGenerator)]
impl GraphGeneratorWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(root: String) -> Self {
        Self {
            inner: RefCell::new(GraphGenerator::new(root)),
        }
    }

    pub fn add_file(&self, file_path: String, symbols: JsValue) {
        let symbols = serde_wasm_bindgen::from_value::<Vec<DocumentSymbol>>(symbols).unwrap();

        self.inner.borrow_mut().add_file(file_path, symbols);
    }

    pub fn add_incoming_calls(&self, file_path: String, position: JsValue, calls: JsValue) {
        let position = serde_wasm_bindgen::from_value::<Position>(position).unwrap();
        let calls =
            serde_wasm_bindgen::from_value::<Vec<CallHierarchyIncomingCall>>(calls).unwrap();

        self.inner
            .borrow_mut()
            .add_incoming_calls(file_path, position, calls);
    }

    pub fn add_outgoing_calls(&self, file_path: String, position: JsValue, calls: JsValue) {
        let position = serde_wasm_bindgen::from_value::<Position>(position).unwrap();
        let calls =
            serde_wasm_bindgen::from_value::<Vec<CallHierarchyOutgoingCall>>(calls).unwrap();

        self.inner
            .borrow_mut()
            .add_outgoing_calls(file_path, position, calls);
    }

    pub fn add_interface_implementations(
        &self,
        file_path: String,
        position: JsValue,
        locations: JsValue,
    ) {
        let position = serde_wasm_bindgen::from_value::<Position>(position).unwrap();
        let locations = serde_wasm_bindgen::from_value::<Vec<Location>>(locations).unwrap();

        self.inner
            .borrow_mut()
            .add_interface_implementations(file_path, position, locations);
    }

    pub fn highlight(&self, file_path: String, position: JsValue) {
        let position = serde_wasm_bindgen::from_value::<Position>(position).unwrap();

        self.inner.borrow_mut().highlight(file_path, position);
    }

    pub fn generate_dot_source(&self) -> String {
        self.inner.borrow().generate_dot_source()
    }
}
