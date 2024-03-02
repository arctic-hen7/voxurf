use sycamore::{component, prelude::*, view};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main() {
    sycamore::render(|cx| {
        let app_state = create_signal(cx, AppState::Available);
        view! { cx, App(app_state) }
    });
}

enum AppState {
    Listening,
    Processing,
    Available,
}

#[component]
fn App<G: Html>(cx: Scope, state: &Signal<AppState>) -> View<G> {
    view! { cx,
        div(class="row-arrange") {
            div(class="left") {
                p(id="transcription") { "Did I hear you right?" }
            }
            div(class="right") {
                DynamicButton(&state)
            }
        }
    }
}

#[component]
fn DynamicButton<G: Html>(cx: Scope, state: &Signal<AppState>) -> View<G> {
    match &*state.get() {
        // TODO: Apply UI in each state.
        AppState::Listening => {
            view! { cx,
                button(disabled=true) { button(src="assets/zrolatency_logo_red_back.png") }
            }
        }
        AppState::Processing => {
            view! { cx,
                button(disabled=true) { button(src="assets/zrolatency_logo_red_back.png") }
            }
        }
        AppState::Available => {
            view! { cx,
                button(on:click=|_| {state.set(AppState::Listening)}) { button(src="assets/zrolatency_logo_red_back.png") }
            }
        }
    }
}
