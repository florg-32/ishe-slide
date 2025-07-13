use gloo::timers::callback::Timeout;
use std::time::Duration;
use web_sys::{OscillatorType, wasm_bindgen::JsValue};

pub fn play_sine_for(freq: f32, end_freq: f32, duration: Duration) -> Result<(), JsValue> {
    let ctx = web_sys::AudioContext::new()?;
    let osc = ctx.create_oscillator()?;
    let volume = ctx.create_gain()?;

    osc.set_type(OscillatorType::Sine);
    osc.frequency().set_value(freq);
    osc.frequency()
        .linear_ramp_to_value_at_time(end_freq, ctx.current_time() + duration.as_secs_f64())?;
    volume.gain().set_value(1.0);
    osc.connect_with_audio_node(&volume)?;
    volume.connect_with_audio_node(&ctx.destination())?;

    osc.start()?;
    Timeout::new(duration.as_millis().try_into().unwrap(), move || {
        volume
            .gain()
            .set_target_at_time(0.01, ctx.current_time(), 0.015)
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

pub fn play_pattern() -> Result<(), JsValue> {
    play_sine_for(400.0, 1000.0, Duration::from_millis(500))?;
    play_sine_for(2000.0, 3000.0, Duration::from_millis(500))
}
