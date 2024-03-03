mod command;
mod openai;

use sycamore::prelude::*;
use wasm_bindgen::prelude::*;

use voxurf::get_message;

#[wasm_bindgen]
pub fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    sycamore::render(|cx| {
        view! { cx, App() }
    })
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    command::execute_command("Get rid of the sidebar on this website.".to_string());
    view! {
        cx,
        p { (get_message()) }
    }
}
