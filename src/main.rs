use dioxus::prelude::*;
use std::time::Duration;

mod audio;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const PICO_CSS: Asset = asset!("/assets/pico.min.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const ISHE_LOGO: Asset = asset!("/assets/ishe.gif");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: PICO_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        main { class: "container",
            Welcome {
                onclick: move |_| audio::play_sine_for(440.0, Duration::from_millis(250)).unwrap()
            }
        }
    }
}

#[component]
fn Welcome(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        img { src: ISHE_LOGO }
        h1 { "Welcome!" }
        button {
            onclick: move |evt| onclick.call(evt),
            "Start"
        }
    }
}
