use sycamore::prelude::*;
use wasm_bindgen::prelude::*;

use voxurf::get_message;

#[wasm_bindgen]
pub fn main() {
    // vars to be loaded for script starts
    let env_vars = 

    // show extension
    sycamore::render(|cx| {
        view! { cx, App() }
    });

    // save vars 
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! {
        cx,
        p { (get_message()) }
        button(on:click=|_| { }) { "THE BUTTON" }
    }
}
