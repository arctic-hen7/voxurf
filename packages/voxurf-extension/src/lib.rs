use sycamore::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main() {
    sycamore::render(|cx| {
        view! { cx, App() }
    });
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! {
        cx,
        div(class="row-arrange") {
            div(class="left") {
                p(id="transcription") { "Did I hear you right?" }
            }
            div(class="right") {
                button() { img(src="assets/zrolatency_logo_red_back.png")}
            }
        }
    }
}
