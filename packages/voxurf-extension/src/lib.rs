mod compute_tree;
mod glue;
mod interface;
mod openai_model;

use crate::{interface::WebExtensionInterface, openai_model::OpenAiModel};
use gloo_net::http::Request;
use sycamore::prelude::*;
use voxurf::{Executor, ExecutorOpts};
use wasm_bindgen::prelude::*;

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
async fn App<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let interface = WebExtensionInterface::new().await;
    let model = OpenAiModel::new(&env!("OPENAI_API_KEY"));
    let executor = Executor::new(
        &interface,
        &model,
        // TODO Make these all configurable
        ExecutorOpts {
            max_round_trips: 5,
            tree_poll_interval_ms: 50,
            stability_threshold_ms: 250,
            stability_timeout_ms: 10_000,
        },
    );
    let state = create_signal(cx, AppState::Idle);

    view! { cx,
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

                                // Execute the user's command
                                interface.pre_execute().await;
                                executor.execute_command(&command).await;
                                interface.post_execute().await;
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
    resp.text().await.unwrap()
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
