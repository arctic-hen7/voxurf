use sycamore::prelude::*;
use wasm_bindgen::prelude::*;

// Code State
// enum State {
//     Listening,
//     Processing,
//     Available,
// }

// impl Display for State {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             State::Listening => write!(f, "Listening"),
//             State::Processing => write!(f, "Processing"),
//             State::Available => write!(f, "Record"),
//         }
//     }
// }

#[wasm_bindgen]
pub fn main() {
    sycamore::render(|cx| {
        view! { cx, App() }
    });
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    // let interactive = create_memo(cx, || state.get() == State::Available);
    view! {
        cx,
        textarea(readonly=true)
        button(style="width:100%") { "RECORD"}
    }
}
