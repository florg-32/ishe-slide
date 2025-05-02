use gloo::timers::callback::Timeout;
use std::time::Duration;
use web_sys::{wasm_bindgen::JsValue, OscillatorType};

pub fn play_sine_for(freq: f32, duration: Duration) -> Result<(), JsValue> {
    let ctx = web_sys::AudioContext::new()?;
    let osc = ctx.create_oscillator()?;
    let volume = ctx.create_gain()?;

    osc.set_type(OscillatorType::Sine);
    osc.frequency().set_value(freq);
    osc.connect_with_audio_node(&volume)?;
    volume.connect_with_audio_node(&ctx.destination())?;

    osc.start()?;
    Timeout::new(duration.as_millis().try_into().unwrap(), move || {
        volume
            .gain()
            .set_target_at_time(0.001, ctx.current_time(), 0.015)
            .unwrap();
        Timeout::new(150, move || {
            osc.stop().unwrap();
            let _ = ctx.close().unwrap();
        })
        .forget();
    })
    .forget();

    Ok(())
}
