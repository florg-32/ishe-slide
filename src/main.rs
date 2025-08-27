use chrono::Local;
use dioxus::prelude::*;
use std::io::Write;
#[cfg(feature = "mobile")]
use std::path::Path;
use std::path::PathBuf;
#[cfg(feature = "web")]
use web_sys::wasm_bindgen::{JsCast, JsValue};
use zip::{ZipWriter, write::SimpleFileOptions};

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

static SAMPLES: GlobalSignal<Vec<(i64, i16)>> = Signal::global(Vec::new);

#[component]
fn App() -> Element {
    let mut state = use_signal(|| State::Start);
    let mut start_time = use_signal(chrono::Local::now);

    let app = match *state.read() {
        State::Start => rsx! {
            Welcome {
                onstart: move |_| { async move {
                    start_time.set(chrono::Local::now());
                    state.set(State::Recording);
                    audio::play_pattern().await;
                }
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
        div {
            img { src: ISHE_LOGO }
            h1 { "Welcome!" }
            button { onclick: move |evt| onstart.call(evt), "Start" }
        }
    }
}

#[component]
fn Recording(
    start_time: chrono::DateTime<chrono::Local>,
    onrestart: EventHandler<MouseEvent>,
) -> Element {
    let mut session_ended = use_signal(|| false);
    let mut file_uploaded_class = use_signal(String::new);

    rsx! {
        button { onclick: move |_| session_ended.set(true), "End Session" }
        div { id: "slider",
            input {
                r#type: "range",
                min: -100,
                max: 100,
                step: 1,
                value: 0,
                oninput: move |evt| {
                    SAMPLES
                        .write()
                        .push((
                            (chrono::Local::now() - start_time).num_milliseconds(),
                            evt.value().parse::<i16>().unwrap(),
                        ))
                },
            }
            hr {}
            div { style: "top: 0%;", "+100 😀" }
            div { style: "top: 100%; transform: translateY(-100%);", "-100 😖" }
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
                            save_file(&data, &name);
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

#[cfg(not(feature = "server"))]
fn prepare_file(start_time: chrono::DateTime<Local>, samples: &[(i64, i16)]) -> (String, Vec<u8>) {
    let mut name = start_time.to_rfc3339();
    name.push_str(".csv");

    let mut wtr = csv::Writer::from_writer(vec![]);
    for s in samples {
        wtr.serialize(s).unwrap();
    }
    let data = wtr.into_inner().unwrap();

    (name, data)
}

#[cfg(feature = "server")]
fn prepare_file(start_time: chrono::DateTime<Local>, samples: &[(i64, i16)]) -> (String, Vec<u8>) {
    (String::new(), Vec::new())
}

fn save_file(data: &[u8], filename: &str) {
    #[cfg(feature = "web")]
    {
        let blob = gloo::file::Blob::new(data);
        let url = gloo::file::ObjectUrl::from(blob);

        let document = web_sys::window().unwrap().document().unwrap();
        let anchor = document
            .create_element("a")
            .unwrap()
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .unwrap();
        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.set_hidden(true);

        document.body().unwrap().append_child(&anchor).unwrap();
        anchor.click();
        document.body().unwrap().remove_child(&anchor).unwrap();
    }

    #[cfg(feature = "mobile")]
    {
        let data_path = PathBuf::from(get_cache_dir().unwrap());
        std::fs::write(data_path.join(filename), data).unwrap();
    }
}

#[server]
async fn upload_recording(recording: Vec<u8>, filename: String) -> Result<(), ServerFnError> {
    std::fs::write(filename, &recording).map_err(|e| ServerFnError::new(e))
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
    if !name.ends_with(".csv") {
        return Err(ServerFnError::new("Invalid file"));
    }

    let path = PathBuf::from("./").join(name);
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
                        save_file(&download_all_recordings().await.unwrap(), "recordings.zip");
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

#[cfg(feature = "mobile")]
fn get_cache_dir() -> anyhow::Result<String> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    let ctx = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };
    let cache_dir = env
        .call_method(ctx, "getFilesDir", "()Ljava/io/File;", &[])?
        .l()?;
    let cache_dir: jni::objects::JString = env
        .call_method(&cache_dir, "toString", "()Ljava/lang/String;", &[])?
        .l()?
        .try_into()?;
    let cache_dir = env.get_string(&cache_dir)?;
    let cache_dir = cache_dir.to_str()?;
    Ok(cache_dir.to_string())
}
