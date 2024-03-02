mod command;

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
    command::execute_command("Send an email to John.".to_string());
    view! {
        cx,
        p { (get_message()) }
    }
}
