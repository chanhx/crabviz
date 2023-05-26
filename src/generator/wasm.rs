use {
    super::GraphGenerator,
    crate::lsp_types::{CallHierarchyOutgoingCall, DocumentSymbol, Location, Position},
    std::cell::RefCell,
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen(js_name = GraphGenerator)]
pub struct GraphGeneratorWasm {
    inner: RefCell<GraphGenerator>,
}

#[wasm_bindgen(js_class = GraphGenerator)]
impl GraphGeneratorWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(GraphGenerator::new()),
        }
    }

    #[wasm_bindgen(js_name = add_file)]
    pub fn add_file_wasm(&self, file_path: String, symbols: JsValue) {
        let symbols = serde_wasm_bindgen::from_value::<Vec<DocumentSymbol>>(symbols).unwrap();

        self.inner.borrow_mut().add_file(file_path, symbols);
    }

    #[wasm_bindgen(js_name = add_outgoing_calls)]
    pub fn add_outgoing_calls_wasm(&self, file_path: String, position: JsValue, calls: JsValue) {
        let position = serde_wasm_bindgen::from_value::<Position>(position).unwrap();
        let calls =
            serde_wasm_bindgen::from_value::<Vec<CallHierarchyOutgoingCall>>(calls).unwrap();

        self.inner
            .borrow_mut()
            .add_outgoing_calls(file_path, position, calls);
    }

    #[wasm_bindgen(js_name = add_interface_implementations)]
    pub fn add_interface_implementations_wasm(
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

    #[wasm_bindgen(js_name = generate_dot_source)]
    pub fn generate_dot_source_wasm(&self) -> String {
        self.inner.borrow().generate_dot_source()
    }
}
