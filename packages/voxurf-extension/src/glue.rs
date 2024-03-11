use wasm_bindgen::prelude::*;

// TODO Error handling with `reject(..)` calls  in JS
#[wasm_bindgen(module = "/src/glue.js")]
extern "C" {
    /// Gets the raw accessibility tree from the Chrome debugger API.
    pub async fn get_raw_ax_tree(tab_id: u32) -> JsValue;
    /// Gets the active tab's ID.
    pub async fn get_tab_id() -> JsValue;
    /// Clicks the element with the given CSS selector.
    pub async fn click_element(tab_id: u32, selector: &str);
    /// Fills the element with the given CSS selector with the given text.
    pub async fn fill_element(tab_id: u32, selector: &str, text: &str);
    /// Converts the given backend DOM node ID to a CSS selector by creating
    /// one dynamically on the node.
    pub async fn dom_id_to_selector(id: u32, tab_id: u32) -> JsValue;

    /// Attaches the debugger to the given tab.
    pub async fn attach_debugger(tab_id: u32);
    /// Detached the debugger from the given tab.
    pub async fn detach_debugger(tab_id: u32);

    /// Enables the debugger DOM API.
    pub async fn dom_enable(tab_id: u32);
    /// Disables the debugger DOM API.
    pub async fn dom_disable(tab_id: u32);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
