use sycamore::prelude::*;
use wasm_bindgen::prelude::*;

use voxurf::get_message;

#[wasm_bindgen]
pub fn main() {
    sycamore::render(|cx| {
        view! { cx, App() }
    })
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! {
        cx,
        p { (get_message()) }
    }
}
