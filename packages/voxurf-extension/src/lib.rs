mod command;
mod openai;

use sycamore::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

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
    command::execute_command("Get rid of the sidebar on this website.".to_string());
    view! { cx,
        div(class="row-arrange") {
            div(class="left") {
                p(id="transcription") { "Did I hear you right?" }
            }
            div(class="right") {
                DynamicButton()
            }
        }
    }
}

#[component]
fn DynamicButton<G: Html>(cx: Scope) -> View<G> {
    let app_state = create_signal(cx, AppState::Available);
    // create_effect(cx, || {});
    view! { cx,
        (match &*app_state.get() {
            AppState::Listening => {
                view! { cx,
                    button(disabled=true) {
                        // TODO: Need an animation
                        img(src="assets/zrolatency_logo_blue_back.png")
                        p { "Recording..." }
                    }
                }
            }
            AppState::Processing => {
                view! { cx,
                    button(disabled=true) {
                        img(src="assets/zrolatency_logo_blue_back.png")
                        p { "Processing." }
                    }

                }
            }
            AppState::Available => {
                view! { cx,
                    button(on:click=|_| {app_state.set(AppState::Listening)}, class="btn-active") {
                        img(src="assets/zrolatency_logo_red_back.png", id="button_record")
                    }
                }
            }
        })
    }
}
