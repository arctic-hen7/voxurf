use sycamore::{component, prelude::*, view};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main() {
    sycamore::render(|cx| {
        view! { cx, App() }
    });
}

enum AppState {
    Listening,
    Processing,
    Available,
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let app_state = create_signal(cx, AppState::Listening);
    view! { cx,
        div(class="row-arrange") {
            div(class="left") {
                p(id="transcription") { "Did I hear you right?" }
            }
            div(class="right") {
                DynamicButton(app_state)
            }
        }
    }
}

#[component]
fn DynamicButton<G: Html>(cx: Scope, state: &ReadSignal<AppState>) -> View<G> {
    match &*state.get() {
        // TODO: Apply UI in each state.
        AppState::Listening => {
            view! { cx,
                p { "ListenMode "}
            }
        }
        AppState::Processing => {
            view! { cx,
                p { "ProcessMode "}
            }
        }
        AppState::Available => {
            view! { cx,
                p { "AvailMode "}
            }
        }
    }
}
