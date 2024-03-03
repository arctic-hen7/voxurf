mod command;
mod openai;

use gloo_net::http::Request;
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;

use crate::command::execute_command;

#[wasm_bindgen]
pub fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    sycamore::render(|cx| {
        view! { cx, App() }
    });
}

#[derive(PartialEq, Eq)]
enum AppState {
    Idle,
    Recording,
    Executing,
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let state = create_signal(cx, AppState::Idle);

    view! { cx,
        div(class="row-arrange") {
            div(class="left") {
                p(id="transcription") { "Did I hear you right?" }
            }
            div(class="right") {
                button(
                    class = format!(
                        "rounded-full h-24 w-24 p-2 {}",
                        match *state.get() {
                            AppState::Idle => "bg-red-500",
                            AppState::Recording => "bg-blue-500",
                            AppState::Executing => "bg-emerald-500",
                        }
                    ),
                    disabled = *state.get() == AppState::Executing,
                    on:click = move |_| {
                        sycamore::futures::spawn_local_scoped(cx, async move {
                            match *state.get() {
                                AppState::Idle => {
                                    // Set the state *after* we're ready to record to avoid
                                    // speaking before recording
                                    start_recording().await;
                                    state.set(AppState::Recording);
                                },
                                AppState::Recording => {
                                    // Set the state *before* we start working so we encapsulate
                                    // everything in the execution phase
                                    state.set(AppState::Executing);
                                    let command = stop_recording().await;
                                    execute_command(&command).await;
                                },
                                AppState::Executing => unreachable!(),
                            }
                        });
                    }
                ) {
                    img(src = "assets/logo_core.webp") {}
                }
            }
        }
    }
}

/// Begins the recording on the local server. This will return when recording has
/// started.
async fn start_recording() {
    let resp = Request::get("http://localhost:3000/start-recording")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

/// Ends the recording on the local server, returning the transcribed text.
async fn stop_recording() -> String {
    let resp = Request::get("http://localhost:3000/end-recording")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    resp
        .text()
        .await
        .unwrap()
}

// #[component]
// fn DynamicButton<G: Html>(cx: Scope) -> View<G> {
//     let app_state = create_signal(cx, AppState::Available);
//     // create_effect(cx, || {});
//     view! { cx,
//         (match &*app_state.get() {
//             AppState::Listening => {
//                 view! { cx,
//                     button(disabled=true) {
//                         // TODO: Need an animation
//                         img(src="assets/zrolatency_logo_blue_back.png")
//                         p { "Recording..." }
//                     }
//                 }
//             }
//             AppState::Processing => {
//                 view! { cx,
//                     button(disabled=true) {
//                         img(src="assets/zrolatency_logo_blue_back.png")
//                         p { "Processing." }
//                     }

//                 }
//             }
//             AppState::Available => {
//                 view! { cx,
//                     button(on:click=|_| {app_state.set(AppState::Listening)}, class="btn-active") {
//                         img(src="assets/zrolatency_logo_red_back.png", id="button_record")
//                     }
//                 }
//             }
//         })
//     }
// }
