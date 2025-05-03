use dioxus::prelude::*;
use std::io::Write;
use std::path::{Path, PathBuf};
use web_sys::wasm_bindgen::{JsCast, JsValue};
use zip::{write::SimpleFileOptions, ZipWriter};

mod audio;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const PICO_CSS: Asset = asset!("/assets/pico.min.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const ISHE_LOGO: Asset = asset!("/assets/ishe.gif");

fn main() {
    dioxus::launch(|| {
        rsx! {
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: PICO_CSS }
            document::Link { rel: "stylesheet", href: MAIN_CSS }
            document::Link {
                rel: "stylesheet",
                href: "https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.colors.min.css",
            }
            Router::<Route> {}
        }
    });
}

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[route("/")]
    App {},
    #[route("/list")]
    List {},
}

enum State {
    Start,
    Recording,
}

static SAMPLES: GlobalSignal<Vec<(f64, u8)>> = Signal::global(Vec::new);

#[component]
fn App() -> Element {
    let mut state = use_signal(|| State::Start);
    let mut start_time = use_signal(|| 0.0);

    let app = match *state.read() {
        State::Start => rsx! {
            Welcome {
                onstart: move |_| {
                    audio::play_pattern().unwrap();
                    start_time.set(js_sys::Date::now());
                    state.set(State::Recording);
                },
            }
        },
        State::Recording => rsx! {
            Recording {
                start_time: start_time(),
                onrestart: move |_| {
                    SAMPLES.write().clear();
                    state.set(State::Start);
                },
            }
        },
    };

    rsx! {
        main { class: "container", {app} }
    }
}

#[component]
fn Welcome(onstart: EventHandler<MouseEvent>) -> Element {
    rsx! {
        img { src: ISHE_LOGO }
        h1 { "Welcome!" }
        button { onclick: move |evt| onstart.call(evt), "Start" }
    }
}

#[component]
fn Recording(start_time: f64, onrestart: EventHandler<MouseEvent>) -> Element {
    let mut session_ended = use_signal(|| false);
    let mut file_uploaded_class = use_signal(String::new);

    rsx! {
        button { onclick: move |_| session_ended.set(true), "End Session" }
        input {
            r#type: "range",
            min: 0,
            max: 100,
            step: 1,
            oninput: move |evt| {
                SAMPLES
                    .write()
                    .push((js_sys::Date::now() - start_time, evt.value().parse::<u8>().unwrap()))
            },
        }
        dialog { open: session_ended.read().to_string(),
            article {
                header {
                    a {
                        aria_label: "Close",
                        rel: "prev",
                        onclick: move |_| session_ended.set(false),
                    }
                    h2 { "Thank you!" }
                }
                div {
                    button {
                        onclick: move |_| {
                            let (name, data) = prepare_file(start_time, &SAMPLES.read());
                            download_file(&data, &name).unwrap();
                        },
                        "Save recording"
                    }
                    button {
                        disabled: !file_uploaded_class.read().is_empty(),
                        class: "{file_uploaded_class}",
                        onclick: move |_| {
                            async move {
                                let (name, data) = prepare_file(start_time, &SAMPLES.read());
                                if upload_recording(data, name).await.is_ok() {
                                    file_uploaded_class
                                        .set("disabled pico-background-green-600".to_string());
                                }
                            }
                        },
                        "Upload recording"
                    }
                    hr {}
                    button {
                        class: "pico-background-red-600",
                        onclick: move |evt| onrestart.call(evt),
                        "Restart"
                    }
                }
            }
        }
    }
}

fn prepare_file(start_time: f64, samples: &[(f64, u8)]) -> (String, Vec<u8>) {
    let date = js_sys::Date::new(&JsValue::from_f64(start_time));
    let name = format!("{}-{}.csv", date.to_locale_time_string("de-DE"), date.get_milliseconds());

    let mut wtr = csv::Writer::from_writer(vec![]);
    for s in samples {
        wtr.serialize(s).unwrap();
    }
    let data = wtr.into_inner().unwrap();

    (name, data)
}

fn download_file(data: &[u8], filename: &str) -> Result<(), JsValue> {
    let blob = gloo::file::Blob::new(data);
    let url = gloo::file::ObjectUrl::from(blob);

    let document = web_sys::window().unwrap().document().unwrap();
    let anchor = document
        .create_element("a")?
        .dyn_into::<web_sys::HtmlAnchorElement>()?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.set_hidden(true);

    document.body().unwrap().append_child(&anchor)?;
    anchor.click();
    document.body().unwrap().remove_child(&anchor)?;

    Ok(())
}

#[server]
async fn upload_recording(recording: Vec<u8>, filename: String) -> Result<(), ServerFnError> {
    std::fs::write(filename, &recording);
    Ok(())
}

#[server]
async fn list_recordings() -> Result<Vec<String>, ServerFnError> {
    Ok(std::fs::read_dir("./")
        .unwrap()
        .filter_map(|f| f.ok())
        .map(|f| f.file_name().to_string_lossy().into_owned())
        .filter(|f| f.ends_with(".csv"))
        .collect())
}

#[server]
async fn delete_recording(name: String) -> Result<(), ServerFnError> {
    let path = PathBuf::from(name);
    if !path.parent().unwrap().as_os_str().is_empty() {
        return Err(ServerFnError::new("invalid path"));
    }

    Ok(std::fs::remove_file(path)?)
}

#[server]
async fn download_all_recordings() -> Result<Vec<u8>, ServerFnError> {
    let mut writer = ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for f in list_recordings().await? {
        writer.start_file(&f, SimpleFileOptions::default())?;
        writer.write_all(&std::fs::read(f)?)?;
    }
    Ok(writer.finish()?.into_inner())
}

#[component]
fn List() -> Element {
    let mut recordings = use_server_future(list_recordings)?;

    rsx! {
        main { class: "container",
            input {
                r#type: "button",
                disabled: recordings().unwrap().unwrap().is_empty(),
                value: "Download all",
                onclick: move |_| {
                    async move {
                        download_file(&download_all_recordings().await.unwrap(), "recordings.zip")
                            .unwrap();
                    }
                },
            }
            table { class: "striped",
                tbody {
                    for f in recordings().unwrap().unwrap() {
                        tr {
                            td { "{f}" }
                            td {
                                style: "cursor: pointer; text-align: center;",
                                onclick: move |_| {
                                    let f = f.clone();
                                    async move {
                                        let _ = delete_recording(f).await;
                                        recordings.restart();
                                    }
                                },
                                "❌"
                            }
                        }
                    }
                }
            }
        }
    }
}
