use sycamore::{component, prelude::*, view};
use sycamore_router::{HistoryIntegration, Route, Router};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main() {
    sycamore::render(|cx| {
        view! { cx, App() }
    });
}

#[derive(Route)]
enum AppState {
    #[to("/")]
    Listening,
    #[to("/process")]
    Processing,
    #[to("/free")]
    Available,
    #[not_found]
    NotFound,
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let app_state = create_signal(cx, AppState::Listening);
    view! { cx,
        div(class="row-arrange") {
            Router(
                integration=HistoryIntegration::new(),
                view=|cx, route: &ReadSignal<AppState>| {
                    view! { cx,
                        (match route.get().as_ref() {
                            // TODO: Implement States
                            AppState::Available => view! { cx,
                            },
                            AppState::Listening => view! { cx,
                            },
                            AppState::Processing=> view! { cx,
                            },
                            AppState::NotFound=> view! { cx,
                            }
                        })
                    }
                }
            )
            div(class="left") {
                p(id="transcription") { "Did I hear you right?" }
            }
        }
    }
}
